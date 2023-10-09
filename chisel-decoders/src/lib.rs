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
pub mod ascii;
pub mod common;
pub mod utf8;
