//! ECMAScript Token Kinds

#[derive(Debug, PartialEq)]
pub enum Number {
    Decimal,
    Float,
    Binary,
    Octal,
    Hex,
    BigInt,
}

#[derive(Debug, PartialEq)]
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
    In,
    Import,
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
    LCurly,
    RCurly,
    LParen,
    RParen,
    LBrack,
    RBrack,
    Dot,
    Dot3, // ...
    Semicolon,
    Comma,
    LAngle,
    RAngle,
    LtEq, // <=
    GtEq, // >=
    Eq,
    Eq2,
    Eq3,
    Neq,
    Neq2,
    Plus,
    Plus2,
    PlusEq,
    Minus,
    Minus2,
    MinusEq,
    Star,
    Star2,
    StarEq,
    Star2Eq,
    ShiftLeft,     // <<
    ShiftLeftEq,   // <<=
    ShiftRight,    // >>
    ShiftRightEq,  // >>=
    ShiftRight3,   // >>>
    ShiftRight3Eq, // >>>=
    Amp,           // &
    AmpEq,
    Amp2,
    Amp2Eq,
    Pipe,
    PipeEq,
    Pipe2,
    Pipe2Eq,
    Bang,
    Tilde,
    Question,
    Question2,
    Question2Eq,
    QuestionDot,
    Caret,
    CaretEq,
    Slash,
    SlashEq,
    FatArrow,
    Percent,
    PercentEq,
    Colon,
    // 12.8.1 Null Literals
    Null,
    // 12.8.2 Boolean Literals
    True,
    False,
    // 12.8.3 Numeric Literals
    Number(Number),
    // 12.8.4 String Literals
    Str,
}
