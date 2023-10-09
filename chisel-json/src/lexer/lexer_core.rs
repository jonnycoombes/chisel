//! Lexer used by both DOM and SAX parsers
//!
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use crate::coords::{Coords, Span};
use std::fmt::{Display, Formatter};
use std::io::BufRead;

use crate::lexer::lexer_input::{CharWithCoords, LexerInput};
use crate::lexer_error;
use crate::results::{ParserError, ParserErrorDetails, ParserErrorSource, ParserResult};

/// Default lookahead buffer size
const DEFAULT_BUFFER_SIZE: usize = 4096;
/// Pattern to match for null
const NULL_PATTERN: [char; 4] = ['n', 'u', 'l', 'l'];
/// Pattern to match for true
const TRUE_PATTERN: [char; 4] = ['t', 'r', 'u', 'e'];
/// Pattern to match for false
const FALSE_PATTERN: [char; 5] = ['f', 'a', 'l', 's', 'e'];

/// Enumeration of valid JSON tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Colon,
    Comma,
    Str(String),
    Float(f64),
    Integer(i64),
    Null,
    Boolean(bool),
    EndOfInput,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::StartObject => write!(f, "StartObject"),
            Token::EndObject => write!(f, "EndObject"),
            Token::StartArray => write!(f, "StartArray"),
            Token::EndArray => write!(f, "EndArray"),
            Token::Colon => write!(f, "Colon"),
            Token::Comma => write!(f, "Comma"),
            Token::Str(str) => write!(f, "String(\"{}\")", str),
            Token::Float(num) => write!(f, "Float({})", num),
            Token::Integer(num) => write!(f, "Integer({})", num),
            Token::Null => write!(f, "Null"),
            Token::Boolean(bool) => write!(f, "Boolean({})", bool),
            Token::EndOfInput => write!(f, "EndOfInput"),
        }
    }
}

/// A packed token consists of a [Token] and the [Span] associated with it
pub type PackedToken<'a> = (Token, Span);

/// Convenience macro for packing tokens along with their positional information
macro_rules! packed_token {
    ($t:expr, $s:expr, $e:expr) => {
        Ok(($t, Span { start: $s, end: $e }))
    };
    ($t:expr, $s:expr) => {
        Ok(($t, Span { start: $s, end: $s }))
    };
}

macro_rules! match_zero {
    () => {
        '0'
    };
}

macro_rules! match_minus {
    () => {
        '-'
    };
}

macro_rules! match_plus_minus {
    () => {
        '+' | '-'
    };
}

macro_rules! match_digit {
    () => {
        '0'..='9'
    };
}

macro_rules! match_non_zero_digit {
    () => {
        '1'..='9'
    };
}

macro_rules! match_exponent {
    () => {
        'e' | 'E'
    };
}

macro_rules! match_period {
    () => {
        '.'
    };
}

macro_rules! match_numeric_terminator {
    () => {
        ']' | '}' | ','
    };
}

macro_rules! match_escape {
    () => {
        '\\'
    };
}

macro_rules! match_escape_non_unicode_suffix {
    () => {
        'n' | 't' | 'r' | '\\' | '/' | 'b' | 'f' | '\"'
    };
}

macro_rules! match_escape_unicode_suffix {
    () => {
        'u'
    };
}

macro_rules! match_quote {
    () => {
        '\"'
    };
}

macro_rules! match_newline {
    () => {
        '\n'
    };
}

