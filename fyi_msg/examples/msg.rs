/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		Msg,
		MsgKind,
	};

	Msg::new("This message has no prefix.").println();

	println!();

	MsgKind::new("Pink", 199)
		.into_msg("This message has a custom pink prefix.")
		.println();

	MsgKind::new("Blue", 4)
		.into_msg("This message has a custom blue prefix.")
		.println();

	println!();

	MsgKind::Notice.into_msg("So official!").println();
	MsgKind::Success.into_msg("Hurray! You did it!").println();
	MsgKind::Warning.into_msg("Hold it there, Sparky!").println();
	MsgKind::Error.into_msg("Oopsie.").println();

	println!();

	MsgKind::Debug.into_msg("The devil is in the details.").println();
	MsgKind::Info.into_msg("Details without the word 'bug'.").println();
	MsgKind::Task.into_msg("Let's get to work!").println();

	println!();

	MsgKind::Confirm.into_msg("Choose your own adventure.").println();

	println!();

	MsgKind::Crunched.into_msg("Some hard work just happened.").println();
	MsgKind::Done.into_msg("As the French say, «FIN».").println();

	println!();

	MsgKind::Info.into_msg("Messages can be timestamped.")
		.with_timestamp(true)
		.println();

	println!();

	MsgKind::Info.into_msg("Messages can be indented (0).").println();

	MsgKind::Info.into_msg("Messages can be indented (1).")
		.with_indent(1)
		.println();

	MsgKind::Info.into_msg("Messages can be indented (2).")
		.with_indent(2)
		.println();

	MsgKind::Info.into_msg("Messages can be indented (3).")
		.with_indent(3)
		.println();

	println!();

	if MsgKind::Confirm.into_msg("Was this example useful?").prompt() {
		Msg::new("Great!").println();
	}
	else {
		Msg::new("Oh well. Can't please 'em all!").println();
	}

	println!();
}
