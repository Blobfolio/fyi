/*!
# FYI Msg

This crate contains the objects providing the heart of the FYI command line
application, namely [`Msg`], a simple struct for status-like messages that can be
easily printed to `Stdout` or `Stderr`.



## Macros

| Macro | Equivalent |
| ----- | ---------- |
| `confirm!(…)` | `Msg::new(MsgKind::Confirm, "Some question…").prompt()` |



## Optional Features

| Feature | Description |
| ------- | ----------- |
| `fitted` | Enables [`Msg::fitted`] for obtaining a slice trimmed to a specific display width. |
| `progress` | Enables [`Progless`], a thread-safe CLI progress bar displayer.
| `timestamps` | Enables timestamp-related methods and flags like [`Msg::with_timestamp`]. |



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
#[cfg(feature = "progress")] mod progress;

pub use msg::{
	buffer::BUFFER2,
	buffer::BUFFER3,
	buffer::BUFFER4,
	buffer::BUFFER5,
	buffer::BUFFER6,
	buffer::BUFFER7,
	buffer::BUFFER8,
	buffer::BUFFER9,
	buffer::BUFFER10,
	buffer::MsgBuffer,
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

#[cfg(feature = "progress")] pub use progress::Progless;

#[cfg(feature = "timestamps")]
pub use msg::FLAG_TIMESTAMP;

#[macro_use]
mod macros {
	#[macro_export(local_inner_macros)]
	/// # Confirm.
	macro_rules! confirm {
		($text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text).prompt()
		);
	}
}
