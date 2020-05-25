/*!
# Benchmark: `fyi_progress::Progress`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::Progress;



fn new(c: &mut Criterion) {
	let total = black_box(10000);

	c.bench_function("fyi_progress::Progress/new(10000, None)", move |b| {
		b.iter(|| Progress::new(total, black_box(None::<String>)))
	});

	c.bench_function("fyi_progress::Progress/new(10000, \"Imma Title Wee\")", move |b| {
		b.iter(|| Progress::new(total, black_box(Some("Imma Title Wee"))))
	});
}


criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
