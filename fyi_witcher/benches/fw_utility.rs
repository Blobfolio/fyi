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
	secs_chunks,
	ends_with_ignore_ascii_case,
);
criterion_main!(benches);
