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



fn fitted_range(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
	group.sample_size(30);

	for txt in [
		"Hello World",
		"\x1b[1;31mHello\x1b[0m World",
		"Björk Guðmundsdóttir",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"fitted_range({}, 20)",
				txt,
			)),
			txt.as_bytes(),
			|b, txt| {
				b.iter(||
					utility::fitted_range(txt, 20)
				);
			}
		);
	}

	group.finish();
}

fn hash64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");
	group.sample_size(50);

	for kv in [
		&b"--prefix"[..],
		b"-p",
		b"--prefix-color",
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
	fitted_range,
	hash64,
);
criterion_main!(benches);
