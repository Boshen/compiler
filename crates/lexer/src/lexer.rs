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
            .read_line_terminator()
            .or_else(|| self.read_whitespace())
            .or_else(|| self.read_identifier())
            .or_else(|| Some(Token::new(Kind::Unknown, self.cur, 1)));

        self.cur += token.as_ref().unwrap().len();

        token
    }
}

impl Lexer<'_> {
    /// Section 12.2 Whitespace
    /// TODO read all whitespace codepoints
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

    /// Section 12.3 Line Terminators
    fn read_line_terminator(&mut self) -> Option<Token> {
        let start = self.cur;
        loop {
            let chr = self.bytes[self.cur];
            if matches!(chr, b'\n' | b'\r') {
                self.cur += '\n'.len_utf8();
            } else if matches!(self.unicode_char(), '\u{2028}' | '\u{2029}') {
                self.cur += '\u{2028}'.len_utf8();
            } else {
                break;
            }
        }
        let len = self.cur - start;
        if len == 0 {
            return None;
        }
        Some(Token::new(Kind::Newline, self.cur, len))
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

    /// Get the next unicode character
    fn unicode_char(&self) -> char {
        let string = unsafe { std::str::from_utf8_unchecked(self.bytes.get_unchecked(self.cur..)) };
        if let Some(chr) = string.chars().next() {
            return chr;
        }
        unreachable!()
    }
}
