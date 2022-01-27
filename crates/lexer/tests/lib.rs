#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(test)]
use lexer::Lexer;

fn test_snapshot(name: &str, input: &str) {
    let tokens = Lexer::new(input.trim()).into_iter().collect::<Vec<_>>();
    let snapshot = format!("# Input\n{}\n---\n# Output\n{:#?}", input, tokens);
    insta::with_settings!({
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, snapshot, name);
    });
}

#[test]
fn identifiers() {
    let input = "
// comment
/* multiline comment */
await break case catch class const continue debugger default delete do else enum export extends
false finally for function if import in instanceof new null return super switch this throw true try typeof var void while with yield
undefined
$ _ $a _a abc_$
{ ( ) [ ] . ... ; , < > <= >= == != === !== + - * % ** ++ -- << >> >>> & | ^ ! ~ && || ?? ? : = +=
-= *= %= **= <<= >>= >>>= &= |= ^= &&= ||= ??= =>
?. / /= }
    ";
    test_snapshot("identifiers", input);
}

#[test]
fn numeric_literals() {
    let input = "
0 123
0b1 0B12
0o1 0O12
0x1 0X12
0123 0789
0n 123n
    ";
    test_snapshot("numeric_literals", input);
}

#[test]
fn string_literals() {
    let input = r#"
"12345" '12345'
    "#;
    test_snapshot("string_literals", input);
}
