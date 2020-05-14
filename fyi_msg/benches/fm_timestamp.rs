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
	println!("{}", Timestamp::new());
	c.bench_function("fyi_msg::Timestamp/new()", move |b| {
		b.iter(|| Timestamp::new())
	});
	println!("");
	println!("");
}

criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
