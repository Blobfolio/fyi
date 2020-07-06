/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		Msg,
		MsgKind,
	};

	Msg::from("This message has no prefix.").println();

	println!();

	Msg::new("Pink", 199, "This message has a custom pink prefix.").println();
	Msg::new("Blue", 4, "This message has a custom blue prefix.").println();

	println!();

	MsgKind::Notice.as_msg("So official!").println();
	MsgKind::Success.as_msg("Hurray! You did it!").println();
	MsgKind::Warning.as_msg("Hold it there, Sparky!").println();
	MsgKind::Error.as_msg("Oopsie.").println();

	println!();

	MsgKind::Debug.as_msg("The devil is in the details.").println();
	MsgKind::Info.as_msg("Details without the word 'bug'.").println();
	MsgKind::Task.as_msg("Let's get to work!").println();

	println!();

	MsgKind::Confirm.as_msg("Choose your own adventure.").println();

	println!();

	MsgKind::Crunched.as_msg("Some hard work just happened.").println();
	MsgKind::Done.as_msg("As the French say, «FIN».").println();

	println!();

	let mut tmp = MsgKind::Info.as_msg("Messages can be timestamped.");
	tmp.set_timestamp();
	tmp.println();

	println!();

	tmp = MsgKind::Info.as_msg("Messages can be indented (0).");
	tmp.println();

	tmp = MsgKind::Info.as_msg("Messages can be indented (1).");
	tmp.set_indent(1);
	tmp.println();

	tmp = MsgKind::Info.as_msg("Messages can be indented (2).");
	tmp.set_indent(2);
	tmp.println();

	tmp = MsgKind::Info.as_msg("Messages can be indented (3).");
	tmp.set_indent(3);
	tmp.println();

	println!();
}
