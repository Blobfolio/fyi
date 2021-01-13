/*!
# Benchmark: `fyi_num::nice_int`
*/

use criterion::{
	BenchmarkId,
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_num::NiceInt;



fn from_u64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceInt");
	group.sample_size(30);

	for ints in [
		17_u64,
		6_489_320_013_u64,
		42_489_320_013_u64,
		1_999_999_999_999_u64,
		u64::MAX,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u64>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceInt::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}

fn from_u32(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceInt");
	group.sample_size(30);

	for ints in [
		99_502_u32,
		100_020_u32,
		6_330_004_u32,
		57_444_000_u32,
		777_804_132_u32,
		u32::MAX,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u32>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceInt::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}

fn from_u16(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceInt");
	group.sample_size(30);

	for ints in [
		17_u16,
		301_u16,
		1_620_u16,
		40_999_u16,
		u16::MAX,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u16>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceInt::from(ints)).as_str();
				});
			}
		);
	}

	group.finish();
}

fn from_u8(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_num::NiceInt");
	group.sample_size(30);

	for ints in [0_u8, 18_u8, 101_u8, 200_u8, u8::MAX].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from<u8>({})", ints)),
			ints,
			|b, &ints| {
				b.iter(|| {
					let _ = black_box(NiceInt::from(ints)).as_str();
				});
			}
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
