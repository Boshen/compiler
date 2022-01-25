use crate::{Lexer, Token};

#[test]
fn test_lex() {
    assert_eq!(Lexer::new("").next(), None);
    assert_eq!(Lexer::new("TODO").next(), Some(Token::Null));
}
