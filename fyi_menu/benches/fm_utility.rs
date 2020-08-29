/*!
# Benchmark: `fyi_menu::utility`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



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

fn hash_arg_key(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::utility");

	for kv in [
		"--prefix",
		"-p",
		"--prefix-color",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!("hash_arg_key({:?})", kv.to_string())),
			&kv.to_string(),
			|b, kv| {
				b.iter(|| fyi_menu::utility::hash_arg_key(&kv))
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	esc_arg,
	hash_arg_key,
);
criterion_main!(benches);
