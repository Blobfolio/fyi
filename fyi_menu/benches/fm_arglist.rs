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
			vec![String::from("debug"), String::from("-t"), String::from("The current line count is 5.")],
			|raw| {
				let mut al = ArgList::from(raw);
				let _com = al.expect_command();
				let _help = al.pluck_help();
				let _t = al.pluck_switch(|x| x != "-t" && x != "--timestamp");
				let _i = al.pluck_switch(|x| x != "-i" && x != "--indent");
				let _stderr = al.pluck_switch(|x| x != "--stderr");
				let _e = al.pluck_opt_usize(|x| x == "-e" || x == "--exit").unwrap_or_default();
				let _msg = al.expect_arg();
			}
		)
	});

	group.finish();
}

fn peek(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("peek() <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(Vec::<String>::new()),
			|al| { let _ = al.peek(); }
		)
	});

	group.bench_function("peek() <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|al| { let _ = al.peek(); }
		)
	});

	group.finish();
}

fn pluck_switch(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("pluck_switch(-p) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_switch(|x| x != "-p"); }
		)
	});

	group.bench_function("pluck_switch(-v) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_switch(|x| x != "-v"); }
		)
	});

	group.bench_function("pluck_switch(-v, --version) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_switch(|x| x != "-v" && x != "--version"); }
		)
	});

	group.finish();
}

fn pluck_opt(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("pluck_opt(-p) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_opt(|x| x == "-p"); }
		)
	});

	group.bench_function("pluck_opt(-v) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_opt(|x| x == "-v"); }
		)
	});

	group.bench_function("pluck_opt(-v, --version) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| { let _ = al.pluck_opt(|x| x == "-v" || x == "--version"); }
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	practical,
	peek,
	pluck_switch,
	pluck_opt,
);
criterion_main!(benches);
