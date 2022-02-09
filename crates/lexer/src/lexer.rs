//! Lexer
use unicode_id::UnicodeID;

use crate::constants::{
    ASCII_LINE_TERMINATORS, ASCII_LINE_TERMINATORS_CHAR, ASCII_SPACES, UNICODE_LINE_TERMINATORS,
    UNICODE_SPACES,
};
use crate::kind::{Kind, Number};
use crate::state::State;
use crate::token::Token;

type LexerReturn = Option<(Kind, usize)>;

pub struct Lexer<'a> {
    /// The input string
    bytes: &'a [u8],

    /// The cursor position
    cur: usize,

    /// Are we at the EOF?
    eof: bool,

    /// Lexer State
    state: State,
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // return EOF if the current index is out of bounds
        // uses unlikely for branch prediction for improving performance
        if self.cur >= self.bytes.len() {
            if self.eof {
                return None;
            }
            self.eof = true;
            return Some(Token::new(Kind::EOF, self.cur, 1));
        }

        // find the next token by examining from the current position
        let result = self.dispatch_read(&self.bytes[self.cur..]);
        let token = if let Some((kind, len)) = result {
            self.state.update(&kind);
            Token::new(kind, self.cur, len)
        } else {
            Token::new(Kind::Unknown, self.cur, 1)
        };

        // move the cursor
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

    #[inline]
    fn dispatch_read(&self, bytes: &[u8]) -> LexerReturn {
        match self.bytes[self.cur] {
            b'/' => self.read_slash(bytes),
            b'0' => self.read_zero(bytes),
            b'1'..=b'9' => self.read_number(bytes),
            b'`' => self.read_template_literal(bytes),
            b'\'' | b'"' => self.read_string_literal(bytes),
            9 | 11 | 12 | b' ' => self.read_ascii_whitespaces(bytes),
            b'\n' | b'\r' => self.read_ascii_line_terminators(bytes),
            b'$' | b'_' => self.read_identifier(bytes),
            b'{' => Some((Kind::LCurly, 1)),
            b'}' => Some((Kind::RCurly, 1)),
            b'(' => Some((Kind::LParen, 1)),
            b')' => Some((Kind::RParen, 1)),
            b'[' => Some((Kind::LBrack, 1)),
            b']' => Some((Kind::RBrack, 1)),
            b';' => Some((Kind::Semicolon, 1)),
            b',' => Some((Kind::Comma, 1)),
            b':' => Some((Kind::Colon, 1)),
            b'#' => Some((Kind::Hash, 1)),
            b'~' => Some((Kind::Tilde, 1)),
            b'.' => Some(self.read_dot(bytes)),
            b'<' => Some(self.read_left_angle(bytes)),
            b'>' => Some(self.read_right_angle(bytes)),
            b'=' => Some(self.read_equal(bytes)),
            b'!' => Some(self.read_exclamation(bytes)),
            b'+' => Some(self.read_plus(bytes)),
            b'-' => Some(self.read_minus(bytes)),
            b'^' => Some(self.read_caret(bytes)),
            b'%' => Some(self.read_percent(bytes)),
            b'*' => Some(self.read_star(bytes)),
            b'&' => Some(self.read_ampersand(bytes)),
            b'|' => Some(self.read_pipe(bytes)),
            b'?' => Some(self.read_question(bytes)),
            n if n.is_ascii_alphabetic() => self
                .read_identifier(bytes)
                .map(|(_, len)| (self.read_keyword(&bytes[..len]), len)),
            _ => self
                .read_unicode_whitespaces(bytes)
                .or_else(|| self.read_unicode_line_terminators(bytes))
                .or_else(|| self.read_identifier(bytes)),
        }
    }

    /// Section 12.2 Whitespace
    #[inline]
    fn read_ascii_whitespaces(&self, bytes: &[u8]) -> LexerReturn {
        let len = bytes
            .iter()
            .take_while(|c| ASCII_SPACES.contains(c))
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::WhiteSpace, len))
    }

    #[inline]
    fn read_unicode_whitespaces(&self, bytes: &[u8]) -> LexerReturn {
        let len = Lexer::from_utf8_unchecked(bytes)
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
    #[inline]
    fn read_ascii_line_terminators(&self, bytes: &[u8]) -> LexerReturn {
        let len = bytes
            .iter()
            .take_while(|c| ASCII_LINE_TERMINATORS.contains(c))
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::LineTerminator, len))
    }

    #[inline]
    fn read_unicode_line_terminators(&self, bytes: &[u8]) -> LexerReturn {
        let len = Lexer::from_utf8_unchecked(bytes)
            .chars()
            .take_while(|c| UNICODE_LINE_TERMINATORS.contains(c))
            .map(char::len_utf8)
            .sum();
        if len == 0 {
            return None;
        }
        Some((Kind::LineTerminator, len))
    }

    /// Section 12.4 Single Line Comment
    #[inline]
    fn read_single_comment(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'/');
        assert_eq!(bytes[1], b'/');
        let len = Lexer::from_utf8_unchecked(bytes)
            .chars()
            .skip(2)
            .take_while(|c| {
                !ASCII_LINE_TERMINATORS_CHAR.contains(c) && !UNICODE_LINE_TERMINATORS.contains(c)
            })
            .map(char::len_utf8)
            .sum::<usize>();
        (Kind::Comment, len + 2)
    }

    /// Section 12.4 Multi Line Comment
    #[inline]
    fn read_multiline_comment(&self, bytes: &[u8]) -> (Kind, usize) {
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
        (Kind::MultilineComment, cur)
    }

    /// Section 12.6.1 Identifier Names
    #[inline]
    fn read_identifier(&self, bytes: &[u8]) -> LexerReturn {
        let mut iter = Lexer::from_utf8_unchecked(bytes).chars().peekable();
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
        Some((Kind::Ident, len))
    }

    /// Section 12.6.2 Keywords and Reserved Words
    #[inline]
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

    /// Section 12.8.4 Read `UnicodeEscapeSequence`
    /// \u followed by 4 hex
    /// \u{digit} with 1..=6 TODO reference this source
    #[inline]
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
    #[inline]
    fn read_dot(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'.');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.next_if_eq(&&b'.').is_some() && iter.peek() == Some(&&b'.') {
            return (Kind::Dot3, 3);
        }
        (Kind::Dot, 1)
    }

    #[inline]
    fn read_left_angle(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'<');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::LtEq, 2); // <=
        }
        if iter.next_if_eq(&&b'<').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::ShiftLeftEq, 3); // <<=
            }
            return (Kind::ShiftLeft, 2); // <<
        }
        (Kind::LAngle, 1) // <
    }

    #[inline]
    fn read_right_angle(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'>');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::GtEq, 2); // >=
        }
        if iter.next_if_eq(&&b'>').is_some() {
            if iter.next_if_eq(&&b'>').is_some() {
                if iter.peek() == Some(&&b'=') {
                    return (Kind::ShiftRight3Eq, 4); // >>>=
                }
                return (Kind::ShiftRight3, 3); // >>>
            }
            if iter.peek() == Some(&&b'=') {
                return (Kind::ShiftRightEq, 3); // >>=
            }
            return (Kind::ShiftRight, 2); // >>
        }
        (Kind::RAngle, 1) // <
    }

    #[inline]
    fn read_equal(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'=');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.next_if_eq(&&b'=').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Eq3, 3); // ===
            }
            return (Kind::Eq2, 2); // ==
        }
        if iter.peek() == Some(&&b'>') {
            return (Kind::FatArrow, 2); // =>
        }
        (Kind::Eq, 1)
    }

    #[inline]
    fn read_exclamation(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'!');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.next_if_eq(&&b'=').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Neq2, 3); // !==
            }
            return (Kind::Neq, 2); // !=
        }
        (Kind::Bang, 1)
    }

    #[inline]
    fn read_plus(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'+');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::PlusEq, 2); // +=
        }
        if iter.peek() == Some(&&b'+') {
            return (Kind::Plus2, 2); // ++
        }
        (Kind::Plus, 1) // +
    }

    #[inline]
    fn read_minus(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'-');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::MinusEq, 2); // -=
        }
        if iter.peek() == Some(&&b'-') {
            return (Kind::Minus2, 2); // --
        }
        (Kind::Minus, 1) // -
    }

    #[inline]
    fn read_caret(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'^');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::CaretEq, 2); // ^=
        }
        (Kind::Caret, 1) // ^
    }

    #[inline]
    fn read_percent(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'%');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::PercentEq, 2); // %=
        }
        (Kind::Percent, 1) // %
    }

    #[inline]
    fn read_star(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'*');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::StarEq, 2); // *=
        }
        if iter.next_if_eq(&&b'*').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Star2Eq, 3); // **=
            }
            return (Kind::Star2, 2); // **
        }
        (Kind::Star, 1) // *
    }

    #[inline]
    fn read_ampersand(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'&');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::AmpEq, 2); // &=
        }
        if iter.next_if_eq(&&b'&').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Amp2Eq, 3); // &&=
            }
            return (Kind::Amp2, 2); // &&
        }
        (Kind::Amp, 1) // &
    }

    #[inline]
    fn read_pipe(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'|');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'=') {
            return (Kind::PipeEq, 2); // |=
        }
        if iter.next_if_eq(&&b'|').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Pipe2Eq, 3); // ||=
            }
            return (Kind::Pipe2, 2); // ||
        }
        (Kind::Pipe, 1) // |
    }

    #[inline]
    fn read_question(&self, bytes: &[u8]) -> (Kind, usize) {
        assert_eq!(bytes[0], b'?');
        let mut iter = bytes.iter().skip(1).peekable();
        if iter.peek() == Some(&&b'.') {
            return (Kind::QuestionDot, 2); // ?.
        }
        if iter.next_if_eq(&&b'?').is_some() {
            if iter.peek() == Some(&&b'=') {
                return (Kind::Question2Eq, 3); // ??=
            }
            return (Kind::Question2, 2); // ??
        }
        (Kind::Question, 1) // ?
    }

    /// 12.8.3 Numeric Literals with `0` prefix
    #[inline]
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
            [b'.'] => Some((Kind::Number(Number::Decimal), 2)),
            [b'n', ..] => Some((Kind::Number(Number::BigInt), 2)),
            [b'0'..=b'9', ..] => self.read_legacy_octal(bytes),
            _ => Some((Kind::Number(Number::Decimal), 1)),
        }
    }

    /// 12.8.3 Numeric Literals
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
                b'_' => {
                    len += 1;
                }
                n if n.is_ascii_digit() => {
                    len += 1;
                }
                _ => break,
            }
        }
        Some((Kind::Number(kind), len))
    }

    #[inline]
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

    #[inline]
    fn read_binary(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'b' | b'B'));
        let len = bytes
            .iter()
            .skip(2)
            .take_while(|b| matches!(b, b'0'..=b'1') || b == &&b'_')
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::Number(Number::Binary), len + 2))
    }

    #[inline]
    fn read_octal(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'o' | b'O'));
        let len = bytes
            .iter()
            .skip(2)
            .take_while(|b| matches!(b, b'0'..=b'7') || b == &&b'_')
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::Number(Number::Octal), len + 2))
    }

    #[inline]
    fn read_legacy_octal(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'0'..=b'9'));
        let mut kind = Number::Octal;
        let len = bytes
            .iter()
            .skip(1)
            .take_while(|b| {
                if matches!(b, b'8'..=b'9') {
                    kind = Number::Decimal;
                }
                b.is_ascii_digit() || b == &&b'_'
            })
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::Number(kind), len + 1))
    }

    #[inline]
    fn read_hex(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'0');
        assert!(matches!(bytes[1], b'x' | b'X'));
        let len = bytes
            .iter()
            .skip(2)
            .take_while(|b| b.is_ascii_hexdigit() || b == &&b'_')
            .count();
        if len == 0 {
            return None;
        }
        Some((Kind::Number(Number::Hex), len + 2))
    }

    /// 12.8.4 String Literals
    #[inline]
    fn read_string_literal(&self, bytes: &[u8]) -> LexerReturn {
        assert!(matches!(bytes[0], b'\'' | b'"'));
        let mut iter = Lexer::from_utf8_unchecked(bytes).chars().peekable();
        let quote = iter.next().unwrap();
        let mut len = 1;
        while let Some(b) = iter.next() {
            len += b.len_utf8();
            if b == '\\' && iter.peek().map_or(false, |q| q == &'\\' || q == &quote) {
                len += 1;
                iter.next();
            } else if b == quote {
                return Some((Kind::Str, len));
            }
        }
        None
    }

    /// 12.8.5 Regular Expression Literals
    #[inline]
    fn read_regex(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'/');
        assert_ne!(bytes[1], b'/');
        let mut cur = 1;
        let mut iter = bytes.iter().skip(1).peekable();
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
                    if iter.peek().map_or(false, |q| q == &&b'/' || q == &&b'\\') {
                        cur += 1;
                        iter.next();
                    }
                }
                _ => {}
            }
            cur += 1;
        }
        None
    }

    /// 12.8.6 Template Literal Lexical Components
    #[inline]
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

    /// Read Slash `/`:
    ///   * Single Line Comment //
    ///   * `MultilineComment` /* */
    ///   * Regex /regex/
    ///   * `SlashEq` /=
    ///   * `Slash` /
    #[inline]
    fn read_slash(&self, bytes: &[u8]) -> LexerReturn {
        assert_eq!(bytes[0], b'/');
        match bytes.get(1) {
            Some(b'/') => Some(self.read_single_comment(bytes)),
            Some(b'*') => Some(self.read_multiline_comment(bytes)),
            Some(b'=') => Some((Kind::SlashEq, 2)),
            Some(_) if self.state.allow_read_regex() => self.read_regex(bytes),
            _ => Some((Kind::Slash, 1)),
        }
    }

    /* ---------- utils ---------- */
    /// Section 12.6 Detect `IdentifierStartChar`
    #[inline]
    fn is_identifier_start(&self, c: char) -> bool {
        c == '$' || c == '_' || c.is_id_start() // contains c.is_ascii_alphabetic() check
    }

    /// Section 12.6 Detect `IdentifierPartChar`
    #[inline]
    fn is_identifier_part(&self, c: char) -> bool {
        c == '$' || c == '_' || c.is_id_continue() // contains c.is_ascii_alphanumeric() check
            || c == '\u{200c}' || c == '\u{200d}'
    }

    /// `std::str::from_utf8_unchecked`
    /// Safefy: we assumed byte string is utf8
    #[inline]
    #[must_use]
    const fn from_utf8_unchecked(bytes: &[u8]) -> &str {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}
