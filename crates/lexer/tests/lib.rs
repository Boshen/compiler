#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#[cfg(test)]
use lexer::{Kind, Lexer};

#[allow(clippy::enum_glob_use)]
use lexer::Kind::*;
#[allow(clippy::enum_glob_use)]
use lexer::Number::*;

fn test(kind: Kind, input: &str) {
    let tokens = Lexer::new(input).into_iter().collect::<Vec<_>>();
    assert_eq!(tokens.len() - 1, 1, "{kind:?} {input} {tokens:?}");
    let token = tokens.first().unwrap();
    assert_eq!(token.kind(), &kind, "{kind:?} {input} {tokens:?}");
    assert_eq!(token.range(), 0..input.len(), "{kind:?} {input} {tokens:?}");
}

#[test]
fn eof() {
    let input = "";
    let tokens = Lexer::new(input).into_iter().collect::<Vec<_>>();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.first().unwrap().kind(), &EOF);
}

#[test]
fn whitespace() {
    [
        "\u{0009}", // <Tab>
        "\u{000B}", // <VT>
        "\u{000C}", // <FF>
        "\u{0020}", // <SP>
        "\u{00A0}", // <NBSP>
        "\u{FEFF}", // <ZWNBSP>
        "\u{2002}", // <USP>
        "\u{2003}", // <USP>
    ]
    .into_iter()
    .for_each(|s| {
        test(WhiteSpace, s);
    });
}

#[test]
fn line_terminator() {
    [
        "\u{000A}", // <LF>
        "\u{000D}", // <CR>
        "\u{2028}", // <LS>
        "\u{2029}", // <PS>
    ]
    .into_iter()
    .for_each(|s| {
        test(LineTerminator, s);
    });
}

#[test]
fn single_line_comment() {
    ["//s", "// s"].into_iter().for_each(|s| {
        test(Comment, s);
    });
}

#[test]
fn multi_line_comment() {
    [
        "/* multi line comment */",
        "/* multi * \n / line \n comment */",
    ]
    .into_iter()
    .for_each(|s| {
        test(MultilineComment, s);
    });
}

#[test]
fn reserved_word() {
    [
        (Await, "await"),
        (Break, "break"),
        (Case, "case"),
        (Catch, "catch"),
        (Class, "class"),
        (Const, "const"),
        (Continue, "continue"),
        (Debugger, "debugger"),
        (DefaulT, "default"),
        (Delete, "delete"),
        (Do, "do"),
        (Else, "else"),
        (Enum, "enum"),
        (Export, "export"),
        (Extends, "extends"),
        (FinallY, "finally"),
        (For, "for"),
        (Function, "function"),
        (If, "if"),
        (Import, "import"),
        (In, "in"),
        (Instanceof, "instanceof"),
        (New, "new"),
        (Return, "return"),
        (Super, "super"),
        (Switch, "switch"),
        (This, "this"),
        (Throw, "throw"),
        (Try, "try"),
        (Typeof, "typeof"),
        (Var, "var"),
        (Void, "void"),
        (While, "while"),
        (With, "with"),
        (Yield, "yield"),
        (Null, "null"),
        (True, "true"),
        (False, "false"),
    ]
    .into_iter()
    .for_each(|(kind, s)| {
        test(kind, s);
    });
}

#[test]
fn identifier() {
    [
        "$",
        "_",
        "$a",
        "_a",
        "abc_$",
        r#"\u03bc"#,
        r#"\u{61}"#,
        r#"x\u03bc"#,
        "x\u{61}",
        "x‍",
        "x‌",
    ]
    .into_iter()
    .for_each(|s| test(Ident, s))
}

#[test]
fn punctuator() {
    [
        (Amp, "&"),
        (Amp2, "&&"),
        (Amp2Eq, "&&="),
        (AmpEq, "&="),
        (Bang, "!"),
        (Caret, "^"),
        (CaretEq, "^="),
        (Colon, ":"),
        (Comma, ","),
        (Dot, "."),
        (Dot3, "..."),
        (Eq, "="),
        (Eq2, "=="),
        (Eq3, "==="),
        (FatArrow, "=>"),
        (GtEq, ">="),
        (LAngle, "<"),
        (LBrack, "["),
        (LCurly, "{"),
        (LParen, "("),
        (LtEq, "<="),
        (Minus, "-"),
        (Minus2, "--"),
        (MinusEq, "-="),
        (Neq, "!="),
        (Neq2, "!=="),
        (Percent, "%"),
        (PercentEq, "%="),
        (Pipe, "|"),
        (Pipe2, "||"),
        (Pipe2Eq, "||="),
        (PipeEq, "|="),
        (Plus, "+"),
        (Plus2, "++"),
        (PlusEq, "+="),
        (Question, "?"),
        (Question2, "??"),
        (Question2Eq, "??="),
        (QuestionDot, "?."),
        (RAngle, ">"),
        (RBrack, "]"),
        (RCurly, "}"),
        (RParen, ")"),
        (Semicolon, ";"),
        (ShiftLeft, "<<"),
        (ShiftLeftEq, "<<="),
        (ShiftRight, ">>"),
        (ShiftRight3, ">>>"),
        (ShiftRight3Eq, ">>>="),
        (ShiftRightEq, ">>="),
        (Slash, "/"),
        (SlashEq, "/="),
        (Star, "*"),
        (Star2, "**"),
        (Star2Eq, "**="),
        (StarEq, "*="),
        (Tilde, "~"),
    ]
    .into_iter()
    .for_each(|(kind, s)| {
        test(kind, s);
    });
}

#[test]
fn numeric_literal() {
    [
        "0", "0789", "0.", "0E-1", "0E+1", "0e-12", "0e+12", "0e0", "0e00", "0e01", "1e1", "1e23",
        "123_456",
    ]
    .into_iter()
    .for_each(|s| test(Number(Decimal), s));
    ["0n", "1n", "123n", "1_2n"]
        .into_iter()
        .for_each(|s| test(Number(BigInt), s));
    ["0b1", "0B10", "0b0_1"]
        .into_iter()
        .for_each(|s| test(Number(Binary), s));
    ["0o1", "0O12", "0123", "0o1_2"]
        .into_iter()
        .for_each(|s| test(Number(Octal), s));
    ["0x1", "0X12", "0x1_2"]
        .into_iter()
        .for_each(|s| test(Number(Hex), s));
    ["0.123", "1.0", "1.1", "1.0e1", "1.1_2"]
        .into_iter()
        .for_each(|s| test(Number(Float), s));
}

#[test]
fn string_literal() {
    [
        r#""""#,
        r#"''"#,
        r#""12345""#,
        r#"'12345'"#,
        r#"'\\\n\r\t\b\v\f\'\"'"#,
        r#"'\u1234'"#,
        r#"'\x12'"#,
        r#"'foo \
            '"#,
        r#""\d""#,
        r#""\\""#,
    ]
    .into_iter()
    .for_each(|s| test(Str, s));
}

#[test]
fn regex() {
    [
        r#"/aa/"#,
        r#"/[0-9A-Za-z_\$(|)\[\]\/\\^]/"#,
        r#"/[//]/"#,
        r#"/[/]/"#,
        r#"/\\/"#,
    ]
    .into_iter()
    .for_each(|s| test(Regex, s));
}

#[test]
fn template_literal() {
    [r#"``"#, r#"`123`"#, r#"`\`\r`"#, r#"`\\`"#]
        .into_iter()
        .for_each(|s| test(Template, s));
}
