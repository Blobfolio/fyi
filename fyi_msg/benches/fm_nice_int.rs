/*!
# Benchmark: `fyi_msg::nice_int`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::NiceInt;



fn from_u64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::NiceInt");
	group.sample_size(30);

	for ints in [6_489_320_013_u64, 42_489_320_013_u64, 1_999_999_999_999_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u64>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u32(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::NiceInt");
	group.sample_size(30);

	for ints in [
		99_502_u32,
		100_200_u32,
		6_330_704_u32,
		57_444_000_u32,
		777_804_132_u32,
		4_294_967_295_u32
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u32>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u16(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::NiceInt");
	group.sample_size(30);

	for ints in [301_u16, 1_620_u16, 40_999_u16].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u16>({})", ints)),
			ints,
			|b, &ints| { b.iter(|| NiceInt::from(ints)); }
		);
	}

	group.finish();
}

fn from_u8(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::NiceInt");
	group.sample_size(30);

	for ints in [0_u8, 18_u8, 101_u8, 200_u8].iter() {
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
	from_u16,
	from_u8,
);
criterion_main!(benches);
