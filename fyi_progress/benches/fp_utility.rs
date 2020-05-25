/*!
# Benchmark: `fyi_progress::lapsed`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::utility;



fn chopped_len(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");

	for pairs in [
		("This is a sentence!".as_bytes(), 20),
		("This \x1b[1mis\x1b[0m a \x1b[96msentence!\x1b[0m".as_bytes(), 20),
		("This is a sentence!".as_bytes(), 10),
		("This \x1b[1mis\x1b[0m a \x1b[96msentence!\x1b[0m".as_bytes(), 10),
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"chopped_len(&[{}…], {})",
				pairs.0.len(),
				pairs.1,
			)),
			pairs,
			|b, pairs| {
				b.iter(||
					utility::chopped_len(pairs.0, pairs.1)
				);
			}
		);
	}
}

fn human_elapsed(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");

	for secs in [1, 50, 100, 2121, 37732, 428390].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"human_elapsed({})",
				secs
			)),
			secs,
			|b, &secs| {
				b.iter(||
					utility::human_elapsed(secs)
				);
			}
		);
	}
}

fn int_as_bytes(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");

	for ints in [10, 113, 10502].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"int_as_bytes({})",
				ints,
			)),
			ints,
			|b, &ints| {
				b.iter(||
					utility::int_as_bytes(ints)
				);
			}
		);
	}
}

fn secs_chunks(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::utility");

	for secs in [10, 113, 10502].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"secs_chunks({})",
				secs,
			)),
			secs,
			|b, &secs| {
				b.iter(||
					utility::secs_chunks(secs)
				);
			}
		);
	}
}



criterion_group!(
	benches,
	chopped_len,
	int_as_bytes,
	human_elapsed,
	secs_chunks,
);
criterion_main!(benches);
