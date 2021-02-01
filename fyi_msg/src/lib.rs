/*!
# FYI Msg

This crate contains the objects providing the heart of the FYI command line
application, namely [Msg], a simple struct for status-like messages that can be
easily printed to `Stdout` or `Stderr`.



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

pub use msg::{
	ansi::NiceANSI,
	buffer::MsgBuffer10,
	buffer::MsgBuffer2,
	buffer::MsgBuffer3,
	buffer::MsgBuffer4,
	buffer::MsgBuffer5,
	buffer::MsgBuffer6,
	buffer::MsgBuffer7,
	buffer::MsgBuffer8,
	buffer::MsgBuffer9,
	FLAG_INDENT,
	FLAG_NEWLINE,
	kind::MsgKind,
	Msg,
};

#[cfg(feature = "timestamps")]
pub use msg::FLAG_TIMESTAMP;
