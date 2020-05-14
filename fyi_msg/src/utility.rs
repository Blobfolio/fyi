/*!
# FYI Message: Utility

This mod contains miscellaneous utility functions for the crate.
*/

/// Re-export these helpers.
pub use crate::print::{
	print,
	print_to,
};

#[cfg(feature = "interactive")]
pub use crate::print::prompt;

#[must_use]
#[allow(clippy::too_many_lines)]
/// Return escape sequence.
///
/// Convert a `u8` number into its equivalent byte string ANSI sequence.
///
/// It is worth noting that unfortunately BASH, et al, work on a 1-256 scale
/// instead of the more typical 0-255 that Rust's `u8` works on. Rather than
/// complicate the logic or require people to mentally subtract one from the
/// value they want, #256 is unsupported, and #0 is an empty value.
pub fn ansi_code_bold(num: u8) -> &'static [u8] {
	match num {
		0 => b"",
		1 => b"\x1B[1;38;5;1m",
		2 => b"\x1B[1;38;5;2m",
		3 => b"\x1B[1;38;5;3m",
		4 => b"\x1B[1;38;5;4m",
		5 => b"\x1B[1;38;5;5m",
		6 => b"\x1B[1;38;5;6m",
		7 => b"\x1B[1;38;5;7m",
		8 => b"\x1B[1;38;5;8m",
		9 => b"\x1B[1;38;5;9m",
		10 => b"\x1B[1;38;5;10m",
		11 => b"\x1B[1;38;5;11m",
		12 => b"\x1B[1;38;5;12m",
		13 => b"\x1B[1;38;5;13m",
		14 => b"\x1B[1;38;5;14m",
		15 => b"\x1B[1;38;5;15m",
		16 => b"\x1B[1;38;5;16m",
		17 => b"\x1B[1;38;5;17m",
		18 => b"\x1B[1;38;5;18m",
		19 => b"\x1B[1;38;5;19m",
		20 => b"\x1B[1;38;5;20m",
		21 => b"\x1B[1;38;5;21m",
		22 => b"\x1B[1;38;5;22m",
		23 => b"\x1B[1;38;5;23m",
		24 => b"\x1B[1;38;5;24m",
		25 => b"\x1B[1;38;5;25m",
		26 => b"\x1B[1;38;5;26m",
		27 => b"\x1B[1;38;5;27m",
		28 => b"\x1B[1;38;5;28m",
		29 => b"\x1B[1;38;5;29m",
		30 => b"\x1B[1;38;5;30m",
		31 => b"\x1B[1;38;5;31m",
		32 => b"\x1B[1;38;5;32m",
		33 => b"\x1B[1;38;5;33m",
		34 => b"\x1B[1;38;5;34m",
		35 => b"\x1B[1;38;5;35m",
		36 => b"\x1B[1;38;5;36m",
		37 => b"\x1B[1;38;5;37m",
		38 => b"\x1B[1;38;5;38m",
		39 => b"\x1B[1;38;5;39m",
		40 => b"\x1B[1;38;5;40m",
		41 => b"\x1B[1;38;5;41m",
		42 => b"\x1B[1;38;5;42m",
		43 => b"\x1B[1;38;5;43m",
		44 => b"\x1B[1;38;5;44m",
		45 => b"\x1B[1;38;5;45m",
		46 => b"\x1B[1;38;5;46m",
		47 => b"\x1B[1;38;5;47m",
		48 => b"\x1B[1;38;5;48m",
		49 => b"\x1B[1;38;5;49m",
		50 => b"\x1B[1;38;5;50m",
		51 => b"\x1B[1;38;5;51m",
		52 => b"\x1B[1;38;5;52m",
		53 => b"\x1B[1;38;5;53m",
		54 => b"\x1B[1;38;5;54m",
		55 => b"\x1B[1;38;5;55m",
		56 => b"\x1B[1;38;5;56m",
		57 => b"\x1B[1;38;5;57m",
		58 => b"\x1B[1;38;5;58m",
		59 => b"\x1B[1;38;5;59m",
		60 => b"\x1B[1;38;5;60m",
		61 => b"\x1B[1;38;5;61m",
		62 => b"\x1B[1;38;5;62m",
		63 => b"\x1B[1;38;5;63m",
		64 => b"\x1B[1;38;5;64m",
		65 => b"\x1B[1;38;5;65m",
		66 => b"\x1B[1;38;5;66m",
		67 => b"\x1B[1;38;5;67m",
		68 => b"\x1B[1;38;5;68m",
		69 => b"\x1B[1;38;5;69m",
		70 => b"\x1B[1;38;5;70m",
		71 => b"\x1B[1;38;5;71m",
		72 => b"\x1B[1;38;5;72m",
		73 => b"\x1B[1;38;5;73m",
		74 => b"\x1B[1;38;5;74m",
		75 => b"\x1B[1;38;5;75m",
		76 => b"\x1B[1;38;5;76m",
		77 => b"\x1B[1;38;5;77m",
		78 => b"\x1B[1;38;5;78m",
		79 => b"\x1B[1;38;5;79m",
		80 => b"\x1B[1;38;5;80m",
		81 => b"\x1B[1;38;5;81m",
		82 => b"\x1B[1;38;5;82m",
		83 => b"\x1B[1;38;5;83m",
		84 => b"\x1B[1;38;5;84m",
		85 => b"\x1B[1;38;5;85m",
		86 => b"\x1B[1;38;5;86m",
		87 => b"\x1B[1;38;5;87m",
		88 => b"\x1B[1;38;5;88m",
		89 => b"\x1B[1;38;5;89m",
		90 => b"\x1B[1;38;5;90m",
		91 => b"\x1B[1;38;5;91m",
		92 => b"\x1B[1;38;5;92m",
		93 => b"\x1B[1;38;5;93m",
		94 => b"\x1B[1;38;5;94m",
		95 => b"\x1B[1;38;5;95m",
		96 => b"\x1B[1;38;5;96m",
		97 => b"\x1B[1;38;5;97m",
		98 => b"\x1B[1;38;5;98m",
		99 => b"\x1B[1;38;5;99m",
		100 => b"\x1B[1;38;5;100m",
		101 => b"\x1B[1;38;5;101m",
		102 => b"\x1B[1;38;5;102m",
		103 => b"\x1B[1;38;5;103m",
		104 => b"\x1B[1;38;5;104m",
		105 => b"\x1B[1;38;5;105m",
		106 => b"\x1B[1;38;5;106m",
		107 => b"\x1B[1;38;5;107m",
		108 => b"\x1B[1;38;5;108m",
		109 => b"\x1B[1;38;5;109m",
		110 => b"\x1B[1;38;5;110m",
		111 => b"\x1B[1;38;5;111m",
		112 => b"\x1B[1;38;5;112m",
		113 => b"\x1B[1;38;5;113m",
		114 => b"\x1B[1;38;5;114m",
		115 => b"\x1B[1;38;5;115m",
		116 => b"\x1B[1;38;5;116m",
		117 => b"\x1B[1;38;5;117m",
		118 => b"\x1B[1;38;5;118m",
		119 => b"\x1B[1;38;5;119m",
		120 => b"\x1B[1;38;5;120m",
		121 => b"\x1B[1;38;5;121m",
		122 => b"\x1B[1;38;5;122m",
		123 => b"\x1B[1;38;5;123m",
		124 => b"\x1B[1;38;5;124m",
		125 => b"\x1B[1;38;5;125m",
		126 => b"\x1B[1;38;5;126m",
		127 => b"\x1B[1;38;5;127m",
		128 => b"\x1B[1;38;5;128m",
		129 => b"\x1B[1;38;5;129m",
		130 => b"\x1B[1;38;5;130m",
		131 => b"\x1B[1;38;5;131m",
		132 => b"\x1B[1;38;5;132m",
		133 => b"\x1B[1;38;5;133m",
		134 => b"\x1B[1;38;5;134m",
		135 => b"\x1B[1;38;5;135m",
		136 => b"\x1B[1;38;5;136m",
		137 => b"\x1B[1;38;5;137m",
		138 => b"\x1B[1;38;5;138m",
		139 => b"\x1B[1;38;5;139m",
		140 => b"\x1B[1;38;5;140m",
		141 => b"\x1B[1;38;5;141m",
		142 => b"\x1B[1;38;5;142m",
		143 => b"\x1B[1;38;5;143m",
		144 => b"\x1B[1;38;5;144m",
		145 => b"\x1B[1;38;5;145m",
		146 => b"\x1B[1;38;5;146m",
		147 => b"\x1B[1;38;5;147m",
		148 => b"\x1B[1;38;5;148m",
		149 => b"\x1B[1;38;5;149m",
		150 => b"\x1B[1;38;5;150m",
		151 => b"\x1B[1;38;5;151m",
		152 => b"\x1B[1;38;5;152m",
		153 => b"\x1B[1;38;5;153m",
		154 => b"\x1B[1;38;5;154m",
		155 => b"\x1B[1;38;5;155m",
		156 => b"\x1B[1;38;5;156m",
		157 => b"\x1B[1;38;5;157m",
		158 => b"\x1B[1;38;5;158m",
		159 => b"\x1B[1;38;5;159m",
		160 => b"\x1B[1;38;5;160m",
		161 => b"\x1B[1;38;5;161m",
		162 => b"\x1B[1;38;5;162m",
		163 => b"\x1B[1;38;5;163m",
		164 => b"\x1B[1;38;5;164m",
		165 => b"\x1B[1;38;5;165m",
		166 => b"\x1B[1;38;5;166m",
		167 => b"\x1B[1;38;5;167m",
		168 => b"\x1B[1;38;5;168m",
		169 => b"\x1B[1;38;5;169m",
		170 => b"\x1B[1;38;5;170m",
		171 => b"\x1B[1;38;5;171m",
		172 => b"\x1B[1;38;5;172m",
		173 => b"\x1B[1;38;5;173m",
		174 => b"\x1B[1;38;5;174m",
		175 => b"\x1B[1;38;5;175m",
		176 => b"\x1B[1;38;5;176m",
		177 => b"\x1B[1;38;5;177m",
		178 => b"\x1B[1;38;5;178m",
		179 => b"\x1B[1;38;5;179m",
		180 => b"\x1B[1;38;5;180m",
		181 => b"\x1B[1;38;5;181m",
		182 => b"\x1B[1;38;5;182m",
		183 => b"\x1B[1;38;5;183m",
		184 => b"\x1B[1;38;5;184m",
		185 => b"\x1B[1;38;5;185m",
		186 => b"\x1B[1;38;5;186m",
		187 => b"\x1B[1;38;5;187m",
		188 => b"\x1B[1;38;5;188m",
		189 => b"\x1B[1;38;5;189m",
		190 => b"\x1B[1;38;5;190m",
		191 => b"\x1B[1;38;5;191m",
		192 => b"\x1B[1;38;5;192m",
		193 => b"\x1B[1;38;5;193m",
		194 => b"\x1B[1;38;5;194m",
		195 => b"\x1B[1;38;5;195m",
		196 => b"\x1B[1;38;5;196m",
		197 => b"\x1B[1;38;5;197m",
		198 => b"\x1B[1;38;5;198m",
		199 => b"\x1B[1;38;5;199m",
		200 => b"\x1B[1;38;5;200m",
		201 => b"\x1B[1;38;5;201m",
		202 => b"\x1B[1;38;5;202m",
		203 => b"\x1B[1;38;5;203m",
		204 => b"\x1B[1;38;5;204m",
		205 => b"\x1B[1;38;5;205m",
		206 => b"\x1B[1;38;5;206m",
		207 => b"\x1B[1;38;5;207m",
		208 => b"\x1B[1;38;5;208m",
		209 => b"\x1B[1;38;5;209m",
		210 => b"\x1B[1;38;5;210m",
		211 => b"\x1B[1;38;5;211m",
		212 => b"\x1B[1;38;5;212m",
		213 => b"\x1B[1;38;5;213m",
		214 => b"\x1B[1;38;5;214m",
		215 => b"\x1B[1;38;5;215m",
		216 => b"\x1B[1;38;5;216m",
		217 => b"\x1B[1;38;5;217m",
		218 => b"\x1B[1;38;5;218m",
		219 => b"\x1B[1;38;5;219m",
		220 => b"\x1B[1;38;5;220m",
		221 => b"\x1B[1;38;5;221m",
		222 => b"\x1B[1;38;5;222m",
		223 => b"\x1B[1;38;5;223m",
		224 => b"\x1B[1;38;5;224m",
		225 => b"\x1B[1;38;5;225m",
		226 => b"\x1B[1;38;5;226m",
		227 => b"\x1B[1;38;5;227m",
		228 => b"\x1B[1;38;5;228m",
		229 => b"\x1B[1;38;5;229m",
		230 => b"\x1B[1;38;5;230m",
		231 => b"\x1B[1;38;5;231m",
		232 => b"\x1B[1;38;5;232m",
		233 => b"\x1B[1;38;5;233m",
		234 => b"\x1B[1;38;5;234m",
		235 => b"\x1B[1;38;5;235m",
		236 => b"\x1B[1;38;5;236m",
		237 => b"\x1B[1;38;5;237m",
		238 => b"\x1B[1;38;5;238m",
		239 => b"\x1B[1;38;5;239m",
		240 => b"\x1B[1;38;5;240m",
		241 => b"\x1B[1;38;5;241m",
		242 => b"\x1B[1;38;5;242m",
		243 => b"\x1B[1;38;5;243m",
		244 => b"\x1B[1;38;5;244m",
		245 => b"\x1B[1;38;5;245m",
		246 => b"\x1B[1;38;5;246m",
		247 => b"\x1B[1;38;5;247m",
		248 => b"\x1B[1;38;5;248m",
		249 => b"\x1B[1;38;5;249m",
		250 => b"\x1B[1;38;5;250m",
		251 => b"\x1B[1;38;5;251m",
		252 => b"\x1B[1;38;5;252m",
		253 => b"\x1B[1;38;5;253m",
		254 => b"\x1B[1;38;5;254m",
		255 => b"\x1B[1;38;5;255m",
	}
}

