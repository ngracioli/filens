use crossterm::event::KeyCode;

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
    pub(crate) language: Language,
    pub(crate) tree_focused: bool,
    pub(crate) language_selection: Option<Language>,
}

impl App {
    pub(crate) fn new(language: Language) -> Self {
        Self {
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
                KeyCode::Char('o') => self.language_selection = Some(self.language),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_change_language_and_preserve_panel_focus() {
        let mut app = App::new(Language::English);
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
