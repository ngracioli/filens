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
    let mut tree_selection = None;
    let mut preview_selection: Option<(PreviewPoint, PreviewPoint)> = None;
    let mut scroll_pointer = None;
    let mut dragging = None;
    let mut copy = None;
    let mut copied: Option<(usize, usize)> = None;
    let mut last_tree_click: Option<(usize, Instant)> = None;
    loop {
        let mut preview = Rect::default();
        let mut tree = (Rect::default(), 0);
        let mut viewport = ui::PreviewViewport::default();
        let mut clipboard = String::new();
        terminal.draw(|frame| {
            let footer;
            (preview, tree, viewport, footer) = ui::draw(frame, app);
            if app.options_selection.is_none() {
                if let Some(selection) = tree_selection {
                    let text = highlight(frame.buffer_mut(), tree.0, tree.1, selection);
                    if copy == Some(0) {
                        clipboard = text;
                    }
                }
                if let Some((anchor, end)) = &mut preview_selection {
                    if let Some(pointer) = scroll_pointer.take()
                        && let Some(point) = preview_point(&viewport, pointer)
                    {
                        *end = point;
                    }
                    if anchor != end {
                        let first = (*anchor).min(*end);
                        let last = (*anchor).max(*end);
                        highlight_preview(frame.buffer_mut(), &viewport, first, last);
                        if copy == Some(1) {
                            clipboard =
                                ui::selected_preview_text(app, viewport.area.width, first, last);
                        }
                    }
                }
                if let Some((index, count)) = copied
                    && if index == 0 {
                        tree_selection.is_some()
                    } else {
                        preview_selection.is_some()
                    }
                {
                    ui::draw_copy_footer(frame, footer, app, count);
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
                if app.options_selection.is_none() {
                    match key.code {
                        KeyCode::Up | KeyCode::Down if app.tree_focused => {
                            tree_selection = None;
                            if dragging == Some(0) {
                                dragging = None;
                            }
                        }
                        KeyCode::Enter if app.tree_focused => {
                            tree_selection = None;
                            preview_selection = None;
                            dragging = None;
                        }
                        KeyCode::Char('w') | KeyCode::Left | KeyCode::Right => {
                            preview_selection = None;
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
                    && app.options_selection.is_none()
                    && matches!(
                        key.code,
                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                    ) =>
            {
                if matches!(key.code, KeyCode::Left | KeyCode::Right) {
                    preview_selection = None;
                    dragging = None;
                }
                app.handle_key(key.code);
            }
            Event::Mouse(mouse) if app.options_selection.is_none() => {
                let position = Position::new(mouse.column, mouse.row);
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        dragging = if tree.0.contains(position) {
                            Some(0)
                        } else if viewport.area.contains(position) && !viewport.rows.is_empty() {
                            Some(1)
                        } else {
                            None
                        };
                        if dragging != Some(0) || position.y >= tree.1 {
                            last_tree_click = None;
                        }
                        if dragging.is_some() || preview.contains(position) {
                            tree_selection = None;
                            preview_selection = None;
                            copied = None;
                        }
                        match dragging {
                            Some(0) => tree_selection = Some((position, position)),
                            Some(1) => {
                                preview_selection =
                                    preview_point(&viewport, position).map(|point| (point, point));
                            }
                            _ => {}
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Up(MouseButton::Left) => {
                        let released = mouse.kind == MouseEventKind::Up(MouseButton::Left);
                        if !released {
                            last_tree_click = None;
                        }
                        match dragging {
                            Some(0) => {
                                let (area, text_bottom) = tree;
                                if let Some((anchor, end)) = &mut tree_selection
                                    && area.contains(*anchor)
                                {
                                    *end = Position::new(
                                        position.x.clamp(area.x, area.right() - 1),
                                        position.y.clamp(area.y, area.bottom() - 1),
                                    );
                                    if released {
                                        if *anchor == *end {
                                            if position == *anchor && position.y < text_bottom {
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
                                            tree_selection = None;
                                        } else {
                                            copy = Some(0);
                                        }
                                    }
                                }
                            }
                            Some(1) => {
                                if let Some((anchor, end)) = &mut preview_selection
                                    && let Some(point) = preview_point(&viewport, position)
                                {
                                    *end = point;
                                    if released {
                                        if anchor == end {
                                            preview_selection = None;
                                        } else {
                                            copy = Some(1);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        if released {
                            dragging = None;
                        }
                    }
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                        if preview.contains(position) || dragging == Some(1) =>
                    {
                        last_tree_click = None;
                        app.scroll_preview(if mouse.kind == MouseEventKind::ScrollUp {
                            -1
                        } else {
                            1
                        });
                        if dragging == Some(1) {
                            scroll_pointer = Some(position);
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(..) => {
                last_tree_click = None;
                tree_selection = None;
                preview_selection = None;
                dragging = None;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PreviewPoint {
    row: (usize, usize),
    column: u16,
}

fn preview_point(viewport: &ui::PreviewViewport, position: Position) -> Option<PreviewPoint> {
    let last = viewport.rows.len().checked_sub(1)?;
    let row = usize::from(position.y.saturating_sub(viewport.area.y)).min(last);
    Some(PreviewPoint {
        row: viewport.rows[row],
        column: position
            .x
            .saturating_sub(viewport.area.x)
            .min(viewport.area.width.checked_sub(1)?),
    })
}

pub(super) fn selection_columns(
    row: (usize, usize),
    width: u16,
    first: PreviewPoint,
    last: PreviewPoint,
) -> Option<(u16, u16)> {
    if row < first.row || row > last.row {
        return None;
    }
    let start = if row == first.row { first.column } else { 0 };
    let end = if row == last.row {
        last.column + 1
    } else {
        width
    };
    let end = end.min(width);
    (start < end).then_some((start, end))
}

fn highlight_preview(
    buffer: &mut Buffer,
    viewport: &ui::PreviewViewport,
    first: PreviewPoint,
    last: PreviewPoint,
) {
    let area = viewport.area;
    for (index, &row) in viewport.rows.iter().enumerate() {
        if let Some((start, end)) = selection_columns(row, area.width, first, last) {
            highlight_row(
                buffer,
                area.x,
                area.y + index as u16,
                area.x + start,
                area.x + end,
                area.right(),
            );
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
        let row = highlight_row(buffer, area.x, y, start, end, area.right());
        if y < text_bottom {
            text.push_str(row.trim_end_matches(' '));
        }
        if y < last.0 && y + 1 < text_bottom {
            text.push('\n');
        }
    }
    text
}

fn highlight_row(
    buffer: &mut Buffer,
    left: u16,
    y: u16,
    start: u16,
    end: u16,
    right: u16,
) -> String {
    let mut text = String::new();
    let mut x = left;
    while x < end {
        let symbol = buffer[(x, y)].symbol();
        let next = x.saturating_add(Span::raw(symbol).width().max(1) as u16);
        if next > start {
            text.push_str(symbol);
            for column in x..next.min(right) {
                buffer[(column, y)].modifier.insert(Modifier::REVERSED);
            }
        }
        x = next;
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
    fn preview_selection_follows_source_rows_and_copies_an_offscreen_anchor() {
        let mut app = App::new(crate::app::Language::English, ".".into(), Vec::new());
        app.preview = crate::app::Preview::File {
            path: "text.txt".into(),
            result: Ok(crate::preview::Document {
                text: "ação 🙂fim\nseguinte".into(),
                truncated: false,
            }),
        };
        app.preview_wrap = true;

        let area = Rect::new(4, 0, 8, 2);
        let initial = ui::PreviewViewport {
            area,
            rows: vec![(0, 0), (0, 1)],
            ..Default::default()
        };
        let anchor = preview_point(&initial, Position::new(4, 0)).unwrap();
        let scrolled = ui::PreviewViewport {
            area,
            rows: vec![(1, 0)],
            ..Default::default()
        };
        let end = preview_point(&scrolled, Position::new(6, 0)).unwrap();
        assert_eq!(
            ui::selected_preview_text(&app, area.width, anchor, end),
            "ação\n🙂fim\nseg"
        );

        let mut buffer = Buffer::empty(area);
        buffer.set_string(4, 0, "seguinte", Style::default());
        highlight_preview(&mut buffer, &scrolled, anchor, end);
        assert!(buffer[(6, 0)].modifier.contains(Modifier::REVERSED));
        assert!(!buffer[(7, 0)].modifier.contains(Modifier::REVERSED));

        let mut buffer = Buffer::empty(area);
        buffer.set_string(4, 0, "ação", Style::default());
        buffer.set_string(4, 1, "🙂fim", Style::default());
        highlight_preview(&mut buffer, &initial, anchor, end);
        assert!(buffer[(4, 0)].modifier.contains(Modifier::REVERSED));
        assert!(buffer[(5, 1)].modifier.contains(Modifier::REVERSED));
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
