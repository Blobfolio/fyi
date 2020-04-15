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
	use std::path::PathBuf;

	let paths = [PathBuf::from("/usr/lib/node_modules/npm")];
	let pattern: Cow<str> = Cow::Borrowed(r"(?i).+\.(js|html?)$");

	let w = fyi_core::Witch::new(&paths, Some(pattern.into()));
	w.progress("Apples", |p| {
		std::thread::sleep(std::time::Duration::from_millis(100));
	});



	/*
	c.bench_function("Walk (new)", |b| b.iter(||
		working::Witch::new(black_box(&paths), None)
	));
	*/
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
