extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn numbers_human_bytes(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::numbers::human_bytes");
	for size in [0u64, 999u64, 1000u64, 499288372u64, 99389382145u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::numbers::human_bytes(size)
				);
			}
		);
	}
	group.finish();
}

fn numbers_human_int(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::numbers::human_int");
	for size in [500u64, 5000u64, 5000000u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::numbers::human_int(size)
				);
			}
		);
	}
	group.finish();
}

fn numbers_saved(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::numbers::saved");
	for size in [(0, 500), (500, 500), (1000, 500)].iter() {
		// Come up with a reasonable name.
		let name: String = {
			let (before, after) = size;
			[
				before.to_string(),
				"»".to_string(),
				after.to_string()
			].concat()
		};

		group.bench_with_input(
			BenchmarkId::from_parameter(name),
			size,
			|b, &(before, after)| {
				b.iter(||
					fyi_core::util::numbers::saved(before, after)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(benches, numbers_human_bytes, numbers_human_int, numbers_saved);
criterion_main!(benches);
