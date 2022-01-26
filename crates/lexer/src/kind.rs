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
    // 12.6.2
    AWAIT,
    BREAK,
    CASE,
    CATCH,
    CLASS,
    CONST,
    CONTINUE,
    DEBUGGER,
    DEFAULT,
    DELETE,
    DO,
    ELSE,
    ENUM,
    EXPORT,
    EXTENDS,
    FALSE,
    FINALLY,
    FOR,
    FUNCTION,
    IF,
    IN,
    IMPORT,
    INSTANCEOF,
    NEW,
    NULL,
    RETURN,
    SUPER,
    SWITCH,
    THIS,
    THROW,
    TRY,
    TRUE,
    TYPEOF,
    VAR,
    VOID,
    WHILE,
    WITH,
    YIELD,

    // keyword
    Null,
    Undefined,
    True,
    False,

    // identifier
    Ident,
}
