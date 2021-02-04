/*!
# Benchmark: `fyi_menu::KeyKind`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_menu::KeyKind;

benches!(
	Bench::new("fyi_menu::KeyKind", "from(Hello World)")
		.with(|| KeyKind::from(&b"Hello World"[..])),

	Bench::new("fyi_menu::KeyKind", "from(-p)")
		.with(|| KeyKind::from(&b"-p"[..])),

	Bench::new("fyi_menu::KeyKind", "from(--prefix)")
		.with(|| KeyKind::from(&b"--prefix"[..])),

	Bench::new("fyi_menu::KeyKind", "from(--prefix-color=199)")
		.with(|| KeyKind::from(&b"--prefix-color=199"[..]))
);
