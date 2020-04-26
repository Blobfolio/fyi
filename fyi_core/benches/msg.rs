extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn prefix_new(c: &mut Criterion) {
	let mut group = c.benchmark_group("Prefix::new");
	for pair in [
		("", 2),
		("Something", 199),
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", pair)),
			pair,
			|b, &(prefix, color)| {
				b.iter(||
					fyi_core::Prefix::new(prefix, color)
				);
			}
		);
	}
	group.finish();
}

fn prefix_prefix(c: &mut Criterion) {
	use fyi_core::Prefix;
	use fyi_core::traits::AnsiBitsy;

	let mut group = c.benchmark_group("Prefix::prefix");
	for prefix in [
		Prefix::None,
		Prefix::Error,
		Prefix::new("Custom", 199),
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&prefix.to_string().strip_ansi()),
			prefix,
			|b, prefix| {
				b.iter(||
					prefix.prefix()
				);
			}
		);
	}
	group.finish();
}

fn msg_new(c: &mut Criterion) {
	let mut group = c.benchmark_group("Msg::new");
	for msg in [
		"",
		"Here is a message nice and long!",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", msg)),
			msg,
			|b, &msg| {
				b.iter(||
					fyi_core::Msg::new(msg)
				);
			}
		);
	}
	group.finish();
}

fn msg_msg_straight(c: &mut Criterion) {
	let mut group = c.benchmark_group("Msg::msg");
	for (k, msg) in [
		fyi_core::Msg::new("No prefix."),
		fyi_core::Msg::new("Standard prefix.")
			.with_prefix(fyi_core::Prefix::Success),
		fyi_core::Msg::new("Custom prefix.")
			.with_prefix(fyi_core::Prefix::new("Testerosa", 199)),
		fyi_core::Msg::new("Indented.")
			.with_prefix(fyi_core::Prefix::Success)
			.with_indent(1),
	].iter().enumerate() {
		println!("{}", msg);

		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{}", k)),
			msg,
			|b, msg| {
				b.iter(||
					msg.msg()
				);
			}
		);
	}
	group.finish();
}

fn msg_msg_timestamped(c: &mut Criterion) {
	let mut group = c.benchmark_group("Msg::msg (timestamped)");
	for (k, msg) in [
		fyi_core::Msg::new("No prefix.")
			.with_flags(fyi_core::MSG_TIMESTAMP),
		fyi_core::Msg::new("Standard prefix.")
			.with_prefix(fyi_core::Prefix::Success)
			.with_flags(fyi_core::MSG_TIMESTAMP),
		fyi_core::Msg::new("Custom prefix.")
			.with_prefix(fyi_core::Prefix::new("Testerosa", 199))
			.with_flags(fyi_core::MSG_TIMESTAMP),
		fyi_core::Msg::new("Indented.")
			.with_prefix(fyi_core::Prefix::Success)
			.with_indent(1)
			.with_flags(fyi_core::MSG_TIMESTAMP),
	].iter().enumerate() {
		println!("{}", msg);

		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{}", k)),
			msg,
			|b, msg| {
				b.iter(||
					msg.msg()
				);
			}
		);
	}
	group.finish();
}


criterion_group!(
	benches,
	msg_new,
	msg_msg_straight,
	msg_msg_timestamped,
	prefix_new,
	prefix_prefix,
);
criterion_main!(benches);
