---
title: Filens Internationalization
date_created: 2026-11-24
tags: [architecture, i18n]
---

# Filens Internationalization

## Scope and Language Selection

Launch with the saved language, or English (`en`) by default. Press `o` to open Options, then open Language. Use `↑↓` to select and `Enter` to apply a language, or click its name. Save the confirmed choice to the per-user configuration. Use `Esc` to return to Options and again to restore focus to the previous panel.

## Implementation

Use `rust-i18n` to embed every catalog in `locales/` at compile time. List the embedded locales in the Language submenu and display each locale’s `locale.name`. Rebuild when catalogs are added, edited, or removed. [rust-i18n documentation](https://docs.rs/rust-i18n/latest/rust_i18n/).

- Initialize the catalogs with English fallback using `rust_i18n::i18n!("locales", fallback = "en")`.
- Store the active language in application state and update it when the user selects a language in Options. Pass it explicitly to text lookups with `t!(..., locale = ...)`.
- Use stable English keys such as `panel.tree`, `footer.browser`, and `error.read_file`. Keep complete sentences in the catalogs and interpolate named values such as the path and error cause.
- Translate application-owned text at the terminal or CLI boundary. Keep errors and paths in their original types until display. System-provided error details can follow the system language.
- Use English when a translation is missing. Check catalog keys and placeholders with `python3 scripts/check_locales.py`.

## Verification

- Test that the CLI accepts a single directory argument.
- Check catalog key and placeholder parity. Exercise missing-translation fallback separately with a test fixture.
- Test opening Options, changing language, and restoring the previous panel focus.
- Render the available languages with Ratatui's `TestBackend`. Verify that headings and footer text fit at supported terminal widths and that the focus indicator remains visible.
- Exercise startup and preview errors in the active language. Check that file names, paths, file content, and shortcuts are preserved.
