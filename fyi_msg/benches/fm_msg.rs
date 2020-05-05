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
	Flags,
	Msg,
	traits::Printable,
};



fn new(c: &mut Criterion) {
	let blank_str = black_box("");
	let example_str = "This is an example message!";
	let prefix_str = "Prefix";
	let zero_u8 = black_box(0_u8);
	let one99_u8 = black_box(199_u8);

	Msg::new(blank_str, zero_u8, blank_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"\", 0, \"\")", move |b| {
		b.iter(|| Msg::new(blank_str, zero_u8, blank_str))
	});
	println!("");
	println!("");

	Msg::new(prefix_str, zero_u8, blank_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"Prefix\", 0, \"\")", move |b| {
		b.iter(|| Msg::new(prefix_str, zero_u8, blank_str))
	});
	println!("");
	println!("");

	Msg::new(prefix_str, one99_u8, example_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/new(\"Prefix\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::new(prefix_str, one99_u8, example_str))
	});
	println!("");
	println!("");

	Msg::plain(example_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/plain(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::plain(example_str))
	});
	println!("");
	println!("");

	Msg::notice(example_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/notice(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::notice(example_str))
	});
	println!("");
	println!("");

	Msg::warning(example_str).print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg/warning(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::warning(example_str))
	});
	println!("");
	println!("");
}

fn print(c: &mut Criterion) {
	let example_str = "This is an example message!";
	let zero_u8 = black_box(0_u8);
	let one_u8 = black_box(1_u8);
	let two_u8 = black_box(2_u8);
	let no_ansi_flag = black_box(Flags::NO_ANSI | Flags::TO_NOWHERE);
	let timestamped_flag = black_box(Flags::TIMESTAMPED | Flags::TO_NOWHERE);
	let default_flag = black_box(Flags::TO_NOWHERE);

	let msg = Msg::debug(example_str);
	msg.print(0, Flags::NONE);
	c.bench_function("fyi_msg::Msg::debug(\"This is...\")/print(0, 0)", move |b| {
		b.iter(|| msg.print(zero_u8, default_flag))
	});
	println!("");
	println!("");

	let msg = Msg::debug(example_str);
	msg.print(1, Flags::NONE);
	c.bench_function("fyi_msg::Msg::debug(\"This is...\")/print(1, 0)", move |b| {
		b.iter(|| msg.print(one_u8, default_flag))
	});
	println!("");
	println!("");

	let msg = Msg::debug(example_str);
	msg.print(2, Flags::NONE);
	c.bench_function("fyi_msg::Msg::debug(\"This is...\")/print(2, 0)", move |b| {
		b.iter(|| msg.print(two_u8, default_flag))
	});
	println!("");
	println!("");

	let msg = Msg::debug(example_str);
	msg.print(0, Flags::NO_ANSI);
	c.bench_function("fyi_msg::Msg::debug(\"This is...\")/print(0, Flags::NO_ANSI)", move |b| {
		b.iter(|| msg.print(two_u8, no_ansi_flag))
	});
	println!("");
	println!("");

	let msg = Msg::debug(example_str);
	msg.print(0, Flags::TIMESTAMPED);
	c.bench_function("fyi_msg::Msg::debug(\"This is...\")/print(0, Flags::TIMESTAMPED)", move |b| {
		b.iter(|| msg.print(two_u8, timestamped_flag))
	});
	println!("");
	println!("");
}

criterion_group!(
	benches,
	new,
	print,
);
criterion_main!(benches);
