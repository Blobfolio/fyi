/*!
# FYI
*/

#![forbid(unsafe_code)]

#![deny(
	clippy::allow_attributes_without_reason,
	clippy::correctness,
	unreachable_pub,
)]

#![warn(
	clippy::complexity,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::style,

	clippy::allow_attributes,
	clippy::clone_on_ref_ptr,
	clippy::create_dir,
	clippy::filetype_is_file,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::impl_trait_in_params,
	clippy::lossy_float_literal,
	clippy::missing_assert_message,
	clippy::missing_docs_in_private_items,
	clippy::needless_raw_strings,
	clippy::panic_in_result_fn,
	clippy::pub_without_shorthand,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::semicolon_inside_block,
	clippy::str_to_string,
	clippy::string_to_string,
	clippy::todo,
	clippy::undocumented_unsafe_blocks,
	clippy::unneeded_field_pattern,
	clippy::unseparated_literal_suffix,
	clippy::unwrap_in_result,

	macro_use_extern_crate,
	missing_copy_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![expect(clippy::redundant_pub_crate, reason = "Unresolvable.")]



mod cli;
mod error;

use error::FyiError;
use fyi_msg::{
	Msg,
	MsgKind,
};



/// # Main.
fn main() {
	// Handle errors.
	if let Err(e) = main__() {
		match e {
			FyiError::Passthrough(_) => {},
			FyiError::PrintHelp(x) => return helper(x),
			FyiError::PrintVersion => { println!("{}", FyiError::PrintVersion); },
			_ => { Msg::error(e.to_string()).eprint(); },
		}

		let code = e.exit_code();
		if code != 0 { std::process::exit(code); }
	}
}

#[inline]
/// # Actual Main.
///
/// This lets us more easily bubble errors, which are printed and handled
/// specially.
fn main__() -> Result<(), FyiError> {
	let kind = cli::parse_kind()?;
	if matches!(kind, MsgKind::Blank) { return cli::parse_blank(); }
	let (msg, flags) = cli::parse_msg(kind)?;

	if matches!(kind, MsgKind::Confirm) {
		return
			if msg.prompt_with_default(flags.yes()) { Ok(()) }
			else { Err(FyiError::Passthrough(1)) };
	}

	// Print to `STDERR`.
	if flags.stderr() { msg.eprint(); }
	// Print to `STDOUT`.
	else { msg.print(); }

	// Exit as desired.
	flags.exit()
}

#[cold]
/// # Help Page.
///
/// Print the appropriate help screen given the call details. Most of the sub-
/// commands work the same way, but a few have their own distinct messages.
///
/// The contents are generated via `build.rs`, which lowers the runtime cost
/// and shrinks the binary a touch.
fn helper(cmd: MsgKind) {
	use std::io::Write;

	let writer = std::io::stdout();
	let mut handle = writer.lock();

	/// # Helper: Variable Bits.
	macro_rules! write_help {
		($path:literal) => {
			handle.write_all(include_bytes!(concat!(env!("OUT_DIR"), "/help-", $path, ".txt")))
		};
		($path:literal, true) => {
			write_help!($path).and_then(|()| write_help!("generic-bottom"))
		};
	}

	// The top is always the same.
	write_help!("top").unwrap();

	// The middle section varies by subcommand.
	match cmd {
		MsgKind::Blank => write_help!("blank"),
		MsgKind::Confirm => write_help!("confirm"),
		MsgKind::Crunched => write_help!("crunched", true),
		MsgKind::Custom => write_help!("print"),
		MsgKind::Debug => write_help!("debug", true),
		MsgKind::Done => write_help!("done", true),
		MsgKind::Error => write_help!("error", true),
		MsgKind::Info => write_help!("info", true),
		MsgKind::Notice => write_help!("notice", true),
		MsgKind::Review => write_help!("review", true),
		MsgKind::Skipped => write_help!("skipped", true),
		MsgKind::Success => write_help!("success", true),
		MsgKind::Task => write_help!("task", true),
		MsgKind::Warning => write_help!("warning", true),
		_ => write_help!("help"),
	}.unwrap();

	handle.flush().unwrap();
}
