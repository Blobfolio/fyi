/*!
# Benchmark: `fyi_witcher::Witcher`
*/

use criterion::{
	black_box,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::Witcher;
use std::ffi::OsStr;
use std::path::PathBuf;



fn build(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");
	group.sample_size(50);

	group.bench_function("with_path(/usr/share/man).build()", move |b| {
		b.iter(|| {
			let _ = black_box(
				Witcher::default()
					.with_path("/usr/share/man")
					.build()
			);
		})
	});

	group.finish();
}

fn regex(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");
	group.sample_size(50);

	group.bench_function(r"with_regex((?i).+\.gz$).with_path(/usr/share/man).build()", move |b| {
		b.iter(||{
			let _ = black_box(
				Witcher::default()
					.with_regex(r"(?i).+\.gz$")
					.with_path("/usr/share/man")
					.build()
			);
		})
	});

	group.finish();
}

fn filter(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");
	group.sample_size(50);

	fn cb(path: &PathBuf) -> bool {
		let bytes: &[u8] = unsafe { &*(path.as_os_str() as *const OsStr as *const [u8]) };
		bytes.len() > 3 && bytes[bytes.len()-3..].eq_ignore_ascii_case(b".gz")
	}

	group.bench_function(r"with_filter(cb).with_path(/usr/share/man).build()", move |b| {
		b.iter(|| {
			let _ = black_box(
				Witcher::default()
					.with_filter(cb)
					.with_path("/usr/share/man")
					.build()
			);
		})
	});

	group.finish();
}

fn with_ext(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");
	group.sample_size(30);

	group.bench_function(r"with_ext(.jpg).with_path(/usr/share).build()", move |b| {
		b.iter(|| {
			let _ = black_box(
				Witcher::default()
					.with_ext(b".jpg")
					.with_path("/usr/share")
					.build()
				);
		});
	});

	group.finish();
}



criterion_group!(
	benches,
	build,
	filter,
	regex,
	with_ext,
);
criterion_main!(benches);
