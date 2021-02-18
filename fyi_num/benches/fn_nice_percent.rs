/*!
# Benchmark: `fyi_num::nice_percent`
*/

use brunch::{
	Bench,
	benches,
};
use fyi_num::NicePercent;
use std::time::Duration;

benches!(
	Bench::new("fyi_num::NicePercent", "from(0)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0_f32)),

	Bench::new("fyi_num::NicePercent", "from(0.1)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0.1_f32)),

	Bench::new("fyi_num::NicePercent", "from(0.12)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0.12_f32)),

	Bench::new("fyi_num::NicePercent", "from(0.123)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0.123_f32)),

	Bench::new("fyi_num::NicePercent", "from(0.1234)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0.1234_f32)),

	Bench::new("fyi_num::NicePercent", "from(0.12345)")
		.timed(Duration::from_secs(1))
		.with(|| NicePercent::from(0.12345_f32))
);
