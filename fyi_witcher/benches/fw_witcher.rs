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
	let paths = black_box([ "/usr/share/man/man5" ]);
	let regex = black_box(r"(?i)\.gz$");

	c.bench_function("fyi_witcher::Witcher/new(/usr/share/man/man5, *.gz)", move |b| {
		b.iter(|| Witcher::new(&paths, regex))
	});
}

criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
