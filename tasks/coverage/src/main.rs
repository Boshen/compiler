#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use rayon::prelude::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    result::Result,
    time::Instant,
};
use walkdir::WalkDir;

use coverage::read_metadata;
use lexer::{Lexer, Token};

/// # Panics
/// Invalid Project Root
#[must_use]
pub fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(2)
    .unwrap()
    .to_path_buf()
}

fn main() {
    let root = project_root().join("tasks/coverage/test262/test/");
    let entries = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.path().to_string_lossy().contains("_FIXTURE"))
        .collect::<Vec<_>>();

    println!("Reading {} files.", entries.len());

    let codes = entries
        .par_iter()
        .map(|e| {
            let code = fs::read_to_string(e.path()).unwrap();
            let (code, meta) = read_metadata(code.as_str()).unwrap();
            (e.path(), code.to_string(), meta)
        })
        .collect::<Vec<_>>();

    println!("Running Lexer agains {} files.", codes.len());
    let now = Instant::now();

    let failed = codes
        .par_iter()
        .filter_map(|(path, code, _meta)| {
            let tokens = Lexer::new(code).into_iter().collect::<Vec<_>>();
            if tokens.iter().any(Token::is_unknown) {
                Some((path, code, tokens))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let duration = now.elapsed();

    failed
        .iter()
        .take(10)
        .for_each(|(path, _code, _tokens)| println!("{path:?}"));

    #[allow(clippy::cast_precision_loss)]
    let diff = ((codes.len() - failed.len()) as f64 / codes.len() as f64) * 100.0;

    println!(
        "Lexing Passed: {}/{} ({:.2}%)",
        failed.len(),
        codes.len(),
        diff
    );
    println!(
        "Time Elapased: {}.{}s",
        duration.as_secs(),
        duration.subsec_millis()
    );
}
