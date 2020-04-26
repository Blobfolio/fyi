extern crate criterion;

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_core::{
	PRINT_NOTHING,
	Progress,
};



fn progress_new(c: &mut Criterion) {
	let mut group = c.benchmark_group("Progress::new");
	let size = black_box(1000);
	for msg in [
		"",
		"Hello World!",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", msg)),
			msg,
			|b, &msg| {
				b.iter(||
					Progress::new(msg, size, PRINT_NOTHING)
				);
			}
		);
	}
	group.finish();
}



fn progress_tick(c: &mut Criterion) {
	let bar = black_box(Progress::new("Test", 10000, PRINT_NOTHING));
	bar.increment(5);

	c.bench_function("Progress::tick", move |b| {
		b.iter(|| bar.tick())
	});
}




criterion_group!(
	benches,
	progress_new,
	progress_tick,
);
criterion_main!(benches);
