mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    clipboard::CopyToClipboard,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
    execute,
};
use ratatui::DefaultTerminal;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Modifier,
    text::Span,
};

use crate::app::App;

const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) fn run(app: &mut App) -> io::Result<()> {
    let result = ratatui::try_init().and_then(|mut terminal| {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = execute!(io::stdout(), DisableMouseCapture);
            hook(info);
        }));
        let result = execute!(io::stdout(), EnableMouseCapture)
            .and_then(|()| run_session(&mut terminal, app));
        let mouse_restored = execute!(io::stdout(), DisableMouseCapture);
        result.and(mouse_restored)
    });
    let restored = ratatui::try_restore();
    result.and(restored)
}

fn run_session(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let mut selections = [None; 2];
    let mut dragging = None;
    let mut copy = None;
    let mut copied: Option<(usize, usize)> = None;
    let mut last_tree_click: Option<(usize, Instant)> = None;
    loop {
        let mut preview = Rect::default();
        let mut panels = [(Rect::default(), 0); 2];
        let mut clipboard = String::new();
        terminal.draw(|frame| {
            let footer;
            (preview, panels, footer) = ui::draw(frame, app);
            if app.language_selection.is_none() {
                for (index, selection) in selections.iter().enumerate() {
                    if let Some(selection) = selection {
                        let (area, text_bottom) = panels[index];
                        let text = highlight(frame.buffer_mut(), area, text_bottom, *selection);
                        if copy == Some(index) {
                            clipboard = text;
                        }
                    }
                }
                if let Some((index, count)) = copied
                    && selections[index].is_some()
                {
                    ui::draw_copy_footer(frame, footer, app.language, count);
                }
            }
        })?;
        if let Some(index) = copy.take()
            && !clipboard.is_empty()
        {
            let count = clipboard.chars().count();
            match execute!(io::stdout(), CopyToClipboard::to_clipboard_from(clipboard)) {
                Ok(()) => {
                    copied = Some((index, count));
                    continue;
                }
                Err(error) if error.kind() == io::ErrorKind::Unsupported => {}
                Err(error) => return Err(error),
            }
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                last_tree_click = None;
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
                if app.language_selection.is_none() {
                    match key.code {
                        KeyCode::Up | KeyCode::Down => {
                            selections = [None; 2];
                            dragging = None;
                        }
                        KeyCode::Enter if app.tree_focused => {
                            selections = [None; 2];
                            dragging = None;
                        }
                        KeyCode::Char('o') => dragging = None,
                        _ => {}
                    }
                }
                app.handle_key(key.code);
            }
            Event::Key(key)
                if key.kind == KeyEventKind::Repeat
                    && !app.tree_focused
                    && app.language_selection.is_none()
                    && matches!(key.code, KeyCode::Up | KeyCode::Down) =>
            {
                selections[1] = None;
                dragging = None;
                app.handle_key(key.code);
            }
            Event::Mouse(mouse) if app.language_selection.is_none() => {
                let position = Position::new(mouse.column, mouse.row);
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        dragging = panels.iter().position(|(area, _)| area.contains(position));
                        if dragging != Some(0) || position.y >= panels[0].1 {
                            last_tree_click = None;
                        }
                        if dragging.is_some() || preview.contains(position) {
                            selections = [None; 2];
                            copied = None;
                        }
                        if let Some(index) = dragging {
                            selections[index] = Some((position, position));
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left) => {
                        if matches!(mouse.kind, MouseEventKind::Drag(_)) {
                            last_tree_click = None;
                        }
                        if let Some(index) = dragging {
                            let area = panels[index].0;
                            if let Some((anchor, end)) = &mut selections[index]
                                && area.contains(*anchor)
                            {
                                *end = Position::new(
                                    position.x.clamp(area.x, area.right() - 1),
                                    position.y.clamp(area.y, area.bottom() - 1),
                                );
                                if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
                                    dragging = None;
                                    if *anchor == *end {
                                        if index == 0
                                            && position == *anchor
                                            && position.y < panels[0].1
                                        {
                                            let row = usize::from(position.y - area.y);
                                            let entry = app.tree_selection.offset() + row;
                                            app.tree_selection.select(Some(entry));
                                            app.tree_focused = true;
                                            let now = Instant::now();
                                            if last_tree_click.take().is_some_and(
                                                |(previous, at)| {
                                                    previous == entry
                                                        && now.duration_since(at)
                                                            <= DOUBLE_CLICK_INTERVAL
                                                },
                                            ) {
                                                app.handle_key(KeyCode::Enter);
                                            } else {
                                                last_tree_click = Some((entry, now));
                                            }
                                        }
                                        selections[index] = None;
                                    } else {
                                        copy = Some(index);
                                    }
                                }
                            }
                        }
                    }
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                        if preview.contains(position) =>
                    {
                        last_tree_click = None;
                        selections[1] = None;
                        if dragging == Some(1) {
                            dragging = None;
                        }
                        app.scroll_preview(if mouse.kind == MouseEventKind::ScrollUp {
                            -1
                        } else {
                            1
                        });
                    }
                    _ => {}
                }
            }
            Event::Resize(..) => {
                last_tree_click = None;
                selections = [None; 2];
                dragging = None;
            }
            _ => {}
        }
    }
}

fn highlight(
    buffer: &mut Buffer,
    area: Rect,
    text_bottom: u16,
    (anchor, end): (Position, Position),
) -> String {
    if anchor == end || !area.contains(anchor) || !area.contains(end) {
        return String::new();
    }
    let first = (anchor.y, anchor.x).min((end.y, end.x));
    let last = (anchor.y, anchor.x).max((end.y, end.x));
    let mut text = String::new();
    for y in first.0..=last.0 {
        let start = if y == first.0 { first.1 } else { area.x };
        let end = if y == last.0 {
            last.1 + 1
        } else {
            area.right()
        };
        let row_start = text.len();
        let mut x = area.x;
        while x < end {
            let cell = &mut buffer[(x, y)];
            let symbol = cell.symbol();
            let width = Span::raw(symbol).width().max(1) as u16;
            if x.saturating_add(width) > start {
                if y < text_bottom {
                    text.push_str(symbol);
                }
                for column in x..x.saturating_add(width).min(area.right()) {
                    buffer[(column, y)].modifier.insert(Modifier::REVERSED);
                }
            }
            x = x.saturating_add(width);
        }
        text.truncate(row_start + text[row_start..].trim_end_matches(' ').len());
        if y < last.0 && y + 1 < text_bottom {
            text.push('\n');
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn drag_from_below_the_document_excludes_padding_from_the_copy() {
        let area = Rect::new(0, 0, 12, 5);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(0, 0, "ação", Style::default());
        buffer.set_string(0, 2, "fim", Style::default());

        let selected = highlight(
            &mut buffer,
            area,
            3,
            (Position::new(4, 4), Position::ORIGIN),
        );

        assert_eq!(selected, "ação\n\nfim");
        assert!(buffer[(4, 4)].modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn reverse_drag_copies_complete_wide_characters_and_excludes_the_gutter() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 12, 2));
        buffer.set_string(0, 0, "1 │ ação🙂", Style::default());
        buffer.set_string(0, 1, "2 │ fim", Style::default());
        let selected = highlight(
            &mut buffer,
            Rect::new(4, 0, 8, 2),
            2,
            (Position::new(5, 1), Position::new(9, 0)),
        );
        assert_eq!(selected, "🙂\nfi");
        assert!(buffer[(8, 0)].modifier.contains(Modifier::REVERSED));
        assert!(!buffer[(3, 1)].modifier.contains(Modifier::REVERSED));
    }
}
