//! Lexer

use crate::kind::Kind;
use crate::token::Token;

pub struct Lexer<'a> {
    /// The input string
    bytes: &'a [u8],

    /// The current index position
    cur: usize,

    /// Are we at the EOF?
    eof: bool,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            cur: 0,
            eof: false,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.bytes.len() {
            if self.eof {
                return None;
            }
            self.eof = true;
            return Some(Token::new(Kind::EOF, self.cur, 1));
        }

        let token = self
            .read_whitespace()
            .or_else(|| self.read_identifier())
            .or_else(|| Some(Token::new(Kind::Unknown, self.cur, 1)));

        self.cur += token.as_ref().unwrap().len();

        token
    }
}

impl Lexer<'_> {
    fn read_whitespace(&mut self) -> Option<Token> {
        let mut len = 0;

        for chr in &self.bytes[self.cur..] {
            if !chr.is_ascii_whitespace() {
                break;
            }
            len += 1;
        }

        if len == 0 {
            return None;
        }

        Some(Token::new(Kind::WhiteSpace, self.cur, len))
    }

    fn read_identifier(&mut self) -> Option<Token> {
        let mut len = 0;

        for chr in &self.bytes[self.cur..] {
            if !chr.is_ascii_alphabetic() {
                break;
            }
            len += 1;
        }

        if len == 0 {
            return None;
        }

        let kind = match &self.bytes[self.cur..self.cur + len] {
            b"null" => Kind::Null,
            b"undefined" => Kind::Undefined,
            b"true" => Kind::True,
            b"false" => Kind::False,
            _ => Kind::Ident,
        };

        Some(Token::new(kind, self.cur, len))
    }
}
