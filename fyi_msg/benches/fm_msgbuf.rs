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
use fyi_msg::MsgBuf;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_function("default()", move |b| {
		b.iter(|| MsgBuf::default())
	});

	group.bench_function("splat(5)", move |b| {
		b.iter(|| MsgBuf::splat(black_box(5_usize)))
	});

	group.bench_with_input(
		BenchmarkId::from_parameter("new(b\"Twinkle Twinkle...\", [55])"),
		b"Twinkle twinkle little star, how I wonder what you are!",
		|b, &buf| {
			b.iter(|| MsgBuf::new(&buf, black_box(&[55_usize])));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new(b\"Twinkle Twinkle...\", [15, 13, 27])"),
		b"Twinkle twinkle little star, how I wonder what you are!",
		|b, &buf| {
			b.iter(|| MsgBuf::new(&buf, black_box(&[55_usize])));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("new(b\"Twinkle Twinkle...\", [15, 13])"),
		b"Twinkle twinkle little star, how I wonder what you are!",
		|b, &buf| {
			b.iter(|| MsgBuf::new(&buf, black_box(&[55_usize])));
		}
	);

	group.finish();
}

fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_with_input(
		BenchmarkId::from_parameter("from(b\"Hello World\")"),
		b"Hello World",
		|b, &buf| {
			b.iter(|| MsgBuf::from(&buf));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("from(b\"Twinkle Twinkle...\")"),
		b"Twinkle twinkle little star, how I wonder what you are!",
		|b, &buf| {
			b.iter(|| MsgBuf::from(&buf));
		}
	);

	group.finish();
}

fn from_many(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_with_input(
		BenchmarkId::from_parameter("from_many(1)"),
		&[
			&b"Don't you know what duplicates are?"[..],
		],
		|b, &bufs| {
			b.iter(|| MsgBuf::from_many(&bufs));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("from_many(2)"),
		&[
			&b"Don't you know "[..],
			&b"what duplicates are?"[..],
		],
		|b, &bufs| {
			b.iter(|| MsgBuf::from_many(&bufs));
		}
	);

	group.bench_with_input(
		BenchmarkId::from_parameter("from_many(6)"),
		&[
			&b"Don't "[..],
			&b"you "[..],
			&b"know "[..],
			&b"what "[..],
			&b"duplicates "[..],
			&b"are?"[..],
		],
		|b, &bufs| {
			b.iter(|| MsgBuf::from_many(&bufs));
		}
	);

	group.finish();
}

fn add_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_function("add_part([])", move |b| {
		b.iter_with_setup(||
			MsgBuf::default(),
			|mut parts| parts += black_box(&[])
		)
	});

	group.bench_function("add_part(b\"foobar\")", move |b| {
		b.iter_with_setup(||
			MsgBuf::default(),
			|mut parts| parts += black_box(b"foobar")
		)
	});

	group.finish();
}

fn clear_part(c: &mut Criterion) {
	// Some strings of different lengths.
	const SM1: &[u8] = b"cat";
	const MD1: &[u8] = b"pencil";
	const LG1: &[u8] = b"dinosaurs";

	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_function("[1].clear_part(1)", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(LG1),
			|mut parts| parts.clear_part(black_box(1))
		)
	});

	for i in 1..4 {
		group.bench_function(format!("[3].clear_part({})", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&[SM1, MD1, LG1]),
				|mut parts| parts.clear_part(black_box(i))
			)
		});
	}

	group.finish();
}

fn remove_part(c: &mut Criterion) {
	// Some strings of different lengths.
	const SM1: &[u8] = b"cat";
	const MD1: &[u8] = b"pencil";
	const LG1: &[u8] = b"dinosaurs";

	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	group.bench_function("[1].remove_part(1)", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(LG1),
			|mut parts| parts.remove_part(black_box(1))
		)
	});

	for i in 1..4 {
		group.bench_function(format!("[3].remove_part({})", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&[SM1, MD1, LG1]),
				|mut parts| parts.remove_part(black_box(i))
			)
		});
	}

	group.finish();
}

fn replace_part(c: &mut Criterion) {
	// Some strings of different lengths.
	const SM1: &[u8] = b"cat";
	const SM2: &[u8] = b"dog";
	const MD1: &[u8] = b"pencil";
	const MD2: &[u8] = b"yellow";
	const LG1: &[u8] = b"dinosaurs";
	const LG2: &[u8] = b"arcosaurs";

	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	// Replace smaller, same, bigger.
	for buf in [SM1, MD1, LG1].iter() {
		group.bench_function(format!(
			"[\"yellow\"].replace_part(1, {:?})",
			unsafe { std::str::from_utf8_unchecked(buf) }
		), move |b| {
			b.iter_with_setup(||
				MsgBuf::from(MD2),
				|mut parts| parts.replace_part(black_box(1), black_box(buf))
			)
		});
	}

	// Same thing, but from the middle.
	for buf in [SM1, MD1, LG1].iter() {
		group.bench_function(format!(
			"[..., \"yellow\", ...].replace_part(2, {:?})",
			unsafe { std::str::from_utf8_unchecked(buf) }
		), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&[SM2, MD2, LG2]),
				|mut parts| parts.replace_part(black_box(2), black_box(buf))
			)
		});
	}

	group.finish();
}



criterion_group!(
	benches,
	new,
	from,
	from_many,
	add_part,
	clear_part,
	remove_part,
	replace_part,
);
criterion_main!(benches);
