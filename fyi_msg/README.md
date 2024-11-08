# FYI Msg

[![docs.rs](https://img.shields.io/docsrs/fyi_msg.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_msg/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_msg.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_msg)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/repo/github/blobfolio/fyi/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/repo/github/blobfolio/fyi)<br>
[![license](https://img.shields.io/badge/license-wtfpl-ff1493?style=flat-square)](https://en.wikipedia.org/wiki/WTFPL)

This crate contains the objects providing the heart of the [FYI command line application](https://github.com/blobfolio/fyi), namely `Msg`, a simple struct for status-like messages that can be easily printed to `STDOUT` or `STDERR`.



## Examples

```rust
use fyi_msg::{Msg, MsgKind};

// One way.
Msg::new(MsgKind::Success, "You did it!")
    .with_newline(true)
    .print();

// Another equivalent way.
Msg::success("You did it!").print();
```

For more usage examples, check out the `examples/msg` demo, which covers just about every common use case.



## Macros

| Macro | Equivalent |
| ----- | ---------- |
| `confirm!(…)` | `Msg::new(MsgKind::Confirm, "Some question…").prompt()` |



## Optional Features

| Feature | Description |
| ------- | ----------- |
| `fitted` | Enables `Msg::fitted` for obtaining a slice trimmed to a specific display width. |
| `progress` | Enables `Progless`, a thread-safe CLI progress bar displayer.
| `timestamps` | Enables timestamp-related methods and flags like `Msg::with_timestamp`. |
