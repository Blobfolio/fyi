/*!
# Benchmark: `fyi_menu::Argue`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_menu::Argue;
use std::time::Duration;

/// # Separate Key/Value entries.
fn split() -> Vec<String> {
	vec![
		String::from("print"),
		String::from("--prefix"),
		String::from("Hello"),
		String::from("-c"),
		String::from("199"),
		String::from("-t"),
		String::from("A penny saved is a penny earned."),
	]
}

/// # Merged Key/Value entries.
fn joined() -> Vec<String> {
	vec![
		String::from("print"),
		String::from("--prefix=Hello"),
		String::from("-c199"),
		String::from("-t"),
		String::from("A penny saved is a penny earned."),
	]
}

/// # Merged Key/Value entries, padded to match `split()` length.
fn joined2() -> Vec<String> {
	vec![
		String::from("print"),
		String::from("--prefix=Hello"),
		String::from("-c199"),
		String::from("-t"),
		String::from("more"),
		String::from("more"),
		String::from("A penny saved is a penny earned."),
	]
}

benches!(
	Bench::new("fyi_menu::Argue", "from(-k Val)")
		.timed(Duration::from_secs(2))
		.with_setup(split(), |v| Argue::from(v)),

	Bench::new("fyi_menu::Argue", "from(-kVal)")
		.timed(Duration::from_secs(2))
		.with_setup(joined(), |v| Argue::from(v)),

	Bench::new("fyi_menu::Argue", "from(-kVal + padding)")
		.timed(Duration::from_secs(2))
		.with_setup(joined2(), |v| Argue::from(v)),

	Bench::new("fyi_menu::Argue", "switch(-c)")
		.timed(Duration::from_secs(1))
		.with_setup_ref(Argue::from(split()), |v| v.switch("-c")),

	Bench::new("fyi_menu::Argue", "switch2(-c, --color)")
		.timed(Duration::from_secs(1))
		.with_setup_ref(Argue::from(split()), |v| v.switch2("-c", "--color")),

	Bench::new("fyi_menu::Argue", "option(-c)")
		.timed(Duration::from_secs(1))
		.with_setup_ref(Argue::from(split()), |v| v.option("-c").is_some()),

	Bench::new("fyi_menu::Argue", "option2(-c, --color)")
		.timed(Duration::from_secs(1))
		.with_setup_ref(Argue::from(split()), |v| v.option2("-c", "--color").is_some())
);
