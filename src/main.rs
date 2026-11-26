mod app;
mod preview;
mod terminal;
mod theme;
mod tree;

use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

use rust_i18n::t;

use app::{App, Language};

rust_i18n::i18n!("locales", fallback = "en");

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
    let (language, path) = parse_args(env::args_os().skip(1))?;
    let locale = language.locale();
    let entries = tree::read_directory(&path, 0).map_err(|error| {
        t!(
            "error.read_directory",
            locale = locale,
            path = path.display(),
            cause = error
        )
        .into_owned()
    })?;

    let mut app = App::new(language, path, entries);
    terminal::run(&mut app).map_err(|error| {
        t!(
            "error.terminal",
            locale = app.language.locale(),
            cause = error
        )
        .into_owned()
    })
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<(Language, PathBuf), String> {
    let mut args = args.into_iter().peekable();
    let mut language = Language::default();

    if args.next_if(|arg| arg == "--lang").is_some() {
        language = match args.next() {
            Some(value) if value == "en" => Language::English,
            Some(value) if value == "pt-BR" => Language::BrazilianPortuguese,
            Some(value) => {
                return Err(t!(
                    "error.unsupported_language",
                    locale = "en",
                    language = value.to_string_lossy()
                )
                .into_owned());
            }
            None => return Err(t!("cli.usage", locale = "en").into_owned()),
        };
    }

    args.next_if(|arg| arg == "--");
    match (args.next(), args.next()) {
        (Some(path), None) => Ok((language, PathBuf::from(path))),
        _ => Err(t!("cli.usage", locale = language.locale()).into_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_requested_language_and_preserves_path() {
        for (args, expected) in [
            (vec!["ação"], Language::English),
            (vec!["--lang", "en", "ação"], Language::English),
            (
                vec!["--lang", "pt-BR", "ação"],
                Language::BrazilianPortuguese,
            ),
        ] {
            assert_eq!(
                parse_args(args.into_iter().map(OsString::from)),
                Ok((expected, PathBuf::from("ação"))),
            );
        }
        assert!(
            parse_args(["--lang", "fr", "."].map(OsString::from))
                .unwrap_err()
                .contains("Unsupported language: fr")
        );
    }
}
