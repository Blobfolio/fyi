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
	group.sample_size(50);

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
);
criterion_main!(benches);
