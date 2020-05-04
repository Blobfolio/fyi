/*!
# Benchmark: `fyi_msg::traits::HumanElapsed`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::HumanElapsed;



fn human_elapsed(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::HumanElapsed");

	for secs in [0, 1, 50, 100, 2121, 37732, 428390, 5847294].iter() {
		// Print both implementations for comparison.
		println!("\x1B[1;96mimpl [u8]:\x1B[0m {}", String::from_utf8(<[u8]>::human_elapsed(*secs).to_vec()).unwrap());
		println!("\x1B[1;96mimpl str:\x1B[0m  {}", str::human_elapsed(*secs));

		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"<[u8]>::human_elapsed({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					<[u8]>::human_elapsed(secs)
				);
			}
		);

		// Add a line break; it gets hard to read!
		println!("");
		println!("");
	}
}

fn human_elapsed_short(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::HumanElapsed");

	for secs in [0, 1, 50, 100, 2121, 37732, 428390, 5847294].iter() {
		// Print both implementations for comparison.
		println!("\x1B[1;96mimpl [u8]:\x1B[0m {}", String::from_utf8(<[u8]>::human_elapsed_short(*secs).to_vec()).unwrap());
		println!("\x1B[1;96mimpl str:\x1B[0m  {}", str::human_elapsed_short(*secs));

		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"<[u8]>::human_elapsed_short({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					<[u8]>::human_elapsed_short(secs)
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
	human_elapsed,
	human_elapsed_short,
);
criterion_main!(benches);
