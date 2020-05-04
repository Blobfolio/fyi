/*!
# Benchmark: `fyi_msg::traits::DoubleTime`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::DoubleTime;



fn double_digit_time(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::DoubleTime");

	for secs in [1, 50, 100].iter() {
		// Print both implementations for comparison.
		println!("\x1B[1;96mimpl [u8]:\x1B[0m {}", String::from_utf8(<[u8]>::double_digit_time(*secs).to_vec()).unwrap());
		println!("\x1B[1;96mimpl str:\x1B[0m  {}", str::double_digit_time(*secs));

		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"<[u8]>::double_digit_time({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					<[u8]>::double_digit_time(secs)
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
	double_digit_time,
);
criterion_main!(benches);
