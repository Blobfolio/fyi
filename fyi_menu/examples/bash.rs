/*!
# FYI Menu: Basher Example

This recreates the BASH completions for [`ChannelZ`](https://github.com/blobfolio/channelz).
The results are printed to the screen as it is rather difficult to convey a "working"
completion script just by tabbing along. (But this one is good!)
*/

/// Do it.
fn main() {
	use fyi_menu::{
		Agree,
		AgreeKind,
	};

	let a: Agree = Agree::new(
		"ChannelZ",
		"channelz",
		"1.2.3",
		"ChannelZ is a simple, fast, multi-threaded static Gzip/Brotli encoding tool for the CLI."
	)
		.with_arg(
			AgreeKind::switch("Remove all existing *.gz *.br files before starting.")
				.with_long("--clean")
		)
		.with_arg(
			AgreeKind::switch("Print help information.")
				.with_short("-h")
				.with_long("--help")
		)
		.with_arg(
			AgreeKind::switch("Show progress bar while working.")
				.with_short("-p")
				.with_long("--progress")
		)
		.with_arg(
			AgreeKind::switch("Print program version.")
				.with_short("-V")
				.with_long("--version")
		)
		.with_arg(
			AgreeKind::option("<FILE>", "Read file paths from this text file.", true)
				.with_short("-l")
				.with_long("--list")
		);

	println!("\n\n{}\n\n", a.bash());
}
