/*!
# FYI ANSI

[![docs.rs](https://img.shields.io/docsrs/fyi_ansi.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_ansi/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_ansi.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_ansi)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/crate/fyi_ansi/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/fyi_ansi/)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

This crate exports two simple compile-time ANSI formatting macros — [`csi`] and
[`ansi`] — as well as shortcut helpers for [`blink`], [`bold`], [`dim`],
[`italic`], [`strike`], and [`underline`].

## Examples

```
use fyi_ansi::{ansi, csi};

// The `csi` macro generates (only) the ANSI formatting sequence.
assert_eq!(csi!(bold, underline), "\x1b[1;4m");

// The `ansi` macro generates formatted content strings.
assert_eq!(
    concat!(ansi!((bold, light_red) "Error:"), " Oh no!"),
    "\x1b[1;91mError:\x1b[0m Oh no!",
);
```

The [`bold`], [`dim`], etc., macros are only shortcuts, but can help declutter
your code when there's only the one style being toggled.

```
use fyi_ansi::{blink, bold, dim, italic, strike, underline};

// Same as with `ansi`, they terminate with a blanket reset by default.
assert_eq!(bold!("I'm bold!"),            "\x1b[1mI'm bold!\x1b[0m");
assert_eq!(dim!("I'm dim!"),              "\x1b[2mI'm dim!\x1b[0m");
assert_eq!(italic!("I'm italic!"),        "\x1b[3mI'm italic!\x1b[0m");
assert_eq!(underline!("I'm underlined!"), "\x1b[4mI'm underlined!\x1b[0m");
assert_eq!(blink!("I'm annoying!"),       "\x1b[5mI'm annoying!\x1b[0m");
assert_eq!(strike!("I'm struck!"),        "\x1b[9mI'm struck!\x1b[0m");
```
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



#[macro_use]
/// # Macros.
mod macros {
	// The `csi_code`macro is generated by build.rs to ensure nothing's missed.
	include!(concat!(env!("OUT_DIR"), "/codes.rs"));

	#[macro_export(local_inner_macros)]
	#[doc(hidden)]
	/// # ANSI Codes: Merge.
	macro_rules! ansi_merge {
		// ! is annoyingly its own tt so requires manual matching/regluing.
		(! $last:tt) => ( csi_code!(!$last) );
		(! $next:tt, $($tail:tt)+) => (
			::std::concat!(csi_code!(!$next), ";", ansi_merge!($($tail)+))
		);

		// All but the last style get a trailing ;
		($last:tt) => ( csi_code!($last) );
		($next:tt, $($tail:tt)+) => (
			::std::concat!(csi_code!($next), ";", ansi_merge!($($tail)+))
		);
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI CSI Sequence.
	///
	/// This compile-time macro generates static ANSI [CSI](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences)
	/// sequences from a list of human-friendly keyword(s).
	///
	/// It supports all combinations of `blink`, `bold`, `dim`, `italic`,
	/// `strike`, `underline`, and foreground (text) color — by ANSI-256 number
	/// (`u8::MIN..=u8::MAX`) or (snake_case) name[^names] — as well as their
	/// equal and opposite resets[^reset] (`!blink`, `!bold`, etc.).
	///
	/// It does _not_ currently support background color.
	///
	/// [^names]: For whatever reason, consistent/unique naming was not a
	/// priority when fleshing out the standard, so only some colors can be
	/// toggled by name. The rest are only accessible by number.
	/// [^reset]: The `reset` keyword can be used to reset all styles at once.
	/// This should generally be preferred over per-property resets like `!bold`
	/// as support is near-universal, plus it's smaller.
	/// [^reset_fg]: All color resets are the same so a generic `!fg` is
	/// provided, but individual `!1`, `!red`, etc., keywords are also
	/// supported for consistency.
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::csi;
	///
	/// // Bold/unbold.
	/// assert_eq!(csi!(bold),       "\x1b[1m");
	/// assert_eq!(csi!(!bold),      "\x1b[21m");
	///
	/// // Dim/undim.
	/// assert_eq!(csi!(dim),        "\x1b[2m");
	/// assert_eq!(csi!(!dim),       "\x1b[22m");
	///
	/// // Underline/un-underline.
	/// assert_eq!(csi!(underline),  "\x1b[4m");
	/// assert_eq!(csi!(!underline), "\x1b[24m");
	///
	/// // Red/Un-red:
	/// assert_eq!(csi!(9),          "\x1b[91m"); // Number.
	/// assert_eq!(csi!(light_red),  "\x1b[91m"); // Name.
	/// assert_eq!(csi!(!9),         "\x1b[39m"); // Number.
	/// assert_eq!(csi!(!light_red), "\x1b[39m"); // Name.
	///
	/// // Comma-separated combinations are merged together into a single
	/// // sequence (with ordering preserved).
	/// assert_eq!(csi!(reset, bold, 9),   "\x1b[0;1;91m");
	/// assert_eq!(csi!(bold, !dim, 9),    "\x1b[1;22;91m");
	///
	/// // To fully restore the terminal to its default styles, use the `reset`
	/// // keyword, or call `csi!()` with no arguments.
	/// assert_eq!(csi!(reset), csi!()); // \x1b[0m
	/// ```
	///
	/// The `csi` macro will always output (technically) valid ANSI sequences,
	/// but it lacks the Big Picture view necessary to detect or clean up
	/// nonsensical combinations.
	///
	/// Put another way, it cannot save you from yourself.
	///
	/// ```
	/// use fyi_ansi::csi;
	///
	/// // There's no reason to do this…
	/// assert_eq!(csi!(bold, bold, bold), "\x1b[1;1;1m");
	/// //                    ^     ^              ^ ^ pointless
	///
	/// // Or this…
	/// assert_eq!(csi!(bold, !bold), "\x1b[1;21m");
	/// //              ^                   ^ pointless
	/// ```
	///
	/// ## ANSI 8/16/256 Color Reference
	///
	/// The actual color rendered will vary by software, but should look
	/// something like the following:
	///
	#[doc=include_str!(concat!(env!("OUT_DIR"), "/color-table.rs"))]
	macro_rules! csi {
		// Straight map.
		($($tail:tt)+) => (
			::std::concat!("\x1b[", ansi_merge!($($tail)+), "m")
		);

		// Generic (full) reset.
		() => ( "\x1b[0m" );
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Wrapper.
	///
	/// This compile-time macro can be used to generate static ANSI-formatted
	/// string slices from arbitrary [`concat`]able content.
	///
	/// For a breakdown of the supported styling options, refer to the
	/// documentation for the [`csi`] macro.
	///
	/// Styles work the same way here as there, except they need to be wrapped
	/// in parentheses — `(style1, styleN…)` — to isolate them from the
	/// content argument(s).
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, csi};
	///
	/// // Bold and pink.
	/// assert_eq!(
	///     ansi!((bold, 199) "Hello World"),
	///     "\x1b[1;38;5;199mHello World\x1b[0m",
	///                              // ^ reset *all* styles to default
	/// );
	///
	/// // The `ansi` macro supports multiple contents, so if you need a
	/// // costume change mid-string, chuck a `csi` in the middle.
	/// assert_eq!(
	///     ansi!((bold, 199) "Hello ", csi!(light_cyan), "World"),
	///     "\x1b[1;38;5;199mHello \x1b[96mWorld\x1b[0m",
	/// //                   ^bold/pink    ^bold/cyan  ^default->
	/// );
	/// ```
	///
	/// Nested usage is technically possible, but not recommended as there is
	/// no way to prevent inner styles from overriding outer ones.
	///
	/// ```
	/// use fyi_ansi::{ansi, csi};
	///
	/// // The blanket reset from the inner ansi!() causes that last period
	/// // to render with default styles, not the bold and red the outer
	/// // ansi!() wanted.
	/// assert_eq!(
	///     ansi!(
	///         (bold, red)
	///         "It's okay to be ",
	///         ansi!((white) "different"),
	///         ".",
	///     ),
	///     "\x1b[1;31mIt's okay to be \x1b[97mdifferent\x1b[0m.\x1b[0m",
	/// //             ^-bold--------------------------^
	/// //             ^-red---------^         ^-white-^       ^ default
	/// );
	///
	/// // Better to nest individual `csi` sequences instead, since they don't
	/// // make any assumptions about what comes after.
	/// assert_eq!(
	///     ansi!(
	///         (bold, red)
	///         "It's okay to be ",
	///         csi!(white),
	///         "different",
	///         csi!(red),
	///         ".",
	///     ),
	///     "\x1b[1;31mIt's okay to be \x1b[97mdifferent\x1b[31m.\x1b[0m",
	/// //             ^-bold-----------------------------------^
	/// //             ^-red---------^         ^-white-^        ^ red
	/// );
	/// ```
	macro_rules! ansi {
		// Close with a generic (full) reset.
		(($($style:tt)+) $($content:expr),+ $(,)?) => (
			::std::concat!(
				csi!($($style)+),
				$($content),+,
				csi!(),
			)
		);
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Blink Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((blink) "…")`,
	/// useful in cases where the only style being overridden is blinkness.
	///
	/// Note that this attribute isn't as well supported as `bold`/`dim`.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, blink};
	///
	/// // Blink, then full reset.
	/// assert_eq!(
	///     blink!("Annoying."),
	///     "\x1b[5mAnnoying.\x1b[0m",
	///                   // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(blink!("ABC"), ansi!((blink) "ABC"));
	/// ```
	macro_rules! blink {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((blink) $($content),+));
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Bold Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((bold) "…")`,
	/// useful in cases where the only style being overridden is boldness.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, bold};
	///
	/// // Bold, then full reset.
	/// assert_eq!(
	///     bold!("Valiente."),
	///     "\x1b[1mValiente.\x1b[0m",
	///                   // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(bold!("ABC"), ansi!((bold) "ABC"));
	/// ```
	macro_rules! bold {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((bold) $($content),+));
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Dim Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((dim) "…")`,
	/// useful in cases where the only style being overridden is dimness.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, dim};
	///
	/// // Dim, then full reset.
	/// assert_eq!(
	///     dim!("Not bright."),
	///     "\x1b[2mNot bright.\x1b[0m",
	///                     // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(dim!("ABC"), ansi!((dim) "ABC"));
	/// ```
	macro_rules! dim {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((dim) $($content),+));
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Italic Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((italic) "…")`,
	/// useful in cases where the only style being overridden is italicness.
	///
	/// Note that this attribute isn't as well supported as `bold`/`dim`.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, italic};
	///
	/// // Italic, then full reset.
	/// assert_eq!(
	///     italic!("Lean In."),
	///     "\x1b[3mLean In.\x1b[0m",
	///                  // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(italic!("ABC"), ansi!((italic) "ABC"));
	/// ```
	macro_rules! italic {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((italic) $($content),+));
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Strike Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((strike) "…")`,
	/// useful in cases where the only style being overridden is
	/// strike-out-ness.
	///
	/// Note that this attribute isn't as well supported as `bold`/`dim`.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, strike};
	///
	/// // Strike, then full reset.
	/// assert_eq!(
	///     strike!("Buy cereal."),
	///     "\x1b[9mBuy cereal.\x1b[0m",
	///                     // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(strike!("ABC"), ansi!((strike) "ABC"));
	/// ```
	macro_rules! strike {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((strike) $($content),+));
	}

	#[macro_export(local_inner_macros)]
	/// # ANSI Underline Wrapper.
	///
	/// This macro serves as a shortcut for calling `ansi((underline) "…")`,
	/// useful in cases where the only style being overridden is underlineness.
	///
	/// For more complex styling options, see [`ansi`].
	///
	/// ## Examples
	///
	/// ```
	/// use fyi_ansi::{ansi, underline};
	///
	/// // Underline, then full reset.
	/// assert_eq!(
	///     underline!("Emphasized."),
	///     "\x1b[4mEmphasized.\x1b[0m",
	///                     // ^ reset *all* styles to default
	/// );
	///
	/// // The same results can be obtained via `ansi`.
	/// assert_eq!(underline!("ABC"), ansi!((underline) "ABC"));
	/// ```
	macro_rules! underline {
		// Close with a full reset.
		($($content:expr),+ $(,)?) => (ansi!((underline) $($content),+));
	}
}
