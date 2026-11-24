use std::{env, io, path::PathBuf, process::ExitCode};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::Color,
    widgets::{Block, List, Paragraph},
};

fn main() -> ExitCode {
    match open_directory() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("filens: {error}");
            ExitCode::FAILURE
        }
    }
}

fn open_directory() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let path = match (args.next(), args.next()) {
        (Some(path), None) => PathBuf::from(path),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Uso: filens <diretório>",
            ));
        }
    };

    let mut entries = path
        .read_dir()?
        .map(|entry| {
            let entry = entry?;
            let name = entry.file_name();
            let suffix = if entry.file_type()?.is_dir() { "/" } else { "" };
            Ok(format!("  {}{suffix}", name.to_string_lossy()))
        })
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort_unstable();
    entries.insert(0, path.display().to_string());

    let result = ratatui::try_init().and_then(|mut terminal| run(&mut terminal, &entries));
    let restored = ratatui::try_restore();
    result.and(restored)
}

fn run(terminal: &mut DefaultTerminal, entries: &[String]) -> io::Result<()> {
    let mut tree_focused = true;
    loop {
        terminal.draw(|frame| {
            let [body, footer] =
                Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
            let [tree, preview] =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .areas(body);

            let tree_border = panel("Árvore", tree_focused);
            let preview_border = panel("Prévia", !tree_focused);
            frame.render_widget(
                List::new(entries.iter().map(String::as_str)).block(tree_border),
                tree,
            );
            frame.render_widget(preview_border, preview);

            let focus = if tree_focused { "Árvore" } else { "Prévia" };
            frame.render_widget(
                Paragraph::new(format!("Foco: {focus} | Tab alterna | q sai")),
                footer,
            );
        })?;

        if let event::Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Tab => tree_focused = !tree_focused,
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}

fn panel(title: &str, focused: bool) -> Block<'_> {
    let border_color = if focused { Color::Cyan } else { Color::Reset };
    Block::bordered().title(title).border_style(border_color)
}
