//! Lexer

use unicode_xid::UnicodeXID;

use crate::constants::UNICODE_SPACES;
use crate::kind::{Kind, Number};
use crate::state::State;
use crate::token::Token;

type LexerReturn = Option<(Kind, usize)>;

pub struct Lexer<'a> {
    /// The input string
    bytes: &'a [u8],

    /// The current index position
    cur: usize,

    /// Are we at the EOF?
    eof: bool,

    /// Lexer State
    state: State,
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

        let result = match self.bytes[self.cur] {
            b'/' => self.read_slash(bytes),
            b'0' => self.read_zero(bytes),
            b'1'..=b'9' => self.read_number(bytes),
            b'`' => self.read_template_literal(bytes),
            b'\'' | b'"' => self.read_string_literal(bytes),
            _ => self
                .read_whitespaces(bytes)
                .or_else(|| self.read_line_terminators(bytes))
                .or_else(|| self.read_identifier_or_keyword(bytes))
                .or_else(|| self.read_punctuator(bytes)),
        };

        let token = if let Some((kind, len)) = result {
            self.state.update(&kind);
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
            state: State::new(),
        }
    }

    /// Section 12.2 Whitespace
    fn read_whitespaces(&self, bytes: &[u8]) -> LexerReturn {
        let len = std::str::from_utf8(bytes)
            .unwrap()
            .chars()
            .take_while(|c| UNICODE_SPACES.contains(c))
            .map(char::len_utf8)
            .sum();
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

    /// Section 12.4 Single Line Comment
    #[allow(clippy::unnecessary_wraps)]
    fn read_single_comment(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'/');
        assert_eq!(bytes[1], b'/');
        let mut cur = 2;
        while let Some(bytes) = bytes.get(cur..) {
            if let Some(len) = self.read_line_terminator(bytes) {
                cur += len;
                break;
            }
            cur += 1;
        }
        Some((Kind::Comment, cur))
    }

    /// Section 12.4 Multi Line Comment
    #[allow(clippy::unnecessary_wraps)]
    fn read_multiline_comment(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'/');
        assert_eq!(bytes[1], b'*');
        let mut cur = 2;
        while let Some(bytes) = bytes.get(cur..) {
            if bytes.starts_with(&[b'*', b'/']) {
                cur += 2;
                break;
            }
            cur += 1;
        }
        Some((Kind::MultilineComment, cur))
    }

    /// Section 12.6.1 Identifier Names
    fn read_identifier_or_keyword(&self, bytes: &[u8]) -> LexerReturn {
        let mut iter = std::str::from_utf8(bytes).unwrap().chars().peekable();
        let mut len = 0;
        if let Some(c) = iter.next() {
            if self.is_identifier_start(c) {
                len += c.len_utf8();
            } else if c == '\\' && iter.peek() == Some(&'u') {
                if let Some(count) = self.read_unicode_escape_sequence(bytes) {
                    len += count;
                    for _ in 0..count - 1 {
                        iter.next();
                    }
                }
            }
        }
        if len == 0 {
            return None;
        }
        while let Some(c) = iter.next() {
            if self.is_identifier_part(c) {
                len += c.len_utf8();
            } else if c == '\\' && iter.peek() == Some(&'u') {
                if let Some(count) = self.read_unicode_escape_sequence(&bytes[len..]) {
                    len += count;
                    for _ in 0..count - 1 {
                        iter.next();
                    }
                }
            } else {
                break;
            }
        }
        let kind = self.read_keyword(&bytes[..len]);
        Some((kind, len))
    }

    /// Section 12.8.4 Read `UnicodeEscapeSequence`
    /// \u followed by 4 hex
    /// \u{digit} with 1..=6 TODO reference this source
    fn read_unicode_escape_sequence(&self, bytes: &[u8]) -> Option<usize> {
        assert_eq!(bytes[0], b'\\');
        assert_eq!(bytes[1], b'u');
        if bytes.get(2) == Some(&b'{') {
            let mut len = 0;
            while len < 6 {
                if let Some(b) = bytes.get(len + 3) {
                    if b.is_ascii_hexdigit() {
                        len += 1;
                    } else {
                        break;
                    }
                }
            }
            return if bytes.get(len + 3) == Some(&b'}') {
                Some(len + 4)
            } else {
                None
            };
        } else if let Some(bytes) = bytes.get(2..6) {
            if bytes.iter().all(u8::is_ascii_hexdigit) {
                return Some(6);
            }
        }
        None
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
            b'%' => match bytes[1..] {
                [b'=', ..] => {
                    cur += 1;
                    Kind::PercentEq
                }
                _ => Kind::Percent,
            },
            b':' => Kind::Colon,
            b'#' => Kind::Hash,
            _ => return None,
        };
        Some((kind, cur + 1))
    }

    /// Section 12.6.2 Keywords and Reserved Words
    const fn read_keyword(&self, bytes: &[u8]) -> Kind {
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

    /// 12.8.3 Numeric Literals with `0` prefix
    fn read_zero(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        match bytes[1..] {
            [b'b' | b'B', _, ..] => self.read_binary(bytes),
            [b'o' | b'O', _, ..] => self.read_octal(bytes),
            [b'x' | b'X', _, ..] => self.read_hex(bytes),
            [b'e' | b'E', _, ..] => self
                .read_number_exponent_part(&bytes[1..])
                .map(|len| (Kind::Number(Number::Decimal), len + 1)),
            [b'.', n, ..] => {
                if n.is_ascii_digit() {
                    self.read_number(bytes)
                } else {
                    Some((Kind::Number(Number::Float), 2))
                }
            }
            [b'n', ..] => Some((Kind::Number(Number::BigInt), 2)),
            [b'0'..=b'9', ..] => self.read_legacy_octal(bytes),
            _ => Some((Kind::Number(Number::Decimal), 1)),
        }
    }

    /// 12.8.3 Numeric Literals
    /// TODO numeric separators
    #[allow(clippy::unnecessary_wraps)]
    fn read_number(&self, bytes: &[u8]) -> LexerReturn {
        assert!(bytes[0].is_ascii_digit());
        let mut kind = Number::Decimal;
        let mut len = 1;
        for b in bytes.iter().skip(1) {
            match &b {
                b'.' => {
                    if kind == Number::Float {
                        break;
                    }
                    len += 1;
                    kind = Number::Float;
                }
                b'n' => {
                    if kind != Number::Decimal {
                        break;
                    }
                    len += 1;
                    kind = Number::BigInt;
                    break;
                }
                b'e' | b'E' => {
                    if let Some(count) = self.read_number_exponent_part(&bytes[len..]) {
                        return Some((Kind::Number(kind), len + count));
                    }
                }
                n if n.is_ascii_digit() => {
                    len += 1;
                }
                _ => break,
            }
        }
        Some((Kind::Number(kind), len))
    }
    fn read_number_exponent_part(&self, bytes: &[u8]) -> Option<usize> {
        assert!(matches!(bytes[0], b'e' | b'E'));
        let mut iter = bytes.iter().skip(1).peekable();
        let mut len = 1;
        if iter.next_if(|b| b == &&b'-' || b == &&b'+').is_some() {
            len += 1;
        }
        let digits = iter.take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 {
            return None;
        }
        Some(len + digits)
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
        assert_eq!(bytes[0], b'/');
        assert_ne!(bytes[1], b'/');
        let mut cur = 1;
        let mut iter = bytes[cur..].iter();
        let mut bracket = false;
        while let Some(b) = iter.next() {
            match &b {
                b'[' => {
                    bracket = true;
                }
                b']' => {
                    bracket = false;
                }
                b'/' => {
                    if bracket {
                        cur += 1;
                        continue;
                    }
                    return Some((Kind::Regex, cur + 1));
                }
                b'\\' => {
                    if bytes.get(cur + 1) == Some(&b'/') {
                        cur += 2;
                        iter.next();
                        continue;
                    }
                }
                _ => {}
            }
            cur += 1;
        }
        // TODO error
        None
    }

    /// 12.8.6 Template Literal Lexical Components
    fn read_template_literal(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'`');
        let mut iter = bytes.iter().enumerate().skip(1).peekable();
        while let Some((len, b)) = iter.next() {
            match &b {
                b'\\' => {
                    iter.next_if(|t| t.1 == &b'`' || t.1 == &b'\\');
                }
                b'`' => return Some((Kind::Template, len + 1)),
                _ => {}
            }
        }
        None
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

    /// Read Slash `/`:
    ///   * Single Line Comment //
    ///   * `MultilineComment` /* */
    ///   * Regex /regex/
    ///   * `SlashEq` /=
    ///   * `Slash` /
    fn read_slash(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'/');
        match bytes.get(1) {
            Some(b'/') => self.read_single_comment(bytes),
            Some(b'*') => self.read_multiline_comment(bytes),
            Some(b'=') => Some((Kind::SlashEq, 2)),
            Some(_) if self.state.allow_read_regex() => self.read_regex(bytes),
            _ => Some((Kind::Slash, 1)),
        }
    }

    /* ---------- utils ---------- */

    /// Section 12.6 Detect `IdentifierStartChar`
    fn is_identifier_start(&self, c: char) -> bool {
        c == '$' || c == '_' || c.is_xid_start() // contains c.is_ascii_alphabetic() check
    }

    /// Section 12.6 Detect `IdentifierPartChar`
    fn is_identifier_part(&self, c: char) -> bool {
        c == '$' || c == '_' || c.is_xid_continue() // contains c.is_ascii_alphanumeric() check
            || c == '\u{200c}' || c == '\u{200d}'
    }

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
            .map(char::len_utf8)
    }
}
