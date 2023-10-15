//!
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(dead_code)]

use std::fmt::{Display, Formatter};

use chisel_common::char::coords::Coords;
use chisel_common::char::span::Span;

use crate::json::tokens::{PackedToken, Token};
use crate::scanner::{CharWithCoords, Scanner};

/// JSON lexer backend result type
pub type LexerResult<T> = Result<T, LexerError>;

/// A global enumeration of error codes
#[derive(Debug, Clone, PartialEq)]
pub enum LexerErrorDetails {
    /// An invalid file has been specified.  It might not exist, or might not be accessible
    InvalidFile,
    /// We can't parse nothing.
    ZeroLengthInput,
    /// End of input has been reached. This is used as a stopping condition at various points.
    EndOfInput,
    /// If pulling bytes from an underlying stream (or [BufRead]) of some description, and an
    /// error occurs, this will be returned.
    StreamFailure,
    /// Dodgy UTF8 has been found in the input.
    NonUtf8InputDetected,
    /// Edge case error condition. This means that something has gone horribly wrong with the
    /// parse.
    UnexpectedToken(Token),
    /// KV pair is expected but not detected.
    PairExpected,
    /// Supplied JSON doesn't have an object or array as a root object.
    InvalidRootObject,
    /// The parse of an object has failed.
    InvalidObject,
    /// The parse of an array has failed.
    InvalidArray,
    /// An invalid character has been detected within the input.
    InvalidCharacter(char),
    /// Whilst looking for a literal string token (null, true, false) a match couldn't be found
    MatchFailed(String, String),
    /// A number has been found with an incorrect string representation.
    InvalidNumericRepresentation(String),
    /// An invalid escape sequence has been found within the input.
    InvalidEscapeSequence(String),
    /// An invalid unicode escape sequence (\uXXX) has been found within the input.
    InvalidUnicodeEscapeSequence(String),
}

impl Display for LexerErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorDetails::InvalidFile => write!(f, "invalid file specified"),
            LexerErrorDetails::ZeroLengthInput => write!(f, "zero length input"),
            LexerErrorDetails::EndOfInput => write!(f, "end of input reached"),
            LexerErrorDetails::StreamFailure => write!(f, "failure in the underlying stream"),
            LexerErrorDetails::NonUtf8InputDetected => write!(f, "non-UTF8 input"),
            LexerErrorDetails::UnexpectedToken(token) => {
                write!(f, "unexpected token found: {}", token)
            }
            LexerErrorDetails::PairExpected => {
                write!(f, "pair expected, something else was found")
            }
            LexerErrorDetails::InvalidRootObject => write!(f, "invalid JSON"),
            LexerErrorDetails::InvalidObject => write!(f, "invalid object"),
            LexerErrorDetails::InvalidArray => write!(f, "invalid array"),
            LexerErrorDetails::InvalidCharacter(ch) => write!(f, "invalid character: \'{}\'", ch),
            LexerErrorDetails::MatchFailed(first, second) => write!(
                f,
                "a match failed. Looking for \"{}\", found \"{}\"",
                first, second
            ),
            LexerErrorDetails::InvalidNumericRepresentation(repr) => {
                write!(f, "invalid number representation: \"{}\"", repr)
            }
            LexerErrorDetails::InvalidEscapeSequence(seq) => {
                write!(f, "invalid escape sequence: \"{}\"", seq)
            }
            LexerErrorDetails::InvalidUnicodeEscapeSequence(seq) => {
                write!(f, "invalid unicode escape sequence: \"{}\"", seq)
            }
        }
    }
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct LexerError {
    /// The global error code for the error
    pub details: LexerErrorDetails,
    /// Parser [Coords]
    pub coords: Option<Coords>,
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.coords.is_some() {
            write!(
                f,
                "details: {}, coords: {}",
                self.details,
                self.coords.unwrap()
            )
        } else {
            write!(f, "details: {}", self.details)
        }
    }
}

/// Wrap a [LexerError] in a top level [Err]
macro_rules! wrapped_lexer_error {
    ($details: expr, $coords : expr) => {
        Err(LexerError {
            details: $details,
            coords: Some($coords),
        })
    };
    ($details: expr) => {
        Err(LexerError {
            details: $details,
            coords: None,
        })
    };
}

