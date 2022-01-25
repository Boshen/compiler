//! Lexer

use crate::token::Token;

pub struct Lexer<'a> {
    /// The input string
    input: &'a str,

    /// The current index position
    cur: usize,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Self { input, cur: 0 }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.input.len() {
            return None;
        }
        Some(Token::Null)
    }
}
