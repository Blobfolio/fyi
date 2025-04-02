/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		AnsiColor,
		confirm,
		Msg,
		MsgKind,
	};

	Msg::from("This message has no prefix.")
		.with_newline(true)
		.print();

	println!();

	Msg::new(("Pink", AnsiColor::Misc199), "This message has a custom pink prefix.")
		.with_newline(true)
		.print();

	Msg::new(("Blue", AnsiColor::Blue), "This message has a custom blue prefix.")
		.with_newline(true)
		.print();

	println!();

	// The built-ins.
	for kind in MsgKind::ALL {
		if kind.prefix_color().is_some() {
			Msg::new(kind, "Built-in prefix.").with_newline(true).print();
		}
	}

	println!();

	#[cfg(feature = "timestamps")]
	{
		Msg::info("Messages can be timestamped.")
			.with_timestamp(true)
			.print();

		println!();
	}

	Msg::info("Messages can be indented (0).")
		.with_indent(0)
		.print();

	Msg::info("Messages can be indented (1).")
		.with_indent(1)
		.print();

	Msg::info("Messages can be indented (2).")
		.with_indent(2)
		.print();

	Msg::info("Messages can be indented (3).")
		.with_indent(3)
		.print();

	Msg::info("Messages can be indented (4).")
		.with_indent(4)
		.print();

	println!();

	Msg::info("The formatting can be stripped too.")
		.without_ansi()
		.print();

	#[cfg(feature = "timestamps")]
	Msg::info("So pale! Ew!")
		.with_timestamp(true)
		.without_ansi()
		.print();

	println!();

	// A prompt to STDERR.
	if Msg::new(MsgKind::Confirm, "Did this print to STDERR?").eprompt_with_default(true) {
		Msg::from("Great!")
			.with_newline(true)
			.print();
	}
	else {
		Msg::from("Are you sure?!")
			.with_newline(true)
			.print();
	}

	// Here's that macro we mentioned earlier.
	if confirm!(yes: "Was this example useful?") {
		Msg::from("Great!")
			.with_newline(true)
			.print();
	}
	else {
		Msg::from("Oh well. Can't please 'em all!")
			.with_newline(true)
			.print();
	}

	// Test confirmation with indentation.
	if confirm!(yes: "Is this confirmation indented?", 1) {
		Msg::from("Great!")
			.with_indent(1)
			.with_newline(true)
			.print();
	}
	else {
		Msg::from("Are you sure?!")
			.with_indent(1)
			.with_newline(true)
			.print();
	}

	if confirm!("Do you hate fun?", 2) {
		Msg::from("That's weird.")
			.with_indent(2)
			.with_newline(true)
			.print();
	}
	else {
		Msg::from("Just checking…")
			.with_indent(2)
			.with_newline(true)
			.print();
	}

	println!();
}