/// Create a [LexerError]
macro_rules! lexer_error {
    ($details: expr, $coords: expr) => {
        LexerError {
            details: $details,
            coords: Some($coords),
        }
    };
    ($details: expr) => {
        LexerError {
            details: $details,
            coords: None,
        }
    };
}

/// Default lookahead buffer size
const DEFAULT_BUFFER_SIZE: usize = 4096;
/// Pattern to match for null
const NULL_PATTERN: [char; 4] = ['n', 'u', 'l', 'l'];
/// Pattern to match for true
const TRUE_PATTERN: [char; 4] = ['t', 'r', 'u', 'e'];
/// Pattern to match for false
const FALSE_PATTERN: [char; 5] = ['f', 'a', 'l', 's', 'e'];

macro_rules! packed_token {
    ($t:expr, $s:expr, $e:expr) => {
        Ok(($t, Span { start: $s, end: $e }))
    };
    ($t:expr, $s:expr) => {
        Ok(($t, Span { start: $s, end: $s }))
    };
}

/// Pattern matching macro
macro_rules! match_zero {
    () => {
        '0'
    };
}

/// Pattern matching macro
macro_rules! match_minus {
    () => {
        '-'
    };
}

/// Pattern matching macro
macro_rules! match_plus_minus {
    () => {
        '+' | '-'
    };
}

/// Pattern matching macro
macro_rules! match_digit {
    () => {
        '0'..='9'
    };
}

/// Pattern matching macro
macro_rules! match_non_zero_digit {
    () => {
        '1'..='9'
    };
}

/// Pattern matching macro
macro_rules! match_exponent {
    () => {
        'e' | 'E'
    };
}

/// Pattern matching macro
macro_rules! match_period {
    () => {
        '.'
    };
}

/// Pattern matching macro
macro_rules! match_numeric_terminator {
    () => {
        ']' | '}' | ','
    };
}

/// Pattern matching macro
macro_rules! match_escape {
    () => {
        '\\'
    };
}

/// Pattern matching macro
macro_rules! match_escape_non_unicode_suffix {
    () => {
        'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' | '\"'
    };
}

/// Pattern matching macro
macro_rules! match_escape_unicode_suffix {
    () => {
        'u'
    };
}

/// Pattern matching macro
macro_rules! match_quote {
    () => {
        '\"'
    };
}

/// Pattern matching macro
macro_rules! match_newline {
    () => {
        '\n'
    };
}

