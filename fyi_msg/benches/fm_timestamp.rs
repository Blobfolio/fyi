/*!
# Benchmark: `fyi_msg::Timestamp`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::Timestamp;



fn new(c: &mut Criterion) {
	c.bench_function("fyi_msg::Timestamp/new()", move |b| {
		b.iter(|| Timestamp::new())
	});
}

criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
