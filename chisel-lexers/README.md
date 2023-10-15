# chisel-lexers

[![Workflow Status](https://img.shields.io/github/actions/workflow/status/jonnycoombes/chisel-core/rust.yml)](https://img.shields.io/github/actions/workflow/status/jonnycoombes/chisel-core/rust.yml)

[![crates.io](https://img.shields.io/crates/v/chisel-lexers.svg)](https://crates.io/crates/chisel-lexers)

[![crates.io](https://img.shields.io/crates/l/chisel-lexers.svg)](https://crates.io/crates/chisel-lexers)

This crate contains a lexical analysis backend for JSON-related parsers.

## Building and Testing

| What               | Command       |
|--------------------|---------------|
| Build crate        | `cargo build` |
| Test crate         | `cargo test`  |
| Run all benchmarks | `cargo bench` |

This crate comes contains two specific benchmarks which may be run in isolation:

| Description                           | Command                          |
|---------------------------------------|----------------------------------|
| JSON lexing (tokenisation) benchmarks | `cargo bench --bench json_lexer` |
| Scanning (char munching) benchmarks   | `cargo bench --bench scanner`    |

## Suggestions and Requests

If you have any suggestions, requests or even just comments relating to this crate, then please just add an issue and
I'll try and take a look when I get change.  Please feel free to fork this repo if you want to utilise/modify this code
in any of your own work.
