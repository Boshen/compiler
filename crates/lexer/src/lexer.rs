//! Lexer

use crate::kind::{Kind, Number};
use crate::token::Token;

type LexerReturn = Option<(Kind, usize)>;

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

        let result = self
            .read_whitespaces(bytes)
            .or_else(|| self.read_line_terminators(bytes))
            .or_else(|| self.read_comment(bytes))
            .or_else(|| self.read_name_or_keyword(bytes))
            .or_else(|| self.read_regex(bytes))
            .or_else(|| self.read_punctuator(bytes))
            .or_else(|| self.read_number(bytes))
            .or_else(|| self.read_string_literal(bytes))
            .or_else(|| self.read_template_literal(bytes));

        let token = if let Some((kind, len)) = result {
            Token::new(kind, self.cur, len)
        } else {
            Token::new(Kind::Unknown, self.cur, 1)
        };

        self.cur += token.len();

        Some(token)
    }
}

#[allow(clippy::unused_self)]
impl<'a> Lexer<'a> {
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self {
            bytes: source.as_bytes(),
            cur: 0,
            eof: false,
        }
    }

    /// Section 12.2 Whitespace
    /// TODO read all whitespace codepoints
    fn read_whitespaces(&self, bytes: &[u8]) -> LexerReturn {
        let len = bytes.iter().take_while(|b| b.is_ascii_whitespace()).count();
        if len == 0 {
            return None;
        }
        Some((Kind::WhiteSpace, len))
    }

    /// Section 12.3 Line Terminators
    fn read_line_terminators(&self, bytes: &[u8]) -> LexerReturn {
        let mut cur = 0;
        while let Some(bytes) = bytes.get(cur..) {
            if let Some(len) = self.read_line_terminator(bytes) {
                cur += len;
            } else {
                break;
            }
        }
        if cur == 0 {
            return None;
        }
        Some((Kind::LineTerminator, cur))
    }

    /// Section 12.4 Comments
    fn read_comment(&self, bytes: &[u8]) -> LexerReturn {
        if bytes.starts_with(&[b'/', b'/']) {
            let mut cur = 2;
            while let Some(bytes) = bytes.get(cur..) {
                if let Some(len) = self.read_line_terminator(bytes) {
                    cur += len;
                    break;
                }
                cur += 1;
            }
            return Some((Kind::Comment, cur));
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
            return Some((Kind::MultilineComment, cur));
        }
        // TODO Error
        None
    }

    /// Section 12.6 Names and Keywords
    fn read_name_or_keyword(&self, bytes: &[u8]) -> LexerReturn {
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
        Some((kind, cur))
    }

    /// Section 12.7 Punctuators
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn read_punctuator(&self, bytes: &[u8]) -> LexerReturn {
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
                // TODO fix regex here
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
        Some((kind, cur + 1))
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
    /// TODO float
    fn read_number(&self, bytes: &[u8]) -> LexerReturn {
        match bytes[0] {
            b'0' => match bytes[1..] {
                [b'b' | b'B', _, ..] => self.read_binary(bytes),
                [b'o' | b'O', _, ..] => self.read_octal(bytes),
                [b'x' | b'X', _, ..] => self.read_hex(bytes),
                [b'e' | b'E', n, ..] => {
                    if n.is_ascii_digit() {
                        let len = bytes[2..]
                            .iter()
                            .take_while(|b| b.is_ascii_hexdigit())
                            .count();
                        Some((Kind::Number(Number::Decimal), len + 2))
                    } else {
                        None
                    }
                }
                [b'n', ..] => Some((Kind::Number(Number::BigInt), 2)),
                [b'0'..=b'9', ..] => self.read_legacy_octal(bytes),
                _ => Some((Kind::Number(Number::Decimal), 1)),
            },
            n if n.is_ascii_digit() => {
                let mut len = 1 + bytes[1..].iter().take_while(|b| b.is_ascii_digit()).count();
                let mut kind = Number::Decimal;
                if let Some(b) = bytes.get(len) {
                    if b == &b'n' {
                        len += 1;
                        kind = Number::BigInt;
                    }
                }
                Some((Kind::Number(kind), len))
            }
            _ => None,
        }
    }

    fn read_binary(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'b' | b'B'));
        let len = bytes[2..]
            .iter()
            .take_while(|b| matches!(b, b'0'..=b'1'))
            .count();
        if len == 0 {
            None
        } else {
            Some((Kind::Number(Number::Binary), len + 2))
        }
    }

    fn read_octal(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'o' | b'O'));
        let len = bytes[2..]
            .iter()
            .take_while(|b| matches!(b, b'0'..=b'7'))
            .count();
        if len == 0 {
            None
        } else {
            Some((Kind::Number(Number::Octal), len + 2))
        }
    }

    fn read_legacy_octal(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'0'..=b'9'));
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
        if len == 0 {
            None
        } else {
            Some((Kind::Number(kind), len + 1))
        }
    }

    fn read_hex(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'x' | b'X'));
        let len = bytes[2..]
            .iter()
            .take_while(|b| b.is_ascii_hexdigit())
            .count();
        if len == 0 {
            None
        } else {
            Some((Kind::Number(Number::Hex), len + 2))
        }
    }

    /// 12.8.4 String Literals
    fn read_string_literal(&self, bytes: &[u8]) -> LexerReturn {
        match bytes[0] {
            quote @ (b'\'' | b'"') => {
                let mut cur = 1;
                let mut iter = bytes[cur..].iter();
                while let Some(b) = iter.next() {
                    if b == &b'\\' {
                        if let Some(size) = self.read_escape_sequence(&bytes[cur..]) {
                            cur += size;
                            (0..size - 1).for_each(|_| {
                                iter.next();
                            });
                        } else {
                            // TODO error
                            return None;
                        }
                    } else if b == &quote {
                        return Some((Kind::Str, cur + 1));
                    } else {
                        cur += 1;
                    }
                }
                // TODO error
                None
            }
            _ => None,
        }
    }

    /// 12.8.5 Regular Expression Literals
    fn read_regex(&self, bytes: &[u8]) -> LexerReturn {
        match bytes[0] {
            // TODO combine this check with comment slash and single slash
            b'/' => {
                if bytes.get(1) == Some(&b'/') {
                    return None;
                }
                let mut cur = 1;
                let mut iter = bytes[cur..].iter();
                while let Some(b) = iter.next() {
                    if b == &b'\\' && bytes.get(cur + 1) == Some(&b'/') {
                        cur += 2;
                        iter.next();
                    } else if b == &b'/' {
                        return Some((Kind::Regex, cur + 1));
                    } else {
                        cur += 1;
                    }
                }
                // TODO error
                None
            }
            _ => None,
        }
    }

    /// 12.8.6 Template Literal Lexical Components
    fn read_template_literal(&self, bytes: &[u8]) -> LexerReturn {
        match bytes[0] {
            b'`' => {
                let len = bytes[1..].iter().take_while(|b| b != &&b'`').count();
                Some((Kind::Template, len + 2))
            }
            _ => None,
        }
    }

    fn read_escape_sequence(&self, bytes: &[u8]) -> Option<usize> {
        assert_eq!(bytes[0], b'\\');
        if let Some(b) = bytes.get(1) {
            if let Some(len) = self.read_line_terminator(&bytes[1..]) {
                return Some(len);
            }
            return match b {
                b'\\' | b'n' | b'r' | b't' | b'b' | b'v' | b'f' | b'\'' | b'"' => Some(2),
                b'u' => Some(5),
                b'x' => Some(3),
                _ => self.read_unicode_char(&bytes[1..]),
            };
        }
        None
    }

    /* ---------- utils ---------- */

    /// Read line terminator and return its length
    fn read_line_terminator(&self, bytes: &[u8]) -> Option<usize> {
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

    /// Read the next unicode character by converting it to string
    fn read_unicode_char(&self, bytes: &[u8]) -> Option<usize> {
        std::str::from_utf8(bytes)
            .ok()
            .and_then(|str| str.chars().next())
            .map(|c| c.len_utf8())
    }
}
