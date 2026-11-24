---
title: Filens Architecture
date_created: 2026-11-24
tags: [architecture]
---

# Filens Architecture

Filens is a terminal file browser with a tree on the left and a read-only preview on the right. The architecture uses one Cargo package with one executable and internal modules organized by responsibility.

## 1. Scope

This layout guides development from the implementation in `src/main.rs`. Use a directory for each feature area shown below. Add its files when the corresponding behavior enters the project. Preserve `filens <directory>` and `filens .` as executable entry points.

## 2. Layout and Responsibilities

```text
Cargo.toml
Cargo.lock
src/
  main.rs       # Arguments, fatal messages, and exit status
  app.rs        # Focus, selection, and state transitions
  terminal/
    mod.rs      # Session lifecycle, keyboard input, and resizing
    ui.rs       # Ratatui layout, panels, and footer
  tree/
    mod.rs      # Directory entries, expansion, and display order
  preview/
    mod.rs      # Reading, decoding, and preview content
tests/
  cli.rs        # Observable executable behavior
```

- `main.rs` validates arguments and prepares the root before entering full-screen mode. It reports fatal errors after attempting to restore the terminal.
- `terminal/mod.rs` manages raw mode, the alternate screen, and the event loop. It translates Crossterm key events into application actions.
- `app.rs` receives actions such as switching focus, moving the selection, and quitting. The tree and preview provide concrete operations for actions that need file system access.
- `terminal/ui.rs` reads state, computes the layout, and draws the frame. File reads happen when an action requests new data. Redrawing the screen or switching focus reuses the data already available.
- `tree/mod.rs` keeps the actual path of each entry separate from its label. `preview/mod.rs` uses that path to read the selected file. When syntax highlighting is added, its state belongs to the open file.

## 3. Contracts

| Boundary | Contract |
| --- | --- |
| Arguments → application | Native path in a `PathBuf`. Incorrect usage or an inaccessible root produces a nonzero exit status. |
| Terminal → application | Actions use plain Rust types. Tests can exercise state and actions without creating a terminal. |
| Tree → preview | Path of the selected item. A read failure appears in the preview while navigation stays available. |
| Application → interface | The interface borrows state and renders with Ratatui. Text indicates which panel has focus. |

## 4. Acceptance Criteria and Tests

- Test focus and selection transitions in their owning modules with `#[cfg(test)]`.
- For interface tests, use `TestBackend` to check relevant cells, including the footer at small terminal sizes.
- Exercise CLI failures through the executable in `tests/cli.rs` using `CARGO_BIN_EXE_filens`.
- For terminal lifecycle changes, run `cargo run -- .`, switch focus in both directions, and confirm that `q` restores the terminal. Test errors during the session when the change affects an error path.

## 5. Rationale and Evolution

A single package follows Cargo conventions and keeps internal boundaries private. Feature directories give related implementation and tests a predictable home. Add submodules such as `preview/decode.rs` and `preview/highlight.rs` as those responsibilities grow. Split into crates when a separate consumer emerges or a component needs to be built and distributed independently. Add `src/lib.rs` within the package when a consumer needs its API or an integration test needs to exercise that API directly.

Start with synchronous execution. For large directories or slow files, measure the delay users experience before introducing concurrency. If a read noticeably blocks keyboard input, isolate the slow operation and discard results from earlier selections.

## 6. Dependencies and Further Reading

- Rust and Cargo for the package and tests. [Cargo package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html), [test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html).
- Crossterm for events and terminal control. [Crossterm documentation](https://docs.rs/crossterm/0.29.0/crossterm/).
- Ratatui for rendering and in-memory tests. [Application patterns](https://ratatui.rs/concepts/application-patterns/), [TestBackend](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html).
- [Rust practices](../docs/rust-practices.md) and [portability and support](../docs/portability.md).
