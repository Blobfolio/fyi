/*!
# Benchmark: `fyi_msg::partitions::Partitions`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::Partitions;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	// No parts.
	group.bench_function("default()", move |b| {
		b.iter(|| Partitions::default())
	});

	group.bench_with_input(
		BenchmarkId::from_parameter("new(&[])"),
		&[],
		|b, &parts| {
			b.iter(|| Partitions::new(&parts));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new(&[2])"),
		&[2],
		|b, &parts| {
			b.iter(|| Partitions::new(&parts));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new(&[2, 10])"),
		&[2, 10],
		|b, &parts| {
			b.iter(|| Partitions::new(&parts));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new(&[1, 2, 5, 4, 3, 3, 0, 8, 9, 1])"),
		&[1, 2, 5, 4, 3, 3, 0, 8, 9, 1],
		|b, &parts| {
			b.iter(|| Partitions::new(&parts));
		}
	);

	group.finish();
}

fn new_bounded(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	// No parts.
	group.bench_with_input(
		BenchmarkId::from_parameter("new_bounded(&[], 0)"),
		&[],
		|b, &parts| {
			b.iter(|| Partitions::new_bounded(&parts, black_box(0)));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new_bounded(&[], 10)"),
		&[],
		|b, &parts| {
			b.iter(|| Partitions::new_bounded(&parts, black_box(10)));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new_bounded(&[2, 10], 12)"),
		&[2, 10],
		|b, &parts| {
			b.iter(|| Partitions::new_bounded(&parts, black_box(12)));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new_bounded(&[2, 10], 20)"),
		&[2, 10],
		|b, &parts| {
			b.iter(|| Partitions::new_bounded(&parts, black_box(20)));
		}
	);

	group.finish();
}

fn new_one_splat(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_with_input(
		BenchmarkId::from_parameter("one(10)"),
		&10,
		|b, &size| {
			b.iter(|| Partitions::one(size));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("splat(10)"),
		&10,
		|b, &size| {
			b.iter(|| Partitions::splat(size));
		}
	);

	group.finish();
}

fn clear(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[].clear()", move |b| {
		b.iter_with_setup(|| Partitions::default(), |mut parts| parts.clear())
	});
	group.bench_function("[10].clear()", move |b| {
		b.iter_with_setup(|| Partitions::one(10), |mut parts| parts.clear())
	});
	group.bench_function("[10, 11, 11, 15, 20, 30, 35, 50, 50, 51].clear()", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[10, 1, 0, 4, 5, 10, 5, 15, 0, 1]),
			|mut parts| parts.clear()
		)
	});

	group.finish();
}

fn flatten(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[].flatten()", move |b| {
		b.iter_with_setup(|| Partitions::default(), |mut parts| parts.flatten())
	});

	group.bench_function("[10, 11, 11, 15, 20, 30, 35, 50, 50, 51].flatten()", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[10, 1, 0, 4, 5, 10, 5, 15, 0, 1]),
			|mut parts| parts.flatten()
		)
	});

	group.finish();
}

fn add_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("add_part(0)", move |b| {
		b.iter_with_setup(|| Partitions::default(), |mut parts| parts.add_part(black_box(0)))
	});

	group.bench_function("add_part(10)", move |b| {
		b.iter_with_setup(|| Partitions::default(), |mut parts| parts.add_part(black_box(10)))
	});

	group.finish();
}

fn grow_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[10].grow_part(1, 10)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.grow_part(black_box(1), black_box(10))
		)
	});

	group.bench_function("[1, 2, 3, 4, 5, 6, 7, 8, 9, 1].grow_part(2, 10)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.grow_part(black_box(2), black_box(10))
		)
	});

	group.finish();
}

fn insert_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[10].insert_part(1, 0)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.insert_part(black_box(1), black_box(0))
		)
	});

	group.bench_function("[1, 2, 3, 4, 5, 6, 7, 8, 9, 1].insert_part(1, 0)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.insert_part(black_box(1), black_box(0))
		)
	});

	group.bench_function("[10].insert_part(1, 15)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.insert_part(black_box(1), black_box(15))
		)
	});

	group.bench_function("[1, 2, 3, 4, 5, 6, 7, 8, 9, 1].insert_part(1, 15)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.insert_part(black_box(1), black_box(15))
		)
	});

	group.finish();
}

fn remove_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[10].remove_part(1)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.remove_part(black_box(1))
		)
	});

	group.bench_function("[1, 2, 0, 4, 5, 6, 7, 8, 9, 1].remove_part(1)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 0, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.remove_part(black_box(1))
		)
	});

	group.bench_function("[1, 2, 0, 4, 5, 6, 7, 8, 9, 1].remove_part(3)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 0, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.remove_part(black_box(3))
		)
	});

	group.bench_function("[1, 2, 0, 4, 5, 6, 7, 8, 9, 1].remove_part(5)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 2, 0, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.remove_part(black_box(5))
		)
	});

	group.finish();
}

fn shrink_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Partitions");

	group.bench_function("[10].shrink_part(1, 10)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.shrink_part(black_box(1), black_box(10))
		)
	});

	group.bench_function("[10].shrink_part(1, 5)", move |b| {
		b.iter_with_setup(||
			Partitions::one(10),
			|mut parts| parts.shrink_part(black_box(1), black_box(5))
		)
	});

	group.bench_function("[1, 10, 3, 4, 5, 6, 7, 8, 9, 1].shrink_part(2, 10)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 10, 3, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.shrink_part(black_box(2), black_box(10))
		)
	});

	group.bench_function("[1, 10, 3, 4, 5, 6, 7, 8, 9, 1].shrink_part(2, 5)", move |b| {
		b.iter_with_setup(||
			Partitions::new(&[1, 10, 3, 4, 5, 6, 7, 8, 9, 1]),
			|mut parts| parts.shrink_part(black_box(2), black_box(5))
		)
	});

	group.finish();
}



criterion_group!(
	benches,

	new,
	new_bounded,
	new_one_splat,
	clear,
	flatten,

	add_part,
	grow_part,
	insert_part,
	remove_part,
	shrink_part,
);
criterion_main!(benches);
