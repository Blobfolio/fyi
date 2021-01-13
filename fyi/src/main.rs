/*!
# FYI

FYI is a simple CLI tool for x86-64 Linux machines that prints an arbitrary
status-style message, optionally indented, timestamped, and/or prefixed.
You know, stuff like:

* <strong><span style="color: red;">Error:</span></strong> Something broke!
* <strong><span style="color: gold;">Warning:</span></strong> I can't keep doing this!
* <strong><span style="color: green;">Success:</span></strong> Life is good!

That's it!



## Why?

The main idea is to make it easier to communicate successes and errors from
within shell scripts, particularly in regards to color. ANSI formatting isn't
difficult, but all that extra code is difficult to read.

Take for example:
```bash
# The manual way.
echo "\033[1;91mError:\033[0m Something broke!"

# Using FYI:
fyi error "Something broke!"
```



## Installation

This application is written in Rust and can be installed using Cargo.

For stable Rust (>= `1.47.0`), run:
```bash
RUSTFLAGS="-C link-arg=-s" cargo install \
    --git https://github.com/Blobfolio/fyi.git \
    --bin fyi \
    --target x86_64-unknown-linux-gnu
```



## Usage

The primary usage is to generate a message with one of the default prefixes,
like: `fyi <PREFIX> [flags] [options] <MSG>`, where the prefix is one of:
* <span style="color: green;">crunched</span>
* <span style="color: darkcyan;">debug</span>
* <span style="color: green;">done</span>
* <span style="color: red;">error</span>
* <span style="color: blueviolet;">info</span>
* <span style="color: blueviolet;">notice</span>
* <span style="color: green;">success</span>
* <span style="color: deeppink;">task</span>
* <span style="color: gold;">warning</span>

The following flags and options are available.
```bash
-e, --exit <num>   Exit with this status code after printing. [default: 0]
-h, --help         Print this screen.
-i, --indent       Indent the line.
    --stderr       Print to STDERR instead of STDOUT.
-t, --timestamp    Include a timestamp.
```

### Custom Prefix:

To use a custom prefix (or no prefix), run `fyi print [flags] [options] <MSG>`,
using the following additional options:
```bash
-p, --prefix <txt>          Set a custom prefix. [default: ]
-c, --prefix-color <num>    Use this color for the prefix. [default: 199]
```

The color should be a `u8` corresponding to a [BASH color number](https://misc.flogisoft.com/bash/tip_colors_and_formatting#colors1).
Note: to avoid the cost of re-alignment, only values in the range of `1..=255` are supported.

### Confirmation Prompt:

To prompt a user for a Y/N (and exit with a corresponding status code), run
`fyi confirm [flags] [options] <MSG>`. Confirmation supports the same flags as
the other built-in prefixes.

### Blank Lines:

And finally, there is a convenient `blank` subcommand that does nothing but
print a certain number of blank lines for you. Run
`fyi blank [flags] [options]`, which supports the following:
```bash
-h, --help           Print this screen.
    --stderr         Print to STDERR instead of STDOUT.
-c, --count <num>    Number of empty lines to print. [default: 1]
```



## License

Copyright © 2020 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

This work is free. You can redistribute it and/or modify it under the terms of the Do What The Fuck You Want To Public License, Version 2.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    Version 2, December 2004

    Copyright (C) 2004 Sam Hocevar <sam@hocevar.net>

    Everyone is permitted to copy and distribute verbatim or modified
    copies of this license document, and changing it is allowed as long
    as the name is changed.

    DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
    TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

    0. You just DO WHAT THE FUCK YOU WANT TO.

*/

#![warn(clippy::filetype_is_file)]
#![warn(clippy::integer_division)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::suboptimal_flops)]
#![warn(clippy::unneeded_field_pattern)]
#![warn(macro_use_extern_crate)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::map_err_ignore)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]



use fyi_menu::{
	Argue,
	FLAG_REQUIRED,
	FLAG_SUBCOMMAND,
};
use fyi_msg::{
	Msg,
	MsgKind,
	FLAG_INDENT,
	FLAG_NEWLINE,
	FLAG_TIMESTAMP,
};



#[doc(hidden)]
/// Main.
fn main() {
	// Parse CLI arguments.
	let mut args = Argue::new(FLAG_REQUIRED | FLAG_SUBCOMMAND)
		.with_version("FYI", env!("CARGO_PKG_VERSION"))
		.with_help(helper);

	match MsgKind::from(unsafe { args.peek_unchecked() }) {
		MsgKind::Blank => blank(&mut args),
		MsgKind::None => {
			Msg::error("Invalid message type.").die(1);
			unreachable!();
		},
		kind => msg(kind, &mut args),
	}
}

