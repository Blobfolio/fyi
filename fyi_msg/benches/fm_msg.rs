/*!
# Benchmark: `fyi_msg::Msg`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::Msg;



fn new(c: &mut Criterion) {
	let blank_str = black_box("");
	let example_str = "This is an example message!";
	let prefix_str = "Prefix:";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	c.bench_function("fyi_msg::Msg/default()", move |b| {
		b.iter(|| Msg::default())
	});

	println!("{}", Msg::new(blank_str, zero_u8, example_str));
	c.bench_function("fyi_msg::Msg/new(\"\", 0, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(blank_str, zero_u8, example_str))
	});

	println!("{}", Msg::new(prefix_str, one99_u8, example_str));
	c.bench_function("fyi_msg::Msg/new(\"Prefix:\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});

	println!("{}", Msg::error(example_str));
	c.bench_function("fyi_msg::Msg/error(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::error(example_str))
	});

	println!("{}", Msg::debug(example_str));
	c.bench_function("fyi_msg::Msg/debug(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::debug(example_str))
	});
}



criterion_group!(
	benches,
	new,
);
criterion_main!(benches);
