# FYI

[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/fyi/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/fyi)
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

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

Debian and Ubuntu users can just grab the pre-built `.deb` package from the [latest release](https://github.com/Blobfolio/fyi/releases/latest).

This application is written in [Rust](https://www.rust-lang.org/) and can alternatively be built/installed from source using [Cargo](https://github.com/rust-lang/cargo):

```bash
# See "cargo install --help" for more options.
cargo install \
    --git https://github.com/Blobfolio/fyi.git \
    --bin fyi
```



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

| Short | Long | Value | Description | Default |
| ----- | ---- | ----- | ----------- | ------- |
| `-e` | `--exit` | `<num>` | Exit with this status code after printing. | 0 |
| `-h` | `--help` | | Print help information and exit. | |
| `-i` | `--indent` | | Indent the line. | |
| | `--stderr` | | Print to STDERR instead of STDOUT. | |
| `-t` | `--timestamp` | | Include a timestamp. | |

### Custom Prefix:

To use a custom prefix (or no prefix), run `fyi print [flags] [options] <MSG>`,
using the following additional options:

| Short | Long | Value | Description | Default |
| ----- | ---- | ----- | ----------- | ------- |
| `-p` | `--prefix` | `<txt>` | Set a custom prefix. | |
| `-c` | `--prefix-color` | `<num>` | Prefix color. | 199 |

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

| Short | Long | Value | Description | Default |
| ----- | ---- | ----- | ----------- | ------- |
| `-h` | `--help` | | Print help information and exit. | |
| | `--stderr` | | Print to STDERR instead of STDOUT. | |
| `-c` | `--count` | `<num>` | Number of empty lines to print. | 1 |
