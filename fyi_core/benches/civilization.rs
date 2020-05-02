extern crate criterion;

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_core::traits::{
	AnsiBitsy,
	Elapsed,
	Inflection,
	MebiSaved,
	OxfordGlue,
	OxfordJoin,
	Shorty,
	ToMebi,
};



fn ansi_bitsy_chars_len(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::AnsiBitsy.chars_len");

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
					text.chars_len()
				);
			}
		);
	}
	group.finish();
}

fn ansi_bitsy_lines_len(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::AnsiBitsy::lines_len");

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
					text.lines_len()
				);
			}
		);
	}
	group.finish();
}

fn ansi_bitsy_strip_ansi(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::AnsiBitsy::strip_ansi");

	for text in [
		"Björk Guðmundsdóttir",
		"\x1B[1mBjörk\x1B[0m Guðmundsdóttir",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(text),
			text,
			|b, &text| {
				b.iter(||
					text.strip_ansi()
				);
			}
		);
	}
	group.finish();
}

fn ansi_bitsy_width(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::AnsiBitsy::width");

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
					text.width()
				);
			}
		);
	}
	group.finish();
}

fn elapsed_elapsed_chunks(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::spacetime::Elapsed::elapsed_chunks");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					size.elapsed_chunks()
				);
			}
		);
	}
	group.finish();
}

fn elapsed_elapsed(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::spacetime::Elapsed::elapsed");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		println!("\x1B[96m{}\x1B[0m", size.elapsed());

		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					size.elapsed()
				);
			}
		);
	}
	group.finish();
}

fn elapsed_elapsed_short(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::spacetime::Elapsed::elapsed_short");
	for size in [0, 50, 100, 10000, 1000000].iter() {
		println!("\x1B[96m{}\x1B[0m", size.elapsed_short());

		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					size.elapsed_short()
				);
			}
		);
	}
	group.finish();
}

fn inflect(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::Inflection::inflect");

	let singular = black_box("book");
	let plural = black_box("books");

	for size in [0, 1, 2, 1000].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					size.inflect(singular, plural)
				);
			}
		);
	}
	group.finish();
}

fn mebi_saved(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::Mebi::saved");
	for size in [(0, 500), (500, 500), (1000, 500)].iter() {
		// Come up with a reasonable name.
		let name: String = {
			let (before, after) = size;
			[
				before.to_string(),
				"»".to_string(),
				after.to_string()
			].concat()
		};

		group.bench_with_input(
			BenchmarkId::from_parameter(name),
			size,
			|b, &(before, after)| {
				b.iter(||
					before.saved(after)
				);
			}
		);
	}
	group.finish();
}

fn mebi_to_mebi(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::Mebi::to_mebi");
	for size in [0u64, 999u64, 1000u64, 499288372u64, 99389382145u64].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			size,
			|b, &size| {
				b.iter(||
					size.to_mebi()
				);
			}
		);
	}
	group.finish();
}

fn oxford_join(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::OxfordJoin.oxford_join");

	let data = black_box(vec![
		"apples",
		"bananas",
		"carrots",
		"dates",
	]);

	for size in (0..5).into_iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(size),
			&size,
			|b, &size| {
				b.iter(||
					data[0..size].oxford_join(&OxfordGlue::And)
				);
			}
		);
	}
	group.finish();
}

fn shorty_shorten(c: &mut Criterion) {
	let mut group = c.benchmark_group("traits::Shorty::shorten");

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
					t.shorten(w)
				);
			}
		);
	}
	group.finish();
}

fn shorty_shorten_reverse(c: &mut Criterion) {
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
					t.shorten_reverse(w)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(
	benches,
	ansi_bitsy_chars_len,
	ansi_bitsy_lines_len,
	ansi_bitsy_strip_ansi,
	ansi_bitsy_width,
	elapsed_elapsed,
	elapsed_elapsed_chunks,
	elapsed_elapsed_short,
	inflect,
	mebi_saved,
	mebi_to_mebi,
	oxford_join,
	shorty_shorten,
	shorty_shorten_reverse,
);
criterion_main!(benches);
