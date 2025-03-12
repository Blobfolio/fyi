/*!
# FYI: Build
*/

#![allow(unused_mut, reason = "It is conditionally used.")]

use argyle::FlagsBuilder;
use std::{
	fs::File,
	io::Write,
	path::PathBuf,
};



/// # Build `MsgFlags`.
fn main() {
	let mut builder = FlagsBuilder::new("MsgFlags")
		.public()
		.with_docs("# Message Flags.

Used by [`Msg::with_flags`] to set multiple properties in one go.")
		.with_flag("Indent", Some("# Indent Message (four spaces).\n\nEquivalent to passing one to [`Msg::with_indent`]."))
		.with_flag("Newline", Some("# End Message w/ Line Break.\n\nEquivalent to passing true to [`Msg::with_newline`]."));

	#[cfg(feature = "timestamps")]
	{
		builder = builder.with_flag("Timestamp", Some("# Timestamp Message.\n\nEquivalent to passing true to [`Msg::with_timestamp`]."));
	}

	// Save it manually so we can note the timestamps feature-gate.
	let out = builder.to_string()
		.replace(
			"\t#[doc = \"# Timestamp",
			"\t#[cfg_attr(docsrs, doc(cfg(feature = \"timestamps\")))]\n\t#[doc = \"# Timestamp",
		);

	File::create(out_path("msg-flags.rs"))
		.and_then(|mut f| f.write_all(out.as_bytes()).and_then(|()| f.flush()))
		.expect("Unable to save msg-flags.rs");
}

/// # Output Path.
///
/// Append the sub-path to OUT_DIR and return it.
fn out_path(stub: &str) -> PathBuf {
	std::fs::canonicalize(std::env::var("OUT_DIR").expect("Missing OUT_DIR."))
		.expect("Missing OUT_DIR.")
		.join(stub)
}
