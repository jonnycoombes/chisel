//! Implementation of an LA(1) scanner backend.
//!
//! # Usage
//!
//! Usage of the scanner is pretty straightforward. Construct an instance based on a supplied
//! decoder (which is responsible for decoding byte streams into streams of UTF8 characters),
//! and then use the [Scanner::advance] and [Scanner::advance_n] functions to move through the
//! underlying input and populate the internal scanner buffer.
//!
//! To look into the scanner buffer, the [Scanner::front] and [Scanner::back] functions allow
//! access to the first and last elements.  To grab the entire contents of the buffer, functions
//! such as [Scanner::buffer_as_char_array] may be used.
//!
//! Once a chunk of input has been processed, the scanner state (i.e. the buffer) can be reset
//! with a call to [Scanner::clear].
//!
//! # Examples
//!
//! ```rust
//!  use std::io::BufReader;
//!  use chisel_common::reader_from_bytes;
//!  use chisel_decoders::utf8::Utf8Decoder;
//!  use chisel_lexers::scanner::Scanner;
//!
//!  // construct a new scanner instance, based on a decoded byte source
//!  let buffer: &[u8] = "let goodly sin and sunshine in".as_bytes();
//!  let mut reader = BufReader::new(buffer);
//!  let mut decoder = Utf8Decoder::new(&mut reader);
//!  let mut scanner = Scanner::new(&mut decoder);
//!  
//! // consume from the scanner...
//! let first = scanner.advance(true);
//! assert!(first.is_ok());
//! assert_eq!(scanner.front().unwrap().ch, 'l');
//! assert_eq!(scanner.front().unwrap().coords.column, 1);
//!
//! // reset the scanner state
//! scanner.clear();
//!
//! ```
#![allow(dead_code)]
use chisel_common::char::coords::Coords;
use chisel_common::char::span::Span;
use std::fmt::{Display, Formatter};

/// Result type for the scanner
pub type ScannerResult<T> = Result<T, ScannerError>;

/// An enumeration of possible faults
#[derive(Debug, Clone, PartialEq)]
pub enum ScannerErrorDetails {
    EndOfInput,
}

/// Convert specific fault codes into human-readable strings
impl Display for ScannerErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerErrorDetails::EndOfInput => write!(f, "end of input reached"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScannerError {
    /// The error code associated with the error
    pub details: ScannerErrorDetails,
    /// [Coords] providing location information relating to the error
    pub coords: Option<Coords>,
}

/// Convert a [ScannerError] into a human-readable format
impl Display for ScannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.coords {
            Some(coords) => write!(f, "details: {}, coords: {}", self.details, coords),
            None => write!(f, "details: {}", self.details),
        }
    }
}

/// Helper macro for the quick definition of a [ScannerError]
macro_rules! scanner_error {
    ($details: expr, $coords : expr) => {
        Err(ScannerError {
            details: $details,
            coords: Some($coords),
        })
    };
    ($details : expr) => {
        Err(ScannerError {
            details: $details,
            coords: None,
        })
    };
}

/// A [char] and a [Coord] providing positional information
pub struct CharWithCoords {
    pub ch: char,
    pub coords: Coords,
}

/// A [String] along with the [Span] it occupies in the input
pub struct StringWithSpan {
    pub str: String,
    pub span: Span,
}

/// Just clone a [CharWithCoords] structure
macro_rules! clone_char_with_coords {
    ($src : expr) => {
        CharWithCoords {
            ch: $src.ch,
            coords: $src.coords.clone(),
        }
    };
}

/// Shorthand for the creation of a [CharWithCoords]
macro_rules! char_with_coords {
    ($ch : expr, $col : expr, $line : expr, $abs : expr) => {
        CharWithCoords {
            ch: $ch,
            coords: Coords {
                column: $col,
                line: $line,
                absolute: $abs,
            },
        }
    };
}

/// Simple scanner which wraps itself around a source of [char]s and converts raw characters
/// into [CharWithCoords] structures. Provides a running buffer which can be used to accumulate
/// input characters, prior to extracting them for further downstream processing.
#[derive()]
pub struct Scanner<'a> {
    /// The underlying source of characters
    source: &'a mut dyn Iterator<Item = char>,

    /// Accumulation buffer
    accumulator: Vec<CharWithCoords>,

    /// Input buffer
    buffer: Vec<CharWithCoords>,

    /// Overall position
    position: Coords,

    /// Newline flag in order ensure correct position reporting
    newline: bool,
}

