/*!
# FYI Msg: Utility Methods
*/



#[allow(clippy::too_many_lines)] // There's a lot of values!
#[must_use]
/// Return Escape Sequence.
///
/// Convert a `u8` number into its equivalent byte string ANSI sequence.
///
/// It is worth noting that unfortunately BASH, et al, work on a 1-256 scale
/// instead of the more typical 0-255 that Rust's `u8` works on. Rather than
/// complicate the logic or require people to mentally subtract one from the
/// value they want, #256 is unsupported.
///
/// # Examples
///
/// ```
/// assert_eq!(fyi_msg::utility::ansi_code_bold(199), b"\x1B[1;38;5;199m");
/// ```
pub fn ansi_code_bold(num: u8) -> &'static [u8] {
	static ANSI: [&[u8]; 256] = [
		b"\x1b[0m",
		b"\x1b[1;38;5;1m",
		b"\x1b[1;38;5;2m",
		b"\x1b[1;38;5;3m",
		b"\x1b[1;38;5;4m",
		b"\x1b[1;38;5;5m",
		b"\x1b[1;38;5;6m",
		b"\x1b[1;38;5;7m",
		b"\x1b[1;38;5;8m",
		b"\x1b[1;38;5;9m",
		b"\x1b[1;38;5;10m",
		b"\x1b[1;38;5;11m",
		b"\x1b[1;38;5;12m",
		b"\x1b[1;38;5;13m",
		b"\x1b[1;38;5;14m",
		b"\x1b[1;38;5;15m",
		b"\x1b[1;38;5;16m",
		b"\x1b[1;38;5;17m",
		b"\x1b[1;38;5;18m",
		b"\x1b[1;38;5;19m",
		b"\x1b[1;38;5;20m",
		b"\x1b[1;38;5;21m",
		b"\x1b[1;38;5;22m",
		b"\x1b[1;38;5;23m",
		b"\x1b[1;38;5;24m",
		b"\x1b[1;38;5;25m",
		b"\x1b[1;38;5;26m",
		b"\x1b[1;38;5;27m",
		b"\x1b[1;38;5;28m",
		b"\x1b[1;38;5;29m",
		b"\x1b[1;38;5;30m",
		b"\x1b[1;38;5;31m",
		b"\x1b[1;38;5;32m",
		b"\x1b[1;38;5;33m",
		b"\x1b[1;38;5;34m",
		b"\x1b[1;38;5;35m",
		b"\x1b[1;38;5;36m",
		b"\x1b[1;38;5;37m",
		b"\x1b[1;38;5;38m",
		b"\x1b[1;38;5;39m",
		b"\x1b[1;38;5;40m",
		b"\x1b[1;38;5;41m",
		b"\x1b[1;38;5;42m",
		b"\x1b[1;38;5;43m",
		b"\x1b[1;38;5;44m",
		b"\x1b[1;38;5;45m",
		b"\x1b[1;38;5;46m",
		b"\x1b[1;38;5;47m",
		b"\x1b[1;38;5;48m",
		b"\x1b[1;38;5;49m",
		b"\x1b[1;38;5;50m",
		b"\x1b[1;38;5;51m",
		b"\x1b[1;38;5;52m",
		b"\x1b[1;38;5;53m",
		b"\x1b[1;38;5;54m",
		b"\x1b[1;38;5;55m",
		b"\x1b[1;38;5;56m",
		b"\x1b[1;38;5;57m",
		b"\x1b[1;38;5;58m",
		b"\x1b[1;38;5;59m",
		b"\x1b[1;38;5;60m",
		b"\x1b[1;38;5;61m",
		b"\x1b[1;38;5;62m",
		b"\x1b[1;38;5;63m",
		b"\x1b[1;38;5;64m",
		b"\x1b[1;38;5;65m",
		b"\x1b[1;38;5;66m",
		b"\x1b[1;38;5;67m",
		b"\x1b[1;38;5;68m",
		b"\x1b[1;38;5;69m",
		b"\x1b[1;38;5;70m",
		b"\x1b[1;38;5;71m",
		b"\x1b[1;38;5;72m",
		b"\x1b[1;38;5;73m",
		b"\x1b[1;38;5;74m",
		b"\x1b[1;38;5;75m",
		b"\x1b[1;38;5;76m",
		b"\x1b[1;38;5;77m",
		b"\x1b[1;38;5;78m",
		b"\x1b[1;38;5;79m",
		b"\x1b[1;38;5;80m",
		b"\x1b[1;38;5;81m",
		b"\x1b[1;38;5;82m",
		b"\x1b[1;38;5;83m",
		b"\x1b[1;38;5;84m",
		b"\x1b[1;38;5;85m",
		b"\x1b[1;38;5;86m",
		b"\x1b[1;38;5;87m",
		b"\x1b[1;38;5;88m",
		b"\x1b[1;38;5;89m",
		b"\x1b[1;38;5;90m",
		b"\x1b[1;38;5;91m",
		b"\x1b[1;38;5;92m",
		b"\x1b[1;38;5;93m",
		b"\x1b[1;38;5;94m",
		b"\x1b[1;38;5;95m",
		b"\x1b[1;38;5;96m",
		b"\x1b[1;38;5;97m",
		b"\x1b[1;38;5;98m",
		b"\x1b[1;38;5;99m",
		b"\x1b[1;38;5;100m",
		b"\x1b[1;38;5;101m",
		b"\x1b[1;38;5;102m",
		b"\x1b[1;38;5;103m",
		b"\x1b[1;38;5;104m",
		b"\x1b[1;38;5;105m",
		b"\x1b[1;38;5;106m",
		b"\x1b[1;38;5;107m",
		b"\x1b[1;38;5;108m",
		b"\x1b[1;38;5;109m",
		b"\x1b[1;38;5;110m",
		b"\x1b[1;38;5;111m",
		b"\x1b[1;38;5;112m",
		b"\x1b[1;38;5;113m",
		b"\x1b[1;38;5;114m",
		b"\x1b[1;38;5;115m",
		b"\x1b[1;38;5;116m",
		b"\x1b[1;38;5;117m",
		b"\x1b[1;38;5;118m",
		b"\x1b[1;38;5;119m",
		b"\x1b[1;38;5;120m",
		b"\x1b[1;38;5;121m",
		b"\x1b[1;38;5;122m",
		b"\x1b[1;38;5;123m",
		b"\x1b[1;38;5;124m",
		b"\x1b[1;38;5;125m",
		b"\x1b[1;38;5;126m",
		b"\x1b[1;38;5;127m",
		b"\x1b[1;38;5;128m",
		b"\x1b[1;38;5;129m",
		b"\x1b[1;38;5;130m",
		b"\x1b[1;38;5;131m",
		b"\x1b[1;38;5;132m",
		b"\x1b[1;38;5;133m",
		b"\x1b[1;38;5;134m",
		b"\x1b[1;38;5;135m",
		b"\x1b[1;38;5;136m",
		b"\x1b[1;38;5;137m",
		b"\x1b[1;38;5;138m",
		b"\x1b[1;38;5;139m",
		b"\x1b[1;38;5;140m",
		b"\x1b[1;38;5;141m",
		b"\x1b[1;38;5;142m",
		b"\x1b[1;38;5;143m",
		b"\x1b[1;38;5;144m",
		b"\x1b[1;38;5;145m",
		b"\x1b[1;38;5;146m",
		b"\x1b[1;38;5;147m",
		b"\x1b[1;38;5;148m",
		b"\x1b[1;38;5;149m",
		b"\x1b[1;38;5;150m",
		b"\x1b[1;38;5;151m",
		b"\x1b[1;38;5;152m",
		b"\x1b[1;38;5;153m",
		b"\x1b[1;38;5;154m",
		b"\x1b[1;38;5;155m",
		b"\x1b[1;38;5;156m",
		b"\x1b[1;38;5;157m",
		b"\x1b[1;38;5;158m",
		b"\x1b[1;38;5;159m",
		b"\x1b[1;38;5;160m",
		b"\x1b[1;38;5;161m",
		b"\x1b[1;38;5;162m",
		b"\x1b[1;38;5;163m",
		b"\x1b[1;38;5;164m",
		b"\x1b[1;38;5;165m",
		b"\x1b[1;38;5;166m",
		b"\x1b[1;38;5;167m",
		b"\x1b[1;38;5;168m",
		b"\x1b[1;38;5;169m",
		b"\x1b[1;38;5;170m",
		b"\x1b[1;38;5;171m",
		b"\x1b[1;38;5;172m",
		b"\x1b[1;38;5;173m",
		b"\x1b[1;38;5;174m",
		b"\x1b[1;38;5;175m",
		b"\x1b[1;38;5;176m",
		b"\x1b[1;38;5;177m",
		b"\x1b[1;38;5;178m",
		b"\x1b[1;38;5;179m",
		b"\x1b[1;38;5;180m",
		b"\x1b[1;38;5;181m",
		b"\x1b[1;38;5;182m",
		b"\x1b[1;38;5;183m",
		b"\x1b[1;38;5;184m",
		b"\x1b[1;38;5;185m",
		b"\x1b[1;38;5;186m",
		b"\x1b[1;38;5;187m",
		b"\x1b[1;38;5;188m",
		b"\x1b[1;38;5;189m",
		b"\x1b[1;38;5;190m",
		b"\x1b[1;38;5;191m",
		b"\x1b[1;38;5;192m",
		b"\x1b[1;38;5;193m",
		b"\x1b[1;38;5;194m",
		b"\x1b[1;38;5;195m",
		b"\x1b[1;38;5;196m",
		b"\x1b[1;38;5;197m",
		b"\x1b[1;38;5;198m",
		b"\x1b[1;38;5;199m",
		b"\x1b[1;38;5;200m",
		b"\x1b[1;38;5;201m",
		b"\x1b[1;38;5;202m",
		b"\x1b[1;38;5;203m",
		b"\x1b[1;38;5;204m",
		b"\x1b[1;38;5;205m",
		b"\x1b[1;38;5;206m",
		b"\x1b[1;38;5;207m",
		b"\x1b[1;38;5;208m",
		b"\x1b[1;38;5;209m",
		b"\x1b[1;38;5;210m",
		b"\x1b[1;38;5;211m",
		b"\x1b[1;38;5;212m",
		b"\x1b[1;38;5;213m",
		b"\x1b[1;38;5;214m",
		b"\x1b[1;38;5;215m",
		b"\x1b[1;38;5;216m",
		b"\x1b[1;38;5;217m",
		b"\x1b[1;38;5;218m",
		b"\x1b[1;38;5;219m",
		b"\x1b[1;38;5;220m",
		b"\x1b[1;38;5;221m",
		b"\x1b[1;38;5;222m",
		b"\x1b[1;38;5;223m",
		b"\x1b[1;38;5;224m",
		b"\x1b[1;38;5;225m",
		b"\x1b[1;38;5;226m",
		b"\x1b[1;38;5;227m",
		b"\x1b[1;38;5;228m",
		b"\x1b[1;38;5;229m",
		b"\x1b[1;38;5;230m",
		b"\x1b[1;38;5;231m",
		b"\x1b[1;38;5;232m",
		b"\x1b[1;38;5;233m",
		b"\x1b[1;38;5;234m",
		b"\x1b[1;38;5;235m",
		b"\x1b[1;38;5;236m",
		b"\x1b[1;38;5;237m",
		b"\x1b[1;38;5;238m",
		b"\x1b[1;38;5;239m",
		b"\x1b[1;38;5;240m",
		b"\x1b[1;38;5;241m",
		b"\x1b[1;38;5;242m",
		b"\x1b[1;38;5;243m",
		b"\x1b[1;38;5;244m",
		b"\x1b[1;38;5;245m",
		b"\x1b[1;38;5;246m",
		b"\x1b[1;38;5;247m",
		b"\x1b[1;38;5;248m",
		b"\x1b[1;38;5;249m",
		b"\x1b[1;38;5;250m",
		b"\x1b[1;38;5;251m",
		b"\x1b[1;38;5;252m",
		b"\x1b[1;38;5;253m",
		b"\x1b[1;38;5;254m",
		b"\x1b[1;38;5;255m",
	];

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
///
/// # Examples
///
/// ```
/// assert_eq!(fyi_msg::utility::time_format_dd(3), b"03");
/// assert_eq!(fyi_msg::utility::time_format_dd(10), b"10");
/// ```
pub fn time_format_dd(num: u32) -> &'static [u8] {
	static TIME: [&[u8]; 60] = [
		b"00",
		b"01",
		b"02",
		b"03",
		b"04",
		b"05",
		b"06",
		b"07",
		b"08",
		b"09",
		b"10",
		b"11",
		b"12",
		b"13",
		b"14",
		b"15",
		b"16",
		b"17",
		b"18",
		b"19",
		b"20",
		b"21",
		b"22",
		b"23",
		b"24",
		b"25",
		b"26",
		b"27",
		b"28",
		b"29",
		b"30",
		b"31",
		b"32",
		b"33",
		b"34",
		b"35",
		b"36",
		b"37",
		b"38",
		b"39",
		b"40",
		b"41",
		b"42",
		b"43",
		b"44",
		b"45",
		b"46",
		b"47",
		b"48",
		b"49",
		b"50",
		b"51",
		b"52",
		b"53",
		b"54",
		b"55",
		b"56",
		b"57",
		b"58",
		b"59",
	];

	TIME[59.min(num as usize)]
}

