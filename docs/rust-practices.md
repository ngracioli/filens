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

## Practices from Rust Terminal Projects

| Topic | Do | Avoid |
| --- | --- | --- |
| Tree identity | Store a native path and file type for each entry. Sort explicitly and handle errors on individual entries. Keep working with readable siblings when possible. | Recovering an entry's path or kind from its display label, assuming directory iteration is ordered, or following directory links recursively by default. [ripgrep](https://github.com/BurntSushi/ripgrep/blob/master/crates/ignore/src/walk.rs), [read_dir](https://doc.rust-lang.org/std/fs/fn.read_dir.html) |
| Preview pipeline | Share syntax definitions, create fresh highlighting state for each file, and preserve original line numbers before wrapping. Bound file reads and show recoverable errors in the preview. | Sharing parser state between files, counting visual rows as source lines, or reading files while drawing a frame. [bat](https://github.com/sharkdp/bat/blob/master/src/controller.rs), [delta](https://github.com/dandavison/delta/blob/main/src/paint.rs) |
| Terminal input | Give an open modal first control of its navigation keys. Restore prior focus when it closes. As bindings grow, translate terminal events into app actions at the terminal boundary. | Sending modal navigation to the background panel or handling shortcuts in rendering code. [Helix](https://github.com/helix-editor/helix/blob/master/helix-term/src/compositor.rs), [Zellij](https://github.com/zellij-org/zellij/blob/main/zellij-utils/src/input/actions.rs) |
| Module boundaries | Keep terminal lifecycle, tree, and preview behavior in focused modules within the Cargo package. Isolate platform-specific behavior at its boundary and test it on the affected OS. | Scattering platform branches through renderers or creating a separate crate for each internal module. [Herdr](https://github.com/motionharvest/herdr/blob/main/src/main.rs), [Alacritty](https://github.com/alacritty/alacritty/blob/master/alacritty/src/main.rs) |
| Responsiveness | Measure slow reads before adding workers. If workers are needed, bound queues and discard results for an outdated selection. Compute layout from the current frame size. | Queuing unlimited work, letting stale reads overwrite the visible preview, or reusing geometry after a resize. [fd](https://github.com/sharkdp/fd/blob/master/src/walk.rs), [bottom](https://github.com/ClementTsang/bottom/blob/main/src/canvas.rs) |

## Rust Failure Modes to Avoid

- **Invalid UTF-8 slicing and display width.** Use valid UTF-8 byte boundaries for slicing and terminal columns for layout. Test accented text, combining characters, wide characters, and tabs. A byte offset, a character count, and a terminal column measure different things. [Rust Book on strings](https://doc.rust-lang.org/book/ch08-02-strings.html).
- **Overflow in navigation.** Clamp bounded movement and use checked arithmetic or conversions where overflow should report an error. Exercise edge indices in debug and release builds. [Rust Book on integer overflow](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow).
- **Races and blocked reads.** Handle open failures even after successful metadata checks. A file can move or disappear between the two operations. Avoid holding locks during blocking I/O and use bounded reads for previews. [Path API](https://doc.rust-lang.org/std/path/struct.Path.html), [Read](https://doc.rust-lang.org/std/io/trait.Read.html), [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
- **Process-global environment mutation.** Keep the active locale in app state. If a future child process needs an environment override, set it on that process with `Command::env`. Rust 2024 marks `set_var` and `remove_var` unsafe because concurrent access can be unsound on some platforms. [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/newly-unsafe-functions.html).
- **Shell argument handling.** Keep previews in process. If a future feature needs an external command, use a known executable with structured arguments, handle spawn errors, and verify behavior on Windows. Keep supported toolchains patched. Rust's Windows batch-file advisories document why this boundary needs platform tests. [CVE-2024-24576](https://blog.rust-lang.org/2024/04/09/cve-2024-24576/), [CVE-2024-43402](https://blog.rust-lang.org/2024/09/04/cve-2024-43402/).

## Verifying Changes

For Rust changes, run the commands below. Clippy is an additional recommended check. Run it with the same toolchain used for compilation.

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

For interface changes, also run `cargo run -- .` in a terminal. Use app-state tests for transitions and Ratatui buffer tests for visible focus, modal overlap, and narrow layouts. Verify terminal restoration in an interactive terminal or pseudoterminal. Exercise errors during setup or a session when modifying lifecycle code. [Ratatui TestBackend](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html), [GitUI lifecycle](https://github.com/gitui-org/gitui/blob/master/src/main.rs).

## Sources

- [Native paths in Rust](https://doc.rust-lang.org/std/path/index.html) and [process arguments](https://doc.rust-lang.org/std/env/fn.args_os.html).
- [Choosing between Result and panic](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html).
- [Terminal restoration in Ratatui](https://docs.rs/ratatui/latest/ratatui/fn.try_restore.html).
- [Clippy in continuous integration](https://doc.rust-lang.org/clippy/continuous_integration/index.html) and [continuous integration with Cargo](https://doc.rust-lang.org/cargo/guide/continuous-integration.html).
