/*!
# Benchmark: `fyi_msg::utility`
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

	for color in [1, 50, 100].iter() {
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

	group.finish();
}

fn time_format_dd(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");

	for num in [0, 10, 100].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("time_format_dd({})", num)),
			num,
			|b, &num| {
				b.iter(||
					utility::time_format_dd(num)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	ansi_code_bold,
	time_format_dd,
);
criterion_main!(benches);
