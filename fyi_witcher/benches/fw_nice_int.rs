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
	group.sample_size(50);

	for ints in [42_489_320_013_u64, 1_999_999_999_999_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u64>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u32(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceInt");
	group.sample_size(50);

	for ints in [99_502_u32, 777_804_132_u32, 4_294_967_295_u32].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u32>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u8(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::NiceInt");
	group.sample_size(50);

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
	from_u32,
	from_u8,
);
criterion_main!(benches);
