/*!
# Benchmark: `fyi_menu::ArgList`
*/

use criterion::{
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::ArgList;



fn practical(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("practical: fyi debug -t \"The current line count is 5.\"", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec![String::from("debug"), String::from("-t"), String::from("The current line count is 5.")]),
			|mut al| {
				let _com = al.expect_command();
				let _help = al.wants_help();
				let _t = al.extract_switch_cb(|x| x != "-t" && x != "--timestamp");
				let _i = al.extract_switch_cb(|x| x != "-i" && x != "--indent");
				let _stderr = al.extract_switch_cb(|x| x != "--stderr");
				let _e = al.extract_opt_usize_cb(|x| x == "-e" || x == "--exit").unwrap_or_default();
				let _msg = al.expect_arg();
			}
		)
	});

	group.finish();
}

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
	practical,
	peek_first,
	extract_switch,
	extract_opt,
);
criterion_main!(benches);