#[must_use]
/// Whitespace Reservoir.
///
/// This method borrows whitespace from a static reference, useful for
/// quickly padding strings, etc.
///
/// Let's don't get crazy, though. A maximum of 255 spaces can be returned.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: [u8; 255] = [32; 255];
	&WHITES[0..255.min(num)]
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn t_ansi_code_bold() {
		for i in 1..=255 {
			assert_eq!(
				ansi_code_bold(i),
				format!("\x1B[1;38;5;{}m", i).as_bytes(),
				"Ansi for {} is incorrect: {:?}",
				i,
				ansi_code_bold(i),
			);
		}
	}

	#[test]
	fn t_time_format_dd() {
		// Test the supported values.
		for i in 0..=59 {
			assert_eq!(
				time_format_dd(i),
				format!("{:02}", i).as_bytes(),
				"DD for {} is incorrect: {:?}",
				i,
				time_format_dd(i)
			);
		}

		// And make sure overflow works.
		assert_eq!(
			time_format_dd(60),
			time_format_dd(59),
			"DD for 60 is incorrect: {:?}",
			time_format_dd(60)
		);
	}

	#[test]
	fn t_whitespace() {
		assert_eq!(whitespace(0), b"");
		assert_eq!(whitespace(5), b"     ");
		assert_eq!(whitespace(10), b"          ");
	}
}
