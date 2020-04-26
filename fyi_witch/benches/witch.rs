extern crate criterion;

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_witch::Witch;
use regex::Regex;
use std::path::PathBuf;



fn witch_new_straight(c: &mut Criterion) {
	let mut group = c.benchmark_group("Witch::new_straight");
	for path in [
		vec![PathBuf::from("/usr/share/php")],
		vec![
			PathBuf::from("/usr/share/man/man5"),
			PathBuf::from("/usr/share/man/man7"),
		],
		vec![
			PathBuf::from("/usr/share/man/man5"),
			PathBuf::from("/usr/share/man/man7"),
			PathBuf::from("/usr/share/man/man8"),
		],
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", path.len())),
			path,
			|b, path| {
				b.iter(||
					Witch::new_straight(&path)
				);
			}
		);
	}
	group.finish();
}

fn witch_new_filtered(c: &mut Criterion) {
	let r = black_box(Regex::new(r"(?i).+\.(gz|php)$").unwrap());
	let mut group = c.benchmark_group("Witch::new_filtered");
	for path in [
		vec![PathBuf::from("/usr/share/php")],
		vec![
			PathBuf::from("/usr/share/man/man5"),
			PathBuf::from("/usr/share/man/man7"),
		],
		vec![
			PathBuf::from("/usr/share/man/man5"),
			PathBuf::from("/usr/share/man/man7"),
			PathBuf::from("/usr/share/man/man8"),
		],
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", path.len())),
			path,
			|b, path| {
				b.iter(||
					Witch::new_filtered(&path, &r)
				);
			}
		);
	}
	group.finish();
}

fn witch_du(c: &mut Criterion) {
	let witch = black_box(Witch::new_filtered(
		&[PathBuf::from("/usr/share/php")],
		&Regex::new(r"(?i).+\.php$").unwrap(),
	));

	c.bench_function("Witch::du", move |b| {
		b.iter(|| witch.du())
	});
}



criterion_group!(
	benches,
	witch_new_straight,
	witch_new_filtered,
	witch_du,
);
criterion_main!(benches);
