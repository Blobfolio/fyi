/*!
# Benchmark: `fyi_menu::ArgList`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::ArgList;



fn peek_first(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("peek_first() <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(Vec::<String>::new()),
			|al| { let _ = al.peek_first(); }
		)
	});

	group.bench_function("peek_first() <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|al| { let _ = al.peek_first(); }
		)
	});

	group.finish();
}

fn extract_switch(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("extract_switch(&[-p]) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_switch(&["-p"]); }
		)
	});

	group.bench_function("extract_switch(&[-v]) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_switch(&["-v"]); }
		)
	});

	group.bench_function("extract_switch(&[-v, --version]) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_switch(&["-v", "--version"]); }
		)
	});

	group.finish();
}

fn extract_opt(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("extract_opt(&[-p]) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_opt(&["-p"]); }
		)
	});

	group.bench_function("extract_opt(&[-v]) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_opt(&["-v"]); }
		)
	});

	group.bench_function("extract_opt(&[-v, --version]) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.extract_opt(&["-v", "--version"]); }
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	peek_first,
	extract_switch,
	extract_opt,
);
criterion_main!(benches);
