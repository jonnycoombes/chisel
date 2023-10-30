use crate::json::numerics::LazyNumeric;
use chisel_common::char::span::Span;
use std::fmt::{Display, Formatter};

/// Enumeration of generated JSON tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Start of a JSON object
    StartObject,
    /// End of a JSON object
    EndObject,
    /// Start of a JSON array
    StartArray,
    /// End of a JSON array
    EndArray,
    /// A colon (KV separator)
    Colon,
    /// A lowly comma
    Comma,
    /// A string value
    Str(String),
    /// A float value
    Float(f64),
    /// An integral value
    Integer(i64),
    /// A lazy numeric value
    LazyNumeric(LazyNumeric),
    /// A null value
    Null,
    /// A boolean value (true/false)
    Boolean(bool),
    /// The end of input token
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
            Token::LazyNumeric(lazy) => {
                let value: f64 = lazy.into();
                write!(f, "Lazy({})", value)
            }
            Token::Null => write!(f, "Null"),
            Token::Boolean(bool) => write!(f, "Boolean({})", bool),
            Token::EndOfInput => write!(f, "EndOfInput"),
        }
    }
}

/// A packed token consists of a [Token] and the [Span] associated with it
pub type PackedToken<'a> = (Token, Span);
