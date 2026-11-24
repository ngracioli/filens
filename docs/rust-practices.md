# Rust Practices for Filens

| Topic | Do | Avoid |
| --- | --- | --- |
| Paths | Read arguments with `args_os`. Use `&Path` to borrow a path and `PathBuf` to store an owned path. Convert paths to text for display. | Operating on tree labels, calling `to_str().unwrap()`, or concatenating path separators. Lossy conversion can give distinct names the same label. |
| Errors | Propagate failures with `Result` and `?`. Include the operation, relevant path, and original cause. Show a message where a recoverable item failure occurs. | Using `unwrap()` on user input, ignoring read errors, or quitting because a file disappeared between listing and opening it. |
| State | Keep selection and focus in application state. Borrow data for reads and use distinct types when states require different handling. | Cloning the tree or preview for every frame, using global state, or spreading selection state across widgets. |
| Reading | Expand directories on demand. Limit preview reads and report truncation. Define and test symbolic link behavior. | Unbounded recursion, following links recursively, or reading devices and pipes as regular files. |
| Terminal | Centralize setup and cleanup. Attempt to restore the session on normal exit and on errors. Use Ratatui's panic restoration mechanisms and test lifecycle changes. | Writing diagnostics directly to the active screen, calling `process::exit` from inside the loop, or scattering ANSI escape sequences through the code. |
| Text | Treat content and file names as data. Display control characters safely. Measure visual width in terminal columns. | Passing raw control bytes through to the terminal or using byte length as display width. |
| Dependencies | Prefer the standard library and existing dependencies. Before adding a crate, check its license, maintenance, native requirements, and minimum Rust version. Track `Cargo.lock` and use `--locked` in checks. | Introducing an async runtime, a foreign function interface, or a system dependency for work the existing tools already cover. |
| Diagnostics | Collect the Filens version, operating system, architecture, terminal, command, and error message. Use sample names or anonymized data in reports. | Including file contents or dumping personal paths in logs by default. |

## Verifying Changes

For Rust changes, run the commands below. Clippy is an additional recommended check. Run it with the same toolchain used for compilation.

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

For interface changes, also run `cargo run -- .` in a terminal. Test regressions where they occur. State transitions can be tested without a terminal. Screen restoration requires a terminal session.

## Sources

- [Native paths in Rust](https://doc.rust-lang.org/std/path/index.html) and [process arguments](https://doc.rust-lang.org/std/env/fn.args_os.html).
- [Choosing between Result and panic](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html).
- [Terminal restoration in Ratatui](https://docs.rs/ratatui/latest/ratatui/fn.try_restore.html).
- [Clippy in continuous integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html) and [continuous integration with Cargo](https://doc.rust-lang.org/cargo/guide/continuous-integration.html).
