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

        self.read_whitespace()
            .or_else(|| self.read_line_terminator())
            .or_else(|| self.read_comment())
            .or_else(|| self.read_name_or_keyword())
            .or_else(|| Some(Token::new(Kind::Unknown, self.cur, 1)))
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
        self.cur += len;
        Some(Token::new(Kind::WhiteSpace, self.cur, len))
    }

    /// Section 12.3 Line Terminators
    fn read_line_terminator(&mut self) -> Option<Token> {
        let start = self.cur;
        while let Some(len) = self.is_line_terminator() {
            self.cur += len;
        }
        let len = self.cur - start;
        if len == 0 {
            return None;
        }
        Some(Token::new(Kind::LineTerminator, self.cur, len))
    }

    /// Section 12.4 Comments
    fn read_comment(&mut self) -> Option<Token> {
        let start = self.cur;
        if self.bytes[self.cur] != b'/' {
            return None;
        }
        match self.bytes.get(self.cur + 1) {
            Some(b'/') => {
                self.cur += 2;
                while self.next().is_some() {
                    if self.is_line_terminator().is_some() {
                        return Some(Token::new(Kind::Comment, self.cur, self.cur - start));
                    }
                }
                // we should be at the EOF here
                assert_eq!(self.cur, self.bytes.len());
                Some(Token::new(Kind::Comment, self.cur, self.cur - start))
            }
            Some(b'*') => {
                self.cur += 2;
                while let Some(b) = self.next() {
                    if b == b'*' && self.bytes.get(self.cur + 1) == Some(&b'/') {
                        self.cur += 2;
                        return Some(Token::new(
                            Kind::MultilineComment,
                            self.cur,
                            self.cur - start,
                        ));
                    }
                }
                // TODO multi line comment error
                None
            }
            _ => None,
        }
    }

    /// Section 12.6 Names and Keywords
    fn read_name_or_keyword(&mut self) -> Option<Token> {
        let start = self.cur;
        while self.bytes.get(self.cur).is_some() {
            if self.is_ident() {
                self.cur += 1;
            } else {
                break;
            }
        }
        let len = self.cur - start;
        if len == 0 {
            return None;
        }
        let kind = Lexer::read_keyword(&self.bytes[start..self.cur]);
        Some(Token::new(kind, self.cur, len))
    }

    /// Advance to the next position and return its byte
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.cur += 1;
        self.bytes.get(self.cur).copied()
    }

    /// Check if the next bytes is a line terminator
    fn is_line_terminator(&mut self) -> Option<usize> {
        let chr = self.bytes[self.cur];
        if matches!(chr, b'\n' | b'\r') {
            return Some('\n'.len_utf8());
        }
        if matches!(self.unicode_char(), '\u{2028}' | '\u{2029}') {
            return Some('\u{2028}'.len_utf8());
        }
        None
    }

    /// Get the next unicode character
    fn unicode_char(&self) -> char {
        let string = unsafe { std::str::from_utf8_unchecked(self.bytes.get_unchecked(self.cur..)) };
        if let Some(chr) = string.chars().next() {
            return chr;
        }
        unreachable!()
    }

    /// Section 12.6
    // TODO read Unicode
    const fn is_ident(&self) -> bool {
        let b = self.bytes[self.cur];
        b.is_ascii_alphabetic() || matches!(b, b'_' | b'$')
    }

    /// Section 12.6.2 Keywords and Reserved Words
    const fn read_keyword(bytes: &[u8]) -> Kind {
        match bytes {
            b"await" => Kind::AWAIT,
            b"break" => Kind::BREAK,
            b"case" => Kind::CASE,
            b"catch" => Kind::CATCH,
            b"class" => Kind::CLASS,
            b"const" => Kind::CONST,
            b"continue" => Kind::CONTINUE,
            b"debugger" => Kind::DEBUGGER,
            b"default" => Kind::DEFAULT,
            b"delete" => Kind::DELETE,
            b"do" => Kind::DO,
            b"else" => Kind::ELSE,
            b"enum" => Kind::ENUM,
            b"export" => Kind::EXPORT,
            b"extends" => Kind::EXTENDS,
            b"false" => Kind::FALSE,
            b"finally" => Kind::FINALLY,
            b"for" => Kind::FOR,
            b"function" => Kind::FUNCTION,
            b"if" => Kind::IF,
            b"in" => Kind::IN,
            b"import" => Kind::IMPORT,
            b"instanceof" => Kind::INSTANCEOF,
            b"new" => Kind::NEW,
            b"null" => Kind::NULL,
            b"return" => Kind::RETURN,
            b"super" => Kind::SUPER,
            b"switch" => Kind::SWITCH,
            b"this" => Kind::THIS,
            b"throw" => Kind::THROW,
            b"try" => Kind::TRY,
            b"true" => Kind::TRUE,
            b"typeof" => Kind::TYPEOF,
            b"var" => Kind::VAR,
            b"void" => Kind::VOID,
            b"while" => Kind::WHILE,
            b"with" => Kind::WITH,
            b"yield" => Kind::YIELD,
            _ => Kind::Ident,
        }
    }
}
