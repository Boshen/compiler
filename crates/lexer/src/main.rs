#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use lexer::Lexer;

pub fn main() {
    let tokens = Lexer::new("null").collect::<Vec<_>>();
    println!("{tokens:?}");
}