#[doc(hidden)]
/// Shoot Blanks.
///
/// Print one or more blank lines to `Stdout` or `Stderr`.
fn blank(args: &mut Argue) {
	// How many lines should we print?
	let msg = Msg::plain("\n".repeat(
		args.option2("-c", "--count")
			.and_then(|c| c.parse::<usize>().ok())
			.map_or(1, |c| 1_usize.max(c))
	));

	// Print it to `Stderr`.
	if args.switch("--stderr") { msg.eprint(); }
	// Print it to `Stdout`.
	else { msg.print(); }
}

#[doc(hidden)]
/// Basic Message.
fn msg(kind: MsgKind, args: &mut Argue) {
	// Exit code.
	let exit: i32 = args.option2("-e", "--exit")
		.map_or(0, |x| x.parse::<i32>().unwrap_or(0));

	// Basic flags.
	let mut flags: u8 = FLAG_NEWLINE;
	if args.switch2("-i", "--indent") { flags |= FLAG_INDENT; }
	if args.switch2("-t", "--timestamp") { flags |= FLAG_TIMESTAMP; }

	// The main message.
	let msg =
		// Custom message prefix.
		if MsgKind::Custom == kind {
			args.option2("-p", "--prefix")
				.map_or_else(
					|| Msg::plain(args.arg(0).unwrap_or_default()),
					|prefix| {
						let color: u8 = args.option2("-c", "--prefix-color")
							.and_then(|x| x.parse::<u8>().ok())
							.unwrap_or(199);
						Msg::custom(prefix, color, args.arg(0).unwrap_or_default())
							.with_flags(flags)
					}
				)
		}
		// Built-in prefix.
		else {
			Msg::new(kind, args.take_arg())
				.with_flags(flags)
		};

	// It's a prompt!
	if MsgKind::Confirm == kind {
		if ! msg.prompt() {
			std::process::exit(1);
		}
		return;
	}
	// Print to `Stderr`.
	else if args.switch("--stderr") { msg.eprint(); }
	// Print to `Stdout`.
	else { msg.print(); }

	// Special exit?
	if 0 != exit {
		std::process::exit(exit);
	}
}

#[doc(hidden)]
/// Help Page.
///
/// Print the appropriate help screen given the call details. Most of the sub-
/// commands work the same way, but a few have their own distinct messages.
fn helper(cmd: Option<&str>) {
	Msg::plain(
		format!(
			r#"
                      ;\
                     |' \
  _                  ; : ;
 / `-.              /: : |
|  ,-.`-.          ,': : |
\  :  `. `.       ,'-. : |
 \ ;    ;  `-.__,'    `-.|         {}{}{}
  \ ;   ;  :::  ,::'`:.  `.        Simple CLI status messages.
   \ `-. :  `    :.    `.  \
    \   \    ,   ;   ,:    (\
     \   :., :.    ,'o)): ` `-.
    ,/,' ;' ,::"'`.`---'   `.  `-._
  ,/  :  ; '"      `;'          ,--`.
 ;/   :; ;             ,:'     (   ,:)
   ,.,:.    ; ,:.,  ,-._ `.     \""'/
   '::'     `:'`  ,'(  \`._____.-'"'
      ;,   ;  `.  `. `._`-.  \\
      ;:.  ;:       `-._`-.\  \`.
       '`:. :        |' `. `\  ) \
          ` ;:       |    `--\__,'
            '`      ,'
                 ,-'

{}
"#,
			"\x1b[38;5;199mFYI\x1b[0;38;5;69m v",
			env!("CARGO_PKG_VERSION"),
			"\x1b[0m",
			match cmd {
				Some("blank") => include_str!("../help/blank.txt").to_string(),
				Some("print") => include_str!("../help/print.txt").to_string(),
				Some("confirm") | Some("prompt") => include_str!("../help/confirm.txt").to_string(),
				Some(x) if MsgKind::from(x) != MsgKind::None => format!(
					include_str!("../help/generic.txt"),
					x,
					Msg::new(MsgKind::from(x), "Hello World").as_str(),
					x.to_lowercase(),
				),
				_ => include_str!("../help/help.txt").to_string(),
			}
		)
	).print();
}
