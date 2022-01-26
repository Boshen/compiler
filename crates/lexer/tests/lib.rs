#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(test)]
use lexer::Lexer;

#[test]
fn test_identifiers() {
    let input = "
    // comment
    /* multiline comment */
    null
    undefined
    true
    false
    aaa
    ";
    let tokens = Lexer::new(input.trim()).into_iter().collect::<Vec<_>>();
    let snapshot = format!("# Input\n{}\n---\n# Output\n{:#?}", input, tokens);
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("identifiers", snapshot, "identifiers");
    });
}
