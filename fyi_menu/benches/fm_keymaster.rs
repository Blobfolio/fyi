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

	group.bench_function("default(7).insert(Hello World, 2)", move |b| {
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
				k
			},
			|mut k| k.insert("Hello World", 2)
		)
	});

	group.finish();
}

fn contains(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyMaster");

	group.bench_function(".contains(0/8)", move |b| {
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
				k
			},
			|k| k.contains("This")
		)
	});

	group.bench_function(".contains(5/8)", move |b| {
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
				k
			},
			|k| k.contains("That")
		)
	});

	group.bench_function(".contains(-/8)", move |b| {
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
				k
			},
			|k| k.contains("Missing")
		)
	});

	group.finish();
}

fn simd(c: &mut Criterion) {
	use packed_simd::u16x32;
	let mut group = c.benchmark_group("simd");

	fn arr_add5(src: &mut [u16], idx: usize) {
		src.iter_mut().skip(idx * 2 + 1).for_each(|x| *x += 5);
	}

	fn simd_add5(src: &mut u16x32, idx: usize) {
		*src += match idx {
			0 => u16x32::new(0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			1 => u16x32::new(0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			2 => u16x32::new(0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			3 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			4 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			5 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			6 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			7 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			8 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			9 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			10 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			11 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 5),
			12 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5),
			13 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5),
			14 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5),
			15 => u16x32::new(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5),
			_ => panic!("Out of range!"),
		};
	}

	group.bench_function("array add", move |b| {
		b.iter(|| {
			let mut rg = [0_16, 0, 11, 19, 46, 46, 55, 55, 64, 64, 84, 85, 101, 102, 110, 115, 120, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
			arr_add5(&mut rg, 4);
			rg
		})
	});

	group.bench_function("simd add", move |b| {
		b.iter(|| {
			let mut rg = u16x32::new(0_16, 0, 11, 19, 46, 46, 55, 55, 64, 64, 84, 85, 101, 102, 110, 115, 120, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
			simd_add5(&mut rg, 4);
			rg
		})
	});

	group.finish();
}



criterion_group!(
	benches,
	insert,
	contains,
	simd
);
criterion_main!(benches);
