#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod tests;

#[must_use]
pub fn lex() -> String {
    "hello lexer!".to_string()
}
