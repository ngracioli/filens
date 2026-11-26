use std::{io, path::PathBuf};

use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::{
    preview,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    English,
    BrazilianPortuguese,
}

impl Language {
    pub(crate) fn locale(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::BrazilianPortuguese => "pt-BR",
        }
    }
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
    pub(crate) language: Language,
    pub(crate) tree_focused: bool,
    pub(crate) language_selection: Option<Language>,
}

impl App {
    pub(crate) fn new(language: Language, root: PathBuf, entries: Vec<Entry>) -> Self {
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
            language_selection: None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode) {
        if let Some(selected) = &mut self.language_selection {
            match key {
                KeyCode::Esc => self.language_selection = None,
                KeyCode::Up => *selected = Language::English,
                KeyCode::Down => *selected = Language::BrazilianPortuguese,
                KeyCode::Enter => self.language = *selected,
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
                KeyCode::Char('o') => self.language_selection = Some(self.language),
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

    pub(crate) fn scroll_preview(&mut self, delta: isize) {
        if self.language_selection.is_none()
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
        let mut app = App::new(Language::English, ".".into(), Vec::new());
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Char('o'));
        assert_eq!(app.language_selection, Some(Language::English));
        app.handle_key(KeyCode::Down);
        assert_eq!(app.language_selection, Some(Language::BrazilianPortuguese));
        assert_eq!(app.language, Language::English);
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.language, Language::BrazilianPortuguese);
        app.handle_key(KeyCode::Up);
        assert_eq!(app.language_selection, Some(Language::English));
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.language_selection, None);
        assert_eq!(app.language, Language::BrazilianPortuguese);
        assert!(!app.tree_focused);
    }
}
