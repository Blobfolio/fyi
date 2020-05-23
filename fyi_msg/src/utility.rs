/*!
# FYI Message: Utility

This mod contains miscellaneous utility functions for the crate.
*/

#[must_use]
/// Return escape sequence.
///
/// Convert a `u8` number into its equivalent byte string ANSI sequence.
///
/// It is worth noting that unfortunately BASH, et al, work on a 1-256 scale
/// instead of the more typical 0-255 that Rust's `u8` works on. Rather than
/// complicate the logic or require people to mentally subtract one from the
/// value they want, #256 is unsupported, and #0 is an empty value.
pub fn ansi_code_bold(num: u8) -> &'static [u8] {
	static ANSI: &[&[u8]] = &[&[], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 51, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 52, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 53, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 54, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 55, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 56, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 57, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 48, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 49, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 50, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 51, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 52, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 53, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 54, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 55, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 56, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 49, 57, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 48, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 49, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 50, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 51, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 53, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 54, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 55, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 56, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 52, 57, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 48, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 49, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 50, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 51, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 52, 109], &[27, 91, 49, 59, 51, 56, 59, 53, 59, 50, 53, 53, 109]];
	ANSI[num as usize]
}

#[must_use]
/// Time Number to String.
///
/// This is a simple conversion table for turning `u32` representations of
/// numbers 0–59 into double-digit strings like "00" and "59". It is faster
/// having these ready than trying to `itoa::write` them on-the-fly.
///
/// Out of range always comes back as "59".
pub fn time_format_dd(num: u32) -> &'static [u8] {
	static TIME: &[&[u8]] = &[&[48, 48], &[48, 49], &[48, 50], &[48, 51], &[48, 52], &[48, 53], &[48, 54], &[48, 55], &[48, 56], &[48, 57], &[49, 48], &[49, 49], &[49, 50], &[49, 51], &[49, 52], &[49, 53], &[49, 54], &[49, 55], &[49, 56], &[49, 57], &[50, 48], &[50, 49], &[50, 50], &[50, 51], &[50, 52], &[50, 53], &[50, 54], &[50, 55], &[50, 56], &[50, 57], &[51, 48], &[51, 49], &[51, 50], &[51, 51], &[51, 52], &[51, 53], &[51, 54], &[51, 55], &[51, 56], &[51, 57], &[52, 48], &[52, 49], &[52, 50], &[52, 51], &[52, 52], &[52, 53], &[52, 54], &[52, 55], &[52, 56], &[52, 57], &[53, 48], &[53, 49], &[53, 50], &[53, 51], &[53, 52], &[53, 53], &[53, 54], &[53, 55], &[53, 56], &[53, 57]];
	TIME[usize::min(59, num as usize)]
}

#[must_use]
/// Whitespace maker.
///
/// This method borrows whitespace from a static reference, useful for
/// quickly padding strings, etc.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: &[u8] = &[32; 255];
	&WHITES[0..usize::min(255, num)]
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ansi_code_bold() {
		_ansi_code_bold(0, "");
		_ansi_code_bold(1, "\x1B[1;38;5;1m");
		_ansi_code_bold(9, "\x1B[1;38;5;9m");
		_ansi_code_bold(50, "\x1B[1;38;5;50m");
		_ansi_code_bold(100, "\x1B[1;38;5;100m");
	}

	fn _ansi_code_bold(num: u8, expected: &str) {
		assert_eq!(
			ansi_code_bold(num),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected.as_bytes()
		);
	}

	#[test]
	fn t_time_format_dd() {
		_time_format_dd(0, "00");
		_time_format_dd(1, "01");
		_time_format_dd(9, "09");
		_time_format_dd(50, "50");
		_time_format_dd(60, "59");
	}

	fn _time_format_dd(num: u32, expected: &str) {
		assert_eq!(
			time_format_dd(num),
			expected.as_bytes(),
			"{} should be equivalent to {:?}",
			num,
			expected
		);
	}

	#[test]
	fn t_whitespace() {
		assert_eq!(whitespace(0), b"");
		assert_eq!(whitespace(5), b"     ");
		assert_eq!(whitespace(10), b"          ");
	}
}
