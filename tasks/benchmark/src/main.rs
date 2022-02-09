use benchmark::get_code;
use criterion::{black_box, Criterion, Throughput};
use pico_args::Arguments;
use std::time::Duration;

use lexer::Lexer;

pub fn main() {
    let mut args = Arguments::from_env();
    let baseline: Option<String> = args.opt_value_from_str("--save-baseline").unwrap();

    let mut criterion = Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(10, 0));

    if let Some(ref baseline) = baseline {
        criterion = criterion.save_baseline(baseline.to_string());
    }

    let mut group = criterion.benchmark_group("lexer");

    let libs = include_str!("./libs.txt").lines();

    for lib in libs {
        let (id, code) = get_code(lib).unwrap();
        let code = code.as_str();

        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_function(&id, |b| {
            b.iter(|| {
                black_box(Lexer::new(code).into_iter().count());
            });
        });
    }

    group.finish();
}
