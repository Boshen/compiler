#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

#[cfg(test)]
mod tests;

mod lexer;
mod token;

pub use crate::lexer::Lexer;
pub use crate::token::Token;
