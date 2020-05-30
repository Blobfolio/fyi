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
use packed_simd::usizex2;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::MsgBuf");

	let blank_part: &[usizex2] = black_box(&[]);
	let hello_buf = black_box(b"Hello World");
	let hello_part = black_box(usizex2::new(0, 11));
	let hello_2parts = black_box([usizex2::new(0, 6), usizex2::new(6, 11)]);

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
		b.iter(|| MsgBuf::new(hello_buf, &hello_2parts))
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
}



criterion_group!(
	benches,
	new,
	from,
	from_many,
	with_parts,
	replace_part,
);
criterion_main!(benches);
