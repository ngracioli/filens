use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, Padding, Paragraph, Wrap},
};
use rust_i18n::t;

use crate::{
    app::{App, Language, Preview},
    preview,
};

const SURFACE: Color = Color::Rgb(21, 29, 43);
const RAISED: Color = Color::Rgb(31, 43, 62);
const BORDER: Color = Color::Rgb(88, 107, 133);
const TEXT: Color = Color::Rgb(225, 234, 246);
const MUTED: Color = Color::Rgb(168, 185, 207);
const ACCENT: Color = Color::Rgb(125, 224, 201);
const DIRECTORY: Color = Color::Rgb(242, 193, 132);

#[derive(Default)]
pub(super) struct PreviewViewport {
    pub(super) area: Rect,
    pub(super) rows: Vec<(usize, usize)>,
    pub(super) last_column: usize,
}

pub(super) fn draw(
    frame: &mut Frame<'_>,
    app: &mut App,
) -> (Rect, (Rect, u16), PreviewViewport, Rect) {
    let locale = app.language.locale();
    let browser_help = browser_help(app);
    let count_help = t!("footer.copied", locale = locale, count = 999999);
    let footer_height = match frame.area().width {
        0..=39 => 3,
        40..=79 => 2,
        width
            if app.language_selection.is_none()
                && usize::from(width)
                    < Span::raw(format!("{browser_help} {count_help}")).width() =>
        {
            2
        }
        _ => 1,
    };
    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(footer_height)])
        .areas(frame.area());

    let options_open = app.language_selection.is_some();
    let [tree, preview] = Layout::horizontal([
        Constraint::Length((body.width / 4).max(12).min(body.width / 2)),
        Constraint::Min(0),
    ])
    .areas(body);
    let tree_title = t!("panel.tree", locale = locale);
    let preview_title = t!("panel.preview", locale = locale);
    let tree_rows = app.entries.iter().map(|entry| {
        let (marker, style) = if entry.kind.is_dir() {
            let marker = if entry.expanded { "▾ " } else { "▸ " };
            (marker, Style::new().fg(DIRECTORY))
        } else {
            ("  ", Style::new().fg(TEXT))
        };
        let indent = "  ".repeat(entry.depth);
        let name = display_path(&entry.name().to_string_lossy());
        Line::styled(format!("{indent}{marker}{name}"), style)
    });
    let tree_focused = app.tree_focused && !options_open;
    let tree_panel = panel(&tree_title, tree_focused, RAISED)
        .title_bottom(display_path(&app.root.to_string_lossy()));
    let tree_text = tree_panel.inner(tree);
    let selection_style = if tree_focused {
        active_style()
    } else {
        Style::new().bg(BORDER)
    };
    frame.render_stateful_widget(
        List::new(tree_rows)
            .block(tree_panel)
            .highlight_style(selection_style),
        tree,
        &mut app.tree_selection,
    );
    let preview_panel = panel(&preview_title, !app.tree_focused && !options_open, SURFACE)
        .padding(Padding::new(1, 1, 1, 0));
    let inner = preview_panel.inner(preview);
    frame.render_widget(preview_panel, preview);
    let mut preview_text = PreviewViewport::default();
    match &app.preview {
        Preview::Empty => draw_message(frame, inner, &t!("preview.empty", locale = locale)),
        Preview::DirectoryError { path, cause } => draw_message(
            frame,
            inner,
            &t!(
                "error.read_directory",
                locale = locale,
                path = display_path(&path.to_string_lossy()),
                cause = cause
            ),
        ),
        Preview::File { path, result } => {
            let [path_row, content] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
            frame.render_widget(
                Paragraph::new(display_path(&path.to_string_lossy())).style(Style::new().fg(MUTED)),
                path_row,
            );
            match result {
                Ok(document) => {
                    let visible_lines =
                        usize::from(content.height).saturating_sub(usize::from(document.truncated));
                    let line_count = document.text.lines().count();
                    app.preview_last_offset = line_count.saturating_sub(if app.preview_wrap {
                        1
                    } else {
                        visible_lines.max(1)
                    });
                    preview_text = draw_document(
                        frame,
                        content,
                        document,
                        locale,
                        app.preview_offset,
                        app.preview_wrap,
                        app.preview_column,
                    );
                    app.preview_last_column = preview_text.last_column;
                }
                Err(error) => {
                    let message = match error {
                        preview::Error::Read(cause) => {
                            t!("preview.read_error", locale = locale, cause = cause)
                        }
                        preview::Error::NotRegular => t!("preview.not_regular", locale = locale),
                        preview::Error::Binary => t!("preview.binary", locale = locale),
                        preview::Error::Encoding => t!("preview.encoding", locale = locale),
                    };
                    draw_message(frame, content, &message);
                }
            }
        }
    }

    let help = if let Some(selected) = app.language_selection {
        let rows = [
            Line::styled(
                t!("options.language", locale = locale),
                Style::new().fg(MUTED),
            ),
            option_row(
                t!("options.english", locale = locale).as_ref(),
                selected == Language::English,
            ),
            option_row(
                t!("options.portuguese", locale = locale).as_ref(),
                selected == Language::BrazilianPortuguese,
            ),
        ];
        let width = body.width.min(34);
        let height = body.height.min(5);
        let modal = Rect::new(
            body.x + (body.width - width) / 2,
            body.y + (body.height - height).div_ceil(2),
            width,
            height,
        );
        frame.render_widget(Clear, modal);
        frame.render_widget(
            List::new(rows).block(
                panel(&t!("panel.options", locale = locale), true, RAISED)
                    .padding(Padding::horizontal(1)),
            ),
            modal,
        );
        t!("footer.options", locale = locale)
    } else {
        browser_help
    };
    draw_footer(frame, footer, &help);
    let tree_rows = tree_text.height.min(
        app.entries
            .len()
            .saturating_sub(app.tree_selection.offset())
            .try_into()
            .unwrap_or(u16::MAX),
    );
    (
        preview,
        (tree_text, tree_text.y + tree_rows),
        preview_text,
        footer,
    )
}

