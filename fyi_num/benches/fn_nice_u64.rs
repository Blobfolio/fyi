/*!
# Benchmark: `fyi_num::nice_u64`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_num::NiceU64;
use std::time::Duration;

benches!(
	Bench::new("fyi_num::NiceU64", "from(0)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU64::from(0_u64)),

	Bench::new("fyi_num::NiceU64", "from(6_489_320_013)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU64::from(6_489_320_013_u64)),

	Bench::new("fyi_num::NiceU64", "from(42_489_320_013)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU64::from(42_489_320_013_u64)),

	Bench::new("fyi_num::NiceU64", "from(1_999_999_999_999)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU64::from(1_999_999_999_999_u64)),

	Bench::new("fyi_num::NiceU64", "from(u64::MAX)")
		.timed(Duration::from_secs(1))
		.with(|| NiceU64::from(u64::MAX))
);