pub struct Lexer<'a> {
    /// Input coordinate state
    input: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a mut impl Iterator<Item = char>) -> Self {
        Lexer {
            input: Scanner::new(chars),
        }
    }

    /// Get the front of the input
    fn front(&self) -> Option<CharWithCoords> {
        self.input.front()
    }

    /// Get the back of the input
    fn back(&self) -> Option<CharWithCoords> {
        self.input.back()
    }

    /// Grab the front character
    fn front_char(&self) -> char {
        self.input.front().unwrap().ch
    }

    /// Grab the back character
    fn back_char(&self) -> char {
        self.input.back().unwrap().ch
    }

    /// Grab the front input coordinates
    fn front_coords(&self) -> Coords {
        self.input.front().unwrap().coords
    }

    /// Grab the back input coordinates
    fn back_coords(&self) -> Coords {
        self.input.back().unwrap().coords
    }

    /// Grab the current absolute input coordinates
    fn absolute_position(&self) -> Coords {
        self.input.position()
    }

    /// Advance the input by one
    fn advance(&mut self, skip_whitespace: bool) -> LexerResult<()> {
        self.input
            .advance(skip_whitespace)
            .map_err(|e| lexer_error!(LexerErrorDetails::EndOfInput))
    }

    /// Advance the input by n
    fn advance_n(&mut self, n: usize, skip_whitespace: bool) -> LexerResult<()> {
        self.input
            .advance_n(n, skip_whitespace)
            .map_err(|e| lexer_error!(LexerErrorDetails::EndOfInput))
    }

    /// Grab the current input string
    fn current_string(&mut self) -> String {
        self.input.buffer_as_string_with_span().str
    }

    /// Grab the current input character array
    fn current_chars(&mut self) -> Vec<char> {
        self.input.buffer_as_char_array()
    }

    /// Grab the current input byte array
    fn current_bytes(&mut self) -> Vec<u8> {
        self.input.buffer_as_byte_array()
    }

    /// Consume the next [Token] from the input
    pub fn consume(&mut self) -> LexerResult<PackedToken> {
        self.input.clear();
        match self.advance(true) {
            Ok(_) => match self.input.front() {
                Some(CharWithCoords { ch: '{', coords }) => {
                    packed_token!(Token::StartObject, coords)
                }
                Some(CharWithCoords { ch: '}', coords }) => packed_token!(Token::EndObject, coords),
                Some(CharWithCoords { ch: '[', coords }) => {
                    packed_token!(Token::StartArray, coords)
                }
                Some(CharWithCoords { ch: ']', coords }) => packed_token!(Token::EndArray, coords),
                Some(CharWithCoords { ch: ':', coords }) => packed_token!(Token::Colon, coords),
                Some(CharWithCoords { ch: ',', coords }) => packed_token!(Token::Comma, coords),
                Some(CharWithCoords { ch: '\"', coords }) => self.match_string(),
                Some(CharWithCoords { ch: 'n', coords }) => self.match_null(),
                Some(CharWithCoords { ch: 't', coords }) => self.match_true(),
                Some(CharWithCoords { ch: 'f', coords }) => self.match_false(),
                Some(CharWithCoords { ch: '-', coords }) => self.match_number(),
                Some(CharWithCoords { ch: d, coords }) if d.is_ascii_digit() => self.match_number(),
                Some(CharWithCoords { ch, coords }) => wrapped_lexer_error!(
                    LexerErrorDetails::InvalidCharacter(ch.clone()),
                    coords.clone()
                ),
                None => wrapped_lexer_error!(LexerErrorDetails::EndOfInput),
            },
            Err(err) => match err.details {
                LexerErrorDetails::EndOfInput => {
                    packed_token!(Token::EndOfInput, self.input.position())
                }
                _ => match err.coords {
                    Some(coords) => wrapped_lexer_error!(err.details, coords),
                    None => wrapped_lexer_error!(err.details),
                },
            },
        }
    }

    /// Match on a valid Json string.
    #[inline]
    fn match_string(&mut self) -> LexerResult<PackedToken> {
        loop {
            match self.advance(false) {
                Ok(_) => match self.front_char() {
                    match_escape!() => match self.input.advance(false) {
                        Ok(_) => match self.front_char() {
                            match_escape_non_unicode_suffix!() => (),
                            match_escape_unicode_suffix!() => self.check_unicode_sequence()?,
                            _ => {
                                return wrapped_lexer_error!(
                                    LexerErrorDetails::InvalidEscapeSequence(self.current_string()),
                                    self.back_coords()
                                );
                            }
                        },
                        Err(err) => {
                            return wrapped_lexer_error!(
                                LexerErrorDetails::EndOfInput,
                                err.coords.unwrap()
                            );
                        }
                    },
                    match_quote!() => {
                        return packed_token!(
                            Token::Str(self.current_string()),
                            self.back_coords(),
                            self.front_coords()
                        );
                    }
                    _ => (),
                },
                Err(err) => return wrapped_lexer_error!(err.details, err.coords.unwrap()),
            }
        }
    }

    /// Check for a valid unicode escape sequence of the form '\uXXXX'
    #[inline]
    fn check_unicode_sequence(&mut self) -> LexerResult<()> {
        let start_position = self.absolute_position();
        for i in 1..=4 {
            match self.advance(false) {
                Ok(_) => {
                    if !self.front_char().is_ascii_hexdigit() {
                        return wrapped_lexer_error!(
                            LexerErrorDetails::InvalidUnicodeEscapeSequence(self.current_string()),
                            start_position
                        );
                    }
                }
                Err(e) => {
                    return wrapped_lexer_error!(
                        LexerErrorDetails::EndOfInput,
                        self.absolute_position()
                    );
                }
            }
        }
        Ok(())
    }

    /// Match on a valid Json number representation, taking into account valid prefixes allowed
    /// within Json but discarding anything that may be allowed by a more general representations.
    ///
    /// Few rules are applied here, leading to different error conditions:
    /// - All representations must have a valid prefix
    /// - Only a single exponent can be specified
    /// - Only a single decimal point can be specified
    /// - Exponents must be well-formed
    /// - An non-exponent alphabetic found in the representation will result in an error
    /// - Numbers can be terminated by commas, brackets and whitespace only (end of pair, end of array)
    #[inline]
    fn match_number(&mut self) -> LexerResult<PackedToken> {
        let mut have_exponent = false;
        let mut have_decimal = false;

        match self.match_valid_number_prefix() {
            Ok(integral) => {
                have_decimal = !integral;
                loop {
                    match self.advance(false) {
                        Ok(_) => match self.front_char() {
                            match_digit!() => (),
                            match_exponent!() => {
                                if !have_exponent {
                                    self.check_following_exponent()?;
                                    have_exponent = true;
                                } else {
                                    return wrapped_lexer_error!(
                                        LexerErrorDetails::InvalidNumericRepresentation(
                                            self.current_string()
                                        ),
                                        self.back_coords()
                                    );
                                }
                            }
                            match_period!() => {
                                if !have_decimal {
                                    have_decimal = true;
                                } else {
                                    return wrapped_lexer_error!(
                                        LexerErrorDetails::InvalidNumericRepresentation(
                                            self.current_string()
                                        ),
                                        self.back_coords()
                                    );
                                }
                            }
                            match_numeric_terminator!() => {
                                self.input.pushback();
                                break;
                            }
                            ch if ch.is_ascii_whitespace() => {
                                self.input.pushback();
                                break;
                            }
                            ch if ch.is_alphabetic() => {
                                return wrapped_lexer_error!(
                                    LexerErrorDetails::InvalidNumericRepresentation(
                                        self.current_string()
                                    ),
                                    self.back_coords()
                                );
                            }
                            _ => {
                                return wrapped_lexer_error!(
                                    LexerErrorDetails::InvalidNumericRepresentation(
                                        self.current_string()
                                    ),
                                    self.back_coords()
                                );
                            }
                        },
                        Err(err) => {
                            return match err.coords {
                                Some(coords) => wrapped_lexer_error!(err.details, coords),
                                None => wrapped_lexer_error!(err.details),
                            };
                        }
                    }
                }
            }
            Err(err) => {
                return match err.coords {
                    Some(coords) => wrapped_lexer_error!(err.details, coords),
                    None => wrapped_lexer_error!(err.details),
                };
            }
        }

        self.parse_numeric(!have_decimal)
    }

    #[inline]
    fn check_following_exponent(&mut self) -> LexerResult<()> {
        self.advance(false).and_then(|_| {
            return match self.front_char() {
                match_plus_minus!() => Ok(()),
                _ => wrapped_lexer_error!(
                    LexerErrorDetails::InvalidNumericRepresentation(self.current_string()),
                    self.absolute_position()
                ),
            };
        })
    }

    #[cfg(not(feature = "mixed_numerics"))]
    #[inline]
    fn parse_numeric(&mut self, integral: bool) -> LexerResult<PackedToken> {
        packed_token!(
            Token::Float(fast_float::parse(self.input.buffer_as_byte_array()).unwrap()),
            self.back_coords(),
            self.front_coords()
        )
    }

    #[cfg(feature = "mixed_numerics")]
    #[inline]
    fn parse_numeric(&mut self, integral: bool) -> LexerResult<PackedToken> {
        if integral {
            packed_token!(
                Token::Integer(lexical::parse(self.input.buffer_as_byte_array()).unwrap()),
                self.back_coords(),
                self.front_coords()
            )
        } else {
            packed_token!(
                Token::Float(fast_float::parse(self.input.buffer_as_byte_array()).unwrap()),
                self.back_coords(),
                self.front_coords()
            )
        }
    }

    /// Check that a numeric representation is prefixed correctly.
    ///
    /// A few rules here:
    /// - A leading minus must be followed by a digit
    /// - A leading minus must be followed by at most one zero before a period
    /// - Any number > zero can't have a leading zero in the representation
    #[inline]
    fn match_valid_number_prefix(&mut self) -> LexerResult<bool> {
        let ch = self.back_char();
        assert!(ch.is_ascii_digit() || ch == '-');
        match ch {
            match_minus!() => self
                .input
                .advance(false)
                .map_err(|e| lexer_error!(LexerErrorDetails::EndOfInput))
                .and_then(|_| self.check_following_minus()),
            match_zero!() => self
                .input
                .advance(false)
                .map_err(|e| lexer_error!(LexerErrorDetails::EndOfInput))
                .and_then(|_| self.check_following_zero()),
            _ => Ok(true),
        }
    }

    /// Check for valid characters following a zero
    #[inline]
    fn check_following_zero(&mut self) -> LexerResult<bool> {
        match self.front_char() {
            match_period!() => Ok(false),
            match_digit!() => wrapped_lexer_error!(
                LexerErrorDetails::InvalidNumericRepresentation(self.current_string()),
                self.back_coords()
            ),
            match_newline!() => {
                self.input.pushback();
                Ok(true)
            }
            _ => {
                self.input.pushback();
                Ok(true)
            }
        }
    }

    /// Check for valid characters following a minus character
    #[inline]
    fn check_following_minus(&mut self) -> LexerResult<bool> {
        match self.front_char() {
            match_non_zero_digit!() => Ok(true),
            match_zero!() => self.advance(false).and_then(|_| {
                if self.front_char() != '.' {
                    return wrapped_lexer_error!(
                        LexerErrorDetails::InvalidNumericRepresentation(self.current_string()),
                        self.back_coords()
                    );
                }
                Ok(false)
            }),
            match_newline!() => {
                self.input.pushback();
                Ok(true)
            }
            _ => wrapped_lexer_error!(
                LexerErrorDetails::InvalidNumericRepresentation(self.current_string()),
                self.back_coords()
            ),
        }
    }

    /// Match on a null token
    #[inline]
    fn match_null(&mut self) -> LexerResult<PackedToken> {
        self.input
            .advance_n(3, false)
            .map_err(|e| lexer_error!(LexerErrorDetails::EndOfInput, self.absolute_position()))
            .and_then(|_| {
                if self.current_chars() == NULL_PATTERN {
                    packed_token!(Token::Null, self.back_coords(), self.front_coords())
                } else {
                    wrapped_lexer_error!(
                        LexerErrorDetails::MatchFailed(
                            String::from_iter(NULL_PATTERN.iter()),
                            self.current_string()
                        ),
                        self.back_coords()
                    )
                }
            })
    }

    /// Match on a true token
    #[inline]
    fn match_true(&mut self) -> LexerResult<PackedToken> {
        self.advance_n(3, false).and_then(|_| {
            if self.current_chars() == TRUE_PATTERN {
                packed_token!(
                    Token::Boolean(true),
                    self.back_coords(),
                    self.front_coords()
                )
            } else {
                wrapped_lexer_error!(
                    LexerErrorDetails::MatchFailed(
                        String::from_iter(TRUE_PATTERN.iter()),
                        self.current_string()
                    ),
                    self.back_coords()
                )
            }
        })
    }

    /// Match on a false token
    #[inline]
    fn match_false(&mut self) -> LexerResult<PackedToken> {
        self.advance_n(4, false).and_then(|_| {
            if self.current_chars() == FALSE_PATTERN {
                packed_token!(
                    Token::Boolean(false),
                    self.back_coords(),
                    self.front_coords()
                )
            } else {
                wrapped_lexer_error!(
                    LexerErrorDetails::MatchFailed(
                        String::from_iter(FALSE_PATTERN.iter()),
                        self.current_string()
                    ),
                    self.back_coords()
                )
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::time::Instant;

    use chisel_common::char::span::Span;
    use chisel_common::{lines_from_relative_file, reader_from_bytes};
    use chisel_decoders::utf8::Utf8Decoder;

    use crate::json::lexer::{Lexer, LexerError, LexerResult};
    use crate::json::tokens::{PackedToken, Token};

    #[test]
    fn should_parse_basic_tokens() {
        let mut reader = reader_from_bytes!("{}[],:");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut lexer = Lexer::new(&mut decoder);
        let mut tokens: Vec<Token> = vec![];
        let mut spans: Vec<Span> = vec![];
        for _ in 1..=7 {
            let token = lexer.consume().unwrap();
            tokens.push(token.0);
            spans.push(token.1);
        }
        assert_eq!(
            tokens,
            [
                Token::StartObject,
                Token::EndObject,
                Token::StartArray,
                Token::EndArray,
                Token::Comma,
                Token::Colon,
                Token::EndOfInput
            ]
        );
    }

    #[test]
    fn should_parse_null_and_booleans() {
        let mut reader = reader_from_bytes!("null true    falsetruefalse");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut lexer = Lexer::new(&mut decoder);
        let mut tokens: Vec<Token> = vec![];
        let mut spans: Vec<Span> = vec![];
        for _ in 1..=6 {
            let token = lexer.consume().unwrap();
            tokens.push(token.0);
            spans.push(token.1);
        }
        assert_eq!(
            tokens,
            [
                Token::Null,
                Token::Boolean(true),
                Token::Boolean(false),
                Token::Boolean(true),
                Token::Boolean(false),
                Token::EndOfInput
            ]
        );
    }

    #[test]
    fn should_parse_strings() {
        let lines = lines_from_relative_file!("fixtures/utf-8/strings.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let mut reader = reader_from_bytes!(l);
                let mut decoder = Utf8Decoder::new(&mut reader);
                let mut lexer = Lexer::new(&mut decoder);
                let token = lexer.consume().unwrap();
                match token.0 {
                    Token::Str(str) => {
                        assert_eq!(str, l)
                    }
                    _ => panic!(),
                }
            }
        }
    }

    #[test]
    fn should_report_correct_error_char_position() {
        let mut reader = reader_from_bytes!("{\"abc\" : \nd}");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut lexer = Lexer::new(&mut decoder);
        let mut results = vec![];
        for _ in 0..4 {
            results.push(lexer.consume())
        }
        assert!(&results[3].is_err());
        let coords = results[3].clone().err().unwrap().coords.unwrap();
        assert_eq!(coords.absolute, 11);
        assert_eq!(coords.line, 2)
    }

    #[test]
    fn should_parse_numerics() {
        let start = Instant::now();
        let lines = lines_from_relative_file!("fixtures/utf-8/numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                println!("Parsing {}", l);
                let mut reader = reader_from_bytes!(l);
                let mut decoder = Utf8Decoder::new(&mut reader);
                let mut lexer = Lexer::new(&mut decoder);
                let token = lexer.consume().unwrap();
                match token.0 {
                    Token::Integer(_) => {
                        assert_eq!(
                            token.0,
                            Token::Integer(l.replace(',', "").parse::<i64>().unwrap())
                        );
                    }
                    Token::Float(_) => {
                        assert_eq!(
                            token.0,
                            Token::Float(fast_float::parse(l.replace(',', "")).unwrap())
                        );
                    }
                    _ => panic!(),
                }
            }
        }
        println!("Parsed numerics in {:?}", start.elapsed());
    }

    #[test]
    fn should_correctly_handle_invalid_numbers() {
        let lines = lines_from_relative_file!("fixtures/utf-8/invalid_numbers.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let mut reader = reader_from_bytes!(l);
                let mut decoder = Utf8Decoder::new(&mut reader);
                let mut lexer = Lexer::new(&mut decoder);
                let token = lexer.consume();
                assert!(token.is_err());
            }
        }
    }

    #[test]
    fn should_correctly_identity_dodgy_strings() {
        let lines = lines_from_relative_file!("fixtures/utf-8/dodgy_strings.txt");
        for l in lines.flatten() {
            if !l.is_empty() {
                let mut reader = reader_from_bytes!(l);
                let mut decoder = Utf8Decoder::new(&mut reader);
                let mut lexer = Lexer::new(&mut decoder);
                let mut error_token: Option<LexerError> = None;
                loop {
                    let token = lexer.consume();
                    match token {
                        Ok(packed) => {
                            if packed.0 == Token::EndOfInput {
                                break;
                            }
                        }
                        Err(err) => {
                            error_token = Some(err.clone());
                            println!("Dodgy string found: {} : {}", l, err.coords.unwrap());
                            break;
                        }
                    }
                }
                assert!(error_token.is_some());
            }
        }
    }

    #[test]
    fn should_correctly_report_errors_for_booleans() {
        let mut reader = reader_from_bytes!("true farse");
        let mut decoder = Utf8Decoder::new(&mut reader);
        let mut lexer = Lexer::new(&mut decoder);
        let mut results: Vec<LexerResult<PackedToken>> = vec![];
        for _ in 1..=2 {
            results.push(lexer.consume());
        }

        // check that we've got the correct types of results
        assert!(results[0].is_ok());
        assert!(results[1].is_err());

        // check that we've located a boolean in the correct position
        if results[0].is_ok() {
            match &results[0] {
                Ok(packed) => {
                    assert_eq!((*packed).1.start.column, 1)
                }
                Err(_) => {}
            }
        }

        // check that the dodgy boolean has been picked up at the correct location
        if results[1].is_err() {
            match &results[1] {
                Ok(_) => {}
                Err(err) => {
                    assert_eq!(err.coords.unwrap().column, 6)
                }
            }
        }

        println!("Parse error: {:?}", results[1]);
    }
}
