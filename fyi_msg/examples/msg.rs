/*!
# FYI Msg Example: See the Prefixes and options.
*/

/// Do it.
fn main() {
	use fyi_msg::Msg;

	Msg::from("This message has no prefix.").println();

	println!();

	Msg::new("Pink", 199, "This message has a custom pink prefix.").println();
	Msg::new("Blue", 4, "This message has a custom blue prefix.").println();

	println!();

	Msg::notice("So official!").println();
	Msg::success("Hurray! You did it!").println();
	Msg::warning("Hold it there, Sparky!").println();
	Msg::error("Oopsie.").println();

	println!();

	Msg::debug("The devil is in the details.").println();
	Msg::info("Details without the word 'bug'.").println();
	Msg::task("Let's get to work!").println();

	println!();

	Msg::confirm("Choose your own adventure.").println();

	println!();

	Msg::crunched("Some hard work just happened.").println();
	Msg::done("As the French say, «FIN».").println();

	println!();

	let mut tmp = Msg::info("Messages can be timestamped.");
	tmp.set_timestamp();
	tmp.println();

	println!();

	tmp = Msg::info("Messages can be indented (0).");
	tmp.println();

	tmp = Msg::info("Messages can be indented (1).");
	tmp.set_indent(1);
	tmp.println();

	tmp = Msg::info("Messages can be indented (2).");
	tmp.set_indent(2);
	tmp.println();

	tmp = Msg::info("Messages can be indented (3).");
	tmp.set_indent(3);
	tmp.println();

	println!();
}
