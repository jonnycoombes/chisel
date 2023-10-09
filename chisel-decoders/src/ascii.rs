#![allow(dead_code)]
#![allow(clippy::transmute_int_to_char)]
//! A character-oriented decoder implementation that will take an underlying [std::u8] (byte) source
//! and produce a stream of decoded ASCII characters
use std::io::BufRead;
use std::mem::transmute;

use crate::common::*;
use crate::decoder_error;

/// An ASCIIdecoder, which takes a ref to a [BufRead] instance.
pub struct AsciiDecoder<'a, B: BufRead> {
    /// The input stream
    input: &'a mut B,
    /// Staging buffer
    buffer: Vec<u8>,
    /// Initialisation flag
    init: bool,
    /// The current index into the input
    index: usize,
}

impl<'a, Buffer: BufRead> AsciiDecoder<'a, Buffer> {
    /// Create a new decoder with a default buffer size
    pub fn new(r: &'a mut Buffer) -> Self {
        AsciiDecoder {
            input: r,
            buffer: vec![],
            init: false,
            index: 0,
        }
    }

    /// Initialise and read the input into an internal buffer for decoding
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

    /// Attempt to decode the next character in the underlying stream.
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

        if self.buffer[self.index] >> 7 == 0 {
            return unsafe {
                self.index += 1;
                Ok(transmute(self.buffer[self.index - 1] as u32))
            };
        } else {
            return Err(decoder_error!(
                DecoderErrorCode::OutOfRange,
                "non-ascii character detected"
            ));
        }
    }
}

impl<'a, B: BufRead> Iterator for AsciiDecoder<'a, B> {
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

    use crate::ascii::AsciiDecoder;
    use crate::common::DecoderErrorCode;

    fn utf8_fuzz_file() -> File {
        File::open("fixtures/fuzz.txt").unwrap()
    }
    fn ascii_fuzz_file() -> File {
        File::open("fixtures/json/bench/ascii/asciiart.json").unwrap()
    }
    fn complex_file() -> File {
        File::open("fixtures/json/bench/utf8/twitter.json").unwrap()
    }

    #[test]
    fn can_create_from_array() {
        let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
        let mut reader = BufReader::new(buffer);
        let mut decoder = AsciiDecoder::new(&mut reader);
        let mut _count = 0;
        while decoder.decode_next().is_ok() {
            _count += 1;
        }
    }

    #[test]
    fn can_create_from_file() {
        let mut reader = BufReader::new(utf8_fuzz_file());
        let _decoder = AsciiDecoder::new(&mut reader);
    }

    #[test]
    fn should_out_of_range_utf8() {
        let mut reader = BufReader::new(utf8_fuzz_file());
        let mut decoder = AsciiDecoder::new(&mut reader);
        loop {
            match decoder.decode_next() {
                Ok(_) => (),
                Err(e) => {
                    assert_eq!(e.code, DecoderErrorCode::OutOfRange);
                    break;
                }
            }
        }
    }

    #[test]
    fn should_pass_an_ascii_fuzz_test() {
        let mut reader = BufReader::new(ascii_fuzz_file());
        let mut decoder = AsciiDecoder::new(&mut reader);
        let mut count = 0;
        while decoder.decode_next().is_ok() {
            count += 1;
        }
        assert_eq!(count, 6406307);
    }

    #[test]
    fn should_be_an_iterator() {
        let start = Instant::now();
        let mut reader = BufReader::new(ascii_fuzz_file());
        let decoder = AsciiDecoder::new(&mut reader);
        assert_eq!(decoder.count(), 6406307);
        println!("Counted fuzz file in {:?}", start.elapsed());
    }
}
