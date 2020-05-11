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
	println!(
		"{:?}",
		String::from_utf8([27, 91, 50, 109, 91, 27, 91, 50, 50, 59, 49, 109, 48, 48, 58, 48, 48, 58, 48, 48, 27, 91, 50, 50, 59, 50, 109, 93, 27, 91, 48, 109, 32, 32][12..20].to_vec())
	);
	std::process::exit(0);

	let mut group = c.benchmark_group("fyi_progress::lapsed");

	for secs in [0, 1, 50, 100, 2121, 37732, 428390, 5847294].iter() {
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



criterion_group!(
	benches,
	compact,
);
criterion_main!(benches);
