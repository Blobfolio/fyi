extern crate criterion;

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};


fn strings_inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::inflect");

	let singular = black_box("book");
	let plural = black_box("books");

	for size in [0, 1, 2, 1000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::inflect(size, singular, plural)
				);
			}
		);
	}
	group.finish();
}

fn strings_oxford_join(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::oxford_join");

	let data = black_box(vec![
		"apples".to_string(),
		"bananas".to_string(),
		"carrots".to_string(),
	]);

	let glue = black_box("and");

	for size in (0..4).into_iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			&size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::oxford_join(&data[0..size], glue)
				);
			}
		);
	}
	group.finish();
}

fn strings_whitespace(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::whitespace");
	for size in [0, 50, 100, 200].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::whitespace(size)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(benches, strings_inflect, strings_oxford_join, strings_whitespace);
criterion_main!(benches);
