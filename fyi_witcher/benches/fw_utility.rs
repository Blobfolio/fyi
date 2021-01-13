/*!
# Benchmark: `fyi_witcher::utility`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::utility;



fn hash64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
	group.sample_size(50);

	for kv in [
		&b"--prefix"[..],
		b"-p",
		b"/usr/share/man/man1/fyi-confirm.1.gz",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!(
				"hash64({:?})",
				unsafe { std::str::from_utf8_unchecked(kv) }
			)),
			kv,
			|b, kv| {
				b.iter(|| utility::hash64(&kv))
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	hash64,
);
criterion_main!(benches);
