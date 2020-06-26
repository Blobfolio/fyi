/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::{
		Msg,
		traits::PrintyPlease,
	};

	Msg::from("This message has no prefix.").fyi_println();

	println!();

	Msg::new("Pink", 199, "This message has a custom pink prefix.").fyi_println();
	Msg::new("Blue", 4, "This message has a custom blue prefix.").fyi_println();

	println!();

	Msg::notice("So official!").fyi_println();
	Msg::success("Hurray! You did it!").fyi_println();
	Msg::warning("Hold it there, Sparky!").fyi_println();
	Msg::error("Oopsie.").fyi_println();

	println!();

	Msg::debug("The devil is in the details.").fyi_println();
	Msg::info("Details without the word 'bug'.").fyi_println();
	Msg::task("Let's get to work!").fyi_println();

	println!();

	Msg::confirm("Choose your own adventure.").fyi_println();

	println!();

	Msg::crunched("Some hard work just happened.").fyi_println();
	Msg::done("As the French say, «FIN».").fyi_println();

	println!();

	let mut tmp = Msg::info("Messages can be timestamped.");
	tmp.set_timestamp();
	tmp.fyi_println();

	println!();

	tmp = Msg::info("Messages can be indented (0).");
	tmp.fyi_println();

	tmp = Msg::info("Messages can be indented (1).");
	tmp.set_indent(1);
	tmp.fyi_println();

	tmp = Msg::info("Messages can be indented (2).");
	tmp.set_indent(2);
	tmp.fyi_println();

	tmp = Msg::info("Messages can be indented (3).");
	tmp.set_indent(3);
	tmp.fyi_println();

	println!();
}
