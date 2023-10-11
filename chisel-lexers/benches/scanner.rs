use std::fs::File;
use std::io::BufReader;

use chisel_decoders::ascii::AsciiDecoder;
use chisel_decoders::utf8::Utf8Decoder;
use chisel_lexers::scanner::Scanner;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

macro_rules! build_utf8_scanner_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/{}.json", $filename)).unwrap();
            let mut reader = BufReader::new(f);
            let mut decoder = Utf8Decoder::new(&mut reader);
            let mut scanner = Scanner::new(&mut decoder);
            loop {
                if scanner.advance(true).is_err() {
                    break;
                }
            }
        }
    };
}

macro_rules! build_ascii_scanner_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/{}.json", $filename)).unwrap();
            let mut reader = BufReader::new(f);
            let mut decoder = AsciiDecoder::new(&mut reader);
            let mut scanner = Scanner::new(&mut decoder);
            loop {
                if scanner.advance(true).is_err() {
                    break;
                }
            }
        }
    };
}

build_utf8_scanner_benchmark!(canada_utf8, "canada");
build_utf8_scanner_benchmark!(citm_catalog_utf8, "citm_catalog");
build_utf8_scanner_benchmark!(twitter_utf8, "twitter");
build_utf8_scanner_benchmark!(simple_utf8, "simple");
build_utf8_scanner_benchmark!(colours_utf8, "colours");

build_ascii_scanner_benchmark!(canada_ascii, "canada");
build_ascii_scanner_benchmark!(citm_catalog_ascii, "citm_catalog");
build_ascii_scanner_benchmark!(simple_ascii, "simple");
build_ascii_scanner_benchmark!(colours_ascii, "colours");

fn benchmark_canada_utf8(c: &mut Criterion) {
    c.bench_function("UTF-8 scan of canada", |b| b.iter(canada_utf8));
}
fn benchmark_citm_catalog_utf8(c: &mut Criterion) {
    c.bench_function("UTF-8 scan of citm_catalog", |b| b.iter(citm_catalog_utf8));
}
fn benchmark_twitter_utf8(c: &mut Criterion) {
    c.bench_function("UTF-8 scan of twitter", |b| b.iter(twitter_utf8));
}
fn benchmark_simple_utf8(c: &mut Criterion) {
    c.bench_function("UTF-8 scan of simple", |b| b.iter(simple_utf8));
}

fn benchmark_colours_utf8(c: &mut Criterion) {
    c.bench_function("UTF-8 scan of colours", |b| b.iter(colours_utf8));
}

fn benchmark_canada_ascii(c: &mut Criterion) {
    c.bench_function("ASCII scan of canada", |b| b.iter(canada_ascii));
}
fn benchmark_citm_catalog_ascii(c: &mut Criterion) {
    c.bench_function("ASCII scan of citm_catalog", |b| b.iter(citm_catalog_ascii));
}
fn benchmark_simple_ascii(c: &mut Criterion) {
    c.bench_function("ASCII scan of simple", |b| b.iter(simple_ascii));
}

fn benchmark_colours_ascii(c: &mut Criterion) {
    c.bench_function("ASCII scan of colours", |b| b.iter(colours_ascii));
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets= benchmark_twitter_utf8,
    benchmark_citm_catalog_utf8,
    benchmark_citm_catalog_ascii,
    benchmark_canada_utf8,
    benchmark_canada_ascii,
    benchmark_simple_utf8,
    benchmark_simple_ascii,
    benchmark_colours_utf8,
    benchmark_colours_ascii,
}
criterion_main!(benches);
