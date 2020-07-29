/*!
# Benchmark: `fyi_menu::ArgList`
*/

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::{
	ArgList,
	KeyKind,
};



fn keykind_from(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::KeyKind");

	for kv in [
		&b"Hello World"[..],
		&b"--key"[..],
		&b"-k"[..],
		&b"-kValue"[..],
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!(
				"from({})",
				unsafe { std::str::from_utf8_unchecked(kv) }
			)),
			kv,
			|b, &kv| {
				b.iter(||
					KeyKind::from(kv)
				);
			}
		);
	}

	group.finish();
}

fn esc_arg(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::utility");

	for kv in [
		"",
		"Hello",
		"Hello World",
		"Hello Joe\'s World",
	].iter() {
		group.bench_with_input(
			BenchmarkId::from_parameter(&format!("esc_arg({:?})", kv.to_string())),
			&kv.to_string(),
			|b, kv| {
				b.iter_with_setup(||
					kv.to_string(),
					|s| fyi_menu::utility::esc_arg(s)
				);
			}
		);
	}

	group.finish();
}

fn arglist(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	const FLAG_HELP: u8 = 0b0001;
	const FLAG_TIME: u8 = 0b0010;
	const FLAG_ERR: u8  = 0b0100;
	const FLAG_INDT: u8 = 0b1000;

	group.bench_function("practical: fyi debug -t \"The current line count is 5.\"", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("debug"),
				String::from("-t"),
				String::from("The current line count is 5.")
			],
			|raw| {
				black_box(1);
				let mut al = ArgList::from(raw);
				let mut _flags: u8 = 0;
				let _com = al.expect_command();

				al.pluck_flags(
					&mut _flags,
					&[
						"-h", "--help",
						"-t", "--timestamp",
						"-i", "--indent",
						"--stderr",
					],
					&[
						FLAG_HELP, FLAG_HELP,
						FLAG_TIME, FLAG_TIME,
						FLAG_INDT, FLAG_INDT,
						FLAG_ERR,
					],
				);

				let _e = al.pluck_opt(|x| x == "-e" || x == "--exit").unwrap_or_default();
				let _msg = al.expect_arg();

				return al;
			}
		)
	});

	group.finish();
}

fn parse_args(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::parse_args");

	const FLAG_HELP: u8 = 0b0001;
	const FLAG_TIME: u8 = 0b0010;
	const FLAG_ERR: u8  = 0b0100;
	const FLAG_INDT: u8 = 0b1000;

	group.bench_function("practical: fyi debug -t \"The current line count is 5.\"", move |b| {
		b.iter_with_setup(||
			vec![
				String::from("debug"),
				String::from("-t"),
				String::from("The current line count is 5.")
			],
			|mut al| {
				black_box(1);
				fyi_menu::parse_args(&mut al, fyi_menu::FLAG_ALL);
				let _com = al.remove(0);
				let mut _flags: u8 = 0;

				let mut idx: usize = 0;
				let len: usize = al.len();

				while idx < len {
					match al[idx].as_str() {
						"-h" | "--help" => { _flags |= FLAG_HELP; },
						"-t" | "--timestamp" => { _flags |= FLAG_TIME; },
						"-i" | "--indent" => { _flags |= FLAG_INDT; },
						"--stderr" => { _flags |= FLAG_ERR; },
						"-e" | "--exit" => { idx += 1; },
						_ => { break; }
					}

					idx += 1;
				}

				let _msg = al.remove(idx);
				return al;
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

fn pluck_flags(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	const FLAG_H: u8 = 0b0001;
	const FLAG_S: u8 = 0b0010;

	group.bench_function("pluck_flags(-h --help --single) <Err>", move |b| {
		b.iter_with_setup(||
			(0_u8, ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"])),
			|(mut flags, mut al)| al.pluck_flags(&mut flags, &["-h", "--help", "--single"], &[FLAG_H, FLAG_H, FLAG_S])
		)
	});

	group.finish();
}

fn pluck_switch(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("pluck_switch(-p) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_switch(|x| x != "-p")
		)
	});

	group.bench_function("pluck_switch(-v) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_switch(|x| x != "-v")
		)
	});

	group.bench_function("pluck_switch(-v, --version) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_switch(|x| x != "-v" && x != "--version")
		)
	});

	group.finish();
}

fn pluck_opt(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_menu::ArgList");

	group.bench_function("pluck_opt(-p) <Err>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_opt(|x| x == "-p")
		)
	});

	group.bench_function("pluck_opt(-v) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_opt(|x| x == "-v")
		)
	});

	group.bench_function("pluck_opt(-v, --version) <Ok>", move |b| {
		b.iter_with_setup(||
			ArgList::from(vec!["-h", "--help", "-v", "1.0", "-b", "master", "--single", "/path/to/thing"]),
			|mut al| al.pluck_opt(|x| x == "-v" || x == "--version")
		)
	});

	group.finish();
}



criterion_group!(
	benches,
	keykind_from,
	esc_arg,

	arglist,
	parse_args,

	peek,
	pluck_flags,
	pluck_switch,
	pluck_opt,
);
criterion_main!(benches);
