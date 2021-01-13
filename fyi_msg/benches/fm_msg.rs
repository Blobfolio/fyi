/*!
# Benchmark: `fyi_msg::Msg`
*/

use criterion::{
	BenchmarkId,
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
	group.sample_size(30);

	group.bench_function("default()", move |b| {
		b.iter(|| Msg::default())
	});

	group.bench_function("plain(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::plain(example_str))
	});

	group.bench_function("custom(\"Prefix:\", 199, \"This is an example message!\")", move |b| {
		b.iter(|| Msg::custom(prefix_str, one99_u8, example_str))
	});

	group.bench_function("error(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::error(example_str))
	});

	group.bench_function("MsgKind::Error.into_msg(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Error.into_msg(example_str))
	});

	group.bench_function("MsgKind::Debug.into_msg(\"This is an example message!\")", move |b| {
		b.iter(|| MsgKind::Debug.into_msg(example_str))
	});

	group.finish();
}

fn fit_width(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");
	group.sample_size(30);

	for ints in [
		20,
		40,
		50,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("fitted({})", ints)),
			ints,
			|b, &ints| {
				b.iter_with_setup(||
					(Msg::error("This is an example message!").with_newline(true), ints),
					|(msg, w)| {
						let _ = black_box(msg.fitted(w));
					}
				);
			}
		);
	}

	group.finish();
}

fn with_timestamp(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");
	group.sample_size(30);

	group.bench_function("with_timestamp()", move |b| {
		b.iter_with_setup(||
			MsgKind::Error.into_msg("The rain in spain is plain."),
			|m| m.with_timestamp(true)
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	new,
	fit_width,
	with_timestamp,
);
criterion_main!(benches);
