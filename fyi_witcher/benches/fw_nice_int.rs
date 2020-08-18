/*!
# Benchmark: `fyi_witcher::nice_int`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::NiceInt;



fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceInt");

	for ints in [10_u64, 113_u64, 10_502_u64, 46_489_320_013_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from,
);
criterion_main!(benches);