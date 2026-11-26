use std::ops::Range;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Clear, List, ListState, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};
use rust_i18n::t;

use crate::{
    app::{App, OptionsSelection, Preview},
    locales, preview,
    theme::{THEMES, Theme},
};

#[derive(Default)]
pub(super) struct PreviewViewport {
    pub(super) area: Rect,
    pub(super) rows: Vec<(usize, usize)>,
    pub(super) last_column: usize,
}

#[derive(Default)]
pub(super) struct OptionsViewport {
    area: Rect,
    offset: usize,
}

impl OptionsViewport {
    pub(super) fn row_at(&self, position: Position) -> Option<usize> {
        self.area
            .contains(position)
            .then(|| self.offset + usize::from(position.y - self.area.y))
    }
}

pub(super) fn draw(
    frame: &mut Frame<'_>,
    app: &mut App,
) -> (Rect, (Rect, u16), PreviewViewport, Rect, OptionsViewport) {
    let theme_index = match app.options_selection {
        Some(OptionsSelection::Theme(index)) => index,
        _ => app.theme,
    };
    let theme = &THEMES[theme_index];
    let locale = app.language;
    let help = match app.options_selection {
        Some(OptionsSelection::Menu(_)) => t!("footer.options_menu", locale = locale),
        Some(_) => t!("footer.options", locale = locale),
        None => browser_help(app),
    };
    let footer_height = match frame.area().width {
        0..=39 => 3,
        40..=79 => 2,
        width if usize::from(width) < Span::raw(help.as_ref()).width() => 2,
        _ => 1,
    };
    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(footer_height)])
        .areas(frame.area());

    let options_open = app.options_selection.is_some();
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
            (marker, Style::new().fg(theme.directory))
        } else {
            ("  ", Style::new().fg(theme.text))
        };
        let indent = "  ".repeat(entry.depth);
        let name = display_path(&entry.name().to_string_lossy());
        Line::styled(format!("{indent}{marker}{name}"), style)
    });
    let tree_focused = app.tree_focused && !options_open;
    let tree_panel = panel(theme, &tree_title, tree_focused, theme.raised)
        .title_bottom(display_path(&app.root.to_string_lossy()));
    let tree_text = tree_panel.inner(tree);
    let selection_style = if tree_focused {
        active_style(theme)
    } else {
        Style::new().bg(theme.border)
    };
    frame.render_stateful_widget(
        List::new(tree_rows)
            .block(tree_panel)
            .highlight_style(selection_style),
        tree,
        &mut app.tree_selection,
    );
    let preview_panel = panel(
        theme,
        &preview_title,
        !app.tree_focused && !options_open,
        theme.surface,
    )
    .padding(Padding::new(1, 1, 1, 0));
    let inner = preview_panel.inner(preview);
    frame.render_widget(preview_panel, preview);
    let mut preview_text = PreviewViewport::default();
    match &app.preview {
        Preview::Empty => draw_message(frame, inner, theme, &t!("preview.empty", locale = locale)),
        Preview::DirectoryError { path, cause } => draw_message(
            frame,
            inner,
            theme,
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
                Paragraph::new(display_path(&path.to_string_lossy()))
                    .style(Style::new().fg(theme.muted)),
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
                    preview_text = draw_document(frame, content, document, app, theme);
                    app.preview_last_column = preview_text.last_column;
                }
                Err(error) => {
                    let message = preview_error(locale, error);
                    draw_message(frame, content, theme, &message);
                }
            }
        }
    }

    let mut options_viewport = OptionsViewport::default();
    if let Some(selection) = app.options_selection {
        let language = t!("options.language", locale = locale);
        let themes = t!("options.theme", locale = locale);
        let (title, selected, rows) = match selection {
            OptionsSelection::Menu(index) => (
                t!("panel.options", locale = locale),
                index,
                vec![
                    option_row(theme, &format!("{language} ›"), index == 0),
                    option_row(theme, &format!("{themes} ›"), index == 1),
                ],
            ),
            OptionsSelection::Language(index) => (
                language,
                index,
                locales()
                    .iter()
                    .enumerate()
                    .map(|(row, locale)| {
                        option_row(
                            theme,
                            &t!("locale.name", locale = locale.as_ref()),
                            index == row,
                        )
                    })
                    .collect(),
            ),
            OptionsSelection::Theme(index) => (
                themes,
                index,
                THEMES
                    .iter()
                    .enumerate()
                    .map(|(row, item)| {
                        let label = if row == app.theme {
                            format!("{} ✓", item.name)
                        } else {
                            item.name.to_owned()
                        };
                        option_row(theme, &label, row == index)
                    })
                    .collect(),
            ),
        };
        let width = body.width.min(34);
        let row_count = rows.len();
        let height = body.height.saturating_sub(2).max(body.height.min(5));
        let height = height.min(row_count as u16 + 2);
        let mut state = ListState::default().with_selected(Some(selected));
        let modal = Rect::new(
            body.x + (body.width - width) / 2,
            body.y + (body.height - height).div_ceil(2),
            width,
            height,
        );
        let block = panel(theme, &title, true, theme.raised).padding(Padding::horizontal(1));
        let mut options_area = block.inner(modal);
        frame.render_widget(Clear, modal);
        frame.render_stateful_widget(List::new(rows).block(block), modal, &mut state);
        let visible_rows = usize::from(height.saturating_sub(2));
        if row_count > visible_rows && visible_rows > 0 && width >= 4 {
            let mut scrollbar = ScrollbarState::new(row_count - visible_rows + 1)
                .viewport_content_length(visible_rows)
                .position(state.offset());
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .style(Style::new().fg(theme.accent)),
                modal.inner(Margin::new(1, 1)),
                &mut scrollbar,
            );
        }
        options_area.height = options_area
            .height
            .min(row_count.saturating_sub(state.offset()) as u16);
        options_viewport = OptionsViewport {
            area: options_area,
            offset: state.offset(),
        };
    }
    let mut spans = Vec::new();
    for (index, action) in help.split(" · ").enumerate() {
        if index != 0 {
            spans.push(Span::raw(" · "));
        }
        let (key, label) = action.split_once(' ').unwrap_or((action, ""));
        spans.push(Span::styled(key, Style::new().fg(theme.text)));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(label));
    }
    draw_footer(frame, footer, theme, Line::from(spans));
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
        options_viewport,
    )
}

