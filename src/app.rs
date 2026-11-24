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
    pub(crate) options_open: bool,
}

impl App {
    pub(crate) fn new(language: Language) -> Self {
        Self {
            language,
            tree_focused: true,
            options_open: false,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode) {
        if self.options_open {
            match key {
                KeyCode::Esc => self.options_open = false,
                KeyCode::Char('1') => self.language = Language::English,
                KeyCode::Char('2') => self.language = Language::BrazilianPortuguese,
                _ => {}
            }
        } else {
            match key {
                KeyCode::Tab => self.tree_focused = !self.tree_focused,
                KeyCode::Char('o') => self.options_open = true,
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
        assert!(app.options_open);
        app.handle_key(KeyCode::Char('2'));
        assert_eq!(app.language, Language::BrazilianPortuguese);
        app.handle_key(KeyCode::Tab);
        app.handle_key(KeyCode::Esc);
        assert!(!app.options_open);
        assert!(!app.tree_focused);
    }
}
