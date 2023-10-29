use chisel_parsers::json::sax::Parser;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use std::path::PathBuf;
macro_rules! build_parse_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let path = base.join(format!("fixtures/json/bench/{}.json", $filename));
            let parser = Parser::default();
            let _ = parser.parse_file(path, &mut |_evt| Ok(()));
        }
    };
}

build_parse_benchmark!(twitter, "twitter");
build_parse_benchmark!(canada, "canada");
build_parse_benchmark!(citm_catalog, "citm_catalog");
build_parse_benchmark!(simple, "simple");
build_parse_benchmark!(schema, "schema");
build_parse_benchmark!(colours, "colours");
build_parse_benchmark!(ms_formatted, "ms-formatted");
build_parse_benchmark!(ms_minified, "ms-minified");

fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("SAX parse of citm_catalog", |b| b.iter(citm_catalog));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("SAX parse of twitter", |b| b.iter(twitter));
}

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("SAX parse of canada", |b| b.iter(canada));
}

fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("SAX parse of simple", |b| b.iter(simple));
}

fn benchmark_schema(c: &mut Criterion) {
    c.bench_function("SAX parse of schema", |b| b.iter(schema));
}
fn benchmark_colours(c: &mut Criterion) {
    c.bench_function("SAX parse of colours", |b| b.iter(colours));
}

fn benchmark_ms_formatted(c: &mut Criterion) {
    c.bench_function("SAX parse of MS formatted (5Mb)", |b| b.iter(ms_formatted));
}
fn benchmark_ms_minified(c: &mut Criterion) {
    c.bench_function("SAX parse of MS minified (5Mb)", |b| b.iter(ms_minified));
}

criterion_group! {
    name = sax_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_citm_catalog,
    benchmark_twitter,
    benchmark_canada,
    benchmark_simple,
    benchmark_schema,
    benchmark_colours,
    benchmark_ms_formatted,
    benchmark_ms_minified
}
criterion_main!(sax_benches);
