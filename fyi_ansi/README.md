# FYI ANSI

[![docs.rs](https://img.shields.io/docsrs/fyi_ansi.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_ansi/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_ansi.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_ansi)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/fyi/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/fyi)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

This crate exports two simple compile-time ANSI formatting macros — `csi` and
`ansi` — as well as shortcut helpers for `bold`, `dim`, and `underline`.

## Examples

```rust
use fyi_ansi::{ansi, csi};

// The `csi` macro generates (only) the ANSI formatting sequence.
assert_eq!(csi!(bold, underline), "\x1b[1;4m");

// The `ansi` macro generates formatted content strings.
assert_eq!(
    concat!(ansi!((bold, light_red) "Error:"), " Oh no!"),
    "\x1b[1;91mError:\x1b[0m Oh no!",
);
```

The `dim`, `bold`, and `underline` macros are only shortcuts, but can
help declutter your code when there's only the one style being toggled.

```rust
use fyi_ansi::{bold, dim, underline};

// Same as with `ansi`, they terminate with a blanket reset by default.
assert_eq!(bold!("I'm bold!"),              "\x1b[1mI'm bold!\x1b[0m");
assert_eq!(dim!("I'm dim!"),                "\x1b[2mI'm dim!\x1b[0m");
assert_eq!(underline!("I'm underlined!"),   "\x1b[4mI'm underlined!\x1b[0m");

// And same as `ansi`, the ">" modifier will make them terminate more
// selectively instead.
assert_eq!(bold!(> "I'm bold!"),            "\x1b[1mI'm bold!\x1b[21m");
assert_eq!(dim!(> "I'm dim!"),              "\x1b[2mI'm dim!\x1b[22m");
assert_eq!(underline!(> "I'm underlined!"), "\x1b[4mI'm underlined!\x1b[24m");
```
