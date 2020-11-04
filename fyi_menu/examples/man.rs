/*!
# FYI Menu: Man Example

This recreates the MAN page for [`ChannelZ`](https://github.com/blobfolio/channelz).
The code is printed to the screen.
*/

/// Do it.
fn main() {
	use fyi_menu::{
		Agree,
		AgreeKind,
		AgreeSection,
		FLAG_MAN_ALL,
	};

	let m: Agree = Agree::new(
		"ChannelZ",
		"channelz",
		"1.2.3",
		"ChannelZ is a simple, fast, multi-threaded static Gzip/Brotli encoding tool for the CLI."
	)
		.with_flags(FLAG_MAN_ALL)
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
		)
		.with_arg(
			AgreeKind::arg("<PATH(s)…>", "Any number of files and directories to crawl and crunch.")
		)
		.with_section(
			AgreeSection::new("FILE TYPES", false)
				.with_item(
					AgreeKind::paragraph("Static copies will only be generated for files with these extensions:")
						.with_line("css; eot; htm(l); ico; js; json; mjs; otf; rss; svg; ttf; txt; xhtm(l); xml; xsl")
				)
		);

	println!("\n\n{}\n", m.man());
}
