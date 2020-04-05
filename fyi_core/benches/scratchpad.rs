use criterion::{
	black_box,
	criterion_group,
	criterion_main,
	Criterion
};



/// Bench.
fn criterion_benchmark(c: &mut Criterion) {
	use fyi_core::Progress;
	use fyi_core::progress_arc as parc;
	use std::thread;
	use std::time::Duration;
	use rayon::prelude::*;
	use std::path::PathBuf;

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

	/*let text: String = "Björk Guðmundsdóttir".to_string();
	let colored: String = format!("{}", ansi_term::Colour::Blue.bold().paint(&text));

	assert_eq!(&text.fyi_shorten(10), "Björk Guð…");
	assert_eq!(&text.fyi_shorten(100), &text);

	c.bench_function("shorten", |b| b.iter(|| text.fyi_shorten(black_box(10))));
	c.bench_function("shorten", |b| b.iter(|| text.fyi_shorten(black_box(100))));*/
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
