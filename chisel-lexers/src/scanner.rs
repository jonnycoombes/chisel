#![allow(dead_code)]
use chisel_common::char::coords::Coords;
use chisel_common::char::span::Span;
use std::fmt::{Display, Formatter};

/// General result type for the scanner
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

/// Aggregate structure consisting of a character and it's position within the input stream
pub struct CharWithCoords {
    pub ch: char,
    pub coords: Coords,
}

/// A tuple consisting of a string-like thing, and associated span
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

/// Structure to manage input state information for the lexer.  Allows for an absolute position as well as a sliding
/// buffer of (as of yet) unconsumed entries
#[derive()]
pub struct Scanner<'a> {
    /// Single lookahead character
    lookahead: Option<char>,

    /// The underlying source of characters
    chars: &'a mut dyn Iterator<Item = char>,

    /// The absolute [Coords]
    position: Coords,

    /// Input buffer
    buffer: Vec<CharWithCoords>,

    /// Pushback buffer
    pushbacks: Vec<CharWithCoords>,
}

/// An input adapter used by the lexer. A [Scanner] is responsible for managing input
/// state to to provide access to segments (or individual characters) from within the source input.
impl<'a> Scanner<'a> {
    /// Create a new state instance with all the defaults
    pub fn new(chars: &'a mut dyn Iterator<Item = char>) -> Self {
        Scanner {
            lookahead: None,
            chars,
            position: Coords::default(),
            buffer: vec![],
            pushbacks: vec![],
        }
    }

    /// Reset the state without resetting the state of the underlying char iterator
    pub fn clear(&mut self) {
        self.buffer = vec![];
    }

    /// Push the last read character (and it's coords) onto the pushback buffer
    pub fn pushback(&mut self) {
        if !self.buffer.is_empty() {
            let last = self.buffer.remove(self.buffer.len() - 1);
            self.pushbacks.push(last);
        }
    }

    /// Get the absolute position in the underlying input
    pub fn position(&self) -> Coords {
        self.position.clone()
    }

    /// Get the optional [char] at the front of the buffer
    pub fn front(&self) -> Option<CharWithCoords> {
        return if !self.buffer.is_empty() {
            Some(clone_char_with_coords!(self.buffer.last().unwrap()))
        } else {
            None
        };
    }

    /// Get the optional [char] at the back of the buffer
    pub fn back(&self) -> Option<CharWithCoords> {
        return if !self.buffer.is_empty() {
            Some(clone_char_with_coords!(self.buffer.first().unwrap()))
        } else {
            None
        };
    }

    /// Advance the input to the next available character, optionally skipping whitespace.
    pub fn advance(&mut self, skip_whitespace: bool) -> ScannerResult<()> {
        // skip any whitespace, which may populate pushback or lookahead
        if skip_whitespace {
            self.skip_whitespace()?;
        }

        // check that we haven't pushed back during whitespace skipping
        if !self.pushbacks.is_empty() {
            self.buffer.push(self.pushbacks.pop().unwrap());
            self.position = self.buffer.last().unwrap().coords;
            return Ok(());
        }

        // otherwise, just grab the next available character from either the underlying input,
        // or from the lookahead buffer
        return match self.next_char() {
            Some(next) => {
                self.inc_position(false);
                match next {
                    (ch, Some(coords)) => self.buffer.push(CharWithCoords { ch, coords }),
                    (ch, None) => self.buffer.push(CharWithCoords {
                        ch,
                        coords: self.position,
                    }),
                }
                Ok(())
            }
            None => scanner_error!(ScannerErrorDetails::EndOfInput, self.position),
        };
    }

    /// Look ahead one in the input stream
    fn try_lookahead(&mut self) -> Option<char> {
        self.lookahead = self.chars.next();
        self.lookahead
    }

    /// Clear the lookahead
    fn clear_lookahead(&mut self) {
        self.lookahead = None;
    }

