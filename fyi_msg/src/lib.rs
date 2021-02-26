/*!
# FYI Msg

This crate contains the objects providing the heart of the FYI command line
application, namely [`Msg`], a simple struct for status-like messages that can be
easily printed to `STDOUT` or `STDERR`.



## Examples

```no_run
use fyi_msg::{Msg, MsgKind};

// One way.
Msg::new(MsgKind::Success, "You did it!")
    .with_newline(true)
    .print();

// Another equivalent way.
Msg::success("You did it!").print();
```

For more usage examples, check out the `examples/msg` demo, which covers just
about every common use case.



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

#![allow(clippy::module_name_repetitions)] // This is fine.



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
pub use fitted::{
	length_width,
	width,
};

#[cfg(feature = "progress")]   pub use progress::Progless;
#[cfg(feature = "timestamps")] pub use msg::FLAG_TIMESTAMP;

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
	macro_rules! confirm {
		($text:expr) => (
			$crate::Msg::new($crate::MsgKind::Confirm, $text).prompt()
		);
	}
}
