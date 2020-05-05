/*!
# Benchmark: `fyi_msg::traits::AnsiCodeBold`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::AnsiCodeBold;



fn ansi_code_bold(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::AnsiCodeBold");

	for color in [0, 1, 50, 100].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"<[u8]>::ansi_code_bold({})",
				color
			)),
			color,
			|b, &color| {
				b.iter(||
					<[u8]>::ansi_code_bold(color)
				);
			}
		);
	}
}

criterion_group!(
	benches,
	ansi_code_bold,
);
criterion_main!(benches);
