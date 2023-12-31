//! The DOM parser
//!
//!
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chisel_decoders::{default_decoder, new_decoder, Encoding};
use chisel_lexers::json::lexer::Lexer;
use chisel_lexers::json::tokens::Token;

use crate::json::{JsonKeyValue, JsonNumeric, JsonValue};
use crate::{parser_error, ParserError, ParserErrorDetails, ParserResult};

/// Main JSON parser struct
pub struct Parser {
    /// The current encoding
    encoding: Encoding,
}

impl Default for Parser {
    /// The default encoding is Utf-8
    fn default() -> Self {
        Self {
            encoding: Default::default(),
        }
    }
}

impl Parser {
    /// Create a new instance of the parser using a specific [Encoding]
    pub fn with_encoding(encoding: Encoding) -> Self {
        Self { encoding }
    }

    ///
    pub fn parse_file<PathLike: AsRef<Path>>(&self, path: PathLike) -> ParserResult<JsonValue> {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let mut chars = new_decoder(&mut reader, self.encoding);
                self.parse(&mut chars)
            }
            Err(_) => {
                parser_error!(ParserErrorDetails::InvalidFile)
            }
        }
    }

    pub fn parse_bytes(&self, bytes: &[u8]) -> ParserResult<JsonValue> {
        let mut reader = BufReader::new(bytes);
        let mut chars = default_decoder(&mut reader);
        self.parse(&mut chars)
    }

    pub fn parse_str(&self, str: &str) -> ParserResult<JsonValue> {
        let mut reader = BufReader::new(str.as_bytes());
        let mut chars = default_decoder(&mut reader);
        self.parse(&mut chars)
    }

    /// Parse the contents of a buffer (e.g. implementation of [BufRead])
    pub fn parse_buffer(&self, buffer: &mut impl BufRead) -> ParserResult<JsonValue> {
        let mut chars = default_decoder(buffer);
        self.parse(&mut chars)
    }

    pub fn parse(&self, chars: &mut impl Iterator<Item = char>) -> ParserResult<JsonValue> {
        let mut lexer = Lexer::new(chars);
        match lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(&mut lexer),
            (Token::StartArray, _) => self.parse_array(&mut lexer),
            (_, span) => {
                parser_error!(ParserErrorDetails::InvalidRootObject, span.start)
            }
        }
    }

    #[inline]
    fn parse_value(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        match lexer.consume()? {
            (Token::StartObject, _) => self.parse_object(lexer),
            (Token::StartArray, _) => self.parse_array(lexer),
            (Token::Str(str), _) => Ok(JsonValue::String(Cow::Owned(str))),
            (Token::LazyNumeric(value), _) => Ok(JsonValue::Number(JsonNumeric::Lazy(value))),
            (Token::Float(value), _) => Ok(JsonValue::Number(JsonNumeric::Float(value))),
            (Token::Integer(value), _) => Ok(JsonValue::Number(JsonNumeric::Integer(value))),
            (Token::Boolean(value), _) => Ok(JsonValue::Boolean(value)),
            (Token::Null, _) => Ok(JsonValue::Null),
            (token, span) => {
                parser_error!(
                    ParserErrorDetails::UnexpectedToken(token.to_string()),
                    span.start
                )
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        let mut pairs = vec![];
        loop {
            match lexer.consume()? {
                (Token::Str(str), _) => {
                    let should_be_colon = lexer.consume()?;
                    match should_be_colon {
                        (Token::Colon, _) => pairs.push(JsonKeyValue {
                            key: str,
                            value: self.parse_value(lexer)?,
                        }),
                        (_, _) => {
                            return parser_error!(
                                ParserErrorDetails::PairExpected,
                                should_be_colon.1.start
                            )
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, _) => return Ok(JsonValue::Object(pairs)),
                (_token, span) => {
                    return parser_error!(ParserErrorDetails::InvalidObject, span.start);
                }
            }
        }
    }

    /// An array is just a list of comma separated values, but we need to do additional checking
    /// to make sure that we don't have consecutive commas, we do allow for empty arrays etc...
    fn parse_array(&self, lexer: &mut Lexer) -> ParserResult<JsonValue> {
        let mut values: Vec<JsonValue> = vec![];
        let mut expect_value: bool = true;
        loop {
            match lexer.consume()? {
                (Token::StartArray, _) => {
                    values.push(self.parse_array(lexer)?);
                }
                (Token::EndArray, span) => {
                    return if !expect_value || values.is_empty() {
                        Ok(JsonValue::Array(values))
                    } else {
                        parser_error!(ParserErrorDetails::ValueExpected, span.start)
                    }
                }
                (Token::StartObject, _) => values.push(self.parse_object(lexer)?),
                (Token::Str(str), _) => values.push(JsonValue::String(Cow::Owned(str))),
                (Token::LazyNumeric(value), _) => {
                    values.push(JsonValue::Number(JsonNumeric::Lazy(value)))
                }
                (Token::Float(value), _) => {
                    values.push(JsonValue::Number(JsonNumeric::Float(value)))
                }
                (Token::Integer(value), _) => {
                    values.push(JsonValue::Number(JsonNumeric::Integer(value)))
                }
                (Token::Boolean(value), _) => values.push(JsonValue::Boolean(value)),
                (Token::Null, _) => values.push(JsonValue::Null),
                (Token::Comma, span) => {
                    if expect_value {
                        return parser_error!(ParserErrorDetails::ValueExpected, span.start);
                    }
                }
                (_token, span) => {
                    return parser_error!(ParserErrorDetails::InvalidArray, span.start);
                }
            }
            expect_value = !expect_value
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_macros)]
    use crate::json::dom::Parser;
    use crate::json::specs;
    use bytesize::ByteSize;
    use chisel_common::char::coords::Coords;
    use chisel_common::relative_file;
    use std::path::PathBuf;
    use std::time::Instant;
    use std::{env, fs};

    #[test]
    fn should_parse_char_iterators_directly() {
        let source = r#"{
            "test" : 1232.0,
            "some other" : "thasdasd",
            "a bool" : true,
            "an array" : [1,2,3,4,5.8,6,7.2,7,8,10]
        }"#;
        let parser = Parser::default();
        let parsed = parser.parse(&mut source.chars());
        println!("{parsed:?}");
        assert!(parsed.is_ok())
    }

    #[test]
    fn should_parse_lengthy_arrays() {
        let path = relative_file!("fixtures/json/valid/bc_block.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path);
        println!("{parsed:?}");
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_parse_simple_schema() {
        let path = relative_file!("fixtures/json/valid/simple_schema.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path);
        println!("{parsed:?}");
        assert!(parsed.is_ok());
    }
    #[test]
    fn should_successfully_handle_basic_invalid_inputs() {
        for spec in specs::invalid_json_specs() {
            let path = relative_file!(spec.filename);
            let parser = Parser::default();
            let parse_result = parser.parse_file(&path);
            println!("Parse result = {:?}", parse_result);
            assert!(&parse_result.is_err());
            let err = parse_result.err().unwrap();
            let err_coords = Coords::from_coords(&err.coords.unwrap());
            assert_eq!(err_coords.line, spec.expected.coords.line);
            assert_eq!(err_coords.column, spec.expected.coords.column)
        }
    }

    #[test]
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            println!("Parsing {:?}", &path);
            if path.is_file() {
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let path = relative_file!(path.to_str().unwrap());
                let parser = Parser::default();
                let parsed = parser.parse_file(&path);
                if parsed.is_err() {
                    println!("Parse of {:?} failed!", &path);
                    println!("Parse failed with errors: {:?}", &parsed)
                }
                assert!(parsed.is_ok());
                println!(
                    "Parsed {} in {:?} [{:?}]",
                    ByteSize(len),
                    start.elapsed(),
                    path,
                );
            }
        }
    }
}
