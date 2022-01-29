#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#[cfg(test)]
use lexer::Lexer;
use unindent::unindent;

fn test_snapshot(name: &str, input: &str) {
    let input = unindent(input);
    let tokens = Lexer::new(input.trim())
        .into_iter()
        .filter(|t| !t.kind().is_whitespace())
        .collect::<Vec<_>>();
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
    let input = r#"
        $ _ $a _a abc_$
        \u03bc
        \u{61}
    "#;
    test_snapshot("identifier", input);
}

#[test]
fn punctuator() {
    let input = "
        { ( ) [ ] . ... ; , < > <= >= == != === !== + - * % ** ++ -- << >> >>> & | ^ ! ~ && || ?? ? : = +=
        -= *= %= **= <<= >>= >>>= &= |= ^= &&= ||= ??= =>
        ?. / /= }
        #
    ";
    test_snapshot("punctuator", input);
}

#[test]
fn numeric_literal() {
    let input = "
        0 0n
        0b1 0B12
        0o1 0O12
        0x1 0X12
        0123 0789
        0. 0.0 0.123
        123n
        1.0 1.1
    ";
    test_snapshot("numeric_literal", input);
}

#[test]
fn string_literal() {
    let input = r#"
        "12345" '12345'
        "" ''
        '\\\n\r\t\b\v\f\'\"'
        '\\\n\r\t\b\v\f\'\"'
        '\u1234' '\x12'
        'foo \
        '
        "\d"
    "#;
    test_snapshot("string_literal", input);
}

#[test]
fn regex() {
    let input = r#"
        /aa/;
        /[0-9A-Za-z_\$(|)\[\]\/\\^]/;
        /[//]/;
        /[/]/;
    "#;
    test_snapshot("regex", input);
}

#[test]
fn template_literal() {
    let input = r#"
        ``
        `123`
        `\`\r`
    "#;
    test_snapshot("template_literal", input);
}
