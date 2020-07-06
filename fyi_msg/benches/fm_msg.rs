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
	let blank_str = black_box("");
	let example_str = "This is an example message!";
	let prefix_str = "Prefix";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("default()", move |b| {
		b.iter(|| Msg::default())
	});

	group.bench_function("new(\"\", 0, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(blank_str, zero_u8, example_str))
	});

	group.bench_function("new(\"Prefix:\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});

	group.bench_function("error(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Error.as_msg(example_str))
	});

	group.bench_function("debug(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Debug.as_msg(example_str))
	});

	group.finish();
}

fn set_indent(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("set_indent(1)", move |b| {
		b.iter_with_setup(||
			MsgKind::Success.as_msg("This is an example message!"),
			|mut msg| msg.set_indent(black_box(1))
		)
	});

	group.finish();
}

fn set_timestamp(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("set_timestamp()", move |b| {
		b.iter_with_setup(||
			MsgKind::Success.as_msg("This is an example message!"),
			|mut msg| msg.set_timestamp()
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	new,
	set_indent,
	set_timestamp,
);
criterion_main!(benches);
