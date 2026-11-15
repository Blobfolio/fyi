/*!
# FYI Msg: Windows Console Support.
*/

use windows_sys::Win32::System::Console::{
	ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle,
	STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
};



#[expect(unsafe_code, reason = "Windows console mode APIs.")]
/// # Enable Windows ANSI Output.
///
/// Enable ANSI processing for console stdout and stderr, preserving other
/// console mode flags. Redirected streams and unavailable consoles are left
/// alone. This can be called before printing messages or starting progress.
///
/// Failures are ignored so output can still be redirected or used without a
/// console. This does not install any interrupt handlers.
///
/// ## Examples
///
/// ```
/// #[cfg(windows)]
/// fyi_msg::enable_console_ansi();
/// ```
pub fn enable_console_ansi() {
	for stream in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
		// SAFETY: GetConsoleMode checks that the standard handle is a console
		// before its existing mode flags are updated.
		unsafe {
			let handle = GetStdHandle(stream);
			let mut mode = 0;
			if GetConsoleMode(handle, &raw mut mode) != 0 {
				SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
			}
		}
	}
}



#[cfg(feature = "signals_sigint")]
pub(crate) use signals::{sigint_default, sigint_keepalive, sigint_two_strike};

#[cfg(feature = "signals_sigint")]
/// # Windows Console Interrupt Policies.
mod signals {
	use crate::{Progless, progress::signals::sigint_error};
	use std::sync::{
		Arc, OnceLock,
		atomic::{AtomicBool, Ordering::SeqCst},
	};
	use windows_sys::Win32::{
		Storage::FileSystem::WriteFile,
		System::{
			Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, GetStdHandle, STD_ERROR_HANDLE, SetConsoleCtrlHandler},
			Threading::ExitProcess,
		},
	};

	#[derive(Clone, Copy)]
	/// # Process-wide Interrupt Policy.
	enum Policy {
		/// # Exit on the First Interrupt.
		Default,
		/// # Exit on the Second Interrupt.
		TwoStrike,
		/// # Report Interrupts without Exiting.
		Keepalive,
	}

	/// # Handler State.
	///
	/// Published before registration so a callback can never race with an
	/// incomplete outer `OnceLock` initialization.
	static STATE: OnceLock<(Arc<AtomicBool>, Policy)> = OnceLock::new();

	/// # Default Interrupt Handling.
	pub(crate) fn sigint_default() -> Arc<AtomicBool> { register(Policy::Default) }

	/// # Two-Strike Interrupt Handling.
	pub(crate) fn sigint_two_strike() -> Arc<AtomicBool> { register(Policy::TwoStrike) }

	/// # Keepalive Interrupt Handling.
	pub(crate) fn sigint_keepalive() -> Arc<AtomicBool> { register(Policy::Keepalive) }

	#[expect(unsafe_code, reason = "Windows console handler registration.")]
	/// # Register a Process-wide Console Handler.
	fn register(policy: Policy) -> Arc<AtomicBool> {
		let (switch, _) = STATE.get_or_init(|| (Arc::new(AtomicBool::new(false)), policy));
		// SAFETY: the callback has the required ABI and uses only static state.
		// Register before enabling Ctrl+C, which a parent such as Git Bash may
		// have disabled. A custom callback alone does not clear that inherited
		// setting; opting into our policy must activate interrupt delivery too.
		let registered = unsafe {
			SetConsoleCtrlHandler(Some(interrupt), 1) != 0 &&
			SetConsoleCtrlHandler(None, 0) != 0
		};
		if ! registered { sigint_error(); }
		Arc::clone(switch)
	}

	#[expect(unsafe_code, reason = "Windows console callback and immediate process exit.")]
	/// # Handle Ctrl+C and Ctrl+Break.
	unsafe extern "system" fn interrupt(event: u32) -> i32 {
		if ! matches!(event, CTRL_C_EVENT | CTRL_BREAK_EVENT) { return 0; }
		let Some((switch, policy)) = STATE.get() else { return 0; };
		let received = switch.swap(true, SeqCst);
		if ! received { unhide_cursor(); }
		if matches!(policy, Policy::Default) || (received && matches!(policy, Policy::TwoStrike)) {
			// SAFETY: immediate exit matches the Unix signal handler and avoids
			// running Rust cleanup while another thread might hold a stdio lock.
			unsafe { ExitProcess(1); }
		}
		1
	}

	#[expect(unsafe_code, reason = "Restore the cursor without acquiring Rust stdio locks in a callback.")]
	/// # Restore Cursor Visibility.
	fn unhide_cursor() {
		let mut written = 0;
		// SAFETY: the static byte string is valid for the entire synchronous
		// write. WriteFile validates the standard handle, including redirection.
		unsafe {
			WriteFile(
				GetStdHandle(STD_ERROR_HANDLE), Progless::CURSOR_UNHIDE.as_ptr(),
				u32::try_from(Progless::CURSOR_UNHIDE.len()).unwrap(),
				&raw mut written, std::ptr::null_mut(),
			);
		}
	}
}