    /// Skip any whitespace within the input, making sure to update our overall position in the
    /// input correctly. As we skip whitespace, we may "overrun" and so need to populate the
    /// pushback buffer, or alternatively in the case of line endings, we may need to populate
    /// the lookahead with a character.  Populating the pushback buffer should update coordinate
    /// information, populating the lookahead will not
    fn skip_whitespace(&mut self) -> ScannerResult<()> {
        loop {
            let next = self.next_char();
            match next {
                Some((ch, _)) => match ch.is_whitespace() {
                    true => match ch {
                        '\r' => {
                            self.inc_position(true);
                            match self.try_lookahead() {
                                Some(la) => match la {
                                    '\n' => {
                                        self.inc_position(false);
                                        self.clear_lookahead();
                                    }
                                    _ => {
                                        self.inc_position(false);
                                        self.pushbacks.push(CharWithCoords {
                                            ch: la,
                                            coords: self.position,
                                        })
                                    }
                                },
                                None => {
                                    return scanner_error!(
                                        ScannerErrorDetails::EndOfInput,
                                        self.position
                                    );
                                }
                            }
                        }
                        '\n' => self.inc_position(true),
                        _ => self.inc_position(false),
                    },
                    false => {
                        self.inc_position(false);
                        self.pushbacks.push(CharWithCoords {
                            ch,
                            coords: self.position,
                        });
                        return Ok(());
                    }
                },
                None => {
                    return scanner_error!(ScannerErrorDetails::EndOfInput, self.position);
                }
            }
        }
    }

    /// Grab the next available character, which might come from either the lookahead, or failing
    /// that, from the underlying input.  Return a tuple consisting of a character, and an optional
    /// coordinate position, depending on where we managed to source the character from...
    fn next_char(&mut self) -> Option<(char, Option<Coords>)> {
        // check to see if we have anything in the pushback buffer
        if !self.pushbacks.is_empty() {
            let popped = self.pushbacks.pop().unwrap();
            return Some((popped.ch, Some(popped.coords)));
        }

        // grab from the lookahead, or read from the underlying input
        match self.lookahead {
            Some(ch) => {
                self.lookahead = None;
                Some((ch, None))
            }
            None => self.chars.next().map(|ch| (ch, None)),
        }
    }

    /// Advance the input over n available characters, returning a [ParserError] if it's not
    /// possible to do so. After calling this method the input state should be read using the
    /// other associated functions available for this type
    pub fn advance_n(&mut self, n: usize, skip_whitespace: bool) -> ScannerResult<()> {
        for _ in 0..n {
            self.advance(skip_whitespace)?;
        }
        Ok(())
    }

    /// Increment position, optionally resetting at the beginning of a new line
    fn inc_position(&mut self, newline: bool) {
        // check whether we're on the very first character
        if self.position.line == 0 {
            self.position.line = 1
        }

        // adjust absolute position
        self.position.absolute += 1;

        // adjust based on whether we've hit a newline
        match newline {
            true => {
                self.position.column = 0;
                self.position.line += 1;
            }
            false => {
                self.position.column += 1;
            }
        }
    }

    /// Extract the current buffer as a [StringWithSpan]. Will return an empty string if there's
    /// nothing in the buffer
    pub fn buffer_as_string_with_span(&mut self) -> StringWithSpan {
        return if !self.buffer.is_empty() {
            let mut s = String::with_capacity(self.buffer.len());
            self.buffer.iter().for_each(|cwc| s.push(cwc.ch));
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

    /// Extract the current buffer as a [char] slice
    pub fn buffer_as_char_array(&mut self) -> Vec<char> {
        return if !self.buffer.is_empty() {
            let mut arr: Vec<char> = vec![];
            self.buffer.iter().for_each(|cwc| arr.push(cwc.ch));
            arr
        } else {
            vec![]
        };
    }

    /// Extract the current buffer as a byte buffer.  You just get an empty vec if the buffer is
    /// currently empty
    pub fn buffer_as_byte_array(&self) -> Vec<u8> {
        return if !self.buffer.is_empty() {
            self.buffer.iter().map(|cwc| cwc.ch as u8).collect()
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
}