pub(super) fn draw_save_error(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    error: &std::io::Error,
) {
    let message = t!("error.save_config", locale = app.language, cause = error);
    frame.render_widget(Clear, area);
    draw_footer(frame, area, &THEMES[app.theme], Line::raw(message));
}

fn preview_error(locale: &str, error: &preview::Error) -> std::borrow::Cow<'static, str> {
    match error {
        preview::Error::Read(cause) => {
            t!("preview.read_error", locale = locale, cause = cause)
        }
        preview::Error::NotRegular => t!("preview.not_regular", locale = locale),
        preview::Error::Binary => t!("preview.binary", locale = locale),
        preview::Error::Encoding => t!("preview.encoding", locale = locale),
    }
}

pub(super) fn draw_copy_error(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    error: &preview::Error,
) {
    frame.render_widget(Clear, area);
    draw_footer(
        frame,
        area,
        &THEMES[app.theme],
        Line::raw(preview_error(app.language, error)),
    );
}

pub(super) fn draw_copy_footer(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    count: usize,
    whole_file: bool,
) {
    let locale = app.language;
    let copied = if whole_file {
        t!("footer.file_copied", locale = locale)
    } else {
        t!("footer.copied", locale = locale, count = count)
    };
    frame.render_widget(Clear, area);
    draw_footer(frame, area, &THEMES[app.theme], Line::raw(copied));
}

