/*!
# Benchmark: `fyi_msg::Msg`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::{
	Msg,
	MsgKind,
};



fn new(c: &mut Criterion) {
	let example_str = "This is an example message!";
	let prefix_str = "Prefix";
	let one99_u8 = black_box(199_u8);

	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("default()", move |b| {
		b.iter(|| Msg::default())
	});

	group.bench_function("new(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(example_str))
	});

	group.bench_function("new(\"Prefix:\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::new(prefix_str, one99_u8).into_msg(example_str))
	});

	group.bench_function("error(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Error.into_msg(example_str))
	});

	group.bench_function("debug(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Debug.into_msg(example_str))
	});

	group.finish();
}

fn with_timestamp(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("with_timestamp()", move |b| {
		b.iter_with_setup(||
			MsgKind::Error.into_msg("The rain in spain is plain."),
			|m| m.with_timestamp(true)
		)
	});
}



criterion_group!(
	benches,
	new,
	with_timestamp,
);
criterion_main!(benches);
