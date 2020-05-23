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
	Msg2,
	PrintFlags,
};



fn new(c: &mut Criterion) {
	let blank_str = black_box("");
	let example_str = "This is an example message!";
	let prefix_str = "Prefix:";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	Msg::new(prefix_str, one99_u8, example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"Prefix\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});

	println!("{}", Msg2::new(prefix_str, one99_u8, example_str));
	c.bench_function("fyi_msg::Msg2/new(\"Prefix\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg2::new(prefix_str, one99_u8, example_str))
	});

	Msg::error(example_str).print(PrintFlags::NONE);
	c.bench_function("fyi_msg::Msg/error(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::error(example_str))
	});

	println!("{}", Msg2::error(example_str));
	c.bench_function("fyi_msg::Msg2/error(\"This is an example message!\")", move |b| {
		b.iter(|| Msg2::error(example_str))
	});
}



criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
