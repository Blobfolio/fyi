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



fn from_u64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceInt");

	for ints in [10_u64, 113_u64, 10_502_u64, 46_489_320_013_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u64>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u8(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceInt");

	for ints in [0_u8, 10_u8, 18_u8, 101_u8, 200_u8].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u8>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	from_u64,
	from_u8,
);
criterion_main!(benches);
