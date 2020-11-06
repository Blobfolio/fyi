/*!
# FYI Menu

This crate contains an agnostic CLI argument parser called [`Argue`]. Unlike more
robust libraries like [clap](https://crates.io/crates/clap), [`Argue`] does not
hold information about expected or required arguments; it merely parses the raw
[`std::env::args`] output into a consistent state.

Post-processing is an exercise largely left to the implementing library to do
in its own way, in its own time. [`Argue`] exposes several methods for quickly
querying the individual pieces of the set, but it can also be dereferenced to a
slice of strings or consumed into an owned string vector for fully manual
processing if desired.

For simple applications, this agnostic approach can significantly reduce the
overhead of processing CLI arguments, but because handling is left to the
implementing library, it might be too tedious or limiting for more complex use
cases.

This crate also contains a build tool called [`Agree`] — hidden behind the
`bashman` crate feature flag — that allows you to all the ins and outs of your
app to generate BASH completions and/or MAN page(s).

[`Agree`] is meant to be run from `build.rs`. Done that way, it should not
have any effect on the binary's runtime performance or size.



## Stability: Alpha

This project is under heavy development and subject to change. While the code
in the `master` branch should always be in a "working" state, breaking changes
and major refactors may be introduced between releases.

(This should probably *not* be used in production-ready applications.)
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
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



#[cfg(feature = "bashman")] mod agree;
mod argue;
mod keykind;
pub mod utility;

#[cfg(feature = "bashman")]
pub use agree::{
	Agree,
	AgreeKind,
	AgreeSwitch,
	AgreeOption,
	AgreeItem,
	AgreeParagraph,
	AgreeSection,
};

pub use argue::{
	Argue,
	FLAG_REQUIRED,
	FLAG_SEPARATOR,
	FLAG_SUBCOMMAND,
};

pub use keykind::KeyKind;



/// # Print an Error and Exit.
///
/// This method prints an error message to `Stderr` and terminates the thread
/// with an exit code of `1`.
///
/// This is used instead of traditional panics in many places, given the CLI-
/// based nature of [`Argue`].
///
/// ## Safety
///
/// The `msg` must be valid UTF-8 or undefined things may happen.
pub fn die(msg: &[u8]) {
	unsafe { fyi_msg::Msg::prefixed_unchecked(fyi_msg::MsgKind::Error, msg) }.eprintln();
	std::process::exit(1);
}
