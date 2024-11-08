/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		confirm,
		Msg,
		MsgKind,
	};

	Msg::plain("This message has no prefix.")
		.with_newline(true)
		.print();

	println!();

	Msg::custom("Pink", 199, "This message has a custom pink prefix.")
		.with_newline(true)
		.print();

	Msg::custom("Blue", 4, "This message has a custom blue prefix.")
		.with_newline(true)
		.print();

	println!();

	Msg::notice("So official!").print();
	Msg::success("Hurray! You did it!").print();
	Msg::warning("Hold it there, Sparky!").print();
	Msg::error("Oopsie.").print();

	println!();

	Msg::review("The total was such-and-such.").print();
	Msg::debug("The devil is in the details.").print();
	Msg::info("Details without the word 'bug'.").print();
	Msg::skipped("Wasn't worth doing.").print();
	Msg::task("Let's get to work!").print();

	println!();

	// This can be printed without actually confirming anything by building
	// the message manually. There is no direct `Msg::confirm()` method to
	// avoid confusion with the `fyi_msg::confirm!()` macro.
	MsgKind::Confirm.into_msg("Choose your own adventure.")
		.with_newline(true)
		.print();

	println!();

	Msg::crunched("Some hard work just happened.").print();
	Msg::done("As the French say, «FIN».").print();

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
		Msg::plain("Great!")
			.with_newline(true)
			.print();
	}
	else {
		Msg::plain("Are you sure?!")
			.with_newline(true)
			.print();
	}

	// Here's that macro we mentioned earlier.
	if confirm!(yes: "Was this example useful?") {
		Msg::plain("Great!")
			.with_newline(true)
			.print();
	}
	else {
		Msg::plain("Oh well. Can't please 'em all!")
			.with_newline(true)
			.print();
	}

	// Test confirmation with indentation.
	if confirm!(yes: "Is this confirmation indented?", 1) {
		Msg::plain("Great!")
			.with_indent(1)
			.with_newline(true)
			.print();
	}
	else {
		Msg::plain("Are you sure?!")
			.with_indent(1)
			.with_newline(true)
			.print();
	}

	if confirm!("Do you hate fun?", 2) {
		Msg::plain("That's weird.")
			.with_indent(2)
			.with_newline(true)
			.print();
	}
	else {
		Msg::plain("Just checking…")
			.with_indent(2)
			.with_newline(true)
			.print();
	}

	println!();
}
