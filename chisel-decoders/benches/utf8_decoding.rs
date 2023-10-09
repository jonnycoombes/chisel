use chisel_decoders::utf8::Utf8Decoder;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_decode_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/utf8/{}.json", $filename)).unwrap();
            let mut reader = BufReader::new(f);
            let mut decoder = Utf8Decoder::new(&mut reader);
            let mut _count = 0;
            while let Some(_) = decoder.next() {
                _count += 1;
            }
        }
    };
}

build_decode_benchmark!(canada, "canada");
build_decode_benchmark!(twitter, "twitter");
build_decode_benchmark!(citm_catalog, "citm_catalog");
build_decode_benchmark!(asciiart, "asciiart");

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("(Utf8) iter decode canada.json file", |b| b.iter(canada));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("(Utf8) iter decode twitter.json file", |b| b.iter(twitter));
}

fn benchmark_asciiart(c: &mut Criterion) {
    c.bench_function("(Utf8) iter decode asciiart.json file", |b| {
        b.iter(asciiart)
    });
}

fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("(Utf8) iter decode citm_catalog.json file", |b| {
        b.iter(citm_catalog)
    });
}
criterion_group! {
    name=utf8_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets= benchmark_twitter, benchmark_canada, benchmark_citm_catalog, benchmark_asciiart
}

criterion_main!(utf8_benches);
