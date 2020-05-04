extern crate criterion;

use criterion::{
	black_box,
	BenchmarkId,
	Criterion,
	criterion_group,
	criterion_main,
};



fn msg_new(c: &mut Criterion) {
	let mut group = c.benchmark_group("fyi_msg::Msg::new");
	for pair in [
		("", 2, ""),
		("Hello World", 199, "Test Message!"),
		("Björk Guðmundsdóttir Has Said", 208, "Test Message!"),
	].iter() {
		{
			let (prefix, color, msg) = *pair;
			println!("{}", fyi_msg::Msg::new(prefix, color, msg));
		}

		group.bench_with_input(
			BenchmarkId::from_parameter(format!("{:?}", pair)),
			pair,
			|b, (prefix, color, msg)| {
				b.iter(||
					fyi_msg::Msg::new(*prefix, *color, *msg)
				);
			}
		);

		println!("");
	}

	// Try a built-in or two.
	let msg = black_box("Test Message!");
	println!("{}", fyi_msg::Msg::error(msg.clone()));
	group.bench_function("error/Test Message!", move |b| {
		b.iter(|| fyi_msg::Msg::error(msg))
	});
	println!("");

	group.finish();
}



criterion_group!(
	benches,
	msg_new,
);
criterion_main!(benches);
