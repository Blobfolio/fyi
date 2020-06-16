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
use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;



fn new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	let paths = black_box([ "/usr/share/man/man5" ]);
	let regex = black_box(r"(?i)\.gz$");

	group.bench_function("new(/usr/share/man/man5, *.gz)", move |b| {
		b.iter(|| Witcher::new(&paths, regex))
	});

	group.finish();
}

pub fn custom_cb (p: Result<jwalk::DirEntry<((), ())>, jwalk::Error>) -> Option<PathBuf> {
	// Skip errors, duh.
	if let Ok(path) = p {
		// We don't want directories.
		if path.file_type().is_dir() { None }
		// We need to canonicalize again because symlinks might
		// not actually be living with the parent directory.
		else if let Ok(path) = fs::canonicalize(&path.path()) {
			// Instead of Regex, let's do a simple `ends_with()` test. Not the
			// most robust approach, but a good demonstration of the sort of
			// thing one might wish to do.
			unsafe {
				let p_str: *const OsStr = path.as_os_str();
				if (&*(p_str as *const [u8])).ends_with(&[46, 103, 122]) { Some(path) }
				else { None }
			}
		}
		else { None }
	}
	else { None }
}

fn custom(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	let paths = black_box([ "/usr/share/man/man5" ]);

	unsafe {
		group.bench_function("custom(/usr/share/man/man5, fn<>)", move |b| {
			b.iter(|| Witcher::custom(&paths, custom_cb))
		});
	}

	group.finish();
}

fn simple(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::Witcher");

	let paths = black_box([ "/usr/share/man/man5" ]);

	group.bench_function("simple(/usr/share/man/man5)", move |b| {
		b.iter(|| Witcher::simple(&paths))
	});

	group.finish();
}



criterion_group!(
	benches,
	new,
	custom,
	simple,
);
criterion_main!(benches);
