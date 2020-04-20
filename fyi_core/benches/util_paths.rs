extern crate criterion;

use criterion::{
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn paths_file_extension(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::file_extension");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::file_extension(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_file_name(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::file_name");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::file_name(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_file_size(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::file_size");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::file_size(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_is_executable(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::is_executable");
	for path in [
		"tests",
		"tests/assets/file.txt",
		"tests/assets/is-executable.sh",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::is_executable(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_parent(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::parent");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::parent(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_to_path_buf_abs(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::to_path_buf_abs");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::to_path_buf_abs(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_to_string(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::to_string");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::to_string(path)
				);
			}
		);
	}
	group.finish();
}

fn paths_to_string_abs(c: &mut Criterion) {
	let mut group = c.benchmark_group("util::paths::to_string_abs");
	for path in [
		"tests",
		"src/lib.rs",
		"tests/assets/file.txt",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(path),
			path,
			|b, &path| {
				b.iter(||
					fyi_core::util::paths::to_string_abs(path)
				);
			}
		);
	}
	group.finish();
}



criterion_group!(
	benches,
	paths_file_extension,
	paths_file_name,
	paths_file_size,
	paths_is_executable,
	paths_parent,
	paths_to_path_buf_abs,
	paths_to_string,
	paths_to_string_abs,
);
criterion_main!(benches);
