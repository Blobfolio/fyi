# FYI Msg

[![docs.rs](https://img.shields.io/docsrs/fyi_msg.svg?style=flat-square&label=docs.rs)](https://docs.rs/fyi_msg/)
<br>
[![crates.io](https://img.shields.io/crates/v/fyi_msg.svg?style=flat-square&label=crates.io)](https://crates.io/crates/fyi_msg)
[![ci](https://img.shields.io/github/actions/workflow/status/Blobfolio/fyi/ci.yaml?style=flat-square&label=ci)](https://github.com/Blobfolio/fyi/actions)
[![deps.rs](https://deps.rs/crate/fyi_msg/latest/status.svg?style=flat-square&label=deps.rs)](https://deps.rs/crate/fyi_msg/)<br>
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
| `signals_sigint` | Adds default, two-strike, or keepalive interrupt handling (`SIGINT` on Unix; Ctrl+C/Ctrl+Break on Windows). |
| `signals_sigwinch` | Uses Unix resize signals; Windows continues polling terminal dimensions. |
| `signals` | Enables both signal features. |
| `timestamps` | Enables timestamp-related methods and flags like `Msg::with_timestamp`. |

On Windows, call `fyi_msg::enable_console_ansi()` before printing ANSI-formatted
output or creating a progress bar. It enables ANSI processing for console stdout
and stderr while preserving other mode flags, and leaves redirected streams
alone. This function does not install signal handlers.

The `signals_sigint` policies must be selected before creating the first progress
bar; the first policy registered wins. Windows console-close, sign-out, and
shutdown events retain their system defaults. The `signal_hook` re-export is
available only on Unix.

To try the interrupt policies:

```sh
cargo run -p fyi_msg --example progless-sigint --features signals
cargo run -p fyi_msg --example progless-sigint --features signals -- --keepalive
```

When building this repository on Windows without Git symlink support, first
materialize the two shared source links (published crates already contain the
source files):

```powershell
Copy-Item skel/ansi256.rs fyi_ansi/skel/ansi256.rs -Force
Copy-Item skel/ansi256.rs fyi_msg/skel/ansi256.rs -Force
```
