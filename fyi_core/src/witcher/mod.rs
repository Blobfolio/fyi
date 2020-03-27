/*!
# FYI Core: Miscellany: Paths
*/

pub mod formats;
pub mod mass;
pub mod ops;
pub mod props;



/// Walk Pattern.
///
/// This is just a convenience method, negating the need to add Regex
/// deps directly to projects using this library.
pub fn pattern_to_regex<S> (pat: S) -> regex::Regex
where S: Into<&'static str> {
	regex::Regex::new(pat.into()).expect("Invalid pattern.")
}