/// In-Place u8 Slice Replacement
///
/// It is often more efficient to copy bytes into the middle of an existing
/// buffer than to run a million separate `push()` or `extend_from_slice()`
/// operations.
///
/// This method can be used to do just that for any two `[u8]` slices of
/// matching lengths. (Note: if the lengths don't match, it will panic!)
///
/// Each index from the right is compared against the corresponding index
/// on the left and copied if different.
pub fn slice_swap(lhs: &mut [u8], rhs: &[u8]) {
	let len: usize = lhs.len();
	assert_eq!(len, rhs.len());

	for i in 0..len {
		if lhs[i] != rhs[i] {
			lhs[i] = rhs[i];
		}
	}
}

#[must_use]
/// Term Width
///
/// This is a simple wrapper around `term_size::dimensions()` to provide
/// the current terminal column width. We don't have any use for height,
/// so that property is ignored.
pub fn term_width() -> usize {
	// Reserve one space at the end "just in case".
	if let Some((w, _)) = term_size::dimensions() { w.saturating_sub(1) }
	else { 0 }
}

#[must_use]
/// Whitespace maker.
///
/// This method borrows whitespace from a static reference, useful for
/// quickly padding strings, etc.
pub fn whitespace(num: usize) -> &'static [u8] {
	static WHITES: &[u8; 255] = &[b' '; 255];

	if num >= 255 { &WHITES[..] }
	else { &WHITES[0..num] }
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
	fn t_slice_swap() {
		let mut lhs: Vec<u8> = vec![1, 2, 3, 4];
		let rhs: Vec<u8> = vec![5, 6, 7, 8];

		// Make sure it looks right before the swap.
		assert_eq!(&lhs, &[1, 2, 3, 4]);

		// Make a change and double-check each side.
		slice_swap(&mut lhs[1..3], &rhs[..2]);
		assert_eq!(&lhs, &[1, 5, 6, 4]);
		assert_eq!(&rhs, &[5, 6, 7, 8]);
	}

	#[test]
	fn t_whitespace() {
		assert_eq!(whitespace(0), b"");
		assert_eq!(whitespace(5), b"     ");
		assert_eq!(whitespace(10), b"          ");
	}
}
