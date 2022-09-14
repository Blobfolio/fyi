/*!
# FYI Msg

[![docs.rs](https://img.shields.io/docsrs/fyi_msg.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_msg/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_msg.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_msg)
[![ci](https://img.shields.io/github/workflow/status/Blobfolio/fyi/Build.svg?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/fyi/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/fyi)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

This crate contains the objects providing the heart of the [FYI command line application](https://github.com/blobfolio/fyi), namely [`Msg`], a simple struct for status-like messages that can be easily printed to `STDOUT` or `STDERR`.



## Examples

```
use fyi_msg::{Msg, MsgKind};

// One way.
Msg::new(MsgKind::Success, "You did it!")
    .with_newline(true)
    .print();

// Another equivalent way.
Msg::success("You did it!").print();
```

For more usage examples, check out the `examples/msg` demo, which covers just about every common use case.



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

*/

#![deny(unsafe_code)]

#![warn(
	clippy::filetype_is_file,
	clippy::integer_division,
	clippy::needless_borrow,
	clippy::nursery,
	clippy::pedantic,
	clippy::perf,
	clippy::suboptimal_flops,
	clippy::unneeded_field_pattern,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
)]

#![allow(
	clippy::module_name_repetitions,
	clippy::redundant_pub_crate,
)]

#![cfg_attr(feature = "docsrs", feature(doc_cfg))]



mod msg;
#[cfg(feature = "fitted")]   mod fitted;
#[cfg(feature = "progress")] mod progress;

#[doc(hidden)]
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
};

pub use msg::{
	FLAG_INDENT,
	FLAG_NEWLINE,
	kind::MsgKind,
	Msg,
};

#[cfg(feature = "fitted")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "fitted")))]
pub use fitted::{
	length_width,
	width,
};

#[cfg(feature = "progress")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "progress")))]
pub use progress::{
	ba::BeforeAfter,
	Progless,
	error::ProglessError,
};

#[cfg(feature = "timestamps")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "timestamps")))]
pub use msg::FLAG_TIMESTAMP;

#[macro_use]
mod macros {
	#[macro_export(local_inner_macros)]
	/// # Confirm.
	///
	/// This is a convenience macro for generating a confirmation message,
	/// handling the prompting, and returning the response `bool`.
	///
	/// ## Example
	///
	/// ```no_run
	/// use fyi_msg::{confirm, Msg, MsgKind};
	///
	/// // The manual way:
	/// if Msg::new(MsgKind::Confirm, "Do you like chickens?").prompt() {
	///     println!("That's great! They like you too!");
	/// }
	///
	/// // The macro way:
	/// if confirm!("Do you like chickens?") {
	///     println!("That's great! They like you too!");
	/// }
	///
	/// // Indendation can be set with the macro too by appending a second
	/// // argument:
	/// if confirm!("Do you like chickens?", 1) {
	///     println!("    That's great! They like you too!");
	/// }
	/// ```
	macro_rules! confirm {
		($text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text).prompt()
		);
		($text:expr, $indent:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.prompt()
		);
	}
}
