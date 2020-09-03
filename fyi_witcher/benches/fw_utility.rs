/*!
# Benchmark: `fyi_witcher::utility`
*/

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witcher::utility;



fn count_nl(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");

	fn data() -> &'static [u8] {
		"Witcher Demo: Gzipped MAN Pages
[00:00:01]  [#######---------------------------------------------------------------------------------------------------------------------------------------------------------------------------]  112/2,822  3.97%
    ↳ /usr/share/man/it/man8/apt-cache.8.gz
    ↳ /usr/share/man/man3/HTML::AsSubs.3pm.gz
    ↳ /usr/share/man/man8/tc-bfifo.8.gz
    ↳ /usr/share/man/tr/man8/groupmod.8.gz
    ↳ /usr/share/man/man3/B::Hooks::OP::Check.3pm.gz
    ↳ /usr/share/man/man1/jpegexiforient.1.gz
    ↳ /usr/share/man/man1/bzip2.1.gz
    ↳ /usr/share/man/man8/devlink-region.8.gz\n".as_bytes()
	}

	group.bench_function(r"count_nl", move |b| {
		b.iter_with_setup(||
			data(),
			|b| utility::count_nl(b)
		)
	});

	group.finish();
}

fn ends_with_ignore_ascii_case(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");

	for paths in [
		b"/home/user/Pictures/file01.jpg",
		b"/home/user/Pictures/file01.JPG",
		b"/home/user/Pictures/file01.png",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"ends_with_ignore_ascii_case({}, .jpg)",
				unsafe { std::str::from_utf8_unchecked(&paths[..]) },
			)),
			paths,
			|b, &paths| {
				b.iter(||
					utility::ends_with_ignore_ascii_case(paths, b".jpg")
				);
			}
		);
	}

	group.finish();
}

fn fitted_range(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");

	for txt in [
		"Hello World",
		"\x1b[1;31mHello\x1b[0m World",
		"Björk Guðmundsdóttir",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"fitted_range({}, 20)",
				txt,
			)),
			txt.as_bytes(),
			|b, txt| {
				b.iter(||
					utility::fitted_range(txt, 20)
				);
			}
		);
	}

	group.finish();
}

fn secs_chunks(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_witcher::utility");

	for secs in [10, 113, 10502].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!(
				"secs_chunks({})",
				secs,
			)),
			secs,
			|b, &secs| {
				b.iter(||
					utility::secs_chunks(secs)
				);
			}
		);
	}

	group.finish();
}



criterion_group!(
	benches,
	count_nl,
	ends_with_ignore_ascii_case,
	fitted_range,
	secs_chunks,
);
criterion_main!(benches);
