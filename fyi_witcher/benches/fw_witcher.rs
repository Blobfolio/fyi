/*!
# Benchmark: `fyi_witcher::Witcher`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::Witcher;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	let paths = black_box([ "/usr/share/man/man5" ]);
	let regex = black_box(r"(?i)\.gz$");

	group.bench_function("new(/usr/share/man/man5, *.gz)", move |b| {
		b.iter(|| Witcher::new(&paths, regex))
	});

	group.finish();
}



criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
