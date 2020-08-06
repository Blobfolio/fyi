/*!
# Benchmark: `fyi_menu`
*/

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};
use fyi_menu::KeyKind;



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



criterion_group!(
	benches,
	esc_arg,
	keykind_from,
	parse_args,
);
criterion_main!(benches);
