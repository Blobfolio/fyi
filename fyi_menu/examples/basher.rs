/*!
# FYI Menu: Basher Example

This recreates the BASH completions for [`ChannelZ`](https://github.com/blobfolio/channelz).
The results are printed to the screen as it is rather difficult to convey a "working"
completion script just by tabbing along. (But this one is good!)
*/

/// Do it.
fn main() {
	use fyi_menu::Basher;

	let b: Basher = Basher::new("channelz")
		.with_option(Some("-l"), Some("--list"))
		.with_switch(Some("-h"), Some("--help"))
		.with_switch(Some("-p"), Some("--progress"))
		.with_switch(Some("-V"), Some("--version"));

	println!("\n\n{}\n\n", b);
}
