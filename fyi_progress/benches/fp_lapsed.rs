/*!
# Benchmark: `fyi_progress::lapsed`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::lapsed;



fn compact(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::lapsed");

	for secs in [0, 1, 50, 100, 2121, 37732, 428390].iter() {
		println!("{}", unsafe { std::str::from_utf8_unchecked(&lapsed::compact(*secs)) });

		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"compact({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					lapsed::compact(secs)
				);
			}
		);

		// Add a line break; it gets hard to read!
		println!("");
		println!("");
	}
}

fn full(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::lapsed");

	for secs in [1, 50, 100, 2121, 37732, 428390].iter() {
		println!("{}", unsafe { std::str::from_utf8_unchecked(&lapsed::full(*secs)) });

		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"full({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					lapsed::full(secs)
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
	compact,
	full,
);
criterion_main!(benches);
