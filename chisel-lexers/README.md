# chisel-lexers

[![Workflow Status](https://img.shields.io/github/actions/workflow/status/jonnycoombes/chisel-core/rust.yml)](https://img.shields.io/github/actions/workflow/status/jonnycoombes/chisel-core/rust.yml)

[![crates.io](https://img.shields.io/crates/v/chisel-lexers.svg)](https://crates.io/crates/chisel-lexers)

[![crates.io](https://img.shields.io/crates/l/chisel-lexers.svg)](https://crates.io/crates/chisel-lexers)

## Overview
This crate contains the lexical analysis backends used throughout chisel. The basic architecture
is very simple, the only thing to note being that the scanning is handled separately to the
lexing (production of tokens) by a common scanner implementation.

Each lexer consumes from a *scanner* instance, and attempts to construct *tokens* which may be
consumed by *parsers* further up the stack.

Each lexer defines, and is capable of producing, its own set of distinct tokens
specific to the parsing task in hand.  (For example, the JSON lexer produces JSON-specific
tokens only).

### Scanning the input
The scanner operates through maintaining an internal state:

- An transient buffer of characters, paired with their coordinates in the input
- A transient pushback buffer, which can be used to temporarily push characters back into the input
- A single lookahead character which can be used for LA(1) operations by a lexer
- A current position in the input
- A source of `char`s (typically a byte decoder of some description)

At any point, a lexer can take a look at the front and the back of the buffer in order to decide
whether to either *advance*, *pushback* or *lookahead*:

- *advance* add another character to the scanning buffer
- *pushback* take the last read character from the buffer and add to the pushback buffer
- *lookahead* read one character from the input, but don't add it to the buffer (yet)

If a fault occurs during any of these operations, the scanner returns a `Err` containing
information about the nature of the fault and the location in the input where it occurred.

After several advances, pushbacks lookaheads, a lexer can retrieve the contents of the buffer
in order to form a new token.

### Lexers

#### JSON Lexer



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
