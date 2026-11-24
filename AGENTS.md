# Filens

Filens is a terminal application for browsing directories and previewing files.

## Stack

- Rust 2024 with Cargo.
- Ratatui for the terminal interface.
- Crossterm for terminal input and control.

## Project conventions

- Keep changes focused, follow nearby conventions, and use English identifiers.
- Read files in `.context/` when the user points to a relevant note. Keep that directory out of commits.
- After Rust changes, run `cargo fmt --check`, `cargo check --locked`, and `cargo test --locked`.
- For terminal UI changes, run `cargo run -- .` to verify the affected behavior.
- Commit or push only when the user asks. Follow the [project commit skill](.agents/skills/commit/SKILL.md).
