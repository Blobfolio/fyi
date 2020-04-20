extern crate criterion;

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn strings_chars_len(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::chars_len");

	for text in [
		"Hello World",
		"Björk Guðmundsdóttir",
		"\x1B[1mBjörk\x1B[0m",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(text),
			text,
			|b, &text| {
				b.iter(||
					fyi_core::util::strings::chars_len(text)
				);
			}
		);
	}
	group.finish();
}

fn strings_inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::inflect");

	let singular = black_box("book");
	let plural = black_box("books");

	for size in [0, 1, 2, 1000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::inflect(size, singular, plural)
				);
			}
		);
	}
	group.finish();
}

fn strings_lines_len(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::lines_len");

	for (k, text) in [
		"",
		"Hello World",
		"Hello\nWorld",
		"Hello\nWorld\n",
	].iter().enumerate() {
		group.bench_with_input(
			BenchmarkId::from_parameter(k),
			text,
			|b, &text| {
				b.iter(||
					fyi_core::util::strings::lines_len(text)
				);
			}
		);
	}
	group.finish();
}

fn strings_oxford_join(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::oxford_join");

	let data = black_box(vec![
		"apples".to_string(),
		"bananas".to_string(),
		"carrots".to_string(),
	]);

	let glue = black_box("and");

	for size in (0..4).into_iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			&size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::oxford_join(&data[0..size], glue)
				);
			}
		);
	}
	group.finish();
}

fn strings_shorten(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::shorten");

	for pair in [
		("Hello World", 0),
		("Hello World", 6),
		("Hello World", 11),
		("Björk Guðmundsdóttir", 10),
		("Björk Guðmundsdóttir", 20),
	].iter() {
		let (text, width) = pair;

		group.bench_with_input(
			BenchmarkId::from_parameter(&format!("{}/{}", text, *width)),
			pair,
			|b, &(t, w)| {
				b.iter(||
					fyi_core::util::strings::shorten(t, w)
				);
			}
		);
	}
	group.finish();
}

fn strings_shorten_reverse(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::shorten_reverse");

	for pair in [
		("Hello World", 0),
		("Hello World", 6),
		("Hello World", 11),
		("Björk Guðmundsdóttir", 10),
		("Björk Guðmundsdóttir", 20),
	].iter() {
		let (text, width) = pair;

		group.bench_with_input(
			BenchmarkId::from_parameter(&format!("{}/{}", text, *width)),
			pair,
			|b, &(t, w)| {
				b.iter(||
					fyi_core::util::strings::shorten_reverse(t, w)
				);
			}
		);
	}
	group.finish();
}

fn strings_strip_ansi(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::strip_ansi");

	for text in [
		"Björk Guðmundsdóttir",
		"\x1B[1mBjörk\x1B[0m Guðmundsdóttir",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(text),
			text,
			|b, &text| {
				b.iter(||
					fyi_core::util::strings::strip_ansi(text)
				);
			}
		);
	}
	group.finish();
}

fn strings_whitespace(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::whitespace");
	for size in [0, 50, 100, 200].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					fyi_core::util::strings::whitespace(size)
				);
			}
		);
	}
	group.finish();
}

fn strings_width(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::strings::width");

	for text in [
		"Hello World",
		"Björk Guðmundsdóttir",
		"\x1B[1mBjörk\x1B[0m",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(text),
			text,
			|b, &text| {
				b.iter(||
					fyi_core::util::strings::width(text)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(
	benches,
	strings_chars_len,
	strings_inflect,
	strings_lines_len,
	strings_oxford_join,
	strings_shorten,
	strings_shorten_reverse,
	strings_strip_ansi,
	strings_whitespace,
	strings_width,
);
criterion_main!(benches);
