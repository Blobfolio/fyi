/*!
# Benchmark: `fyi_menu::KeyMaster`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::KeyMaster;



fn insert(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyMaster");

	group.bench_function("default().insert(Hello World, 2)", move |b| {
		b.iter_with_setup(||
			KeyMaster::default(),
			|mut k| k.insert("Hello World", 2)
		)
	});

	group.bench_function("default(10).insert(Hello World, 2)", move |b| {
		b.iter_with_setup(||
			{
				let mut k = KeyMaster::default();
				k.insert("This", 10);
				k.insert("Is", 20);
				k.insert("The", 30);
				k.insert("Song", 40);
				k.insert("That", 50);
				k.insert("Never", 60);
				k.insert("Ends", 70);
				k.insert("It", 80);
				k.insert("Goes", 90);
				k.insert("On", 100);
				k
			},
			|mut k| k.insert("Hello World", 2)
		)
	});

	group.finish();
}

fn contains(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyMaster");

	group.bench_function(".contains(0/10)", move |b| {
		b.iter_with_setup(||
			{
				let mut k = KeyMaster::default();
				k.insert("This", 10);
				k.insert("Is", 20);
				k.insert("The", 30);
				k.insert("Song", 40);
				k.insert("That", 50);
				k.insert("Never", 60);
				k.insert("Ends", 70);
				k.insert("It", 80);
				k.insert("Goes", 90);
				k.insert("On", 100);
				k
			},
			|k| k.contains("This")
		)
	});

	group.bench_function(".contains(5/10)", move |b| {
		b.iter_with_setup(||
			{
				let mut k = KeyMaster::default();
				k.insert("This", 10);
				k.insert("Is", 20);
				k.insert("The", 30);
				k.insert("Song", 40);
				k.insert("That", 50);
				k.insert("Never", 60);
				k.insert("Ends", 70);
				k.insert("It", 80);
				k.insert("Goes", 90);
				k.insert("On", 100);
				k
			},
			|k| k.contains("That")
		)
	});

	group.bench_function(".contains(-/10)", move |b| {
		b.iter_with_setup(||
			{
				let mut k = KeyMaster::default();
				k.insert("This", 10);
				k.insert("Is", 20);
				k.insert("The", 30);
				k.insert("Song", 40);
				k.insert("That", 50);
				k.insert("Never", 60);
				k.insert("Ends", 70);
				k.insert("It", 80);
				k.insert("Goes", 90);
				k.insert("On", 100);
				k
			},
			|k| k.contains("Missing")
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	insert,
	contains,
);
criterion_main!(benches);
