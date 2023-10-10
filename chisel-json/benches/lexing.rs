use std::fs::File;
use std::io::BufReader;

use criterion::{Criterion, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};

use chisel_decoders::utf8::Utf8Decoder;
use chisel_json::lexer::lexer_core::{Lexer, Token};

macro_rules! build_lex_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/{}.json", $filename)).unwrap();
            let mut reader = BufReader::new(f);
            let mut decoder = Utf8Decoder::new(&mut reader);
            let mut lexer = Lexer::new(&mut decoder);
            loop {
                match lexer.consume() {
                    Ok(t) => {
                        if t.0 == Token::EndOfInput {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("error occurred: {:?}", err);
                    }
                }
            }
        }
    };
}

build_lex_benchmark!(canada, "canada");
build_lex_benchmark!(citm_catalog, "citm_catalog");
build_lex_benchmark!(twitter, "twitter");
build_lex_benchmark!(simple, "simple");
build_lex_benchmark!(colours, "colours");

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("lex of canada", |b| b.iter(canada));
}
fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("lex of citm_catalog", |b| b.iter(citm_catalog));
}
fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("lex of twitter", |b| b.iter(twitter));
}
fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("lex of simple", |b| b.iter(simple));
}

fn benchmark_colours(c: &mut Criterion) {
    c.bench_function("lex of colours", |b| b.iter(colours));
}
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets= benchmark_twitter, benchmark_citm_catalog, benchmark_canada, benchmark_simple, benchmark_colours
}
criterion_main!(benches);
