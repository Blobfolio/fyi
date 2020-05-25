/*!
# Benchmark: `fyi_progress::Progress`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_progress::Progress;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::Progress");
	let total = black_box(10000);

	group.bench_function("new(10000, None)", move |b| {
		b.iter(|| Progress::new(total, black_box(None::<String>)))
	});

	group.bench_function("new(10000, \"Imma Title Wee\")", move |b| {
		b.iter(|| Progress::new(total, black_box(Some("Imma Title Wee"))))
	});

	group.finish();
}

fn tick(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_progress::Progress");
	let total = black_box(10000);

	group.bench_function("tick(10000, None)", move |b| {
		let pbar = Progress::new(total, black_box(None::<String>));
		b.iter_with_setup(|| pbar.increment(1), |_| pbar.tick())
	});

	group.bench_function("tick(10000, \"Imma Title Wee\")", move |b| {
		let pbar = Progress::new(total, black_box(Some("Imma Title Wee")));
		b.iter_with_setup(|| pbar.increment(1), |_| pbar.tick())
	});

	group.bench_function("tick(10000, \"Imma Title Wee\") + tasks", move |b| {
		let pbar = Progress::new(total, black_box(Some("Imma Title Wee")));
		pbar.add_task("This is a task!");
		pbar.add_task("This is an ostrich.");
		pbar.add_task("Pick up groceries at the store.");
		pbar.add_task("Add more benchmarks.");
		pbar.add_task("Add more unit tests.");
		b.iter_with_setup(|| pbar.increment(1), |_| pbar.tick())
	});

	group.bench_function("tick(10000, \"Imma Title Wee\") + long tasks", move |b| {
		let pbar = Progress::new(total, black_box(Some("Imma Title Wee")));
		pbar.add_task("This is a task!");
		pbar.add_task("A still more glorious dawn awaits rogue globular star cluster decipherment Cambrian explosion tingling of the spine. The sky calls to us extraordinary claims require extraordinary evidence dream of the mind's eye Apollonius of Perga kindling the energy hidden in matter inconspicuous motes of rock and gas. Colonies kindling the energy hidden in matter a very small stage in a vast cosmic arena from which we spring vastness is bearable only through love the only home we've ever known and billions upon billions upon billions upon billions upon billions upon billions upon billions.");
		pbar.add_task("Add more unit tests.");
		b.iter_with_setup(|| pbar.increment(1), |_| pbar.tick())
	});

	group.finish();
}


criterion_group!(
	benches,
	new,
	tick,
);
criterion_main!(benches);
