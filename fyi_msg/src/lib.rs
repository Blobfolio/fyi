/*!
# FYI Msg

This crate contains the objects providing the heart of the FYI command line
application, namely [Msg], a simple struct for status-like messages that can be
easily printed to `Stdout` or `Stderr`.



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
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



mod msg;

pub use msg::{
	Msg,
	MsgBuffer2,
	MsgBuffer3,
	MsgBuffer4,
	MsgBuffer5,
	MsgBuffer6,
	MsgBuffer7,
	MsgBuffer8,
	MsgBuffer9,
	MsgBuffer10,
	MsgKind,
	FLAG_INDENT,
	FLAG_NEWLINE,
	FLAG_TIMESTAMP,
};
