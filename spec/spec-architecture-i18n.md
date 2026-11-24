---
title: Filens Internationalization
date_created: 2026-11-24
tags: [architecture, i18n]
---

# Filens Internationalization

## Scope and Language Selection

Support English (`en`) and Brazilian Portuguese (`pt-BR`). Use English when the user omits the language, regardless of the operating system settings. Select Brazilian Portuguese for a launch with `filens --lang pt-BR .`. Accept `--lang en` explicitly. Press `o` to open Options. Use `↑` and `↓` to choose a language and `Enter` to apply it immediately for the current session. Press `Esc` to close Options and restore focus to the previous panel. A new launch follows the command-line option or the English default. Report unsupported language values as CLI usage errors in English before entering the alternate screen.

## Implementation

Use `rust-i18n` with `locales/en.toml` and `locales/pt-BR.toml` at the package root. Its compile-time macro embeds both catalogs in the executable and supports English fallback and named interpolation. Both languages travel with every release binary. This fits the MVP's labels and short messages. Revisit Fluent if plural rules or grammatical variants become a recurring requirement. [rust-i18n documentation](https://docs.rs/rust-i18n/latest/rust_i18n/), [Project Fluent for Rust](https://github.com/projectfluent/fluent-rs).

- Initialize the catalogs with English fallback using `rust_i18n::i18n!("locales", fallback = "en")`.
- Resolve the starting language when parsing arguments and store the active language in application state. Update that state when the user selects a language in Options. Pass it explicitly to text lookups with `t!(..., locale = ...)` so tests can render either language independently.
- Use stable English keys such as `panel.tree`, `footer.browser`, and `error.read_file`. Keep complete sentences in the catalogs and interpolate named values such as the path and error cause.
- Translate application-owned text at the terminal or CLI boundary. Keep errors and paths in their original types until display. System-provided error details can follow the system language.
- Use English when a Portuguese message is missing. Keep the same keys and placeholder names in both catalogs for releases.

## Verification

- Test language parsing for the default, both supported values, and an invalid value.
- Check catalog key and placeholder parity. Exercise missing-translation fallback separately with a test fixture.
- Test opening Options, changing language, and restoring the previous panel focus.
- Render both languages with Ratatui's `TestBackend`. Verify that headings and footer text fit at supported terminal widths and that the focus indicator remains visible.
- Exercise CLI and preview errors in both languages. Check that file names, paths, file content, and shortcuts are preserved.
