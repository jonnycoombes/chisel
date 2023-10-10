use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use pprof::criterion::{Output, PProfProfiler};

use chisel_json::parsers::dom::Parser;

macro_rules! build_parse_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let path = base.join(format!("fixtures/json/bench/{}.json", $filename));
            let parser = Parser::default();
            let _ = parser.parse_file(path);
        }
    };
}

build_parse_benchmark!(twitter, "twitter");
build_parse_benchmark!(canada, "canada");
build_parse_benchmark!(citm_catalog, "citm_catalog");
build_parse_benchmark!(simple, "simple");
build_parse_benchmark!(schema, "schema");
build_parse_benchmark!(colours, "colours");

fn benchmark_citm_catalog(c: &mut Criterion) {
    c.bench_function("DOM parse of citm_catalog", |b| b.iter(citm_catalog));
}

fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("DOM parse of twitter", |b| b.iter(twitter));
}

fn benchmark_canada(c: &mut Criterion) {
    c.bench_function("DOM parse of canada", |b| b.iter(canada));
}

fn benchmark_simple(c: &mut Criterion) {
    c.bench_function("DOM parse of simple", |b| b.iter(simple));
}

fn benchmark_schema(c: &mut Criterion) {
    c.bench_function("DOM parse of schema", |b| b.iter(schema));
}

fn benchmark_colours(c: &mut Criterion) {
    c.bench_function("DOM parse of colours", |b| b.iter(colours));
}

criterion_group! {
    name = dom_benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_citm_catalog, benchmark_twitter, benchmark_canada, benchmark_simple, benchmark_schema, benchmark_colours
}
criterion_main!(dom_benches);
