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
	let mut group = c.benchmark_group("fyi_menu::Argue");
	group.sample_size(50);

	group.bench_function("from_iter(debug -t A penny saved...)", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("debug"),
				String::from("-t"),
				String::from("A penny saved is a penny earned."),
			].into_iter(),
			|v| Argue::from(v)
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
			].into_iter(),
			|v| Argue::from(v)
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
			|a| a.switch("-c")
		)
	});

	group.bench_function("switch2()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| a.switch2("-c", "--prefix-color")
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
			|a| { let _ = a.option("-c"); }
		)
	});

	group.bench_function("option2()", move |b| {
		b.iter_with_setup(||
			test_data(),
			|a| { let _ = a.option2("-c", "--prefix-color"); }
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
