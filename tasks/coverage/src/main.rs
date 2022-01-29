#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
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
    let entries = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_string_lossy().to_string())
        .filter(|path| !path.contains("_FIXTURE"))
        .collect::<Vec<_>>();

    println!("Reading {} files.", entries.len());

    let codes = entries
        .par_iter()
        // TODO: unicode
        .filter(|path| !path.contains("unicode"))
        .map(|path| {
            let code = fs::read_to_string(path).unwrap();
            (
                path.strip_prefix(root.clone().as_path().to_str().unwrap())
                    .unwrap(),
                code,
            )
        })
        .collect::<Vec<_>>();

    let mut files = SimpleFiles::new();
    let mut lexers = Vec::with_capacity(codes.len());
    for (path, code) in &codes {
        let (code, meta) = read_metadata(code.as_str()).unwrap();
        // TODO: re-enable negative tests
        if meta.negative.is_none() {
            let file_id = files.add(path, code);
            lexers.push((file_id, Lexer::new(code)));
        }
    }

    println!("Running Lexer ...");

    let now = Instant::now();

    let failed = lexers
        .into_par_iter()
        .filter_map(|(file_id, lexer)| {
            lexer
                .into_iter()
                .find(Token::is_unknown)
                .map(|token| (file_id, token))
        })
        .collect::<Vec<_>>();

    let duration = now.elapsed();

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    failed.iter().take(5).for_each(|(file_id, token)| {
        let diagnostic = Diagnostic::error()
            .with_message("Unknown Token")
            .with_labels(vec![Label::primary(*file_id, token.range())]);
        term::emit(&mut writer.lock(), &config, &files, &diagnostic).ok();
    });

    let passed = codes.len() - failed.len();
    #[allow(clippy::cast_precision_loss)]
    let diff = (passed as f64 / codes.len() as f64) * 100.0;

    println!("Lexing Passed: {}/{} ({:.2}%)", passed, codes.len(), diff);
    println!(
        "Time Elapased: {}.{}s",
        duration.as_secs(),
        duration.subsec_millis()
    );
}
