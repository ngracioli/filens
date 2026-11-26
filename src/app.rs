use std::{io, path::PathBuf};

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::{
    locales, preview,
    theme::THEMES,
    tree::{self, Entry},
};

pub(crate) enum Preview {
    Empty,
    File {
        path: PathBuf,
        result: Result<preview::Document, preview::Error>,
    },
    DirectoryError {
        path: PathBuf,
        cause: io::Error,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionsSelection {
    Menu(usize),
    Language(usize),
    Theme(usize),
}

pub(crate) struct App {
    pub(crate) root: PathBuf,
    pub(crate) entries: Vec<Entry>,
    pub(crate) tree_selection: ListState,
    pub(crate) preview: Preview,
    pub(crate) preview_offset: usize,
    pub(crate) preview_last_offset: usize,
    pub(crate) preview_column: usize,
    pub(crate) preview_last_column: usize,
    pub(crate) preview_wrap: bool,
    pub(crate) language: &'static str,
    pub(crate) tree_focused: bool,
    pub(crate) options_selection: Option<OptionsSelection>,
    pub(crate) theme: usize,
}

impl App {
    pub(crate) fn new(language: &'static str, root: PathBuf, entries: Vec<Entry>) -> Self {
        let selected = (!entries.is_empty()).then_some(0);
        Self {
            root,
            entries,
            tree_selection: ListState::default().with_selected(selected),
            preview: Preview::Empty,
            preview_offset: 0,
            preview_last_offset: 0,
            preview_column: 0,
            preview_last_column: 0,
            preview_wrap: false,
            language,
            tree_focused: true,
            options_selection: None,
            theme: 0,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode) {
        if let Some(selection) = &mut self.options_selection {
            use OptionsSelection::{Language as LanguageMenu, Menu, Theme};

            match key {
                KeyCode::Esc => {
                    self.options_selection = match selection {
                        Menu(_) => None,
                        LanguageMenu(_) => Some(Menu(0)),
                        Theme(_) => Some(Menu(1)),
                    };
                }
                KeyCode::Up | KeyCode::Down => {
                    let (index, last) = match selection {
                        Menu(index) => (index, 1),
                        LanguageMenu(index) => (index, locales().len() - 1),
                        Theme(index) => (index, THEMES.len() - 1),
                    };
                    *index = if key == KeyCode::Up {
                        index.saturating_sub(1)
                    } else {
                        (*index + 1).min(last)
                    };
                }
                KeyCode::Enter => match *selection {
                    Menu(0) => {
                        let index = locales()
                            .iter()
                            .position(|locale| locale == self.language)
                            .unwrap_or_default();
                        *selection = LanguageMenu(index);
                    }
                    Menu(_) => *selection = Theme(self.theme),
                    LanguageMenu(index) => self.language = &locales()[index],
                    Theme(index) => self.theme = index,
                },
                _ => {}
            }
        } else {
            match key {
                KeyCode::Tab => self.tree_focused = !self.tree_focused,
                KeyCode::Char('w') => self.preview_wrap = !self.preview_wrap,
                KeyCode::Left if !self.tree_focused && !self.preview_wrap => {
                    self.preview_column = self.preview_column.saturating_sub(1);
                }
                KeyCode::Right if !self.tree_focused && !self.preview_wrap => {
                    self.preview_column = self
                        .preview_column
                        .saturating_add(1)
                        .min(self.preview_last_column.max(self.preview_column));
                }
                KeyCode::Char('o') => {
                    self.options_selection = Some(OptionsSelection::Menu(0));
                }
                KeyCode::Up if self.tree_focused => {
                    if let Some(index) = self.tree_selection.selected() {
                        self.tree_selection.select(Some(index.saturating_sub(1)));
                    }
                }
                KeyCode::Down if self.tree_focused => {
                    if let Some(index) = self.tree_selection.selected() {
                        self.tree_selection
                            .select(Some((index + 1).min(self.entries.len() - 1)));
                    }
                }
                KeyCode::Enter if self.tree_focused => self.activate_selection(),
                KeyCode::Up => self.scroll_preview(-1),
                KeyCode::Down => self.scroll_preview(1),
                _ => {}
            }
        }
    }

    pub(crate) fn click_option(&mut self, row: usize) {
        if let Some(selection) = &mut self.options_selection {
            let index = match selection {
                OptionsSelection::Menu(index)
                | OptionsSelection::Language(index)
                | OptionsSelection::Theme(index) => index,
            };
            *index = row;
            self.handle_key(KeyCode::Enter);
        }
    }

    pub(crate) fn scroll_preview(&mut self, delta: isize) {
        if self.options_selection.is_none()
            && matches!(self.preview, Preview::File { result: Ok(_), .. })
        {
            self.preview_offset = self
                .preview_offset
                .saturating_add_signed(delta)
                .min(self.preview_last_offset.max(self.preview_offset));
        }
    }

    fn activate_selection(&mut self) {
        let Some(index) = self.tree_selection.selected() else {
            return;
        };
        let entry = &mut self.entries[index];
        if !entry.kind.is_dir() {
            self.preview_offset = 0;
            self.preview_column = 0;
            self.preview = Preview::File {
                path: entry.path.clone(),
                result: preview::open(&entry.path),
            };
        } else if entry.expanded {
            entry.expanded = false;
            let depth = entry.depth;
            let start = index + 1;
            let descendants = self.entries[start..]
                .iter()
                .take_while(|entry| entry.depth > depth)
                .count();
            self.entries.drain(start..start + descendants);
        } else {
            match tree::read_directory(&entry.path, entry.depth + 1) {
                Ok(children) => {
                    entry.expanded = true;
                    self.entries.splice(index + 1..index + 1, children);
                }
                Err(cause) => {
                    self.preview = Preview::DirectoryError {
                        path: entry.path.clone(),
                        cause,
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_change_language_and_preserve_panel_focus() {
        let mut app = App::new("en", ".".into(), Vec::new());
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Char('o'));
        assert_eq!(app.options_selection, Some(OptionsSelection::Menu(0)));
        app.handle_key(KeyCode::Enter);
        let portuguese = locales()
            .iter()
            .position(|locale| locale == "pt-BR")
            .unwrap();
        assert_eq!(app.language, "en");
        app.click_option(portuguese);
        assert_eq!(app.language, "pt-BR");
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.options_selection, Some(OptionsSelection::Menu(0)));
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.options_selection, None);
        assert_eq!(app.language, "pt-BR");
        assert!(!app.tree_focused);
    }
}
