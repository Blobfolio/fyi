/*!
# Benchmark: `fyi_menu`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::Argue;



fn from_iter(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::Argue");
	group.sample_size(30);

	// These two styles should match.
	assert_eq!(
		*Argue::from(vec![
			String::from("print"),
			String::from("--prefix"),
			String::from("Hello"),
			String::from("-c"),
			String::from("199"),
			String::from("-t"),
			String::from("A penny saved is a penny earned."),
		].into_iter()),
		*Argue::from(vec![
			String::from("print"),
			String::from("--prefix"),
			String::from("Hello"),
			String::from("-c"),
			String::from("199"),
			String::from("-t"),
			String::from("A penny saved is a penny earned."),
		].into_iter())
	);

	// Keys and values split.
	group.bench_function("from_iter(print --prefix hello -c 199 -t …)", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("print"),
				String::from("--prefix"),
				String::from("Hello"),
				String::from("-c"),
				String::from("199"),
				String::from("-t"),
				String::from("A penny saved is a penny earned."),
			].into_iter(),
			|v| { let _ = black_box(Argue::from(v)); }
		)
	});

	// Keys and values merged.
	group.bench_function("from_iter(print --prefix=hello -c199 -t …)", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("print"),
				String::from("--prefix=Hello"),
				String::from("-c199"),
				String::from("-t"),
				String::from("more"),
				String::from("more"),
				String::from("A penny saved is a penny earned."),
			].into_iter(),
			|v| { let _ = black_box(Argue::from(v)); }
		)
	});

	group.finish();
}

fn switch(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::Argue");
	group.sample_size(30);

	fn test_data() -> Argue {
		Argue::from(vec![
			String::from("print"),
			String::from("--prefix"),
			String::from("Hello"),
			String::from("-c"),
			String::from("199"),
			String::from("-t"),
			String::from("A penny saved is a penny earned."),
		].into_iter())
	}

	group.bench_function("switch()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| { let _ = black_box(a.switch("-c")); }
		)
	});

	group.bench_function("switch2()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| { let _ = black_box(a.switch2("-c", "--prefix-color")); }
		)
	});

	group.finish();
}

fn option(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::Argue");
	group.sample_size(30);

	fn test_data() -> Argue {
		Argue::from(vec![
			String::from("print"),
			String::from("--prefix"),
			String::from("Hello"),
			String::from("-c"),
			String::from("199"),
			String::from("-t"),
			String::from("A penny saved is a penny earned."),
		].into_iter())
	}

	group.bench_function("option()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| { let _ = black_box(a.option("-c")); }
		)
	});

	group.bench_function("option2()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| { let _ = black_box(a.option2("-c", "--prefix-color")); }
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	from_iter,
	switch,
	option,
);
criterion_main!(benches);
