# Qtx

Qtx is an asynchronous GUI framework based on Qt Widgets. The goal of this project is not expose
Qt's APIs (AKA bindings) but a GUI framework building on top of Qt Widgets.

## Features

- Native asynchronous.
- Safe and ergonomic APIs[^1].

## Non-goals

- Non-desktop platforms.
- Supports multiple Qt versions.

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Qtx
by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[^1]: There is one unavoidable unsafe function you need to use. This function is safe to call if you
      don't have other Qt bindings.
