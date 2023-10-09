#![allow(dead_code)]
#![allow(clippy::transmute_int_to_char)]
//! A character-oriented decoder implementation that will take an underlying [std::u8] (byte) source
//! and produce a stream of decoded Unicode (UTF-8) characters
use std::io::BufRead;
use std::mem::transmute;

use crate::common::*;
use crate::utf8::SequenceType::Unrecognised;
use crate::{decoder_error, invalid_byte_sequence};

enum SequenceType {
    Single,
    Pair,
    Triple,
    Quad,
    Unrecognised,
}

/// Mask for extracting 7 bits from a single byte sequence
const SINGLE_BYTE_MASK: u32 = 0b0111_1111;
/// Mask for extracting initial 5 bits within a double byte UTF-8 sequence
const DOUBLE_BYTE_MASK: u32 = 0b0001_1111;
/// Mask for extracting initial 4 bits within a triple byte UTF-8 ssequence
const TRIPLE_BYTE_MASK: u32 = 0b0000_1111;
/// Mask for extracting initial 3 bits within a quad byte UTF-8 ssequence
const QUAD_BYTE_MASK: u32 = 0b0000_0111;
/// Mask for extracting 6 bits from following byte UTF-8 ssequences
const FOLLOWING_BYTE_MASK: u32 = 0b0011_1111;

/// Low bound for checking excluded triples
const TRIPLE_EXCLUDED_LOW_BOUND: u32 = 0xd800;

/// High bound for checking excluded triples
const TRIPLE_EXCLUDED_HIGH_BOUND: u32 = 0xdfff;

/// High bound for checking quads
const QUAD_HIGH_BOUND: u32 = 0x10ffff;

