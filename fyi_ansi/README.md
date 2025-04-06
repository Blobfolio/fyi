# FYI ANSI

[![docs.rs](https://img.shields.io/docsrs/fyi_ansi.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_ansi/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_ansi.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_ansi)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/fyi/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/fyi)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

This crate exports two simple compile-time ANSI formatting macros: `csi` and `ansi`.

## Examples

```rust
use fyi_ansi::{csi, ansi};

// The `csi` macro generates (only) the ANSI formatting sequence.
assert_eq!(csi!(bold, underline), "\x1b[1;4m");

// The `ansi` macro generates formatted content strings.
assert_eq!(
    concat!(ansi!((bold, light_red) "Error:"), " Oh no!"),
    "\x1b[1;91mError:\x1b[0m Oh no!",
);
```
