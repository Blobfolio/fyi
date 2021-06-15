# FYI Msg

[![Documentation](https://docs.rs/fyi_msg/badge.svg)](https://docs.rs/fyi_msg/)
[![crates.io](https://img.shields.io/crates/v/fyi_msg.svg)](https://crates.io/crates/fyi_msg)

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



## License

Copyright © 2021 [Blobfolio, LLC](https://blobfolio.com) &lt;hello@blobfolio.com&gt;

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
