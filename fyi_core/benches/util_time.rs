extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn time_chunked(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::time::chunked");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::time::chunked(size)
				);
			}
		);
	}
	group.finish();
}

fn time_elapsed(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::time::elapsed");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::time::elapsed(size)
				);
			}
		);
	}
	group.finish();
}

fn time_elapsed_short(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::time::elapsed_short");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::time::elapsed_short(size)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(benches, time_chunked, time_elapsed, time_elapsed_short);
criterion_main!(benches);
