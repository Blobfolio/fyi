mod ansi_bitsy;
mod inflection;
mod mebi;
mod oxford_join;
mod shorty;
mod spacetime;
mod thousands;

#[cfg(feature = "witcher")]
mod io;

/// ANSI and display.
pub use self::ansi_bitsy::AnsiBitsy;
/// Mebibyte love.
pub use self::mebi::{MebiSaved, ToMebi};
/// Inflection.
pub use self::inflection::Inflection;
/// Oxford Join.
pub use self::oxford_join::{OxfordGlue, OxfordJoin};
/// String shortening.
pub use self::shorty::Shorty;
/// Human-readable elapsed time.
pub use self::spacetime::{
	AbsPath,
	Elapsed,
	PathProps,
};
/// Numbers with thousands separation.
pub use self::thousands::Thousands;

#[cfg(feature = "witcher")]
/// IO traits.
pub use self::io::FYIPathIO;
