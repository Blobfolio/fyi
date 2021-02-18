/*!
# Benchmark: `fyi_num::nice_u32`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_num::NiceU32;
use std::time::Duration;

benches!(
	Bench::new("fyi_num::NiceU32", "from(0)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(0_u32)),

	Bench::new("fyi_num::NiceU32", "from(100_020)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(100_020_u32)),

	Bench::new("fyi_num::NiceU32", "from(6_330_004)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(6_330_004_u32)),

	Bench::new("fyi_num::NiceU32", "from(57_444_000)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(57_444_000_u32)),

	Bench::new("fyi_num::NiceU32", "from(777_804_132)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(777_804_132_u32)),

	Bench::new("fyi_num::NiceU32", "from(u32::MAX)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU32::from(u32::MAX))
);
