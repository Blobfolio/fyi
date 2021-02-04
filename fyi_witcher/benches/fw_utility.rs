/*!
# Benchmark: `fyi_witcher::utility`
*/

use fyi_bench::{
	Bench,
	benches,
};
use fyi_witcher::utility::hash64;

benches!(
	Bench::new("fyi_witcher::utility", "hash64(-p)")
		.with(|| hash64(&b"-p"[..])),

	Bench::new("fyi_witcher::utility", "hash64(--prefix)")
		.with(|| hash64(&b"--prefix"[..])),

	Bench::new("fyi_witcher::utility", "hash64(/usr/share/...)")
		.with(|| hash64(&b"/usr/share/man/man1/fyi-confirm.1.gz"[..]))
);
