/*!
# FYI Core: Miscellany: Paths
*/

pub mod formats;
pub mod ops;
pub mod props;
pub mod witch;



/// Walk Pattern.
///
/// This is just a convenience method, possibly mooting the need for
/// each child project to explicitly require the Regex crate.
pub fn pattern_to_regex<S> (pat: S) -> regex::Regex
where S: Into<&'static str> {
	regex::Regex::new(pat.into()).expect("Invalid pattern.")
}

