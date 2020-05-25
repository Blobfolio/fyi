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
	let prefix_str = "Prefix";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("default()", move |b| {
		b.iter(|| Msg::default())
	});

	println!("{}", Msg::new(blank_str, zero_u8, example_str));
	group.bench_function("new(\"\", 0, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(blank_str, zero_u8, example_str))
	});

	println!("{}", Msg::new(prefix_str, one99_u8, example_str));
	group.bench_function("new(\"Prefix:\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});

	println!("{}", Msg::error(example_str));
	group.bench_function("error(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::error(example_str))
	});

	println!("{}", Msg::debug(example_str));
	group.bench_function("debug(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::debug(example_str))
	});

	group.finish();
}

fn set_timestamp(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");

	group.bench_function("set_timestamp(false)", move |b| {
		b.iter_with_setup(|| Msg::success("This is an example message!"), |mut msg| msg.set_timestamp(false))
	});

	group.finish();
}



criterion_group!(
	benches,
	new,
	set_timestamp,
);
criterion_main!(benches);
