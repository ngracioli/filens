use std::{ops::Range, path::Path, sync::LazyLock};

use syntect::{
    easy::ScopeRangeIterator,
    parsing::{ParseState, Scope, ScopeStack, SyntaxDefinition, SyntaxReference, SyntaxSet},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Token {
    Comment,
    String,
    Keyword,
    Constant,
    Function,
    Type,
    Tag,
    Heading,
}

pub(crate) type Highlights = Vec<Vec<(Range<usize>, Token)>>;

pub(crate) fn prepare() {
    LazyLock::force(&SYNTAXES);
}

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(|| {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    for source in [
        include_str!("syntaxes/TypeScript.sublime-syntax"),
        include_str!("syntaxes/JavaScript (Babel).sublime-syntax"),
        include_str!("syntaxes/TOML.sublime-syntax"),
        include_str!("syntaxes/DotENV.sublime-syntax"),
    ] {
        if let Ok(syntax) = SyntaxDefinition::load_from_str(source, true, None) {
            builder.add(syntax);
        }
    }
    builder.build()
});

static TOKEN_SCOPES: LazyLock<Vec<(Scope, Token)>> = LazyLock::new(|| {
    [
        ("comment", Token::Comment),
        ("punctuation.definition.comment", Token::Comment),
        ("string", Token::String),
        ("punctuation.definition.string", Token::String),
        ("keyword", Token::Keyword),
        ("storage", Token::Keyword),
        ("constant", Token::Constant),
        ("entity.name.function", Token::Function),
        ("support.function", Token::Function),
        ("entity.name.type", Token::Type),
        ("entity.name.class", Token::Type),
        ("support.type", Token::Type),
        ("entity.name.tag", Token::Tag),
        ("markup.heading", Token::Heading),
    ]
    .into_iter()
    .map(|(scope, token)| (Scope::new(scope).expect("valid built-in scope"), token))
    .collect()
});

pub(super) fn highlight(path: &Path, text: &str) -> Option<Highlights> {
    let name = path.file_name().and_then(|name| name.to_str());
    let extension = path.extension().and_then(|extension| extension.to_str());
    let syntax = name
        .and_then(|name| SYNTAXES.find_syntax_by_extension(name))
        .or_else(|| {
            let name = name?;
            (name.starts_with(".env.") || name.starts_with("env."))
                .then(|| SYNTAXES.find_syntax_by_name("DotENV"))
                .flatten()
        })
        .or_else(|| SYNTAXES.find_syntax_by_extension(extension?))
        .or_else(|| match extension {
            Some(extension)
                if extension.eq_ignore_ascii_case("vue")
                    || extension.eq_ignore_ascii_case("astro") =>
            {
                SYNTAXES.find_syntax_by_extension("html")
            }
            _ => None,
        })
        .or_else(|| SYNTAXES.find_syntax_by_first_line(text.lines().next()?))?;
    highlight_with(&SYNTAXES, syntax, text)
}

fn highlight_with(set: &SyntaxSet, syntax: &SyntaxReference, text: &str) -> Option<Highlights> {
    let mut parser = ParseState::new(syntax);
    let mut scopes = ScopeStack::new();
    text.split_inclusive('\n')
        .map(|line| {
            let operations = parser.parse_line(line, set).ok()?;
            let end = line
                .strip_suffix('\n')
                .map_or(line, |line| line.strip_suffix('\r').unwrap_or(line))
                .len();
            let mut tokens: Vec<(Range<usize>, Token)> = Vec::new();
            for (range, operation) in ScopeRangeIterator::new(&operations, line) {
                scopes.apply(operation).ok()?;
                let end = range.end.min(end);
                if range.start >= end {
                    continue;
                }
                let token = scopes.as_slice().iter().rev().find_map(|scope| {
                    TOKEN_SCOPES
                        .iter()
                        .find_map(|(prefix, token)| prefix.is_prefix_of(*scope).then_some(*token))
                });
                if let Some(token) = token {
                    if let Some((previous, previous_token)) = tokens.last_mut()
                        && *previous_token == token
                        && previous.end == range.start
                    {
                        previous.end = end;
                    } else {
                        tokens.push((range.start..end, token));
                    }
                }
            }
            Some(tokens)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_every_mvp_format() {
        for (name, source) in [
            ("README.md", "# Heading\n"),
            ("index.html", "<div>hello</div>\n"),
            ("style.css", ".note { color: red; }\n"),
            ("app.js", "const name = 'hello';\n"),
            ("app.ts", "interface User { name: string }\n"),
            ("app.jsx", "<section>hello</section>\n"),
            ("app.py", "def greet(): return 'hello'\n"),
            ("app.rs", "fn main() {}\n"),
            ("App.java", "public class App {}\n"),
            ("app.c", "int main(void) { return 0; }\n"),
            ("app.cpp", "namespace app {}\n"),
            ("app.go", "package main\n"),
            ("query.sql", "SELECT 1;\n"),
            ("app.sh", "echo 'hello'\n"),
            ("data.json", "{\"enabled\": true}\n"),
            ("data.yaml", "enabled: true\n"),
            ("Cargo.toml", "[package]\nname = 'hello'\n"),
            ("Cargo.lock", "[[package]]\nname = 'hello'\n"),
            (".env.local", "MODE=\"test\"\n"),
            ("app.vue", "<template><section/></template>\n"),
            ("app.astro", "<section>hello</section>\n"),
            ("script", "#!/bin/bash\necho 'hello'\n"),
        ] {
            let lines = highlight(Path::new(name), source).unwrap_or_else(|| panic!("{name}"));
            if matches!(name, "app.jsx" | "app.vue" | "app.astro") {
                assert!(
                    lines[0].iter().any(|(_, kind)| *kind == Token::Tag),
                    "{name}: {lines:?}"
                );
            } else {
                assert!(lines.iter().any(|line| !line.is_empty()), "{name}");
            }
        }
    }

    #[test]
    fn grammar_error_discards_highlighting_and_next_file_recovers() {
        let grammar = SyntaxDefinition::load_from_str(
            "name: Broken\nscope: source.broken\ncontexts:\n  main:\n    - match: 'first'\n      scope: keyword.control.broken\n    - match: '!fail'\n      push: scope:source.missing\n",
            true,
            None,
        ).unwrap();
        let mut builder = SyntaxSet::new().into_builder();
        builder.add(grammar);
        let set = builder.build();
        let broken = set.find_syntax_by_name("Broken").unwrap();
        assert_eq!(
            highlight_with(&set, broken, "first\n"),
            Some(vec![vec![(0..5, Token::Keyword)]])
        );
        assert!(highlight_with(&set, broken, "first\n!fail\nlast\n").is_none());
        assert!(highlight(Path::new("next.rs"), "fn main() {}\n").is_some());
    }

    #[test]
    fn carries_multiline_state_only_within_each_file() {
        let rust = highlight(Path::new("open.rs"), "/* starts\nstill a comment\n").unwrap();
        assert_eq!(rust[1], vec![(0..15, Token::Comment)]);

        let python = "def greet(): return 'hello'\n";
        let expected = highlight(Path::new("app.py"), python).unwrap();
        assert!(expected[0].iter().any(|(_, kind)| *kind == Token::Keyword));
        assert_eq!(highlight(Path::new("next.py"), python), Some(expected));
        assert!(highlight(Path::new("unknown"), "ordinary text\n").is_none());
    }
}
