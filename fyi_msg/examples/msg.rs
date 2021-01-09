/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
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

	MsgKind::Notice.into_msg("So official!")
		.with_newline(true)
		.print();
	MsgKind::Success.into_msg("Hurray! You did it!")
		.with_newline(true)
		.print();
	MsgKind::Warning.into_msg("Hold it there, Sparky!")
		.with_newline(true)
		.print();
	MsgKind::Error.into_msg("Oopsie.")
		.with_newline(true)
		.print();

	println!();

	MsgKind::Debug.into_msg("The devil is in the details.")
		.with_newline(true)
		.print();
	MsgKind::Info.into_msg("Details without the word 'bug'.")
		.with_newline(true)
		.print();
	MsgKind::Task.into_msg("Let's get to work!")
		.with_newline(true)
		.print();

	println!();

	MsgKind::Confirm.into_msg("Choose your own adventure.")
		.with_newline(true)
		.print();

	println!();

	MsgKind::Crunched.into_msg("Some hard work just happened.")
		.with_newline(true)
		.print();
	MsgKind::Done.into_msg("As the French say, «FIN».")
		.with_newline(true)
		.print();

	println!();

	MsgKind::Info.into_msg("Messages can be timestamped.")
		.with_timestamp(true)
		.with_newline(true)
		.print();

	println!();

	MsgKind::Info.into_msg("Messages can be indented (0).")
		.with_newline(true)
		.print();

	MsgKind::Info.into_msg("Messages can be indented (1).")
		.with_indent(1)
		.with_newline(true)
		.print();

	MsgKind::Info.into_msg("Messages can be indented (2).")
		.with_indent(2)
		.with_newline(true)
		.print();

	MsgKind::Info.into_msg("Messages can be indented (3).")
		.with_indent(3)
		.with_newline(true)
		.print();

	println!();

	if MsgKind::Confirm.into_msg("Was this example useful?").prompt() {
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
