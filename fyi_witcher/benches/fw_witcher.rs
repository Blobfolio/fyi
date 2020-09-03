/*!
# Benchmark: `fyi_witcher::Witcher`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::Witcher;
use std::ffi::OsStr;
use std::path::PathBuf;



fn build(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function("with_path(/usr/share/man).build()", move |b| {
		b.iter(||
			Witcher::default().with_path("/usr/share/man").build()
		)
	});

	group.finish();
}

fn regex(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function(r"with_regex((?i).+\.gz$).with_path(/usr/share/man).build()", move |b| {
		b.iter(||
			Witcher::default().with_regex(r"(?i).+\.gz$").with_path("/usr/share/man").build()
		)
	});

	group.finish();
}

fn filter(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	fn cb(path: &PathBuf) -> bool {
		let bytes: &[u8] = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) };
		bytes.len() > 3 && bytes[bytes.len()-3..].eq_ignore_ascii_case(b".gz")
	}

	group.bench_function(r"with_filter(cb).with_path(/usr/share/man).build()", move |b| {
		b.iter(||
			Witcher::default().with_filter(cb).with_path("/usr/share/man").build()
		)
	});

	group.finish();
}

fn with_ext1(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function(r"with_ext1(.jpg).with_path(/usr/share).build()", move |b| {
		b.iter(||
			Witcher::default().with_ext1(b".jpg").with_path("/usr/share").build()
		)
	});

	group.finish();
}

fn with_ext2(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function(r"with_ext2(.jpg, .png).with_path(/usr/share).build()", move |b| {
		b.iter(||
			Witcher::default().with_ext2(b".jpg", b".png").with_path("/usr/share").build()
		)
	});


	group.finish();
}

fn with_ext3(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	group.bench_function(r"with_ext3(.jpg, .png, .jpeg).with_path(/usr/share).build()", move |b| {
		b.iter(||
			Witcher::default().with_ext3(b".jpg", b".png", b".jpeg").with_path("/usr/share").build()
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	build,
	regex,
	with_ext1,
	with_ext2,
	with_ext3,
	filter,
);
criterion_main!(benches);
