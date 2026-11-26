use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{locales, theme::THEMES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Preferences {
    pub(crate) language: &'static str,
    pub(crate) theme: usize,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: "en",
            theme: 0,
        }
    }
}

pub(crate) fn path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        absolute_env("APPDATA")
    } else if cfg!(target_os = "macos") {
        absolute_env("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        absolute_env("XDG_CONFIG_HOME")
            .or_else(|| absolute_env("HOME").map(|home| home.join(".config")))
    };
    base.map(|base| base.join("filens").join("config.toml"))
}

fn absolute_env(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

pub(crate) fn load(path: &Path) -> Preferences {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| contents.parse::<toml::Table>().ok())
        .map(|table| from_table(&table))
        .unwrap_or_default()
}

fn from_table(table: &toml::Table) -> Preferences {
    let defaults = Preferences::default();
    let language = table
        .get("language")
        .and_then(toml::Value::as_str)
        .and_then(|name| locales().iter().find(|locale| locale.as_ref() == name))
        .map(|locale| locale.as_ref())
        .unwrap_or(defaults.language);
    let theme = table
        .get("theme")
        .and_then(toml::Value::as_str)
        .and_then(|name| THEMES.iter().position(|theme| theme.name == name))
        .unwrap_or(defaults.theme);
    Preferences { language, theme }
}

pub(crate) fn save(path: &Path, preferences: Preferences) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let table = toml::Table::from_iter([
        ("language".into(), preferences.language.into()),
        ("theme".into(), THEMES[preferences.theme].name.into()),
    ]);
    let temporary = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let written = file.write_all(table.to_string().as_bytes());
    drop(file);
    let result = written.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmed_preferences_can_be_saved_loaded_and_replaced() {
        let directory = env::temp_dir().join(format!("filens-config-test-{}", std::process::id()));
        let path = directory.join("filens").join("config.toml");
        assert_eq!(load(&path), Preferences::default());

        let preferences = Preferences {
            language: "pt-BR",
            theme: 1,
        };
        save(&path, preferences).unwrap();
        assert_eq!(load(&path), preferences);

        save(&path, Preferences::default()).unwrap();
        assert_eq!(load(&path), Preferences::default());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unknown_preferences_fall_back_independently() {
        let table = "language = 'removed'\ntheme = 'Nord'".parse().unwrap();
        let preferences = from_table(&table);
        assert_eq!(preferences.language, "en");
        assert_eq!(THEMES[preferences.theme].name, "Nord");

        let table = "language = 'pt-BR'\ntheme = 'removed'".parse().unwrap();
        assert_eq!(
            from_table(&table),
            Preferences {
                language: "pt-BR",
                theme: 0
            },
        );
    }
}
