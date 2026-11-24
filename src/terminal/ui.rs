use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
    widgets::{Block, List, Paragraph, Wrap},
};
use rust_i18n::t;

use crate::app::{App, Language};

pub(super) fn draw(frame: &mut Frame<'_>, app: &App, entries: &[String]) {
    let locale = app.language.locale();
    let footer_height = if frame.area().width < 56 { 2 } else { 1 };
    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(footer_height)])
        .areas(frame.area());

    let help = if app.options_open {
        let english_marker = if app.language == Language::English {
            "*"
        } else {
            " "
        };
        let portuguese_marker = if app.language == Language::BrazilianPortuguese {
            "*"
        } else {
            " "
        };
        let rows = [
            t!("options.language", locale = locale).into_owned(),
            format!(
                "{english_marker} 1  {}",
                t!("options.english", locale = locale)
            ),
            format!(
                "{portuguese_marker} 2  {}",
                t!("options.portuguese", locale = locale)
            ),
        ];
        frame.render_widget(
            List::new(rows).block(panel(&t!("panel.options", locale = locale), true)),
            body,
        );
        t!("footer.options", locale = locale)
    } else {
        let [tree, preview] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(body);
        let tree_title = t!("panel.tree", locale = locale);
        let preview_title = t!("panel.preview", locale = locale);
        frame.render_widget(
            List::new(entries.iter().map(String::as_str))
                .block(panel(&tree_title, app.tree_focused)),
            tree,
        );
        frame.render_widget(panel(&preview_title, !app.tree_focused), preview);

        let focus = if app.tree_focused {
            tree_title
        } else {
            preview_title
        };
        t!("footer.focus", locale = locale, panel = focus)
    };
    frame.render_widget(Paragraph::new(help).wrap(Wrap { trim: true }), footer);
}

fn panel(title: &str, focused: bool) -> Block<'_> {
    let border_color = if focused { Color::Cyan } else { Color::Reset };
    Block::bordered().title(title).border_style(border_color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_both_languages_at_narrow_width() {
        for (language, focus, options, back) in [
            (Language::English, "Focus: Tree", "Options", "Esc back"),
            (
                Language::BrazilianPortuguese,
                "Foco: Árvore",
                "Opções",
                "Esc volta",
            ),
        ] {
            let mut terminal = Terminal::new(TestBackend::new(32, 8)).unwrap();
            let mut app = App::new(language);
            terminal
                .draw(|frame| draw(frame, &app, &["ação.txt".into()]))
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(focus));
            assert!(rendered.contains("ação.txt"));

            app.options_open = true;
            terminal.draw(|frame| draw(frame, &app, &[])).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(options));
            assert!(rendered.contains(back));
        }
    }
}
