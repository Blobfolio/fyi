/*!
# FYI Msg: Traits
*/

mod ansi_code;
mod girth_ext;
mod printable;
mod strip_ansi;

pub use ansi_code::AnsiCodeBold;
pub use girth_ext::GirthExt;
pub use printable::Printable;
pub use strip_ansi::{
	StripAnsi,
	STRIPPER,
	STRIPPER_BYTES,
};
