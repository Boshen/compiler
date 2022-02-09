#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use pico_args::Arguments;
use std::fs::read_to_string;

use lexer::Lexer;

/// # Errors
/// # Panics
pub fn main() -> Result<(), pico_args::Error> {
    let mut args = Arguments::from_env();

    let path = args.free_from_str::<String>()?;

    let code = read_to_string(&path).unwrap();

    let tokens = Lexer::new(code.as_str()).into_iter().collect::<Vec<_>>();

    println!("{tokens:?}");

    Ok(())
}
