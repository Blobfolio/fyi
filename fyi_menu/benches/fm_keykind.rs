/*!
# Benchmark: `fyi_menu::KeyKind`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::KeyKind;



fn keykind_from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyKind");
	group.sample_size(30);

	for kv in [
		&b"Hello World!"[..],
		&b"--prefix"[..],
		&b"--prefix-color=Value"[..],
		&b"-p"[..],
		&b"-c199"[..],
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!(
				"from({})",
				unsafe { std::str::from_utf8_unchecked(kv) }
			)),
			kv,
			|b, &kv| {
				b.iter(||
					KeyKind::from(kv)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	keykind_from,
);
criterion_main!(benches);