pub(super) fn draw_copy_footer(frame: &mut Frame<'_>, area: Rect, app: &App, count: usize) {
    let locale = app.language.locale();
    let help = browser_help(app);
    let copied = t!("footer.copied", locale = locale, count = count);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(format!("{help} ")),
            Span::styled(copied, Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]))
        .style(Style::new().fg(TEXT).bg(RAISED))
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn browser_help(app: &App) -> std::borrow::Cow<'static, str> {
    let locale = app.language.locale();
    let help = t!("footer.browser", locale = locale);
    if !matches!(app.preview, Preview::File { result: Ok(_), .. }) {
        return help;
    }
    let wrap = if app.preview_wrap {
        t!("footer.wrap_on", locale = locale)
    } else {
        t!("footer.wrap_off", locale = locale)
    };
    format!("{wrap} | {help}").into()
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, help: &str) {
    frame.render_widget(
        Paragraph::new(help)
            .style(Style::new().fg(TEXT).bg(RAISED))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_message(frame: &mut Frame<'_>, area: Rect, message: &str) {
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::new().fg(MUTED))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_document(
    frame: &mut Frame<'_>,
    area: Rect,
    document: &preview::Document,
    locale: &str,
    offset: usize,
    wrap: bool,
    column: usize,
) -> PreviewViewport {
    let mut rows = Vec::new();
    if document.truncated {
        rows.push(Line::styled(
            t!("preview.truncated", locale = locale),
            Style::new().fg(MUTED),
        ));
    }
    let visible_lines = usize::from(area.height).saturating_sub(rows.len());
    let number_width = document.text.lines().count().to_string().len();
    let gutter = (number_width as u16 + 3).min(area.width);
    let width = usize::from(area.width - gutter);
    let text_y = area.y.saturating_add(rows.len() as u16);
    let mut source_rows = Vec::new();
    let lines = document
        .text
        .lines()
        .enumerate()
        .skip(offset)
        .take(visible_lines);
    let lines = lines.map(|(index, text)| (index, expand_tabs(text)));

    let mut last_column = 0;
    if !wrap {
        let expanded: Vec<_> = lines.collect();
        let max_width = expanded
            .iter()
            .map(|(_, text)| Span::raw(text).width())
            .max()
            .unwrap_or(0);
        last_column = max_width.saturating_sub(width);
        for (index, text) in expanded {
            source_rows.push((index, 0));
            rows.push(numbered_row(
                Some(index + 1),
                number_width,
                clip_columns(&text, column, width),
            ));
        }
    } else {
        for (index, text) in lines {
            let remaining = visible_lines - source_rows.len();
            for (part, text) in wrapped_rows(&text, width, remaining)
                .into_iter()
                .enumerate()
            {
                source_rows.push((index, part));
                rows.push(numbered_row(
                    (part == 0).then_some(index + 1),
                    number_width,
                    text,
                ));
            }
            if source_rows.len() == visible_lines {
                break;
            }
        }
    }
    frame.render_widget(Paragraph::new(rows).style(Style::new().fg(TEXT)), area);
    PreviewViewport {
        area: Rect::new(
            area.x + gutter,
            text_y,
            area.width - gutter,
            visible_lines as u16,
        ),
        rows: source_rows,
        last_column,
    }
}

fn wrapped_rows(text: &str, width: usize, limit: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut part = String::new();
    let mut used = 0;
    for chunk in text.split_inclusive(char::is_whitespace) {
        let word_width = Span::raw(chunk.trim_end_matches(char::is_whitespace)).width();
        let chunk = Span::raw(chunk);
        for (position, grapheme) in chunk.styled_graphemes(Style::default()).enumerate() {
            let symbol = grapheme.symbol;
            let size = Span::raw(symbol).width();
            if size > width {
                continue;
            }
            let keep_word = position == 0 && word_width <= width && used + word_width > width;
            if keep_word || used + size > width {
                rows.push(std::mem::take(&mut part));
                used = 0;
                if rows.len() == limit {
                    return rows;
                }
            }
            part.push_str(symbol);
            used += size;
        }
    }
    rows.push(part);
    rows
}

pub(super) fn selected_preview_text(
    app: &App,
    width: u16,
    first: super::PreviewPoint,
    last: super::PreviewPoint,
) -> String {
    let Preview::File {
        result: Ok(document),
        ..
    } = &app.preview
    else {
        return String::new();
    };
    let mut selected = Vec::new();
    for (line, text) in document
        .text
        .lines()
        .enumerate()
        .take(last.row.0 + 1)
        .skip(first.row.0)
    {
        let text = expand_tabs(text);
        let width = usize::from(width);
        let parts = if app.preview_wrap {
            let limit = if line == last.row.0 {
                last.row.1 + 1
            } else {
                usize::MAX
            };
            wrapped_rows(&text, width, limit)
        } else {
            vec![clip_columns(&text, app.preview_column, width)]
        };
        for (part, text) in parts.into_iter().enumerate() {
            let Some((start, end)) =
                super::selection_columns((line, part), width as u16, first, last)
            else {
                continue;
            };
            let mut column = 0;
            let mut fragment = String::new();
            for grapheme in Span::raw(text.as_str()).styled_graphemes(Style::default()) {
                let next = column + Span::raw(grapheme.symbol).width();
                if column >= usize::from(end) {
                    break;
                }
                if next > usize::from(start) {
                    fragment.push_str(grapheme.symbol);
                }
                column = next;
            }
            fragment.truncate(fragment.trim_end_matches(' ').len());
            selected.push(fragment);
        }
    }
    selected.join("\n")
}

fn numbered_row(number: Option<usize>, width: usize, text: String) -> Line<'static> {
    let number = number.map(|value| value.to_string()).unwrap_or_default();
    Line::from(vec![
        Span::styled(format!("{number:>width$} │ "), Style::new().fg(MUTED)),
        Span::raw(text),
    ])
}

fn expand_tabs(text: &str) -> String {
    let mut expanded = String::new();
    let mut column = 0;
    for (index, part) in text.split('\t').enumerate() {
        if index != 0 {
            let spaces = 4 - column % 4;
            expanded.extend(std::iter::repeat_n(' ', spaces));
            column += spaces;
        }
        let part = display_path(part);
        column += Span::raw(&part).width();
        expanded.push_str(&part);
    }
    expanded
}

fn clip_columns(text: &str, start: usize, width: usize) -> String {
    let mut column = 0;
    let mut visible = String::new();
    for grapheme in Span::raw(text).styled_graphemes(Style::default()) {
        let size = Span::raw(grapheme.symbol).width();
        let next = column + size;
        if next > start + width {
            break;
        }
        if column >= start {
            visible.push_str(grapheme.symbol);
        } else if next > start {
            visible.extend(std::iter::repeat_n(' ', next - start));
        }
        column = next;
    }
    visible
}

fn display_path(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .collect()
}

fn option_row(label: &str, selected: bool) -> Line<'static> {
    if selected {
        Line::styled(format!("> {label} "), active_style())
    } else {
        Line::from(format!("  {label}"))
    }
}

