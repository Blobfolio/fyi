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

	group.bench_function("[].add_part(\"foobar\")", move |b| {
		b.iter_with_setup(||
			MsgBuf::default(),
			|mut parts| parts.add_part(black_box(b"foobar"))
		)
	});

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
	remove_part,
	replace_part,
);
criterion_main!(benches);



/*
fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	let blank_part: &[(usize, usize)] = black_box(&[]);
	let hello_buf = black_box(b"Hello World");
	let hello_part = black_box((0, 11));
	let hello_2parts = black_box(&[(0, 6), (6, 11)]);

	group.bench_function("default()", move |b| {
		b.iter(|| MsgBuf::default())
	});

	group.bench_function("new(b\"Hello World\", [])", move |b| {
		b.iter(|| MsgBuf::new(hello_buf, blank_part))
	});

	group.bench_function("new(b\"Hello World\", [(0, 11)])", move |b| {
		b.iter(|| MsgBuf::new(hello_buf, &[hello_part]))
	});

	group.bench_function("new(b\"Hello World\", [(0, 6), (6, 11)])", move |b| {
		b.iter(|| MsgBuf::new(hello_buf, hello_2parts))
	});

	group.finish();
}

fn from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	let blank_buf: &[u8] = black_box(&[]);
	let hello_buf = black_box(b"Hello World");
	let bjork_buf = black_box("Björk Guðmundsdóttir is an Icelandic singer, songwriter, record producer, actress, and DJ.".as_bytes());

	for buf in [
		blank_buf,
		hello_buf,
		bjork_buf,
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("from({})", unsafe { std::str::from_utf8_unchecked(buf) })),
			buf,
			|b, &buf| {
				b.iter(||
					MsgBuf::from(buf)
				);
			}
		);
	}

	group.finish();
}

fn from_many(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	let blank_buf: &[u8] = black_box(&[]);
	let hello_buf = black_box(b"Hello World");
	let bjork_buf = black_box("Björk Guðmundsdóttir is an Icelandic singer, songwriter, record producer, actress, and DJ.".as_bytes());
	let ansi1_buf: &[u8] = black_box(&[27, 91, 49, 59, 57, 49, 109]);
	let ansi2_buf: &[u8] = black_box(&[69, 114, 114, 111, 114, 58]);
	let ansi3_buf: &[u8] = black_box(&[27, 91, 48, 109]);

	group.bench_function("from_many(3)", move |b| {
		b.iter(|| MsgBuf::from_many(&[
			blank_buf,
			hello_buf,
			bjork_buf,
		]))
	});

	group.bench_function("from_many(6)", move |b| {
		b.iter(|| MsgBuf::from_many(&[
			blank_buf,
			hello_buf,
			bjork_buf,
			ansi1_buf,
			ansi2_buf,
			ansi3_buf,
		]))
	});

	group.finish();
}

fn with_parts(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	for num in [
		1_usize,
		5_usize,
		10_usize,
		16_usize,
	].iter() {
		assert_eq!(MsgBuf::with_parts(*num).count_partitions(), *num);
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("with_parts({})", num)),
			num,
			|b, num| {
				b.iter(||
					MsgBuf::with_parts(*num)
				);
			}
		);
	}

	group.finish();
}

fn replace_part(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	// First with just one part.
	group.bench_function("replace_part::<1/1>() clear", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(b"The hills are alive with the sound of music."),
			|mut buf| buf.replace_part(black_box(0), black_box(b""))
		)
	});

	group.bench_function("replace_part::<1/1>() shorten", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(b"The hills are alive with the sound of music."),
			|mut buf| buf.replace_part(black_box(0), black_box(b"The hills have eyes!"))
		)
	});

	group.bench_function("replace_part::<1/1>() same-size", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(b"The hills are alive with the sound of music."),
			|mut buf| buf.replace_part(black_box(0), black_box(b"The hills prefer country-and-western, y'all."))
		)
	});

	group.bench_function("replace_part::<1/1>() extend", move |b| {
		b.iter_with_setup(||
			MsgBuf::from(b"The hills are alive with the sound of music."),
			|mut buf| buf.replace_part(black_box(0), black_box(b"The hills are a little bit howly if you ask me. No thank you!"))
		)
	});

	// Now try it out in each position of a 3-parted buffer.
	for i in 1..4 {
		group.bench_function(&format!("replace_part::<{}/3>() clear", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&vec![&b"The hills are alive with the sound of music."[..]; i][..]),
				|mut buf| buf.replace_part(black_box(i - 1), black_box(b""))
			)
		});

		group.bench_function(&format!("replace_part::<{}/3>() shorten", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&vec![&b"The hills are alive with the sound of music."[..]; i][..]),
				|mut buf| buf.replace_part(black_box(i - 1), black_box(b"The hills have eyes!"))
			)
		});

		group.bench_function(&format!("replace_part::<{}/3>() same-size", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&vec![&b"The hills are alive with the sound of music."[..]; i][..]),
				|mut buf| buf.replace_part(black_box(i - 1), black_box(b"The hills prefer country-and-western, y'all."))
			)
		});

		group.bench_function(&format!("replace_part::<{}/3>() extend", i), move |b| {
			b.iter_with_setup(||
				MsgBuf::from_many(&vec![&b"The hills are alive with the sound of music."[..]; i][..]),
				|mut buf| buf.replace_part(black_box(i - 1), black_box(b"The hills are a little bit howly if you ask me. No thank you!"))
			)
		});
	}

	group.finish();
}*/
