//! ECMAScript Token Kinds

#[derive(Debug, PartialEq)]
pub enum Kind {
    Unknown,

    // whitespace
    EOF,
    WhiteSpace,
    Newline,

    // keyword
    Null,
    Undefined,
    True,
    False,

    // identifier
    Ident,
}
