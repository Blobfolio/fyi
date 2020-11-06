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



fn hash64(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");
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

fn write_time(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::utility");
	group.sample_size(30);

	group.bench_function("write_time()", move |b| {
		b.iter_with_setup(||
			([0_u8; 8].as_mut_ptr(), 13, 45, 20), |(v, h, m, s)|
				unsafe { utility::write_time(v, h, m, s, b':') }
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	hash64,
	write_time,
);
criterion_main!(benches);
