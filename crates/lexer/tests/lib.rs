#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(test)]
use lexer::Lexer;

#[test]
fn test_identifiers() {
    let input = "
    // comment
    /* multiline comment */
    await break case catch class const continue debugger default delete do else enum export extends
false finally for function if import in instanceof new null return super switch this throw true try typeof var void while with yield
    undefined
    $ _ $a _a abc_$
    ";
    let tokens = Lexer::new(input.trim()).into_iter().collect::<Vec<_>>();
    let snapshot = format!("# Input\n{}\n---\n# Output\n{:#?}", input, tokens);
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("identifiers", snapshot, "identifiers");
    });
}
