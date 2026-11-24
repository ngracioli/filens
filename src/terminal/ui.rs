use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, Padding, Paragraph, Wrap},
};
use rust_i18n::t;

use crate::app::{App, Language};

const SURFACE: Color = Color::Rgb(21, 29, 43);
const RAISED: Color = Color::Rgb(31, 43, 62);
const BORDER: Color = Color::Rgb(88, 107, 133);
const TEXT: Color = Color::Rgb(225, 234, 246);
const MUTED: Color = Color::Rgb(168, 185, 207);
const ACCENT: Color = Color::Rgb(125, 224, 201);
const DIRECTORY: Color = Color::Rgb(242, 193, 132);

pub(super) fn draw(frame: &mut Frame<'_>, app: &App, entries: &[String]) {
    let locale = app.language.locale();
    let footer_height = if frame.area().width < 56 { 2 } else { 1 };
    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(footer_height)])
        .areas(frame.area());

    let options_open = app.language_selection.is_some();
    let [tree, preview] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(body);
    let tree_title = t!("panel.tree", locale = locale);
    let preview_title = t!("panel.preview", locale = locale);
    let tree_rows = entries.iter().map(|entry| {
        let style = if entry.ends_with('/') {
            Style::new().fg(DIRECTORY)
        } else {
            Style::new().fg(TEXT)
        };
        Line::styled(entry.as_str(), style)
    });
    frame.render_widget(
        List::new(tree_rows).block(panel(
            &tree_title,
            app.tree_focused && !options_open,
            RAISED,
        )),
        tree,
    );
    frame.render_widget(
        Paragraph::new(t!("preview.empty", locale = locale))
            .style(Style::new().fg(MUTED))
            .wrap(Wrap { trim: true })
            .block(
                panel(&preview_title, !app.tree_focused && !options_open, SURFACE)
                    .padding(Padding::new(1, 1, 1, 0)),
            ),
        preview,
    );

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
        t!("footer.browser", locale = locale)
    };
    frame.render_widget(
        Paragraph::new(help)
            .style(Style::new().fg(TEXT).bg(RAISED))
            .wrap(Wrap { trim: true }),
        footer,
    );
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

    #[test]
    fn options_overlay_keeps_the_browser_visible() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut app = App::new(Language::English);
        app.handle_key(crossterm::event::KeyCode::Char('o'));
        terminal
            .draw(|frame| draw(frame, &app, &["example.txt".into()]))
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
            let mut app = App::new(language);
            terminal
                .draw(|frame| draw(frame, &app, &["ação.txt".into()]))
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert_eq!(terminal.backend().buffer()[(2, 0)].bg, ACCENT);
            assert!(!rendered.contains("Focus:") && !rendered.contains("Foco:"));
            assert!(rendered.contains("ação.txt"));
            assert!(rendered.contains(quit));
            assert!(rendered.contains("tab "));

            app.handle_key(crossterm::event::KeyCode::Char('o'));
            terminal.draw(|frame| draw(frame, &app, &[])).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(options));
            assert!(rendered.contains("Esc"));
            assert!(rendered.contains(back));
            assert!(rendered.contains(selection));
            assert!(rendered.contains("↑↓"));
        }
    }
}
