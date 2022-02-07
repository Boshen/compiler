//! ECMAScript Token Kinds

#[derive(Debug, Eq, PartialEq)]
pub enum Number {
    Decimal,
    Float,
    Binary,
    Octal,
    Hex,
    BigInt,
}

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kind {
    Unknown,
    EOF,
    // 12.2 whitespace
    WhiteSpace,
    // 12.3 line terminator
    LineTerminator,
    // 12.4 comment
    Comment,
    MultilineComment,
    // 12.6 identifier
    Ident,
    // 12.6.2 keyword
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    DefaulT,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    FinallY,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Return,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield,
    // 12.7 punctuators
    Amp, // &
    Amp2,
    Amp2Eq,
    AmpEq,
    Bang,
    Caret,
    CaretEq,
    Colon,
    Comma,
    Dot,
    Dot3, // ...
    Eq,
    Eq2,
    Eq3,
    FatArrow,
    GtEq, // >=
    LAngle,
    LBrack,
    LCurly,
    LParen,
    LtEq, // <=
    Minus,
    Minus2,
    MinusEq,
    Neq,
    Neq2,
    Percent,
    PercentEq,
    Pipe,
    Pipe2,
    Pipe2Eq,
    PipeEq,
    Plus,
    Plus2,
    PlusEq,
    Question,
    Question2,
    Question2Eq,
    QuestionDot,
    RAngle,
    RBrack,
    RCurly,
    RParen,
    Semicolon,
    ShiftLeft,     // <<
    ShiftLeftEq,   // <<=
    ShiftRight,    // >>
    ShiftRight3,   // >>>
    ShiftRight3Eq, // >>>=
    ShiftRightEq,  // >>=
    Slash,
    SlashEq,
    Star,
    Star2,
    Star2Eq,
    StarEq,
    Tilde,
    // 12.8.1 Null Literals
    Null,
    Undefined,
    // 12.8.2 Boolean Literals
    True,
    False,
    // 12.8.3 Numeric Literals
    Number(Number),
    // 12.8.4 String Literals
    Str,
    // 12.8.5 Regular Expression Literals
    Regex,
    // 12.8.6 Template Literal Lexical Components
    Template,
    // TODO section
    Hash,
}
#[allow(clippy::enum_glob_use)]
use self::Kind::*;
impl Kind {
    #[must_use]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, WhiteSpace)
    }

    // https://stackoverflow.com/questions/5519596/when-parsing-javascript-what-determines-the-meaning-of-a-slash
    // https://www-archive.mozilla.org/js/language/js20-2002-04/rationale/syntax.html#regular-expressions
    #[must_use]
    pub const fn at_expr(&self) -> bool {
        // punctuators
        match self {
            LCurly | LParen | LBrack | Dot | Dot3 | Semicolon | Comma | LAngle | RAngle | LtEq
            | GtEq | Eq | Eq2 | Eq3 | Neq | Neq2 | Plus | Plus2 | PlusEq | Minus | Minus2
            | MinusEq | Star | Star2 | StarEq | Star2Eq | ShiftLeft | ShiftLeftEq | ShiftRight
            | ShiftRightEq | ShiftRight3 | ShiftRight3Eq | Amp | AmpEq | Amp2 | Amp2Eq | Pipe
            | PipeEq | Pipe2 | Pipe2Eq | Bang | Tilde | Question | Question2 | Question2Eq
            | QuestionDot | Caret | CaretEq | Slash | SlashEq | FatArrow | Percent | PercentEq
            | Colon => return true,
            _ => {}
        };
        // keywords
        match self {
            New | Delete | Void | Typeof | Instanceof | In | Do | Return | Case | Throw | Else => {
                return true
            }
            _ => {}
        };
        false
    }
}
