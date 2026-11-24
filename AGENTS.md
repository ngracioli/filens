# Filens

Filens is a terminal application for browsing directories and previewing files.

## Stack

- Rust 2024 with Cargo.
- Ratatui for the terminal interface.
- Crossterm for terminal input and control.

## Project conventions

- Keep changes focused, follow nearby conventions, and use English identifiers.

## Internationalization

- Localize all Filens-owned interface, help, and error text. Use `rust-i18n` with embedded `locales/en.toml` and `locales/pt-BR.toml` catalogs. English (`en`) is the default and fallback. Brazilian Portuguese (`pt-BR`) is the other supported locale.
- Look up messages with the active locale from application state and stable English keys. Store complete messages in the catalogs and use named placeholders for runtime values. Preserve paths, file contents, and key bindings.
- Update both catalogs when adding or changing a message. Verify matching keys and placeholders, English fallback, and terminal layout in both languages. See the [i18n specification](spec/spec-architecture-i18n.md).

## Development workflow

- Read files in `.context/` when the user points to a relevant note. Keep that directory out of commits.
- After Rust changes, run `cargo fmt --check`, `cargo check --locked`, and `cargo test --locked`.
- For terminal UI changes, run `cargo run -- .` to verify the affected behavior.
- Commit or push only when the user asks. Follow the [project commit skill](.agents/skills/commit/SKILL.md).
