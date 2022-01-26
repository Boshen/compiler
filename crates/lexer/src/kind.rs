//! ECMAScript Token Kinds

#[derive(Debug, PartialEq)]
pub enum Kind {
    Unknown,

    EOF,
    // 12.2
    WhiteSpace,
    // 12.3
    LineTerminator,
    // 12.4
    Comment,
    MultilineComment,

    // keyword
    Null,
    Undefined,
    True,
    False,

    // identifier
    Ident,
}
