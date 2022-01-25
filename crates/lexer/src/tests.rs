#[cfg(test)]
use crate::lex;

#[test]
fn test_lex() {
    assert_eq!(lex(), "hello lexer!");
}
