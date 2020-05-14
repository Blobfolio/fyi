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
use fyi_witcher::utility;



fn inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
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



criterion_group!(
	benches,
	inflect,
);
criterion_main!(benches);
