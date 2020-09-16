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
	group.sample_size(50);

	group.bench_function("default()", move |b| {
		b.iter(|| Msg::default())
	});

	group.bench_function("new(\"This is an example message!\")", move |b| {
		b.iter(|| Msg::from(example_str))
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

fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");
	group.sample_size(50);

	group.bench_function("from::<Vec<u8>>::()", move |b| {
		b.iter_with_setup(||
			b"My dear aunt sally eats cake.".to_vec(),
			|v| Msg::from(v)
		)
	});

	group.bench_function("from::<[&[u8]; 3]>::()", move |b| {
		b.iter_with_setup(||
			[&b"My "[..], b"dear ", b"aunt."],
			|v| Msg::from(v)
		)
	});

	group.bench_function("from::<[&[u8]; 6]>::()", move |b| {
		b.iter_with_setup(||
			[&b"My "[..], b"dear ", b"aunt ", b"sally ", b"eats ", b"cake."],
			|v| Msg::from(v)
		)
	});

	group.finish();
}

fn with_timestamp(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg");
	group.sample_size(50);

	group.bench_function("with_timestamp()", move |b| {
		b.iter_with_setup(||
			MsgKind::Error.into_msg("The rain in spain is plain."),
			|m| m.with_timestamp(true)
		)
	});
}

fn msgkind(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgKind");
	group.sample_size(50);

	group.bench_function("new(Hello Dolly, 199)", move |b| {
		b.iter(||
			MsgKind::new("Hello Dolly", 199)
		)
	});
}



criterion_group!(
	benches,
	new,
	from,
	with_timestamp,
	msgkind,
);
criterion_main!(benches);
