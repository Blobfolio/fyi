/*!
# FYI Msg

[![docs.rs](https://img.shields.io/docsrs/fyi_msg.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_msg/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_msg.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_msg)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/crate/fyi_msg/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/fyi_msg/)<br>
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

#![cfg_attr(docsrs, feature(doc_cfg))]



mod ansi;
mod msg;
#[cfg(feature = "fitted")]   mod fitted;
#[cfg(feature = "progress")] mod progress;

pub use ansi::AnsiColor;
pub use msg::{
	Msg,
	kind::MsgKind,
};

#[cfg(feature = "fitted")]
#[cfg_attr(docsrs, doc(cfg(feature = "fitted")))]
pub use fitted::{
	fit_to_width,
	length_width,
	width,
};

#[cfg(feature = "progress")]
#[cfg_attr(docsrs, doc(cfg(feature = "progress")))]
pub use progress::{
	ba::BeforeAfter,
	Progless,
	error::ProglessError,
	guard::ProglessTaskGuard,
};

// Re-export.
pub use fyi_ansi;
#[cfg_attr(docsrs, doc(cfg(feature = "signal-hook")))]
#[cfg(feature = "signal-hook")] pub use signal_hook;

#[cfg(test)] use brunch as _;
#[cfg(test)] use rayon as _;

#[macro_use]
/// # Macros.
mod macros {
	#[macro_export(local_inner_macros)]
	/// # Confirm.
	///
	/// This convenience macro prints a message, prompts the user for a
	/// yes/no response, and interprets/returns that value as a `bool`.
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
	/// ```
	///
	/// The following modifiers are supported:
	/// * `@indent $literal`: indent the message `$literal` "tabs";
	/// * `@stderr`: pop the question over STDERR (instead of STDOUT);
	/// * `@yes`: default to "Y" (instead of "N");
	///
	/// ```no_run
	/// # use fyi_msg::confirm;
	/// // Indent one "tabs" (four spaces):
	/// if confirm!(@indent 1 "Do you like chickens?") {
	///     println!("    That's great! They like you too!");
	/// }
	///
	/// // Print to STDERR instead of STDOUT:
	/// if confirm!(@stderr "Do you like chickens?") {
	///     println!("That's great! They like you too!");
	/// }
	///
	/// // Default to yes instead of no.
	/// if confirm!(@yes "Do you like chickens?") {
	///     println!("That's great! They like you too!");
	/// }
	/// ```
	///
	/// Modifiers can be stacked together any which way.
	///
	/// ```no_run
	/// # use fyi_msg::confirm;
	/// // Indent three "tabs" _and_ print to STDERR _and_ default to yes:
	/// if confirm!(@indent 3 @stderr @yes "Do you like chickens?") {
	///     println!("            That's great! They like you too!");
	/// }
	///
	/// // Same as above.
	/// if confirm!(@stderr @indent 3 @yes "Do you like chickens?") {
	///     println!("            That's great! They like you too!");
	/// }
	///
	/// // Same again.
	/// if confirm!(@stderr @yes @indent 3 "Do you like chickens?") {
	///     println!("            That's great! They like you too!");
	/// }
	///
	/// // …
	/// ```
	macro_rules! confirm {
		// Maybe-indent.
		($( @indent $indent:literal )? $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				$( .with_indent($indent) )?
				.prompt()
		);

		// Maybe-indent, yes.
		($( @indent $indent:literal )? @yes $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				$( .with_indent($indent) )?
				.prompt_with_default(true)
		);
		// Maybe-indent, yes.
		(@yes $( @indent $indent:literal )? $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				$( .with_indent($indent) )?
				.prompt_with_default(true)
		);

		// Maybe-indent, STDERR.
		($( @indent $indent:literal )? @stderr $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				$( .with_indent($indent) )?
				.eprompt()
		);
		(@stderr $( @indent $indent:literal )? $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				$( .with_indent($indent) )?
				.eprompt()
		);

		// STDERR, yes.
		(@stderr @yes $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.eprompt_with_default(true)
		);
		(@yes @stderr $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.eprompt_with_default(true)
		);

		// Indent, STDERR, yes.
		(@indent $indent:literal @stderr @yes $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
		(@indent $indent:literal @yes @stderr $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
		(@stderr @indent $indent:literal @yes $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
		(@stderr @yes @indent $indent:literal $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
		(@yes @indent $indent:literal @stderr $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
		(@yes @stderr @indent $indent:literal $text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text)
				.with_indent($indent)
				.eprompt_with_default(true)
		);
	}
}
