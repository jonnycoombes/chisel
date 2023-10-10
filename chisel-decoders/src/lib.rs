//! ## Overview
//!
//! This crate contains a very simple, lean implementations of decoders that will consume `u8` bytes from a given
//! `Read` implementation, and decode into the Rust internal `char` type using either UTF-8 or ASCII.
//!
//! The decoder implementations are pretty fast and loose: under the covers they utilise some bit-twiddlin' in
//! conjunction with the *unsafe* `transmute` function to do the conversions.
//!
//! *No string allocations are used during conversion*.
//!
//! ### Usage
//!
//! Usage is very simple, provided you have something that implements `Read` in order to source some bytes:
//!
//! ### Create from a slice
//!
//! Just wrap your array in a `mut` reader, and then plug it into a new instance of either `Utf8Decoder`:
//!
//! ```rust
//!     # use std::io::BufReader;
//!     # use chisel_decoders::utf8::Utf8Decoder;
//!
//!     let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
//!     let mut reader = BufReader::new(buffer);
//!     let _decoder = Utf8Decoder::new(&mut reader);
//! ```
//! If you're fairly certain that you're dealing with ASCII only, then just pick the `AsciiDecoder` instead:
//!
//! ```rust
//!     # use std::io::BufReader;
//!     # use chisel_decoders::ascii::AsciiDecoder;
//!
//!     let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
//!     let mut reader = BufReader::new(buffer);
//!     let _decoder = AsciiDecoder::new(&mut reader);
//! ```
//!
//! ### Create from a file
//!
//! Just crack open your file, wrap in a `Read` instance and then plug into a new instance of `Utf8Decoder`:
//!
//! ```rust
//!     # use std::fs::File;
//!     # use std::io::BufReader;
//!     # use std::path::PathBuf;
//!     # use chisel_decoders::utf8::Utf8Decoder;
//!
//!     let path = PathBuf::from("./Cargo.toml");
//!     let f = File::open(path);
//!     let mut reader = BufReader::new(f.unwrap());
//!     let _decoder = Utf8Decoder::new(&mut reader);
//! ```
//! ### Consuming Decoded `chars`
//!
//! Once you've created an instance of a specific decoder, you simply iterate over the `chars` in
//! order to pull out the decoded characters (a decoder implements `Iterator<Item=char>`):
//!
//! ```rust
//!     # use std::fs::File;
//!     # use std::io::BufReader;
//!     # use std::path::PathBuf;
//!     # use chisel_decoders::utf8::Utf8Decoder;
//!
//!     let path = PathBuf::from("./Cargo.toml");
//!     let f = File::open(path);
//!     let mut reader = BufReader::new(f.unwrap());
//!     let decoder = Utf8Decoder::new(&mut reader);
//!     for c in decoder {
//!        println!("char: {}", c)
//!     }
//! ```
//!
use crate::ascii::AsciiDecoder;
use crate::utf8::Utf8Decoder;
use std::io::BufRead;

pub mod ascii;
pub mod common;
pub mod utf8;

/// Enumeration of different supported encoding types
#[derive(Copy, Clone)]
pub enum Encoding {
    Utf8,
    Ascii,
}

/// Default encoding is UTF-8
impl Default for Encoding {
    fn default() -> Self {
        Self::Utf8
    }
}

/// Helper function for constructing a default decoder, wrapped around an input buffer
pub fn default_decoder<'a, Buffer: BufRead>(
    buffer: &'a mut Buffer,
) -> Box<dyn Iterator<Item = char> + 'a> {
    Box::new(Utf8Decoder::new(buffer))
}

/// Helper function for constructing a specific decoder, wrapped around an input buffer
pub fn new_decoder<'a, Buffer: BufRead>(
    buffer: &'a mut Buffer,
    encoding: Encoding,
) -> Box<dyn Iterator<Item = char> + 'a> {
    match encoding {
        Encoding::Ascii => Box::new(AsciiDecoder::new(buffer)),
        Encoding::Utf8 => Box::new(Utf8Decoder::new(buffer)),
    }
}

#[cfg(test)]
mod lib {

    use crate::{default_decoder, new_decoder, Encoding};
    use std::fs::File;
    use std::io::BufReader;

    fn fuzz_file() -> File {
        File::open("fixtures/fuzz.txt").unwrap()
    }

    #[test]
    fn should_create_a_default_decoder() {
        let mut reader = BufReader::new(fuzz_file());
        let decoder = default_decoder(&mut reader);
        assert!(decoder.count() > 0)
    }

    #[test]
    fn should_create_a_new_ascii_decoder() {
        let mut reader = BufReader::new(fuzz_file());
        let decoder = new_decoder(&mut reader, Encoding::Ascii);
        assert!(decoder.count() > 0)
    }

    #[test]
    fn should_create_a_new_utf8_decoder() {
        let mut reader = BufReader::new(fuzz_file());
        let decoder = new_decoder(&mut reader, Encoding::Utf8);
        assert!(decoder.count() > 0)
    }
}
