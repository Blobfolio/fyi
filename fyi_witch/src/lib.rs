/*!
# FYI Witch
*/

#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]

#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]

extern crate jwalk;
extern crate nix;
extern crate rayon;
extern crate regex;
extern crate tempfile;
extern crate tempfile_fast;

mod witch;
pub mod traits;

/// The Witch!
pub use witch::Witch;
