#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#[cfg(test)]
use lexer::Lexer;
use unindent::unindent;

fn test_snapshot(name: &str, input: &str) {
    let input = unindent(input);
    let tokens = Lexer::new(input.trim()).into_iter().collect::<Vec<_>>();
    let snapshot = format!("# Input\n{}\n---\n# Output\n{:#?}", input, tokens);
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, snapshot, name);
    });
}

#[test]
fn comment() {
    let input = "
        // comment
        /* multiline comment */
    ";
    test_snapshot("comment", input);
}

#[test]
fn keyword() {
    let input = "
        await break case catch class const continue debugger default delete do else enum export extends
        false finally for function if import in instanceof new null return super switch this throw true try typeof var void while with yield
        undefined
    ";
    test_snapshot("keyword", input);
}

#[test]
fn identifier() {
    let input = "
        $ _ $a _a abc_$
    ";
    test_snapshot("identifier", input);
}

#[test]
fn punctuator() {
    let input = "
        { ( ) [ ] . ... ; , < > <= >= == != === !== + - * % ** ++ -- << >> >>> & | ^ ! ~ && || ?? ? : = +=
        -= *= %= **= <<= >>= >>>= &= |= ^= &&= ||= ??= =>
        ?. / /= }
    ";
    test_snapshot("punctuator", input);
}

#[test]
fn numeric_literal() {
    let input = "
        0 123
        0b1 0B12
        0o1 0O12
        0x1 0X12
        0123 0789
        0n 123n
    ";
    test_snapshot("numeric_literal", input);
}

#[test]
fn string_literal() {
    let input = r#"
        "12345" '12345'
    "#;
    test_snapshot("string_literal", input);
}

#[test]
fn regex() {
    let input = r#"
        /aa/
    "#;
    test_snapshot("regex", input);
}

#[test]
fn template_literal() {
    let input = r#"
`123`
    "#;
    test_snapshot("template_literal", input);
}
