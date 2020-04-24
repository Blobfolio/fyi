use criterion::{
	black_box,
	criterion_group,
	criterion_main,
	Criterion
};

mod working {

}



/// Bench.
fn criterion_benchmark(c: &mut Criterion) {
	use std::borrow::Cow;

	let paths = ["/usr/lib/node_modules/npm"];
	let pattern: Cow<str> = Cow::Borrowed(r"(?i).+\.(js|html?)$");

	let w = fyi_core::Witch::new(&paths, Some(pattern.into()));
	w.progress("Apples", |_| {
		std::thread::sleep(std::time::Duration::from_millis(100));
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