fn browser_help(app: &App) -> std::borrow::Cow<'static, str> {
    let locale = app.language;
    if app.tree_focused {
        return t!("footer.tree", locale = locale);
    }
    let focus = t!("footer.focus_tree", locale = locale);
    if !matches!(app.preview, Preview::File { result: Ok(_), .. }) {
        return focus;
    }
    let scroll = t!("footer.scroll", locale = locale);
    let copy = t!("footer.copy_file", locale = locale);
    if app.preview_wrap {
        let wrap = t!("footer.wrap_on", locale = locale);
        format!("{focus} · {scroll} · {copy} · {wrap}").into()
    } else {
        let horizontal = t!("footer.scroll_horizontal", locale = locale);
        let wrap = t!("footer.wrap_off", locale = locale);
        format!("{focus} · {scroll} · {horizontal} · {copy} · {wrap}").into()
    }
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, theme: &Theme, help: Line<'_>) {
    frame.render_widget(
        Paragraph::new(help)
            .style(Style::new().fg(Color::Rgb(153, 153, 153)).bg(theme.surface))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_message(frame: &mut Frame<'_>, area: Rect, theme: &Theme, message: &str) {
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::new().fg(theme.muted))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_document(
    frame: &mut Frame<'_>,
    area: Rect,
    document: &preview::Document,
    app: &App,
    theme: &Theme,
) -> PreviewViewport {
    let locale = app.language;
    let mut rows = Vec::new();
    if document.truncated {
        rows.push(Line::styled(
            t!("preview.truncated", locale = locale),
            Style::new().fg(theme.muted),
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
        .skip(app.preview_offset)
        .take(visible_lines);
    let lines = lines.map(|(index, text)| {
        let tokens = document
            .highlights
            .as_ref()
            .map_or(&[][..], |lines| &lines[index]);
        let (text, offsets) = expand_tabs(text);
        // As expansões mudam os bytes visíveis, mas os tokens apontam para a linha original.
        let colors: Vec<_> = tokens
            .iter()
            .map(|(range, token)| {
                let project = |source| {
                    let index = offsets.partition_point(|(end, _)| *end <= source);
                    let (original, expanded) = index.checked_sub(1).map_or((0, 0), |i| offsets[i]);
                    expanded + (source - original)
                };
                (project(range.start)..project(range.end), *token)
            })
            .collect();
        (index, text, colors)
    });

    let mut last_column = 0;
    if !app.preview_wrap {
        let expanded: Vec<_> = lines.collect();
        let max_width = expanded
            .iter()
            .map(|(_, text, _)| Span::raw(text).width())
            .max()
            .unwrap_or(0);
        last_column = max_width.saturating_sub(width);
        for (index, text, colors) in expanded {
            source_rows.push((index, 0));
            rows.push(numbered_row(
                theme,
                Some(index + 1),
                number_width,
                clip_columns(
                    &text,
                    app.preview_column,
                    width,
                    syntax_style(theme, &colors),
                ),
            ));
        }
    } else {
        for (index, text, colors) in lines {
            let remaining = visible_lines - source_rows.len();
            for (part, text) in wrapped_rows(&text, width, remaining, syntax_style(theme, &colors))
                .into_iter()
                .enumerate()
            {
                source_rows.push((index, part));
                rows.push(numbered_row(
                    theme,
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
    frame.render_widget(
        Paragraph::new(rows).style(Style::new().fg(theme.text)),
        area,
    );
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

fn syntax_style<'a>(
    theme: &'a Theme,
    colors: &'a [(Range<usize>, preview::Token)],
) -> impl Fn(usize) -> Style + 'a {
    |offset| {
        let index = colors.partition_point(|(range, _)| range.end <= offset);
        colors
            .get(index)
            .filter(|(range, _)| range.contains(&offset))
            .map_or_else(Style::new, |(_, token)| {
                Style::new().fg(theme.syntax[*token as usize])
            })
    }
}

fn push_span(spans: &mut Vec<Span<'static>>, symbol: &str, style: Style) {
    if let Some(last) = spans.last_mut()
        && last.style == style
    {
        last.content.to_mut().push_str(symbol);
    } else {
        spans.push(Span::styled(symbol.to_owned(), style));
    }
}

fn wrapped_rows(
    text: &str,
    width: usize,
    limit: usize,
    style: impl Fn(usize) -> Style,
) -> Vec<Vec<Span<'static>>> {
    let mut rows = Vec::new();
    let mut part = Vec::new();
    let mut used = 0;
    let mut offset = 0;
    for chunk in text.split_inclusive(char::is_whitespace) {
        let word_width = Span::raw(chunk.trim_end_matches(char::is_whitespace)).width();
        let chunk = Span::raw(chunk);
        for (position, grapheme) in chunk.styled_graphemes(Style::default()).enumerate() {
            let symbol = grapheme.symbol;
            let size = Span::raw(symbol).width();
            let style = style(offset);
            offset += symbol.len();
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
            push_span(&mut part, symbol, style);
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
        let (text, _) = expand_tabs(text);
        let width = usize::from(width);
        let parts = if app.preview_wrap {
            let limit = if line == last.row.0 {
                last.row.1 + 1
            } else {
                usize::MAX
            };
            wrapped_rows(&text, width, limit, |_| Style::new())
        } else {
            vec![clip_columns(&text, app.preview_column, width, |_| {
                Style::new()
            })]
        };
        for (part, spans) in parts.into_iter().enumerate() {
            let Some((start, end)) =
                super::selection_columns((line, part), width as u16, first, last)
            else {
                continue;
            };
            let mut column = 0;
            let mut fragment = String::new();
            for grapheme in spans
                .iter()
                .flat_map(|span| span.styled_graphemes(Style::default()))
            {
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

fn numbered_row(
    theme: &Theme,
    number: Option<usize>,
    width: usize,
    mut spans: Vec<Span<'static>>,
) -> Line<'static> {
    let number = number.map(|value| value.to_string()).unwrap_or_default();
    spans.insert(
        0,
        Span::styled(format!("{number:>width$} │ "), Style::new().fg(theme.muted)),
    );
    Line::from(spans)
}

fn expand_tabs(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut expanded = String::new();
    let mut offsets = Vec::new();
    let mut source = 0;
    let mut column = 0;
    for (index, part) in text.split('\t').enumerate() {
        if index != 0 {
            let spaces = 4 - column % 4;
            expanded.extend(std::iter::repeat_n(' ', spaces));
            column += spaces;
            source += 1;
            offsets.push((source, expanded.len()));
        }
        let start = expanded.len();
        for character in part.chars() {
            source += character.len_utf8();
            if character.is_control() {
                expanded.push('�');
                offsets.push((source, expanded.len()));
            } else {
                expanded.push(character);
            }
        }
        column += Span::raw(&expanded[start..]).width();
    }
    (expanded, offsets)
}

fn clip_columns(
    text: &str,
    start: usize,
    width: usize,
    style: impl Fn(usize) -> Style,
) -> Vec<Span<'static>> {
    let mut column = 0;
    let mut visible = Vec::new();
    let mut offset = 0;
    for grapheme in Span::raw(text).styled_graphemes(Style::default()) {
        let symbol = grapheme.symbol;
        let size = Span::raw(symbol).width();
        let next = column + size;
        if next > start + width {
            break;
        }
        let style = style(offset);
        offset += symbol.len();
        if column >= start {
            push_span(&mut visible, symbol, style);
        } else if next > start {
            push_span(&mut visible, &" ".repeat(next - start), style);
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

fn option_row(theme: &Theme, label: &str, selected: bool) -> Line<'static> {
    if selected {
        Line::styled(format!("> {label} "), active_style(theme))
    } else {
        Line::from(format!("  {label}"))
    }
}

fn active_style(theme: &Theme) -> Style {
    Style::new()
        .fg(theme.surface)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
}

fn panel<'a>(theme: &Theme, title: &'a str, focused: bool, background: Color) -> Block<'a> {
    let border = if focused { theme.accent } else { theme.border };
    let title_style = if focused {
        active_style(theme)
    } else {
        Style::new().fg(theme.muted).add_modifier(Modifier::BOLD)
    };
    Block::bordered()
        .style(Style::new().fg(theme.text).bg(background))
        .title(Span::styled(format!(" {title} "), title_style))
        .border_style(Style::new().fg(border))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn app_with_entry(language: &'static str, name: &str) -> App {
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
        let mut app = App::new("en", root.clone(), entries);
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
        let mut app = App::new("pt-BR", root.clone(), entries);
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

        let mut app = app_with_entry("en", "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: "first\nsecond\nthird\nfourth\nlast".into(),
                truncated: false,
                highlights: None,
            }),
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 8)).unwrap();
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

        let mut app = app_with_entry("en", "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: format!(
                    "first\n界\te\u{301}{}\nthird\nfourth\nfifth\nsixth\nlast",
                    "x".repeat(100)
                ),
                truncated: false,
                highlights: None,
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
        assert!(browser_help(&app).starts_with("Tab Preview"));
        let tree = render(&mut app);
        assert!(tree.contains("Enter Open"));
        assert!(!tree.contains("Copy file"));
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Down);
        assert!(browser_help(&app).starts_with("Tab Tree"));
        let preview = render(&mut app);
        assert!(preview.contains("2 │ 界  e\u{301}"));
        assert!(preview.contains("Copy file"));
        assert!(preview.contains("←→"));
        assert!(!preview.contains("Enter Open"));
        app.handle_key(KeyCode::Right);
        assert!(render(&mut app).contains("2 │    e\u{301}"));
        app.handle_key(KeyCode::Right);
        assert!(render(&mut app).contains("2 │   e\u{301}"));
        app.handle_key(KeyCode::Char('w'));
        let wrapped = render(&mut app);
        assert!(wrapped.contains("Unwrap lines"));
        assert!(!wrapped.contains("←→"));
        assert!(wrapped.contains("2 │ 界  e\u{301}"));
        assert!(wrapped.contains("  │ xxx"));
        assert!(wrapped.contains("3 │ third"));
        app.handle_key(KeyCode::Char('w'));
        assert!(render(&mut app).contains("2 │   e\u{301}"));

        app.language = "pt-BR";
        assert!(render(&mut app).contains("Quebrar linhas"));
        app.handle_key(KeyCode::Char('w'));
        assert!(render(&mut app).contains("Desativar quebra"));
    }

    #[test]
    fn markdown_keeps_source_numbers_and_theme_colors_through_layout() {
        let path = std::env::temp_dir().join(format!("filens-markdown-{}.md", std::process::id()));
        std::fs::write(
            &path,
            "# Heading with enough words to wrap\nOrdinary text\n",
        )
        .unwrap();
        let document = preview::open(&path).ok().unwrap();
        std::fs::remove_file(path).unwrap();
        let mut app = app_with_entry("en", "sample.md");
        let mut terminal = Terminal::new(TestBackend::new(20, 8)).unwrap();

        for theme in [&THEMES[0], &THEMES[5]] {
            for wrap in [false, true] {
                app.preview_wrap = wrap;
                let mut rows = Vec::new();
                terminal
                    .draw(|frame| {
                        rows = draw_document(frame, frame.area(), &document, &app, theme).rows;
                    })
                    .unwrap();
                let buffer = terminal.backend().buffer();
                let heading_color = theme.syntax[preview::Token::Heading as usize];
                assert_ne!(heading_color, theme.text);
                assert_eq!(buffer[(0, 0)].symbol(), "1");
                assert_eq!(buffer[(4, 0)].fg, heading_color);
                if wrap {
                    assert_eq!(buffer[(0, 1)].symbol(), " ");
                    assert_eq!(buffer[(4, 1)].fg, heading_color);
                }
                let second_line = rows.iter().position(|row| *row == (1, 0)).unwrap() as u16;
                assert_eq!(buffer[(0, second_line)].symbol(), "2");
                assert_eq!(buffer[(4, second_line)].fg, theme.text);
            }
        }
    }

    #[test]
    fn wrapping_moves_a_portuguese_word_to_the_next_row() {
        use crossterm::event::KeyCode;

        let mut app = app_with_entry("pt-BR", "example.txt");
        app.preview = Preview::File {
            path: "example.txt".into(),
            result: Ok(preview::Document {
                text: format!("{} ação-livre", "a".repeat(64)),
                truncated: false,
                highlights: None,
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
        let mut app = app_with_entry("en", "example.txt");
        app.handle_key(crossterm::event::KeyCode::Char('o'));
        let theme = &THEMES[app.theme];
        terminal
            .draw(|frame| {
                draw(frame, &mut app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(23, 8)].symbol(), "┌");
        assert_eq!(buffer[(23, 8)].fg, theme.accent);
        assert_eq!(buffer[(30, 10)].bg, theme.raised);
        assert_eq!(buffer[(60, 10)].bg, theme.surface);
        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("Options"));
        assert!(rendered.contains("Language ›"));
        assert!(rendered.contains("Themes ›"));
        assert!(!rendered.contains("English"));
        assert!(!rendered.contains("Tokyo Night"));
        assert!(rendered.contains("Tree"));
        assert!(rendered.contains("Preview"));
        assert!(rendered.contains("example.txt"));
    }

    #[test]
    fn options_apply_theme_to_preview_and_scroll_to_selection() {
        use crossterm::event::KeyCode;

        for (steps, label, background) in [
            (1, "> One Dark", Color::Rgb(40, 44, 52)),
            (30, "> Filens Mono", Color::Rgb(28, 28, 28)),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(32, 8)).unwrap();
            let mut app = app_with_entry("en", "example.txt");
            app.handle_key(KeyCode::Char('o'));
            app.handle_key(KeyCode::Down);
            app.handle_key(KeyCode::Enter);
            for _ in 0..steps {
                app.handle_key(KeyCode::Down);
            }
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            assert!(terminal.backend().to_string().contains(label));
            app.handle_key(KeyCode::Enter);
            app.handle_key(KeyCode::Esc);
            app.handle_key(KeyCode::Esc);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            assert_eq!(terminal.backend().buffer()[(20, 2)].bg, background);
        }
    }

    #[test]
    fn options_preview_theme_until_confirmed_or_cancelled() {
        use crossterm::event::KeyCode;

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut app = app_with_entry("en", "example.txt");
        for (apply, expected) in [(false, THEMES[0].surface), (true, THEMES[1].surface)] {
            app.handle_key(KeyCode::Char('o'));
            app.handle_key(KeyCode::Down);
            app.handle_key(KeyCode::Enter);
            app.handle_key(KeyCode::Down);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            assert_eq!(terminal.backend().buffer()[(60, 10)].bg, THEMES[1].surface);
            if apply {
                app.handle_key(KeyCode::Enter);
                app.handle_key(KeyCode::Down);
                terminal
                    .draw(|frame| {
                        draw(frame, &mut app);
                    })
                    .unwrap();
                assert_eq!(terminal.backend().buffer()[(60, 10)].bg, THEMES[2].surface);
            }
            app.handle_key(KeyCode::Esc);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            assert_eq!(terminal.backend().buffer()[(60, 10)].bg, expected);
        }
    }

    #[test]
    fn language_menu_shows_each_catalog_name_at_narrow_width() {
        use crossterm::event::KeyCode;

        for locale in locales() {
            let mut terminal = Terminal::new(TestBackend::new(32, 8)).unwrap();
            let mut app = app_with_entry(locale, "example.txt");
            app.handle_key(KeyCode::Char('o'));
            app.handle_key(KeyCode::Enter);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            let name = t!("locale.name", locale = locale.as_ref());
            let screen = terminal.backend().to_string();
            assert!(screen.contains(name.as_ref()), "{locale}: {screen}");
        }
    }

    #[test]
    fn renders_both_languages_at_narrow_width() {
        for (language, options, back, selection, quit) in [
            ("en", "Options", "Close", "> English", "Quit"),
            ("pt-BR", "Opções", "Fechar", "> Português do Brasil", "Sair"),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(32, 8)).unwrap();
            let mut app = app_with_entry(language, "ação.txt");
            let theme = &THEMES[app.theme];
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert_eq!(terminal.backend().buffer()[(2, 0)].bg, theme.accent);
            assert!(!rendered.contains("Focus:") && !rendered.contains("Foco:"));
            assert!(rendered.contains("ação.txt"));
            assert!(rendered.contains(quit));
            assert!(rendered.contains("Tab "));

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
            assert!(rendered.contains("↑↓"));
            assert!(!rendered.contains(selection));

            app.handle_key(crossterm::event::KeyCode::Enter);
            terminal
                .draw(|frame| {
                    draw(frame, &mut app);
                })
                .unwrap();
            assert!(terminal.backend().to_string().contains(selection));
        }
    }
}