/// An input adapter used by the lexer. A [Scanner] is responsible for managing input
/// state to to provide access to segments (or individual characters) from within the source input.
impl<'a> Scanner<'a> {
    /// New instance, based on an [Iterator] of [char]
    pub fn new(chars: &'a mut dyn Iterator<Item = char>) -> Self {
        Scanner {
            source: chars,
            accumulator: vec![],
            buffer: vec![],
            position: Coords {
                column: 0,
                line: 1,
                absolute: 0,
            },
            newline: false,
        }
    }

    /// Reset the internal state of the scanner, without resetting the state of the underlying char iterator
    pub fn clear(&mut self) {
        self.accumulator = vec![];
    }

    /// Push the last read character (and it's coords) onto the pushback buffer. Noop if there's
    /// currently nothing in the accumulator
    pub fn pushback(&mut self) {
        if !self.accumulator.is_empty() {
            self.buffer.push(self.accumulator.pop().unwrap())
        }
    }

    /// Get the absolute position in the underlying input
    pub fn position(&self) -> Coords {
        self.position
    }

    /// Get the optional [char] at the front of the scanner buffer
    pub fn front(&self) -> Option<CharWithCoords> {
        return if !self.accumulator.is_empty() {
            Some(clone_char_with_coords!(self.accumulator.last().unwrap()))
        } else {
            None
        };
    }

    /// Get the optional [char] at the back of the scanner buffer
    pub fn back(&self) -> Option<CharWithCoords> {
        return if !self.accumulator.is_empty() {
            Some(clone_char_with_coords!(self.accumulator.first().unwrap()))
        } else {
            None
        };
    }

    /// Advance the scanner to the next available character, optionally skipping whitespace.
    pub fn advance(&mut self, skip_whitespace: bool) -> ScannerResult<()> {
        loop {
            match self.next() {
                Some(cwc) => {
                    // update overall position
                    self.position = cwc.coords;

                    // check for whitespace
                    if skip_whitespace {
                        if !cwc.ch.is_whitespace() {
                            self.accumulator.push(cwc);
                            return Ok(());
                        }
                    } else {
                        self.accumulator.push(cwc);
                        return Ok(());
                    }
                }
                None => return scanner_error!(ScannerErrorDetails::EndOfInput),
            }
        }
    }

    /// Try and look ahead one [char] in the input stream
    pub fn try_lookahead(&mut self) -> Option<&CharWithCoords> {
        return if !self.buffer.is_empty() {
            self.buffer.last()
        } else {
            match self.next() {
                Some(cwc) => {
                    self.buffer.push(cwc);
                    self.buffer.last()
                }
                None => None,
            }
        };
    }

    /// Grab the next available character and update the current position if we retrieve a new
    /// character from the underlying input
    fn next(&mut self) -> Option<CharWithCoords> {
        // early return from the buffer if possible
        return if !self.buffer.is_empty() {
            Some(self.buffer.pop().unwrap())
        } else {
            // check next character and adjust position taking into account line endings
            match self.source.next() {
                Some(ch) => match ch {
                    '\n' => {
                        self.newline = true;
                        Some(char_with_coords!(
                            ch,
                            self.position.column + 1,
                            self.position.line,
                            self.position.absolute + 1
                        ))
                    }
                    _ => {
                        if self.newline {
                            self.newline = false;
                            Some(char_with_coords!(
                                ch,
                                1,
                                self.position.line + 1,
                                self.position.absolute + 1
                            ))
                        } else {
                            Some(char_with_coords!(
                                ch,
                                self.position.column + 1,
                                self.position.line,
                                self.position.absolute + 1
                            ))
                        }
                    }
                },
                None => None,
            }
        };
    }

    /// Advance the scanner over n available characters, returning a [ScannerError] if it's not
    /// possible to do so. After calling this method the input state should be read using the
    /// other associated functions available for this type
    pub fn advance_n(&mut self, n: usize, skip_whitespace: bool) -> ScannerResult<()> {
        for _ in 0..n {
            self.advance(skip_whitespace)?;
        }
        Ok(())
    }

