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
            .or_else(|| self.read_punctuators())
            .or_else(|| {
                self.cur += 1;
                Some(Token::new(Kind::Unknown, self.cur, 1))
            })
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

    /// Section 12.7 Punctuators
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn read_punctuators(&mut self) -> Option<Token> {
        let start = self.cur;
        let kind = match self.bytes[self.cur] {
            b'{' => Kind::LCurly,
            b'}' => Kind::RCurly,
            b'(' => Kind::LParen,
            b')' => Kind::RParen,
            b'[' => Kind::LBrack,
            b']' => Kind::RBrack,
            b'.' => {
                if self.bytes.get(self.cur + 1..=self.cur + 2) == Some(&[b'.', b'.']) {
                    self.cur += 2;
                    Kind::Dot3
                } else {
                    Kind::Dot
                }
            }
            b';' => Kind::Semicolon,
            b',' => Kind::Comma,
            b'<' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::LtEq
                } else if self.bytes.get(self.cur + 1) == Some(&b'<') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::ShiftLeftEq
                    } else {
                        Kind::ShiftLeft
                    }
                } else {
                    Kind::LAngle
                }
            }
            b'>' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::GtEq
                } else if self.bytes.get(self.cur + 1) == Some(&b'>') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'>') {
                        self.cur += 1;
                        if self.bytes.get(self.cur + 1) == Some(&b'=') {
                            self.cur += 1;
                            Kind::ShiftRight3Eq
                        } else {
                            Kind::ShiftRight3
                        }
                    } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::ShiftRightEq
                    } else {
                        Kind::ShiftRight
                    }
                } else {
                    Kind::RAngle
                }
            }
            b'=' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Eq3
                    } else {
                        Kind::Eq2
                    }
                } else if self.bytes.get(self.cur + 1) == Some(&b'>') {
                    self.cur += 1;
                    Kind::FatArrow
                } else {
                    Kind::Eq
                }
            }
            b'!' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Neq2
                    } else {
                        Kind::Neq
                    }
                } else {
                    Kind::Bang
                }
            }
            b'+' => {
                if self.bytes.get(self.cur + 1) == Some(&b'+') {
                    self.cur += 1;
                    Kind::Plus2
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::PlusEq
                } else {
                    Kind::Plus
                }
            }
            b'-' => {
                if self.bytes.get(self.cur + 1) == Some(&b'-') {
                    self.cur += 1;
                    Kind::Minus2
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::MinusEq
                } else {
                    Kind::Minus
                }
            }
            b'*' => {
                if self.bytes.get(self.cur + 1) == Some(&b'*') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Star2Eq
                    } else {
                        Kind::Star2
                    }
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::StarEq
                } else {
                    Kind::Star
                }
            }
            b'&' => {
                if self.bytes.get(self.cur + 1) == Some(&b'&') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Amp2Eq
                    } else {
                        Kind::Amp2
                    }
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::AmpEq
                } else {
                    Kind::Amp
                }
            }
            b'|' => {
                if self.bytes.get(self.cur + 1) == Some(&b'|') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Pipe2Eq
                    } else {
                        Kind::Pipe2
                    }
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::PipeEq
                } else {
                    Kind::Pipe
                }
            }
            b'~' => Kind::Tilde,
            b'?' => {
                if self.bytes.get(self.cur + 1) == Some(&b'?') {
                    self.cur += 1;
                    if self.bytes.get(self.cur + 1) == Some(&b'=') {
                        self.cur += 1;
                        Kind::Question2Eq
                    } else {
                        Kind::Question2
                    }
                } else if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::Question2Eq
                } else if self.bytes.get(self.cur + 1) == Some(&b'.') {
                    self.cur += 1;
                    Kind::QuestionDot
                } else {
                    Kind::Question
                }
            }
            b'^' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::CaretEq
                } else {
                    Kind::Caret
                }
            }
            b'/' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::SlashEq
                } else {
                    Kind::Slash
                }
            }
            b'%' => {
                if self.bytes.get(self.cur + 1) == Some(&b'=') {
                    self.cur += 1;
                    Kind::PercentEq
                } else {
                    Kind::Percent
                }
            }
            b':' => Kind::Colon,
            _ => return None,
        };
        self.cur += 1;
        Some(Token::new(kind, self.cur, self.cur - start))
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
            b"await" => Kind::Await,
            b"break" => Kind::Break,
            b"case" => Kind::Case,
            b"catch" => Kind::Catch,
            b"class" => Kind::Class,
            b"const" => Kind::Const,
            b"continue" => Kind::Continue,
            b"debugger" => Kind::Debugger,
            b"default" => Kind::DefaulT,
            b"delete" => Kind::Delete,
            b"do" => Kind::Do,
            b"else" => Kind::Else,
            b"enum" => Kind::Enum,
            b"export" => Kind::Export,
            b"extends" => Kind::Extends,
            b"false" => Kind::False,
            b"finally" => Kind::FinallY,
            b"for" => Kind::For,
            b"function" => Kind::Function,
            b"if" => Kind::If,
            b"in" => Kind::In,
            b"import" => Kind::Import,
            b"instanceof" => Kind::Instanceof,
            b"new" => Kind::New,
            b"null" => Kind::Null,
            b"return" => Kind::Return,
            b"super" => Kind::Super,
            b"switch" => Kind::Switch,
            b"this" => Kind::This,
            b"throw" => Kind::Throw,
            b"try" => Kind::Try,
            b"true" => Kind::True,
            b"typeof" => Kind::Typeof,
            b"var" => Kind::Var,
            b"void" => Kind::Void,
            b"while" => Kind::While,
            b"with" => Kind::With,
            b"yield" => Kind::Yield,
            _ => Kind::Ident,
        }
    }
}
