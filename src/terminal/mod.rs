mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::app::App;

pub(crate) fn run(app: &mut App) -> io::Result<()> {
    let result = ratatui::try_init().and_then(|mut terminal| run_session(&mut terminal, app));
    let restored = ratatui::try_restore();
    result.and(restored)
}

fn run_session(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
            app.handle_key(key.code);
        }
    }
}
