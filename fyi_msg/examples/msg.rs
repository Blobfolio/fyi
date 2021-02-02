/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		confirm,
		crunched,
		debug,
		done,
		error,
		info,
		Msg,
		MsgKind,
		notice,
		plain,
		success,
		task,
		warning,
	};

	plain!("This message has no prefix.");

	println!();

	Msg::custom("Pink", 199, "This message has a custom pink prefix.")
		.with_newline(true)
		.print();

	Msg::custom("Blue", 4, "This message has a custom blue prefix.")
		.with_newline(true)
		.print();

	println!();

	notice!("So official!");
	success!("Hurray! You did it!");
	warning!("Hold it there, Sparky!");
	error!("Oopsie.");

	println!();

	debug!("The devil is in the details.");
	info!("Details without the word 'bug'.");
	task!("Let's get to work!");

	println!();

	// This can be printed without actually confirming anything by building
	// the message manually.
	MsgKind::Confirm.into_msg("Choose your own adventure.")
		.with_newline(true)
		.print();

	println!();

	crunched!("Some hard work just happened.");
	done!("As the French say, «FIN».");

	println!();

	#[cfg(feature = "timestamps")]
	{
		MsgKind::Info.into_msg("Messages can be timestamped.")
			.with_timestamp(true)
			.with_newline(true)
			.print();

		println!();
	}

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

	if confirm!("Was this example useful?") {
		plain!("Great!");
	}
	else {
		plain!("Oh well. Can't please 'em all!");
	}

	println!();
}
