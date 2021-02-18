/*!
# FYI Msg

This crate contains the objects providing the heart of the FYI command line
application, namely [Msg], a simple struct for status-like messages that can be
easily printed to `Stdout` or `Stderr`.



## Macros

If manual construction isn't your thing, the following macros allow you to quickly print a prefixed message (with trailing linebreak). Passing `true` as a second argument prints to `Stderr` instead of `Stdout`.

| Macro | Equivalent |
| ----- | ---------- |
| `crunched!(…)` | `Msg::new(MsgKind::Crunched, …).with_newline(true).print()` |
| `debug!(…)` | `Msg::new(MsgKind::Debug, …).with_newline(true).print()` |
| `done!(…)` | `Msg::new(MsgKind::Done, …).with_newline(true).print()` |
| `error!(…)` | `Msg::new(MsgKind::Error, …).with_newline(true).print()` |
| `info!(…)` | `Msg::new(MsgKind::Info, …).with_newline(true).print()` |
| `notice!(…)` | `Msg::new(MsgKind::Notice, …).with_newline(true).print()` |
| `success!(…)` | `Msg::new(MsgKind::Success, …).with_newline(true).print()` |
| `task!(…)` | `Msg::new(MsgKind::Task, …).with_newline(true).print()` |
| `warning!(…)` | `Msg::new(MsgKind::Warning, …).with_newline(true).print()` |

You can print an unprefixed message with the following. Like the above, a second parameter of `true` will cause it to print to `Stderr`.

| Macro | Equivalent |
| ----- | ---------- |
| `plain!(…)` | `Msg::plain(…).with_newline(true).print()` |

You can also prompt a confirmation with the following, which will return a `bool`, `true` for Yes, and `false` for No. This one can only be directed to `Stdout`.

| Macro | Equivalent |
| ----- | ---------- |
| `confirm!(…)` | `Msg::new(MsgKind::Confirm, …).prompt()` |



## Optional Features

| Feature | Description |
| ------- | ----------- |
| fitted | Enables [`Msg::fitted`] for obtaining a slice trimmed to a specific display width. |
| timestamps | Enables timestamp-related methods and flags like [`Msg::with_timestamp`]. |



## Stability

Release versions of this library should be in a working state, but as this
project is under perpetual development, code might change from version to
version.
*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod msg;

#[cfg(feature = "fitted")] mod fitted;

pub use msg::{
	buffer::MsgBuffer10,
	buffer::MsgBuffer2,
	buffer::MsgBuffer3,
	buffer::MsgBuffer4,
	buffer::MsgBuffer5,
	buffer::MsgBuffer6,
	buffer::MsgBuffer7,
	buffer::MsgBuffer8,
	buffer::MsgBuffer9,
	buffer::MsgBuffer,
	buffer::DefaultMsgBuffer,
	FLAG_INDENT,
	FLAG_NEWLINE,
	kind::MsgKind,
	Msg,
};

#[cfg(feature = "fitted")]
pub use fitted::{
	length_width,
	width,
};

#[cfg(feature = "timestamps")]
pub use msg::FLAG_TIMESTAMP;

#[macro_use]
mod macros {
	#[macro_export]
	/// # Print a message with prefix to STDERR.
	macro_rules! eprint_msg {
		($prefix:expr, $text:expr) => {
			$crate::Msg::new($prefix, $text).with_newline(true).eprint()
		};
	}

	#[macro_export]
	/// # Print a message with prefix to STDOUT.
	macro_rules! print_msg {
		($prefix:expr, $text:expr) => {
			$crate::Msg::new($prefix, $text).with_newline(true).print()
		};
	}

	#[macro_export(local_inner_macros)]
	/// # Confirm.
	macro_rules! confirm {
		($text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text).prompt()
		);
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Crunched message.
	macro_rules! crunched {
		($text:expr) => { print_msg!($crate::MsgKind::Crunched, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Crunched, $text); };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Debug message.
	macro_rules! debug {
		($text:expr) => { print_msg!($crate::MsgKind::Debug, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Debug, $text); };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Done message.
	macro_rules! done {
		($text:expr) => { print_msg!($crate::MsgKind::Done, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Done, $text); };
	}

	#[macro_export]
	/// # Print an Error.
	macro_rules! error {
		($text:expr) => { $crate::Msg::error($text).print() };
		($text:expr, true) => { $crate::Msg::error($text).eprint() };
	}

	#[macro_export(local_inner_macros)]
	/// # Print an Info message.
	macro_rules! info {
		($text:expr) => { print_msg!($crate::MsgKind::Info, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Info, $text); };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Notice.
	macro_rules! notice {
		($text:expr) => { print_msg!($crate::MsgKind::Notice, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Notice, $text); };
	}

	#[macro_export]
	/// # Print a message with prefix to STDOUT.
	macro_rules! plain {
		($text:expr) => { $crate::Msg::plain($text).with_newline(true).print() };
		($text:expr, true) => { $crate::Msg::plain($text).with_newline(true).eprint() };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Success.
	macro_rules! success {
		($text:expr) => { print_msg!($crate::MsgKind::Success, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Success, $text); };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Task.
	macro_rules! task {
		($text:expr) => { print_msg!($crate::MsgKind::Task, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Task, $text); };
	}

	#[macro_export(local_inner_macros)]
	/// # Print a Warning.
	macro_rules! warning {
		($text:expr) => { print_msg!($crate::MsgKind::Warning, $text); };
		($text:expr, true) => { eprint_msg!($crate::MsgKind::Warning, $text); };
	}
}
