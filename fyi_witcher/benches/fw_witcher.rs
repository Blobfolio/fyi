/*!
# Benchmark: `fyi_witcher`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_witcher::{
	witch,
	Witcher,
};
use std::time::Duration;

benches!(
	Bench::new("fyi_witcher", "witch(/usr/share)")
		.timed(Duration::from_secs(5))
		.with(|| witch(&["/usr/share"])),

	Bench::new("fyi_witcher::Witcher", "with_path(/usr/share)")
		.timed(Duration::from_secs(5))
		.with(|| Witcher::default().with_path("/usr/share").build())
);
