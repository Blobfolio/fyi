/*!
# FYI Core: Progress Bar Example.

This is a simple threaded progress bar demo.

Being that we aren't actually performing work, the impression of
variability is achieved by making each thread live as long as the
presidents' tenure.

The scale is one second per standard four-year term.
*/

/// Do it.
fn main() {
	use std::time::Duration;
	use std::thread;
	use std::sync::Arc;
	use rayon::prelude::*;
	use fyi_core::Progress;
	use fyi_core::Msg;
	use fyi_core::Prefix;



	// This is approximate four years in days.
	const FOUR_YEARS: f64 = 1461.0;

	// Presidents and their term length in days, no particular order.
	let presidents = vec![
		("Richard Nixon", 2027),
		("Donald Trump", 1191),
		("George W. Bush", 2922),
		("Grover Cleveland", 2922),
		("Lyndon B. Johnson", 1886),
		("Gerald Ford", 895),
		("Ulysses S. Grant", 2922),
		("Benjamin Harrison", 1461),
		("Millard Fillmore", 969),
		("Jimmy Carter", 1461),
		("Chester A. Arthur", 1262),
		("Bill Clinton", 2922),
		("Zachary Taylor", 492),
		("James K. Polk", 1461),
		("John Tyler", 1430),
		("John Quincy Adams", 1461),
		("James Monroe", 2922),
		("James A. Garfield", 199),
		("Harry S. Truman", 2840),
		("Abraham Lincoln", 1503),
		("Calvin Coolidge", 2041),
		("William Henry Harrison", 31),
		("Franklin Pierce", 1461),
		("Herbert Hoover", 1461),
		("John Adams", 1460),
		("Theodore Roosevelt", 2728),
		("Barack Obama", 2922),
		("William Howard Taft", 1461),
		("Andrew Jackson", 2922),
		("John F. Kennedy", 1036),
		("Woodrow Wilson", 2922),
		("James Madison", 2922),
		("William McKinley", 1654),
		("Ronald Reagan", 2922),
		("Thomas Jefferson", 2922),
		("Dwight D. Eisenhower", 2922),
		("Andrew Johnson", 1419),
		("Franklin D. Roosevelt", 4422),
		("Martin Van Buren", 1461),
		("George H. W. Bush", 1461),
		("James Buchanan", 1461),
		("Warren G. Harding", 881),
		("Rutherford B. Hayes", 1461),
		("George Washington", 2865),
	];

	// Start a bar.
	let bar = Arc::new(Progress::new(
		Msg::new("Eviction underway…")
			.with_prefix(Prefix::new("Processing", 199))
			.to_string(),
		presidents.len() as u64,
		0
	));

	// Launch a steady tick.
	let handle = Progress::steady_tick(&bar, None);
	const SET_MSG_TARGET: f64 = 34.0 / 44.0;

	// Do our "work".
	presidents.par_iter().for_each(|(p, d)| {
		let dude: String = p.to_string();
		bar.clone().add_task(&dude);

		// Simulate processing overhead.
		let pause: u64 = (*d as f64 / FOUR_YEARS * 1000.0) as u64;
		thread::sleep(Duration::from_millis(pause));

		// Change the message as we get to the end.
		if bar.clone().percent() == SET_MSG_TARGET {
			bar.clone().update(
				1,
				Some(Msg::new("Almost done!")
					.with_prefix(Prefix::new("Processing", 199))
					.to_string()),
				Some(dude)
			);
		}
		else {
			bar.clone().update(1, None, Some(dude));
		}
	});
	handle.join().unwrap();

	// Print a quick summary.
	bar.finished_in();
}