    /// Extract the scanner buffer as a [StringWithSpan]. Will return an empty string if there's
    /// nothing in the buffer
    pub fn buffer_as_string_with_span(&mut self) -> StringWithSpan {
        return if !self.accumulator.is_empty() {
            let mut s = String::with_capacity(self.accumulator.len());
            self.accumulator.iter().for_each(|cwc| s.push(cwc.ch));
            StringWithSpan {
                str: s,
                span: Span {
                    start: self.back().unwrap().coords,
                    end: self.front().unwrap().coords,
                },
            }
        } else {
            StringWithSpan {
                str: String::new(),
                span: Span {
                    start: self.position,
                    end: self.position,
                },
            }
        };
    }

    /// Extract the scanner buffer as a [char] slice
    pub fn buffer_as_char_array(&mut self) -> Vec<char> {
        return if !self.accumulator.is_empty() {
            let mut arr: Vec<char> = vec![];
            self.accumulator.iter().for_each(|cwc| arr.push(cwc.ch));
            arr
        } else {
            vec![]
        };
    }

    /// Extract the scanner buffer as a byte buffer.  You just get an empty vec if the buffer is
    /// currently empty
    pub fn buffer_as_byte_array(&self) -> Vec<u8> {
        return if !self.accumulator.is_empty() {
            self.accumulator.iter().map(|cwc| cwc.ch as u8).collect()
        } else {
            vec![]
        };
    }
}

#[cfg(test)]
mod test {
    use crate::scanner::Scanner;
    use chisel_common::reader_from_bytes;
    use chisel_decoders::utf8::Utf8Decoder;
    use std::io::BufReader;

    #[test]
    fn should_create_new() {
        let mut reader = reader_from_bytes!("{}[],:");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let _ = Scanner::new(&mut decoder);
    }

    #[test]
    fn should_consume_single_lines_correctly() {
        let mut reader = reader_from_bytes!("this is a test line");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut input = Scanner::new(&mut decoder);
        let result = input.advance(true);
        assert!(result.is_ok());
        assert_eq!(input.front().unwrap().ch, 't');
        for _ in 1..5 {
            let result = input.advance(true);
            assert!(result.is_ok());
        }
        assert_eq!(input.front().unwrap().ch, 'i');
        assert_eq!(input.front().unwrap().coords.column, 6);

        input.clear();
        for _ in 1..5 {
            let result = input.advance(false);
            assert!(result.is_ok());
        }
        assert_eq!(input.front().unwrap().ch, ' ');
        assert_eq!(input.front().unwrap().coords.column, 10)
    }

    #[test]
    fn should_handle_pushbacks_correctly() {
        // construct a new scanner instance, based on a decoded byte source
        let buffer: &[u8] = "let goodly sin and sunshine in".as_bytes();
        let mut reader = BufReader::new(buffer);
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut scanner = Scanner::new(&mut decoder);

        // consume the first character from the scanner...
        let first = scanner.advance(true);
        assert!(first.is_ok());
        assert_eq!(scanner.front().unwrap().ch, 'l');
        assert_eq!(scanner.front().unwrap().coords.column, 1);

        // consume a second character
        assert!(scanner.advance(true).is_ok());

        // ...and then pushback onto the buffer
        scanner.pushback();

        // front of the buffer should still be 'l'
        assert_eq!(scanner.front().unwrap().ch, 'l');

        // advance again - this time char will be taken from the pushback buffer
        let _ = scanner.advance(true);
        assert_eq!(scanner.front().unwrap().ch, 'e');

        // grab the contents of the buffer as a string
        let buffer_contents = scanner.buffer_as_string_with_span();
        assert_eq!(buffer_contents.str, String::from("le"));

        // reset the scanner and empty the buffer
        scanner.clear();

        // buffer should now be empty
        assert!(scanner.buffer_as_string_with_span().str.is_empty());

        // advance yet again
        assert!(scanner.advance(true).is_ok());

        // the third character read will be from the 3rd column in the input
        assert_eq!(scanner.front().unwrap().ch, 't');
        assert_eq!(scanner.front().unwrap().coords.column, 3);
    }
}
