mod app;
mod config;
mod preview;
mod terminal;
mod theme;
mod tree;

use std::{borrow::Cow, env, ffi::OsString, path::PathBuf, process::ExitCode, sync::OnceLock};

use rust_i18n::t;

use app::App;

rust_i18n::i18n!("locales", fallback = "en");

fn locales() -> &'static [Cow<'static, str>] {
    static LOCALES: OnceLock<Vec<Cow<'static, str>>> = OnceLock::new();
    LOCALES.get_or_init(|| {
        let mut locales = rust_i18n::available_locales!();
        locales.sort_unstable();
        locales
    })
}

fn main() -> ExitCode {
    match open_directory() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("filens: {error}");
            ExitCode::FAILURE
        }
    }
}

fn open_directory() -> Result<(), String> {
    let path = parse_args(env::args_os().skip(1))?;
    let config_path = config::path();
    let preferences = config_path.as_deref().map(config::load).unwrap_or_default();
    let language = preferences.language;
    let entries = tree::read_directory(&path, 0).map_err(|error| {
        t!(
            "error.read_directory",
            locale = language,
            path = path.display(),
            cause = error
        )
        .into_owned()
    })?;

    let mut app = App::new(language, path, entries);
    app.theme = preferences.theme;
    terminal::run(&mut app, config_path.as_deref())
        .map_err(|error| t!("error.terminal", locale = app.language, cause = error).into_owned())
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<PathBuf, String> {
    let mut args = args.into_iter().peekable();
    args.next_if(|arg| arg == "--");
    match (args.next(), args.next()) {
        (Some(path), None) => Ok(PathBuf::from(path)),
        _ => Err(t!("cli.usage", locale = "en").into_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_directory_and_preserves_its_path() {
        for args in [vec!["ação"], vec!["--", "ação"]] {
            assert_eq!(
                parse_args(args.into_iter().map(OsString::from)),
                Ok(PathBuf::from("ação")),
            );
        }
        assert!(parse_args([]).is_err());
        assert!(parse_args(["--lang", "pt-BR", "."].map(OsString::from)).is_err());
    }
}
