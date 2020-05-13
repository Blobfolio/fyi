/*!
# Benchmark: `fyi_msg::traits::Inflection`
*/

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::utility;



fn inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");
	let singular = black_box("agendum");
	let plural = black_box("agenda");

	for num in [0_u64, 1_u64, 500_u64, 1000_u64, 86400_u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"inflect({}, \"agendum\", \"agenda\")",
				num
			)),
			num,
			|b, &num| {
				b.iter(||
					utility::inflect(num, singular, plural)
				);
			}
		);
	}
}

fn whitespace(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	for spaces in [0, 4, 50, 100, 250].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("whitespace({})", spaces)),
			spaces,
			|b, &spaces| {
				b.iter(||
					utility::whitespace(spaces)
				);
			}
		);
	}
}

criterion_group!(
	benches,
	inflect,
	whitespace,
);
criterion_main!(benches);
