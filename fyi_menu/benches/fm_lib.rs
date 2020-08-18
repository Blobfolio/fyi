/*!
# Benchmark: `fyi_menu`
*/

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::KeyKind;



fn keykind_from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyKind");

	for kv in [
		&b"Hello World"[..],
		&b"--key"[..],
		&b"-k"[..],
		&b"-kValue"[..],
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

fn esc_arg(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::utility");

	for kv in [
		"",
		"Hello",
		"Hello World",
		"Hello Joe\'s World",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!("esc_arg({:?})", kv.to_string())),
			&kv.to_string(),
			|b, kv| {
				b.iter_with_setup(||
					kv.to_string(),
					|s| fyi_menu::utility::esc_arg(s)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	esc_arg,
	keykind_from,
);
criterion_main!(benches);
