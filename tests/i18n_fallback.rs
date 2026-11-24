rust_i18n::i18n!("tests/fixtures/i18n", fallback = "en");

#[test]
fn falls_back_to_english_for_missing_portuguese_message() {
    assert_eq!(
        rust_i18n::t!("fallback.message", locale = "pt-BR"),
        "English fallback"
    );
}
