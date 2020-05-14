/*!
# Benchmark: `fyi_msg::traits::Inflection`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::utility;



fn ansi_code_bold(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	for color in [0, 1, 50, 100].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"ansi_code_bold({})",
				color
			)),
			color,
			|b, &color| {
				b.iter(||
					utility::ansi_code_bold(color)
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
	ansi_code_bold,
	whitespace,
);
criterion_main!(benches);
