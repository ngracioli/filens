# Filens

Filens is a terminal application for browsing directories and previewing files.

## Stack

- Rust 2024 with Cargo.
- Ratatui for the terminal interface.
- Crossterm for terminal input and control.

## Project conventions

- Keep changes focused, follow nearby conventions, and use English identifiers.
- Read and apply the relevant sections of [Rust practices](docs/rust-practices.md) before writing or reviewing Rust code.

## Internationalization

- Localize all Filens-owned interface, help, and error text with `rust-i18n` and embedded `locales/*.toml` catalogs. English (`en`) is the default, fallback, and source for translations.
- Look up messages with the active locale from application state and stable English keys. Store complete messages in the catalogs and use named placeholders for runtime values. Preserve paths, file contents, and key bindings.
- Add a language by creating `locales/<locale>.toml` with `locale.name` set to its name in that language. The language submenu discovers embedded catalogs automatically. Change the language inside Options for the current session.
- Add new message keys to `en.toml` first and update translations across the catalog set. Run `python3 scripts/check_locales.py` with Python 3.11+ to list missing keys and verify placeholders. Review generated translations and check Unicode rendering and key hints at narrow terminal sizes.

## Development workflow

- Read files in `.context/` when the user points to a relevant note. Keep that directory out of commits.
- After Rust changes, run `cargo fmt --check`, `cargo check --locked`, and `cargo test --locked`.
- For terminal UI changes, run `cargo run -- .` to verify the affected behavior.
- Commit or push only when the user asks. Follow the [project commit skill](.agents/skills/commit/SKILL.md).
