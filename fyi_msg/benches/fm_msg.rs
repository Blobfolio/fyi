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
	PrintFlags,
};



fn new(c: &mut Criterion) {
	let blank_str = black_box("");
	let example_str = "This is an example message!";
	let prefix_str = "Prefix";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	Msg::new(blank_str, zero_u8, blank_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"\", 0, \"\")", move |b| {
		b.iter(|| Msg::new(blank_str, zero_u8, blank_str))
	});

	Msg::new(prefix_str, zero_u8, blank_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"Prefix\", 0, \"\")", move |b| {
		b.iter(|| Msg::new(prefix_str, zero_u8, blank_str))
	});

	Msg::new(prefix_str, one99_u8, example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"Prefix\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});

	Msg::plain(example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/plain(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::plain(example_str))
	});

	Msg::notice(example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/notice(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::notice(example_str))
	});

	Msg::eg(example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/eg(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::eg(example_str))
	});
}



criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
