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



criterion_group!(
	benches,
	build,
	regex,
	filter,
);
criterion_main!(benches);
