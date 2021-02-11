/*!
# Benchmark: `fyi_num::nice_elapsed`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_num::NiceElapsed;
use std::time::Duration;

benches!(
	Bench::new("fyi_num::NiceElapsed", "hms(10)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::hms(10_u32)),

	Bench::new("fyi_num::NiceElapsed", "hms(113)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::hms(113_u32)),

	Bench::new("fyi_num::NiceElapsed", "hms(10502)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::hms(10502_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(1)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(1_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(50)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(50_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(100)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(100_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(2121)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(2121_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(37732)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(37732_u32)),

	Bench::new("fyi_num::NiceElapsed", "from(428390)")
		.timed(Duration::from_secs(1))
		.with(|| NiceElapsed::from(428390_u32))
);
