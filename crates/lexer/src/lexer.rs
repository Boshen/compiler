//! Lexer

use crate::kind::{Kind, Number};
use crate::token::Token;

pub struct Lexer<'a> {
    /// The input string
    bytes: &'a [u8],

    /// The current index position
    cur: usize,

    /// Are we at the EOF?
    eof: bool,
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

        let bytes = &self.bytes[self.cur..];
        let token = self
            .read_whitespaces(bytes)
            .or_else(|| self.read_line_terminators(bytes))
            .or_else(|| self.read_comment(bytes))
            .or_else(|| self.read_name_or_keyword(bytes))
            .or_else(|| self.read_punctuator(bytes))
            .or_else(|| self.read_number(bytes))
            .or_else(|| self.read_string_literal(bytes))
            .or_else(|| Some(Token::new(Kind::Unknown, self.cur, 1)));

        if let Some(t) = token.as_ref() {
            self.cur += t.len();
        }

        token
    }
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

    /// Section 12.2 Whitespace
    /// TODO read all whitespace codepoints
    fn read_whitespaces(&self, bytes: &[u8]) -> Option<Token> {
        let len = bytes.iter().take_while(|b| b.is_ascii_whitespace()).count();
        if len == 0 {
            return None;
        }
        Some(Token::new(Kind::WhiteSpace, self.cur, len))
    }

    /// Section 12.3 Line Terminators
    fn read_line_terminators(&self, bytes: &[u8]) -> Option<Token> {
        let mut cur = 0;
        while let Some(bytes) = bytes.get(cur..) {
            if let Some(len) = Lexer::read_line_terminator(bytes) {
                cur += len;
            } else {
                break;
            }
        }
        if cur == 0 {
            return None;
        }
        Some(Token::new(Kind::LineTerminator, self.cur, cur))
    }

    /// Section 12.4 Comments
    fn read_comment(&self, bytes: &[u8]) -> Option<Token> {
        if bytes.starts_with(&[b'/', b'/']) {
            let mut cur = 2;
            while let Some(bytes) = bytes.get(cur..) {
                if let Some(len) = Lexer::read_line_terminator(bytes) {
                    cur += len;
                    break;
                }
                cur += 1;
            }
            return Some(Token::new(Kind::Comment, self.cur, cur));
        }
        if bytes.starts_with(&[b'/', b'*']) {
            let mut cur = 2;
            while let Some(bytes) = bytes.get(cur..) {
                if bytes.starts_with(&[b'*', b'/']) {
                    cur += 2;
                    break;
                }
                cur += 1;
            }
            return Some(Token::new(Kind::MultilineComment, self.cur, cur));
        }
        // TODO Error
        None
    }

    /// Section 12.6 Names and Keywords
    fn read_name_or_keyword(&self, bytes: &[u8]) -> Option<Token> {
        // let start = self.cur;
        let mut cur = 0;
        while let Some(bytes) = bytes.get(cur..) {
            if let Some(len) = Lexer::read_identifier(bytes) {
                cur += len;
            } else {
                break;
            }
        }
        if cur == 0 {
            return None;
        }
        let kind = Lexer::read_keyword(&bytes[..cur]);
        Some(Token::new(kind, self.cur, cur))
    }

    /// Section 12.7 Punctuators
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn read_punctuator(&self, bytes: &[u8]) -> Option<Token> {
        let mut cur = 0;
        let kind = match bytes[cur] {
            b'{' => Kind::LCurly,
            b'}' => Kind::RCurly,
            b'(' => Kind::LParen,
            b')' => Kind::RParen,
            b'[' => Kind::LBrack,
            b']' => Kind::RBrack,
            b'.' => {
                if bytes[1..].starts_with(&[b'.', b'.']) {
                    cur += 2;
                    Kind::Dot3
                } else {
                    Kind::Dot
                }
            }
            b';' => Kind::Semicolon,
            b',' => Kind::Comma,
            b'<' => match bytes[1..] {
                [b'<', b'=', ..] => {
                    cur += 2;
                    Kind::ShiftLeftEq
                }
                [b'<', ..] => {
                    cur += 1;
                    Kind::ShiftLeft
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::LtEq
                }
                _ => Kind::LAngle,
            },
            b'>' => match bytes[1..] {
                [b'>', b'>', b'=', ..] => {
                    cur += 3;
                    Kind::ShiftRight3Eq
                }
                [b'>', b'>', ..] => {
                    cur += 2;
                    Kind::ShiftRight3
                }
                [b'>', b'=', ..] => {
                    cur += 2;
                    Kind::ShiftRightEq
                }
                [b'>', ..] => {
                    cur += 1;
                    Kind::ShiftRight
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::GtEq
                }
                _ => Kind::RAngle,
            },
            b'=' => match bytes[1..] {
                [b'=', b'=', ..] => {
                    cur += 2;
                    Kind::Eq3
                }
                [b'>', ..] => {
                    cur += 1;
                    Kind::FatArrow
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::Eq2
                }
                _ => Kind::Eq,
            },
            b'!' => match bytes[1..] {
                [b'=', b'=', ..] => {
                    cur += 2;
                    Kind::Neq2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::Neq
                }
                _ => Kind::Bang,
            },
            b'+' => match bytes[1..] {
                [b'+', ..] => {
                    cur += 1;
                    Kind::Plus2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::PlusEq
                }
                _ => Kind::Plus,
            },
            b'-' => match bytes[1..] {
                [b'-', ..] => {
                    cur += 1;
                    Kind::Minus2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::MinusEq
                }
                _ => Kind::Minus,
            },
            b'*' => match bytes[1..] {
                [b'*', b'=', ..] => {
                    cur += 2;
                    Kind::Star2Eq
                }
                [b'*', ..] => {
                    cur += 1;
                    Kind::Star2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::StarEq
                }
                _ => Kind::Star,
            },
            b'&' => match bytes[1..] {
                [b'&', b'=', ..] => {
                    cur += 2;
                    Kind::Amp2Eq
                }
                [b'&', ..] => {
                    cur += 1;
                    Kind::Amp2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::AmpEq
                }
                _ => Kind::Amp,
            },
            b'|' => match bytes[1..] {
                [b'|', b'=', ..] => {
                    cur += 2;
                    Kind::Pipe2Eq
                }
                [b'|', ..] => {
                    cur += 1;
                    Kind::Pipe2
                }
                [b'=', ..] => {
                    cur += 1;
                    Kind::PipeEq
                }
                _ => Kind::Pipe,
            },
            b'~' => Kind::Tilde,
            b'?' => match bytes[1..] {
                [b'?', b'=', ..] => {
                    cur += 2;
                    Kind::Question2Eq
                }
                [b'?', b'.', ..] => {
                    cur += 2;
                    Kind::QuestionDot
                }
                [b'?', ..] => {
                    cur += 1;
                    Kind::Question2
                }
                _ => Kind::Question,
            },
            b'^' => match bytes[1..] {
                [b'=', ..] => {
                    cur += 1;
                    Kind::CaretEq
                }
                _ => Kind::Caret,
            },
            b'/' => match bytes[1..] {
                [b'=', ..] => {
                    cur += 1;
                    Kind::SlashEq
                }
                _ => Kind::Slash,
            },
            b'%' => match bytes[1..] {
                [b'=', ..] => {
                    cur += 1;
                    Kind::PercentEq
                }
                _ => Kind::Percent,
            },
            b':' => Kind::Colon,
            _ => return None,
        };
        Some(Token::new(kind, self.cur, cur + 1))
    }

    /// Section 12.6 Identifiers
    /// read a single identifier and return its length
    // TODO read Unicode
    const fn read_identifier(bytes: &[u8]) -> Option<usize> {
        if let Some(b) = bytes.first() {
            if b.is_ascii_alphabetic() || matches!(b, b'_' | b'$') {
                return Some(1);
            }
        }
        None
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

    /// 12.8.3 Numeric Literals
    /// TODO numeric separators
    /// TODO exponential
    /// TODO sign
    /// TODO float
    fn read_number(&self, bytes: &[u8]) -> Option<Token> {
        match bytes[0] {
            b'0' => {
                match bytes[1..] {
                    [b'b' | b'B', n, ..] => {
                        if matches!(n, b'0'..=b'1') {
                            let len = bytes[2..]
                                .iter()
                                .take_while(|b| matches!(b, b'0'..=b'1'))
                                .count();
                            Some(Token::new(Kind::Number(Number::Binary), self.cur, len + 2))
                        } else {
                            None
                        }
                    }
                    [b'o' | b'O', n, ..] => {
                        if matches!(n, b'0'..=b'7') {
                            let len = bytes[2..]
                                .iter()
                                .take_while(|b| matches!(b, b'0'..=b'7'))
                                .count();
                            Some(Token::new(Kind::Number(Number::Octal), self.cur, len + 2))
                        } else {
                            None
                        }
                    }
                    [b'x' | b'X', n, ..] => {
                        if n.is_ascii_hexdigit() {
                            let len = bytes[2..]
                                .iter()
                                .take_while(|b| b.is_ascii_hexdigit())
                                .count();
                            Some(Token::new(Kind::Number(Number::Hex), self.cur, len + 2))
                        } else {
                            None
                        }
                    }
                    [b'e' | b'E', n, ..] => {
                        if n.is_ascii_digit() {
                            let len = bytes[2..]
                                .iter()
                                .take_while(|b| b.is_ascii_hexdigit())
                                .count();
                            Some(Token::new(Kind::Number(Number::Decimal), self.cur, len + 2))
                        } else {
                            None
                        }
                    }
                    [n, ..] => {
                        // legacy octal
                        if n.is_ascii_digit() {
                            let mut kind = Number::Octal;
                            let len = bytes[1..]
                                .iter()
                                .take_while(|b| {
                                    if matches!(b, b'8'..=b'9') {
                                        kind = Number::Decimal;
                                    }
                                    b.is_ascii_digit()
                                })
                                .count();
                            Some(Token::new(Kind::Number(kind), self.cur, len + 1))
                        } else if n == b'n' {
                            Some(Token::new(Kind::Number(Number::BigInt), self.cur, 2))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            n if n.is_ascii_digit() => {
                let mut len = 1 + bytes[1..].iter().take_while(|b| b.is_ascii_digit()).count();
                let mut kind = Number::Decimal;
                if let Some(b) = bytes.get(len) {
                    if b == &b'n' {
                        len += 1;
                        kind = Number::BigInt;
                    }
                }
                Some(Token::new(Kind::Number(kind), self.cur, len))
            }
            _ => None,
        }
    }

    // 12.8.4 String Literals
    fn read_string_literal(&self, bytes: &[u8]) -> Option<Token> {
        match bytes[0] {
            b'\'' => {
                let len = bytes[1..].iter().take_while(|b| b != &&b'\'').count();
                Some(Token::new(Kind::Str, self.cur, len + 2))
            }
            b'"' => {
                let len = bytes[1..].iter().take_while(|b| b != &&b'"').count();
                dbg!(&len);
                Some(Token::new(Kind::Str, self.cur, len + 2))
            }
            _ => None,
        }
    }

    /* ---------- utils ---------- */

    /// Read line terminator and return its length
    fn read_line_terminator(bytes: &[u8]) -> Option<usize> {
        match *bytes {
            [b'\n', ..] => Some('\n'.len_utf8()),
            [b'\r', ..] => Some('\r'.len_utf8()),
            _ => {
                if bytes.starts_with("\u{2028}".as_bytes()) {
                    Some('\u{2028}'.len_utf8())
                } else if bytes.starts_with("\u{2029}".as_bytes()) {
                    Some('\u{2029}'.len_utf8())
                } else {
                    None
                }
            }
        }
        //
    }
}