pub struct Lexer<'a> {
    /// Input coordinate state
    input: LexerInput<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a mut impl Iterator<Item = char>) -> Self {
        Lexer {
            input: LexerInput::new(chars),
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
    fn advance(&mut self, skip_whitespace: bool) -> ParserResult<()> {
        self.input.advance(skip_whitespace)
    }

    /// Advance the input by n
    fn advance_n(&mut self, n: usize, skip_whitespace: bool) -> ParserResult<()> {
        self.input.advance_n(n, skip_whitespace)
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
    pub fn consume(&mut self) -> ParserResult<PackedToken> {
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
                Some(CharWithCoords { ch, coords }) => lexer_error!(
                    ParserErrorDetails::InvalidCharacter(ch.clone()),
                    coords.clone()
                ),
                None => lexer_error!(ParserErrorDetails::EndOfInput),
            },
            Err(err) => match err.details {
                ParserErrorDetails::EndOfInput => {
                    packed_token!(Token::EndOfInput, self.input.position())
                }
                _ => match err.coords {
                    Some(coords) => lexer_error!(err.details, coords),
                    None => lexer_error!(err.details),
                },
            },
        }
    }

    /// Match on a valid Json string.
    fn match_string(&mut self) -> ParserResult<PackedToken> {
        loop {
            match self.advance(false) {
                Ok(_) => match self.front_char() {
                    match_escape!() => match self.input.advance(false) {
                        Ok(_) => match self.front_char() {
                            match_escape_non_unicode_suffix!() => (),
                            match_escape_unicode_suffix!() => self.check_unicode_sequence()?,
                            _ => {
                                return lexer_error!(
                                    ParserErrorDetails::InvalidEscapeSequence(
                                        self.current_string()
                                    ),
                                    self.back_coords()
                                );
                            }
                        },
                        Err(err) => {
                            return lexer_error!(err.details, err.coords.unwrap());
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
                Err(err) => return lexer_error!(err.details, err.coords.unwrap()),
            }
        }
    }

    /// Check for a valid unicode escape sequence of the form '\uXXXX'
    fn check_unicode_sequence(&mut self) -> ParserResult<()> {
        let start_position = self.absolute_position();
        for i in 1..=4 {
            match self.advance(false) {
                Ok(_) => {
                    if !self.front_char().is_ascii_hexdigit() {
                        return lexer_error!(
                            ParserErrorDetails::InvalidUnicodeEscapeSequence(self.current_string()),
                            start_position
                        );
                    }
                }
                Err(e) => {
                    return lexer_error!(ParserErrorDetails::EndOfInput, self.absolute_position());
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
    fn match_number(&mut self) -> ParserResult<PackedToken> {
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
                                    return lexer_error!(
                                        ParserErrorDetails::InvalidNumericRepresentation(
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
                                    return lexer_error!(
                                        ParserErrorDetails::InvalidNumericRepresentation(
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
                                return lexer_error!(
                                    ParserErrorDetails::InvalidNumericRepresentation(
                                        self.current_string()
                                    ),
                                    self.back_coords()
                                );
                            }
                            _ => {
                                return lexer_error!(
                                    ParserErrorDetails::InvalidNumericRepresentation(
                                        self.current_string()
                                    ),
                                    self.back_coords()
                                );
                            }
                        },
                        Err(err) => {
                            return match err.coords {
                                Some(coords) => lexer_error!(err.details, coords),
                                None => lexer_error!(err.details),
                            };
                        }
                    }
                }
            }
            Err(err) => {
                return match err.coords {
                    Some(coords) => lexer_error!(err.details, coords),
                    None => lexer_error!(err.details),
                }
            }
        }

        self.parse_numeric(!have_decimal)
    }

    fn check_following_exponent(&mut self) -> ParserResult<()> {
        self.advance(false).and_then(|_| {
            return match self.front_char() {
                match_plus_minus!() => Ok(()),
                _ => lexer_error!(
                    ParserErrorDetails::InvalidNumericRepresentation(self.current_string()),
                    self.absolute_position()
                ),
            };
        })
    }

    #[cfg(not(feature = "mixed_numerics"))]
    fn parse_numeric(
        &mut self,
        integral: bool,
        start_coords: Coords,
        end_coords: Coords,
    ) -> ParserResult<PackedToken> {
        packed_token!(
            Token::Float(fast_float::parse(self.input.buffer_as_bytes()).unwrap()),
            back_input_coords!(),
            front_input_coords!()
        )
    }

    #[cfg(feature = "mixed_numerics")]
    fn parse_numeric(&mut self, integral: bool) -> ParserResult<PackedToken> {
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
    fn match_valid_number_prefix(&mut self) -> ParserResult<bool> {
        let ch = self.back_char();
        assert!(ch.is_ascii_digit() || ch == '-');
        match ch {
            match_minus!() => self
                .input
                .advance(false)
                .and_then(|_| self.check_following_minus()),
            match_zero!() => self
                .input
                .advance(false)
                .and_then(|_| self.check_following_zero()),
            _ => Ok(true),
        }
    }

    fn check_following_zero(&mut self) -> ParserResult<bool> {
        match self.front_char() {
            match_period!() => Ok(false),
            match_digit!() => lexer_error!(
                ParserErrorDetails::InvalidNumericRepresentation(self.current_string()),
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

    fn check_following_minus(&mut self) -> ParserResult<bool> {
        match self.front_char() {
            match_non_zero_digit!() => Ok(true),
            match_zero!() => self.advance(false).and_then(|_| {
                if self.front_char() != '.' {
                    return lexer_error!(
                        ParserErrorDetails::InvalidNumericRepresentation(self.current_string()),
                        self.back_coords()
                    );
                }
                Ok(false)
            }),
            match_newline!() => {
                self.input.pushback();
                Ok(true)
            }
            _ => lexer_error!(
                ParserErrorDetails::InvalidNumericRepresentation(self.current_string()),
                self.back_coords()
            ),
        }
    }

    /// Match on a null token
    fn match_null(&mut self) -> ParserResult<PackedToken> {
        self.input.advance_n(3, false).and_then(|_| {
            if self.current_chars() == NULL_PATTERN {
                packed_token!(Token::Null, self.back_coords(), self.front_coords())
            } else {
                lexer_error!(
                    ParserErrorDetails::MatchFailed(
                        String::from_iter(NULL_PATTERN.iter()),
                        self.current_string()
                    ),
                    self.back_coords()
                )
            }
        })
    }

    /// Match on a true token
    fn match_true(&mut self) -> ParserResult<PackedToken> {
        self.advance_n(3, false).and_then(|_| {
            if self.current_chars() == TRUE_PATTERN {
                packed_token!(
                    Token::Boolean(true),
                    self.back_coords(),
                    self.front_coords()
                )
            } else {
                lexer_error!(
                    ParserErrorDetails::MatchFailed(
                        String::from_iter(TRUE_PATTERN.iter()),
                        self.current_string()
                    ),
                    self.back_coords()
                )
            }
        })
    }

    /// Match on a false token
    fn match_false(&mut self) -> ParserResult<PackedToken> {
        self.advance_n(4, false).and_then(|_| {
            if self.current_chars() == FALSE_PATTERN {
                packed_token!(
                    Token::Boolean(false),
                    self.back_coords(),
                    self.front_coords()
                )
            } else {
                lexer_error!(
                    ParserErrorDetails::MatchFailed(
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

    use chisel_decoders::utf8::Utf8Decoder;

    use crate::coords::Span;
    use crate::lexer::lexer_core::{Lexer, PackedToken, Token};
    use crate::results::{ParserError, ParserResult};
    use crate::{lines_from_relative_file, reader_from_bytes};

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
                let mut error_token: Option<ParserError> = None;
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
        let mut results: Vec<ParserResult<PackedToken>> = vec![];
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
