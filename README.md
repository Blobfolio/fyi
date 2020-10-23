# FYI

FYI is a simple CLI tool for x86-64 Linux machines that prints an arbitrary
status-style message, optionally indented, timestamped, and/or prefixed.
You know, stuff like:

* **Error:** Something broke!
* **Warning:** I can't keep doing this!
* **Success:** Life is good!

(Github doesn't display colored text, but just imagine the error prefix in ![#ff0000](https://via.placeholder.com/15/ff0000/000000?text=+), the success in ![#27ae60](https://via.placeholder.com/15/27ae60/000000?text=+), etc.)

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

This application is written in [Rust](https://www.rust-lang.org/) and can be installed using [Cargo](https://github.com/rust-lang/cargo).

For stable Rust (>= `1.47.0`), run:
```bash
RUSTFLAGS="-C link-arg=-s" cargo install \
    --git https://github.com/Blobfolio/fyi.git \
    --bin fyi \
    --target x86_64-unknown-linux-gnu
```

Pre-built `.deb` packages are also added for each [release](https://github.com/Blobfolio/fyi/releases/latest). They should always work for the latest stable Debian and Ubuntu.



## Usage

The primary usage is to generate a message with one of the default prefixes,
like: `fyi <PREFIX> [flags] [options] <MSG>`, where the prefix is one of:
* ![#27ae60](https://via.placeholder.com/15/27ae60/000000?text=+) crunched
* ![#5dade2](https://via.placeholder.com/15/5dade2/000000?text=+) debug
* ![#27ae60](https://via.placeholder.com/15/27ae60/000000?text=+) done
* ![#ff0000](https://via.placeholder.com/15/ff0000/000000?text=+) error
* ![#9b59b6](https://via.placeholder.com/15/9b59b6/000000?text=+) info
* ![#9b59b6](https://via.placeholder.com/15/9b59b6/000000?text=+) notice 
* ![#27ae60](https://via.placeholder.com/15/27ae60/000000?text=+) success
* ![#ff1493](https://via.placeholder.com/15/ff1493/000000?text=+) task
* ![#f1c40f](https://via.placeholder.com/15/f1c40f/000000?text=+) warning 

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
-c, --prefix-color <num>    Prefix color. [default: 199]
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



## Credits

| Library | License | Author |
| ---- | ---- | ---- |
| [ahash](https://crates.io/crates/ahash) | Apache-2.0 OR MIT | Tom Kaitchuck |
| [chrono](https://crates.io/crates/chrono) | Apache-2.0 OR MIT | Kang Seonghoon, Brandon W Maister |
| [criterion](https://crates.io/crates/criterion) | Apache-2.0 OR MIT | Jorge Aparicio, Brook Heisler |
| [crossbeam-channel](https://crates.io/crates/crossbeam-channel) | Apache-2.0 OR MIT  | The Crossbeam Project Developers |
| [num-format](https://crates.io/crates/num-format) | Apache-2.0 OR MIT  | The Rust Project Developers |
| [num-integer](https://crates.io/crates/num-integer) | Apache-2.0 OR MIT  | The Rust Project Developers |
| [rand](https://crates.io/crates/rand) | Apache-2.0 OR MIT  | The Rand, Rust Project Developers |
| [rayon](https://crates.io/crates/rayon) | Apache-2.0 OR MIT | Niko Matsakis, Josh Stone |
| [regex](https://crates.io/crates/regex) | Apache-2.0 OR MIT | The Rust Project Developers |
| [term_size](https://crates.io/crates/term_size) | Apache-2.0 OR MIT | Kevin K, Benjamin Sago |
| [unicode-width](https://crates.io/crates/unicode-width) | Apache-2.0 OR MIT | kwantam, Manish Goregaokar |



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