use chisel_common::char::coords::Coords;
use chisel_lexers::json::lexer::LexerError;
use std::fmt::{Display, Formatter};

/// JSON parser implementations
pub mod json;

/// Global result type used throughout the parser stages
pub type ParserResult<T> = Result<T, ParserError>;

/// A global enumeration of error codes
#[derive(Debug, Clone, PartialEq)]
pub enum ParserErrorDetails {
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
    UnexpectedToken(String),
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
    /// A bubbled error from the lexical analysis backend
    LexerError(String),
}

impl Display for ParserErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorDetails::InvalidFile => write!(f, "invalid file specified"),
            ParserErrorDetails::ZeroLengthInput => write!(f, "zero length input"),
            ParserErrorDetails::EndOfInput => write!(f, "end of input reached"),
            ParserErrorDetails::StreamFailure => write!(f, "failure in the underlying stream"),
            ParserErrorDetails::NonUtf8InputDetected => write!(f, "non-UTF8 input"),
            ParserErrorDetails::UnexpectedToken(token) => {
                write!(f, "unexpected token found: {}", token)
            }
            ParserErrorDetails::PairExpected => {
                write!(f, "pair expected, something else was found")
            }
            ParserErrorDetails::InvalidRootObject => write!(f, "invalid JSON"),
            ParserErrorDetails::InvalidObject => write!(f, "invalid object"),
            ParserErrorDetails::InvalidArray => write!(f, "invalid array"),
            ParserErrorDetails::InvalidCharacter(ch) => write!(f, "invalid character: \'{}\'", ch),
            ParserErrorDetails::MatchFailed(first, second) => write!(
                f,
                "a match failed. Looking for \"{}\", found \"{}\"",
                first, second
            ),
            ParserErrorDetails::InvalidNumericRepresentation(repr) => {
                write!(f, "invalid number representation: \"{}\"", repr)
            }
            ParserErrorDetails::InvalidEscapeSequence(seq) => {
                write!(f, "invalid escape sequence: \"{}\"", seq)
            }
            ParserErrorDetails::InvalidUnicodeEscapeSequence(seq) => {
                write!(f, "invalid unicode escape sequence: \"{}\"", seq)
            }
            ParserErrorDetails::LexerError(repr) => {
                write!(f, "lexer error reported: \"{}\"", repr)
            }
        }
    }
}

/// The general error structure
#[derive(Debug, Clone)]
pub struct ParserError {
    /// The global error code for the error
    pub details: ParserErrorDetails,
    /// Parser [Coords]
    pub coords: Option<Coords>,
}

impl Display for ParserError {
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

/// Allows conversion from errors arising within the lexical analysis/scanning stage of parsing
impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        ParserError {
            details: ParserErrorDetails::LexerError(value.details.to_string()),
            coords: value.coords,
        }
    }
}

/// Helper macro for cooking up a [ParserError] specific to the DOM parser
#[macro_export]
macro_rules! parser_error {
    ($details: expr, $coords: expr) => {
        Err(ParserError {
            details: $details,
            coords: Some($coords),
        })
    };
    ($details: expr) => {
        Err(ParserError {
            details: $details,
            coords: None,
        })
    };
}
