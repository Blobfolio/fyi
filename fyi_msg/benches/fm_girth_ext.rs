/*!
# Benchmark: `fyi_msg::traits::GirthExt`
*/

extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_msg::traits::GirthExt;



fn count_chars(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::GirthExt");

	for text in [
		"",
		"Hello",
		"Björk Guðmundsdóttir",
		"This is a slightly longer sentence, but nothing crazy.",
	].iter() {
		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{:?}.count_chars()",
				text
			)),
			text.as_bytes(),
			|b, text| {
				b.iter(||
					text.count_chars()
				);
			}
		);
	}
}

fn count_lines(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::GirthExt");

	for text in [
		"",
		"Hello",
		"Hello\nWorld",
		"Hello\n\nWorld",
	].iter() {
		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{:?}.count_lines()",
				text
			)),
			text.as_bytes(),
			|b, text| {
				b.iter(||
					text.count_lines()
				);
			}
		);
	}
}

fn count_width(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::traits::GirthExt");

	for text in [
		"",
		"Hello",
		"\x1b[1mHello",
		"\x1b[1mBjörk Guðmundsdóttir",
		"This is a \x1b[1mslightly\x1b[0m longer sentence, but nothing crazy.",
	].iter() {
		// Test the [u8] implementation as that's the focus; str performance is
		// comparable.
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"{:?}.count_width()",
				text
			)),
			text.as_bytes(),
			|b, text| {
				b.iter(||
					text.count_width()
				);
			}
		);
	}
}

criterion_group!(
	benches,
	count_chars,
	count_lines,
	count_width,
);
criterion_main!(benches);
