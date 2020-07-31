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
use std::path::PathBuf;



fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function("from(/usr/share/man).to_vec()", move |b| {
		b.iter_with_setup(||
			black_box(PathBuf::from("/usr/share/man")),
			|path| Witcher::from(path).to_vec()
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	from,
);
criterion_main!(benches);
