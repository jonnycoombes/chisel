use chisel_decoders::ascii::AsciiDecoder;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use std::fs::File;
use std::io::BufReader;

macro_rules! build_decode_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/json/bench/ascii/{}.json", $filename)).unwrap();
            let mut reader = BufReader::new(f);
            let mut decoder = AsciiDecoder::new(&mut reader);
            let mut _count = 0;
            while let Some(_) = decoder.next() {
                _count += 1;
            }
        }
    };
}

build_decode_benchmark!(asciiart, "asciiart");
build_decode_benchmark!(simple, "simple");

fn benchmark_asciiart(c: &mut Criterion) {
    c.bench_function("(Ascii) iter decode asciiart.json file", |b| {
        b.iter(asciiart)
    });
}

fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("(Ascii) iter decode simple.json file", |b| b.iter(simple));
}

criterion_group! {
    name=ascii_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets= benchmark_asciiart, benchmark_simple
}

criterion_main!(ascii_benches);
