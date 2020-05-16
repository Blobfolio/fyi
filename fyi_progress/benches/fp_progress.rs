/*!
# Benchmark: `fyi_progress::Progress`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::Msg;
use fyi_progress::Progress;



fn new(c: &mut Criterion) {
	let msg = Msg::new("Prefix", 199, "This is an example message!");
	let hundred_k = black_box(100_000_u64);

	c.bench_function("fyi_progress::Progress/new(None, 100000)", move |b| {
		b.iter(|| Progress::new(None, hundred_k))
	});

	c.bench_function("fyi_progress::Progress/new(Some(...), 100000)", move |b| {
		b.iter(|| Progress::new(Some(msg.clone()), hundred_k))
	});
}

fn tick(c: &mut Criterion) {
	let msg = Msg::new("Prefix", 199, "This is an example message!");
	let hundred_k = black_box(100_000_u64);

	let progress = Progress::new(None, hundred_k);
	c.bench_function("fyi_progress::Progress/tick()/{None, 0/100000, None}", move |b| {
		b.iter(|| progress.tick())
	});

	let progress = Progress::new(None, hundred_k);
	progress.increment(100);
	c.bench_function("fyi_progress::Progress/tick()/{None, 100/100000, None}", move |b| {
		b.iter(|| progress.tick())
	});

	let progress = Progress::new(None, hundred_k);
	progress.add_task("The first thing I want to say is, Hello!");
	progress.add_task("The second thing I want to say is, Good Bye!");
	c.bench_function("fyi_progress::Progress/tick()/{None, 0/100000, [2]}", move |b| {
		b.iter(|| progress.tick())
	});

	let progress = Progress::new(None, hundred_k);
	progress.increment(100);
	progress.add_task("The first thing I want to say is, Hello!");
	progress.add_task("The second thing I want to say is, Good Bye!");
	c.bench_function("fyi_progress::Progress/tick()/{None, 100/100000, [2]}", move |b| {
		b.iter(|| progress.tick())
	});

	let progress = Progress::new(None, hundred_k);
	progress.increment(50000);
	progress.add_task("The first thing I want to say is, Hello!");
	progress.add_task("The second thing I want to say is, Good Bye!");
	c.bench_function("fyi_progress::Progress/tick()/{None, 50000/100000, [2]}", move |b| {
		b.iter(|| progress.tick())
	});

	let progress = Progress::new(Some(msg), hundred_k);
	progress.increment(50000);
	progress.add_task("The first thing I want to say is, Hello!");
	progress.add_task("The second thing I want to say is, Good Bye!");
	c.bench_function("fyi_progress::Progress/tick()/{Some(...), 50000/100000, [2]}", move |b| {
		b.iter(|| progress.tick())
	});
}



criterion_group!(
	benches,
	new,
	tick,
);
criterion_main!(benches);
