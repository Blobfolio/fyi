/*!
# Benchmark: `fyi_witcher::Witcher`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};



fn witch(c: &mut Criterion) {
	// println!("{:?}", fyi_witcher::witch(&["/usr/share"]).len());

	let mut group = c.benchmark_group("fyi_witcher");
	group.sample_size(30);

	group.bench_function("witch(/usr/share)", move |b| {
		b.iter_with_setup(||
			std::iter::once("/usr/share"),
			|i| {
				let _ = black_box(fyi_witcher::witch(i));
			}
		)
	});

	group.finish();
}

fn build(c: &mut Criterion) {
	use fyi_witcher::Witcher;

	// println!("{:?}", Witcher::default().with_path("/usr/share").build().len());

	let mut group = c.benchmark_group("fyi_witcher::Witcher");
	group.sample_size(30);

	group.bench_function("with_path(/usr/share).build()", move |b| {
		b.iter(|| {
			let _ = black_box(
				Witcher::default()
					.with_path("/usr/share")
					.build()
			);
		})
	});

	group.finish();
}



criterion_group!(
	benches,
	build,
	witch,
);
criterion_main!(benches);
