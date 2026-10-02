use super::ai_context_menu_token;

#[test]
fn extracts_the_filter_after_a_standalone_at_symbol() {
    assert_eq!(
        ai_context_menu_token("review @app/src", "review @app/src".len()),
        Some((7, "app/src".to_owned()))
    );
}

#[test]
fn accepts_an_empty_filter() {
    assert_eq!(ai_context_menu_token("@", 1), Some((0, String::new())));
}

#[test]
fn ignores_at_symbols_inside_words() {
    assert_eq!(
        ai_context_menu_token("email@example.com", "email@example.com".len()),
        None
    );
}

#[test]
fn closes_when_the_filter_contains_whitespace() {
    assert_eq!(ai_context_menu_token("@src main", 9), None);
}
