/*!
# Benchmark: `fyi_msg::traits::StripAnsi`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::StripAnsi;



fn strip_ansi(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::StripAnsi");

	for text in [
		"Normal",
		"\x1B[1;96mBjörk\x1B[0m Guðmundsdóttir",
	].iter() {
		// Print both implementations for comparison.
		println!("\x1B[1;96mimpl [u8]:\x1B[0m {}", String::from_utf8(text.as_bytes().strip_ansi().to_vec()).unwrap());
		println!("\x1B[1;96mimpl str:\x1B[0m  {}", text.strip_ansi());

		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{:?}.strip_ansi()",
				text
			)),
			text.as_bytes(),
			|b, text| {
				b.iter(||
					text.strip_ansi()
				);
			}
		);

		// Add a line break; it gets hard to read!
		println!("");
		println!("");
	}
}

criterion_group!(
	benches,
	strip_ansi,
);
criterion_main!(benches);
