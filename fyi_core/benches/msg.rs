extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn prefix_new(c: &mut Criterion) {
	let mut group = c.benchmark_group("Prefix::new");
	for pair in [
		("", 2),
		("Something", 199),
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", pair)),
			pair,
			|b, &(prefix, color)| {
				b.iter(||
					fyi_core::Prefix::new(prefix, color)
				);
			}
		);
	}
	group.finish();
}

fn prefix_prefix(c: &mut Criterion) {
	use fyi_core::Prefix;
	use fyi_core::traits::AnsiBitsy;

	let mut group = c.benchmark_group("Prefix.prefix");
	for prefix in [
		Prefix::None,
		Prefix::Error,
		Prefix::new("Custom", 199),
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&prefix.to_string().strip_ansi()),
			prefix,
			|b, prefix| {
				b.iter(||
					prefix.prefix()
				);
			}
		);
	}
	group.finish();
}


criterion_group!(
	benches,
	prefix_new,
	prefix_prefix,
);
criterion_main!(benches);