fn active_style() -> Style {
    Style::new()
        .fg(SURFACE)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

fn panel(title: &str, focused: bool, background: Color) -> Block<'_> {
    let border = if focused { ACCENT } else { BORDER };
    let title_style = if focused {
        active_style()
    } else {
        Style::new().fg(MUTED).add_modifier(Modifier::BOLD)
    };
    Block::bordered()
        .style(Style::new().fg(TEXT).bg(background))
        .title(Span::styled(format!(" {title} "), title_style))
        .border_style(Style::new().fg(border))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn app_with_entry(language: Language, name: &str) -> App {
        let entry = crate::tree::Entry {
            path: name.into(),
            kind: std::fs::metadata("Cargo.toml").unwrap().file_type(),
            depth: 0,
            expanded: false,
        };
        App::new(language, ".".into(), vec![entry])
    }

    #[test]
    fn keyboard_expands_previews_and_hides_descendants() {
        use crossterm::event::KeyCode;

        let root = std::env::temp_dir().join(format!("filens-tree-{}", std::process::id()));
        let folder = root.join("folder");
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("child.txt");
        std::fs::write(&file, "").unwrap();
        let entries = crate::tree::read_directory(&root, 0).unwrap();
        let mut app = App::new(Language::English, root.clone(), entries);
        let mut terminal = Terminal::new(TestBackend::new(120, 12)).unwrap();

        app.handle_key(KeyCode::Enter);
        app.handle_key(KeyCode::Down);
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Up);
        app.handle_key(KeyCode::Enter);
        assert!(matches!(app.preview, Preview::Empty));
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Enter);
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();
        assert!(matches!(&app.preview, Preview::File { path, result: Ok(_) } if path == &file));
        assert!(
            terminal
                .backend()
                .to_string()
                .contains(&file.to_string_lossy().into_owned())
        );

        app.handle_key(KeyCode::Up);
        app.handle_key(KeyCode::Enter);
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();
        let rendered = terminal.backend().to_string();
        assert_eq!(rendered.matches("child.txt").count(), 1);
        assert_eq!(app.entries.len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn previews_numbered_text_and_recovers_from_file_errors() {
        use crossterm::event::KeyCode;

        let root = std::env::temp_dir().join(format!("filens-preview-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let utf16 = [0xfe, 0xff]
            .into_iter()
            .chain("ação\n\nfim".encode_utf16().flat_map(u16::to_be_bytes))
            .collect::<Vec<_>>();
        let files: [(&str, &[u8]); 5] = [
            ("a-binary", b"example\0"),
            ("b-encoding", b"caf\xe9"),
            ("c-removed", b"example"),
            ("d-utf16", &utf16),
            ("e-utf8", " ação\r\n\r\nfim\r\n".as_bytes()),
        ];
        for (name, bytes) in files {
            std::fs::write(root.join(name), bytes).unwrap();
        }
        let entries = crate::tree::read_directory(&root, 0).unwrap();
        std::fs::remove_file(root.join("c-removed")).unwrap();
        let mut app = App::new(Language::BrazilianPortuguese, root.clone(), entries);
        let mut terminal = Terminal::new(TestBackend::new(120, 16)).unwrap();

        for (index, expected) in [
            "Arquivo binário.",
            "Codificação de texto não reconhecida.",
            "Não foi possível ler o arquivo:",
            "1 │ ação",
            "1 │  ação",
        ]
        .into_iter()
        .enumerate()
        {
            app.handle_key(KeyCode::Enter);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(expected), "{rendered}");
            assert!(app.tree_focused);
            if index >= 3 {
                assert!(rendered.contains("2 │"));
                assert!(rendered.contains("3 │ fim"));
                assert!(!rendered.contains("4 │"));
                let (name, original) = files[index];
                assert!(rendered.contains(root.join(name).to_str().unwrap()));
                assert_eq!(std::fs::read(root.join(name)).unwrap(), original);
            }
            app.handle_key(KeyCode::Down);
        }

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preview_arrows_scroll_numbered_lines_and_stop_at_the_end() {
        use crossterm::event::KeyCode;

        let mut app = app_with_entry(Language::English, "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: "first\nsecond\nthird\nfourth\nlast".into(),
                truncated: false,
            }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 9)).unwrap();
        app.handle_key(KeyCode::Tab);
        for _ in 0..8 {
            app.handle_key(KeyCode::Down);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
        }
        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("3 │ third"));
        assert!(rendered.contains("5 │ last"));
        assert!(!rendered.contains("2 │ second"));

        app.handle_key(KeyCode::Up);
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();
        assert!(terminal.backend().to_string().contains("2 │ second"));
    }

    #[test]
    fn preview_wrap_and_horizontal_scroll_preserve_source_numbers() {
        use crossterm::event::KeyCode;

        let mut app = app_with_entry(Language::English, "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: format!(
                    "first\n界\te\u{301}{}\nthird\nfourth\nfifth\nsixth\nlast",
                    "x".repeat(100)
                ),
                truncated: false,
            }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 10)).unwrap();
        let mut render = |app: &mut App| {
            terminal
                .draw(|frame| {
                    draw(frame, app);
                })
                .unwrap();
            terminal.backend().to_string()
        };
        assert!(render(&mut app).contains("Wrap off"));
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Down);
        assert!(render(&mut app).contains("2 │ 界  e\u{301}"));
        app.handle_key(KeyCode::Right);
        assert!(render(&mut app).contains("2 │    e\u{301}"));
        app.handle_key(KeyCode::Right);
        assert!(render(&mut app).contains("2 │   e\u{301}"));
        app.handle_key(KeyCode::Char('w'));
        let wrapped = render(&mut app);
        assert!(wrapped.contains("Wrap on"));
        assert!(wrapped.contains("2 │ 界  e\u{301}"));
        assert!(wrapped.contains("  │ xxx"));
        assert!(wrapped.contains("3 │ third"));
        app.handle_key(KeyCode::Char('w'));
        assert!(render(&mut app).contains("2 │   e\u{301}"));

        app.language = Language::BrazilianPortuguese;
        assert!(render(&mut app).contains("Quebra desligada"));
        app.handle_key(KeyCode::Char('w'));
        assert!(render(&mut app).contains("Quebra ligada"));
    }

    #[test]
    fn wrapping_moves_a_portuguese_word_to_the_next_row() {
        use crossterm::event::KeyCode;

        let mut app = app_with_entry(Language::BrazilianPortuguese, "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: format!("{} ação-livre", "a".repeat(64)),
                truncated: false,
            }),
        };
        app.handle_key(KeyCode::Char('w'));
        let mut terminal = Terminal::new(TestBackend::new(100, 10)).unwrap();
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("  │ ação-livre"));
    }

    #[test]
    fn options_overlay_keeps_the_browser_visible() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut app = app_with_entry(Language::English, "example.txt");
        app.handle_key(crossterm::event::KeyCode::Char('o'));
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(23, 7)].symbol(), "┌");
        assert_eq!(buffer[(23, 7)].fg, ACCENT);
        assert_eq!(buffer[(30, 10)].bg, RAISED);
        assert_eq!(buffer[(60, 10)].bg, SURFACE);
        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("Options"));
        assert!(rendered.contains("Tree"));
        assert!(rendered.contains("Preview"));
        assert!(rendered.contains("example.txt"));
    }

    #[test]
    fn renders_both_languages_at_narrow_width() {
        for (language, options, back, selection, quit) in [
            (Language::English, "Options", "Back", "> English", "Quit"),
            (
                Language::BrazilianPortuguese,
                "Opções",
                "Voltar",
                "> Português do Brasil",
                "Sair",
            ),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(32, 8)).unwrap();
            let mut app = app_with_entry(language, "ação.txt");
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert_eq!(terminal.backend().buffer()[(2, 0)].bg, ACCENT);
            assert!(!rendered.contains("Focus:") && !rendered.contains("Foco:"));
            assert!(rendered.contains("ação.txt"));
            assert!(rendered.contains(quit));
            assert!(rendered.contains("tab "));

            app.handle_key(crossterm::event::KeyCode::Char('o'));
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(options));
            assert!(rendered.contains("Esc"));
            assert!(rendered.contains(back));
            assert!(rendered.contains(selection));
            assert!(rendered.contains("↑↓"));
        }
    }
}
