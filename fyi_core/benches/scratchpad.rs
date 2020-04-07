use criterion::{
	black_box,
	criterion_group,
	criterion_main,
	Criterion
};



/// Bench.
fn criterion_benchmark(c: &mut Criterion) {
	use fyi_core::{
		Progress,
		arc::progress as parc,
	};
	use rayon::prelude::*;
	use std::path::PathBuf;
	use std::thread;
	use std::time::Duration;

	{
		let bar = Progress::new("Funny thing is happening!", 300, 0);
		let looper = parc::looper(&bar, 60);
		(0..300).into_par_iter().for_each(|ref x| {
			let fakep = PathBuf::from(format!("/tmp/{:?}", x));
			parc::add_working(&bar, fakep.to_path_buf());

			thread::sleep(Duration::from_millis(200));

			parc::update(&bar, 1, None, Some(fakep.to_path_buf()));
		});
		parc::finish(&bar);
		looper.join().unwrap();
	}


	c.bench_function("Progress::new", |b| b.iter(|| Progress::new(
		black_box("The rain in Spain rhymes with plain."),
		black_box(100),
		black_box(0),
	)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
