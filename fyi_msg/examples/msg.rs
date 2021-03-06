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

	Msg::debug("The devil is in the details.").print();
	Msg::info("Details without the word 'bug'.").print();
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

	// Here's that macro we mentioned earlier.
	if confirm!("Was this example useful?") {
		Msg::plain("Great!")
			.with_newline(true)
			.print();
	}
	else {
		Msg::plain("Oh well. Can't please 'em all!")
			.with_newline(true)
			.print();
	}

	println!();
}
