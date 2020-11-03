/*!
# FYI Menu: Man Example

This recreates the MAN page for [`ChannelZ`](https://github.com/blobfolio/channelz).
The code is printed to the screen.
*/

/// Do it.
fn main() {
	use fyi_menu::{
		Man,
		ManSection,
		ManSectionItem,
	};

	let m: Man = Man::new("ChannelZ", "channelz", "1.2.3")
		.with_text(
			"DESCRIPTION",
			"ChannelZ is a simple, fast, multi-threaded static Gzip/Brotli encoding tool for the CLI.",
			false
		)
		.with_text("USAGE:", "channelz [FLAGS] [OPTIONS] <PATH(s)…>", true)
		.with_section(
			ManSection::list("FLAGS:")
				.with_item(
					ManSectionItem::new("Remove all existing *.gz *.br files before starting.")
						.with_key("--clean")
				)
				.with_item(
					ManSectionItem::new("Print help information.")
						.with_key("-h")
						.with_key("--help")
				)
				.with_item(
					ManSectionItem::new("Show progress bar while minifying.")
						.with_key("-p")
						.with_key("--progress")
				)
				.with_item(
					ManSectionItem::new("Print version information.")
						.with_key("-V")
						.with_key("--version")
				)
		)
		.with_section(
			ManSection::list("OPTIONS:")
				.with_item(
					ManSectionItem::new("Read file paths from this text file.")
						.with_key("-l")
						.with_key("--list")
						.with_value("<FILE>")
				)
		)
		.with_text(
			"<PATH(s)…>:",
			"Any number of files or directories to crawl and crunch.",
			true
		)
		.with_text(
			"NOTE",
			"Static copies will only be generated for files with these extensions:\n.RE\ncss; eot; htm(l); ico; js; json; mjs; otf; rss; svg; ttf; txt; xhtm(l); xml; xsl",
			false
		);

	println!("\n\n{}\n\n", m);
}