/// Convenience macro for some bit twiddlin'
macro_rules! single_byte_sequence {
    ($byte : expr) => {
        $byte >> 7 == 0
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! double_byte_sequence {
    ($byte : expr) => {
        $byte >> 5 == 0b0000_0110
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! triple_byte_sequence {
    ($byte : expr) => {
        $byte >> 4 == 0b0000_1110
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! quad_byte_sequence {
    ($byte : expr) => {
        $byte >> 3 == 0b0001_1110
    };
}

macro_rules! decode_pair {
    ($buf : expr) => {
        ($buf[1] as u32 & FOLLOWING_BYTE_MASK) | (($buf[0] as u32 & DOUBLE_BYTE_MASK) << 6)
    };
}

macro_rules! decode_triple {
    ($buf : expr) => {
        ($buf[2] as u32 & FOLLOWING_BYTE_MASK)
            | (($buf[1] as u32 & FOLLOWING_BYTE_MASK) << 6)
            | (($buf[0] as u32 & TRIPLE_BYTE_MASK) << 12)
    };
}

macro_rules! decode_quad {
    ($buf : expr) => {
        ($buf[3] as u32 & FOLLOWING_BYTE_MASK)
            | (($buf[2] as u32 & FOLLOWING_BYTE_MASK) << 6)
            | (($buf[1] as u32 & FOLLOWING_BYTE_MASK) << 12)
            | (($buf[0] as u32 & QUAD_BYTE_MASK) << 18)
    };
}

/// Determine what kind of UTF-8 sequence we're dealing with
#[inline]
fn sequence_type(b: u8) -> SequenceType {
    if single_byte_sequence!(b) {
        return SequenceType::Single;
    }
    if triple_byte_sequence!(b) {
        return SequenceType::Triple;
    }
    if double_byte_sequence!(b) {
        return SequenceType::Pair;
    }
    if quad_byte_sequence!(b) {
        return SequenceType::Quad;
    }
    Unrecognised
}

/// A UTF-8 decoder, which takes a ref to a [BufRead] instance.
pub struct Utf8Decoder<'a, B: BufRead> {
    /// The input stream
    input: &'a mut B,
    /// Staging buffer
    buffer: Vec<u8>,
    init: bool,
    index: usize,
}

impl<'a, Buffer: BufRead> Utf8Decoder<'a, Buffer> {
    /// Create a new decoder with a default buffer size
    pub fn new(r: &'a mut Buffer) -> Self {
        Utf8Decoder {
            input: r,
            buffer: vec![],
            init: false,
            index: 0,
        }
    }

    fn init(&mut self) -> DecoderResult<()> {
        match self.input.read_to_end(&mut self.buffer) {
            Ok(_) => {
                self.init = true;
                Ok(())
            }
            Err(_) => Err(decoder_error!(
                DecoderErrorCode::StreamFailure,
                "failed to read input"
            )),
        }
    }

    /// Attempt to decode the next character in the underlying stream. Assumes the maximum
    /// number of unicode bytes is 4 *not* 6
    fn decode_next(&mut self) -> DecoderResult<char> {
        if !self.init {
            self.init()?;
        }

        if self.index >= self.buffer.len() {
            return Err(decoder_error!(
                DecoderErrorCode::EndOfInput,
                "end of input reached"
            ));
        }

        match sequence_type(self.buffer[self.index]) {
            SequenceType::Single => unsafe {
                self.index += 1;
                Ok(transmute(self.buffer[self.index - 1] as u32))
            },
            SequenceType::Pair => unsafe {
                self.index += 2;
                Ok(transmute(decode_pair!(
                    &self.buffer[self.index - 2..self.index]
                )))
            },
            SequenceType::Triple => unsafe {
                self.index += 3;
                let value = decode_triple!(&self.buffer[self.index - 3..self.index]);
                if (TRIPLE_EXCLUDED_LOW_BOUND..=TRIPLE_EXCLUDED_HIGH_BOUND).contains(&value) {
                    Err(decoder_error!(
                        DecoderErrorCode::OutOfRange,
                        "value falls within forbidden range [0xd800, 0xdfff]"
                    ))
                } else {
                    Ok(transmute(value))
                }
            },
            SequenceType::Quad => unsafe {
                self.index += 4;
                let value = decode_quad!(&self.buffer[self.index - 4..self.index]);
                if value > QUAD_HIGH_BOUND {
                    Err(decoder_error!(
                        DecoderErrorCode::OutOfRange,
                        "value falls outside maximum bound 0x10ffff"
                    ))
                } else {
                    Ok(transmute(value))
                }
            },
            Unrecognised => {
                invalid_byte_sequence!()
            }
        }
    }
}

impl<'a, B: BufRead> Iterator for Utf8Decoder<'a, B> {
    type Item = char;
    /// Decode the next character from the underlying stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.decode_next() {
            Ok(c) => Some(c),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    use crate::utf8::Utf8Decoder;

    fn fuzz_file() -> File {
        File::open("fixtures/fuzz.txt").unwrap()
    }

    fn complex_file() -> File {
        File::open("fixtures/json/bench/utf8/twitter.json").unwrap()
    }

    #[test]
    fn can_create_from_array() {
        let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
        let mut reader = BufReader::new(buffer);
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut _count = 0;
        while decoder.decode_next().is_ok() {
            _count += 1;
        }
    }

    #[test]
    fn can_create_from_file() {
        let mut reader = BufReader::new(fuzz_file());
        let _decoder = Utf8Decoder::new(&mut reader);
    }

    #[test]
    fn pass_a_fuzz_test() {
        let start = Instant::now();
        let mut reader = BufReader::new(fuzz_file());
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut count = 0;
        while decoder.decode_next().is_ok() {
            count += 1;
        }
        assert_eq!(count, 35283);
        println!("Decoded fuzz file in {:?}", start.elapsed());
    }

    #[test]
    fn decode_a_complex_document() {
        let mut reader = BufReader::new(complex_file());
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut count = 0;
        while decoder.decode_next().is_ok() {
            count += 1;
        }
        assert_eq!(count, 567916);
    }

    #[test]
    fn should_be_an_iterator() {
        let start = Instant::now();
        let mut reader = BufReader::new(fuzz_file());
        let decoder = Utf8Decoder::new(&mut reader);
        assert_eq!(decoder.count(), 35283);
        println!("Counted fuzz file in {:?}", start.elapsed());
    }
}
