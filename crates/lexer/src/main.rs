#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use lexer::lex;

pub fn main() {
    println!("{}", lex());
}
