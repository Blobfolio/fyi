/*!
# Benchmark: `fyi_menu`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::Argue;



fn from_iter(c: &mut Criterion) {
	use std::iter::FromIterator;

	let mut group = c.benchmark_group("fyi_menu::Argue");

	group.bench_function("from_iter(debug -t A penny saved...)", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("debug"),
				String::from("-t"),
				String::from("A penny saved is a penny earned."),
			],
			|v| Argue::from_iter(v.into_iter())
		)
	});

	group.bench_function("from_iter(print --prefix hello -c 199 -t A penny saved...)", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("print"),
				String::from("--prefix"),
				String::from("Hello"),
				String::from("-c"),
				String::from("199"),
				String::from("-t"),
				String::from("A penny saved is a penny earned."),
			],
			|v| Argue::from_iter(v.into_iter())
		)
	});

	group.finish();
}


criterion_group!(
	benches,
	from_iter,
);
criterion_main!(benches);
