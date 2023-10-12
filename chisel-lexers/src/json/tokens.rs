use chisel_common::char::span::Span;
use std::fmt::{Display, Formatter};

/// Enumeration of generated JSON tokens
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
