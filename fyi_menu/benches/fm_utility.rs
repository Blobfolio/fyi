/*!
# Benchmark: `fyi_menu::utility`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_menu::utility::esc_arg;

benches!(
	Bench::new("fyi_menu::utility", "esc_arg(Hello)")
		.with(|| esc_arg(String::from("Hello"))),

	Bench::new("fyi_menu::utility", "esc_arg(Hello World)")
		.with(|| esc_arg(String::from("Hello World"))),

	Bench::new("fyi_menu::utility", "esc_arg(Eat at Joe's)")
		.with(|| esc_arg(String::from("Eat at Joe's")))
);
