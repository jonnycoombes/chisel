//! The SAX parser
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chisel_common::char::coords::Coords;
use chisel_decoders::{default_decoder, new_decoder, Encoding};
use chisel_json_pointer::JsonPointer;
use chisel_lexers::json::lexer::Lexer;
use chisel_lexers::json::tokens::Token;

use crate::json::events::{Event, Match};
use crate::parser_error;
use crate::{ParserError, ParserErrorDetails, ParserResult};

macro_rules! emit_event {
    ($cb : expr, $m : expr, $span : expr, $path : expr) => {
        $cb(&Event {
            matched: $m,
            span: $span,
            pointer: Some(&$path),
        })
    };
    ($cb : expr, $m : expr, $span : expr) => {
        $cb(&Event {
            matched: $m,
            span: $span,
            pointer: None,
        })
    };
}

/// Main JSON parser struct
pub struct Parser {
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

    pub fn parse_file<PathLike: AsRef<Path>, Callback>(
        &self,
        path: PathLike,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let mut chars = new_decoder(&mut reader, self.encoding);
                self.parse(&mut chars, cb)
            }
            Err(_) => {
                parser_error!(ParserErrorDetails::InvalidFile)
            }
        }
    }

    pub fn parse_bytes<Callback>(&self, bytes: &[u8], cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        if bytes.is_empty() {
            return parser_error!(ParserErrorDetails::ZeroLengthInput, Coords::default());
        }
        let mut reader = BufReader::new(bytes);
        let mut chars = default_decoder(&mut reader);
        self.parse(&mut chars, cb)
    }

    pub fn parse_str<Callback>(&self, str: &str, cb: &mut Callback) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        if str.is_empty() {
            return parser_error!(ParserErrorDetails::ZeroLengthInput, Coords::default());
        }
        let mut reader = BufReader::new(str.as_bytes());
        let mut chars = default_decoder(&mut reader);
        self.parse(&mut chars, cb)
    }

    /// Parse the contents extracted from an instance of [BufRead]
    pub fn parse_buffer<Callback>(
        &self,
        buffer: &mut impl BufRead,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        let mut chars = default_decoder(buffer);
        self.parse(&mut chars, cb)
    }

    pub fn parse<Callback>(
        &self,
        chars: &mut impl Iterator<Item = char>,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        let mut pointer = JsonPointer::root();
        let mut lexer = Lexer::new(chars);
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartOfInput, span)?;
                emit_event!(cb, Match::StartObject, span, pointer)?;
                self.parse_object(&mut lexer, &mut pointer, cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartOfInput, span, pointer)?;
                emit_event!(cb, Match::StartArray, span, pointer)?;
                self.parse_array(&mut lexer, &mut pointer, cb)
            }
            (_, span) => {
                parser_error!(ParserErrorDetails::InvalidRootObject, span.start)
            }
        }
    }

    #[inline]
    fn parse_value<Callback>(
        &self,
        lexer: &mut Lexer,
        pointer: &mut JsonPointer,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        match lexer.consume()? {
            (Token::StartObject, span) => {
                emit_event!(cb, Match::StartObject, span, pointer)?;
                self.parse_object(lexer, pointer, cb)
            }
            (Token::StartArray, span) => {
                emit_event!(cb, Match::StartArray, span, pointer)?;
                self.parse_array(lexer, pointer, cb)
            }
            (Token::Str(str), span) => {
                emit_event!(cb, Match::String(Cow::Borrowed(&str)), span, pointer)
            }
            (Token::LazyNumeric(value), span) => {
                emit_event!(cb, Match::Numeric(value), span, pointer)
            }
            (Token::Float(value), span) => {
                emit_event!(cb, Match::Float(value), span, pointer)
            }
            (Token::Integer(value), span) => {
                emit_event!(cb, Match::Integer(value), span, pointer)
            }
            (Token::Boolean(value), span) => {
                emit_event!(cb, Match::Boolean(value), span, pointer)
            }
            (Token::Null, span) => {
                emit_event!(cb, Match::Null, span, pointer)
            }
            (token, span) => {
                parser_error!(
                    ParserErrorDetails::UnexpectedToken(token.to_string()),
                    span.start
                )
            }
        }
    }

    /// An object is just a list of comma separated KV pairs
    fn parse_object<Callback>(
        &self,
        lexer: &mut Lexer,
        pointer: &mut JsonPointer,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        loop {
            match lexer.consume()? {
                (Token::Str(str), span) => {
                    pointer.push_name(str.replace("\"", ""));
                    emit_event!(cb, Match::ObjectKey(Cow::Borrowed(&str)), span, pointer)?;
                    let should_be_colon = lexer.consume()?;
                    match should_be_colon {
                        (Token::Colon, _) => {
                            self.parse_value(lexer, pointer, cb)?;
                            pointer.pop();
                        }
                        (_, _) => {
                            return parser_error!(
                                ParserErrorDetails::PairExpected,
                                should_be_colon.1.start
                            )
                        }
                    }
                }
                (Token::Comma, _) => (),
                (Token::EndObject, span) => {
                    return emit_event!(cb, Match::EndObject, span, pointer);
                }
                (_token, span) => {
                    return parser_error!(ParserErrorDetails::InvalidArray, span.start)
                }
            }
        }
    }

    /// An array is just a list of comma separated values
    fn parse_array<Callback>(
        &self,
        lexer: &mut Lexer,
        pointer: &mut JsonPointer,
        cb: &mut Callback,
    ) -> ParserResult<()>
    where
        Callback: FnMut(&Event) -> ParserResult<()>,
    {
        let mut index = 0;
        let mut expect_value: bool = true;
        let mut first_pass = true;
        loop {
            pointer.push_index(index);
            match lexer.consume()? {
                (Token::StartArray, span) => {
                    emit_event!(cb, Match::StartArray, span, pointer)?;
                    self.parse_array(lexer, pointer, cb)?;
                }
                (Token::EndArray, span) => {
                    return if !expect_value || first_pass {
                        pointer.pop();
                        emit_event!(cb, Match::EndArray, span, pointer)
                    } else {
                        parser_error!(ParserErrorDetails::ValueExpected, span.start)
                    }
                }
                (Token::StartObject, span) => {
                    emit_event!(cb, Match::StartObject, span, pointer)?;
                    self.parse_object(lexer, pointer, cb)?;
                }
                (Token::Str(str), span) => {
                    emit_event!(cb, Match::String(Cow::Borrowed(&str)), span, pointer)?;
                }
                (Token::LazyNumeric(value), span) => {
                    emit_event!(cb, Match::Numeric(value), span, pointer)?;
                }
                (Token::Float(value), span) => {
                    emit_event!(cb, Match::Float(value), span, pointer)?;
                }
                (Token::Integer(value), span) => {
                    emit_event!(cb, Match::Integer(value), span, pointer)?;
                }
                (Token::Boolean(value), span) => {
                    emit_event!(cb, Match::Boolean(value), span, pointer)?;
                }
                (Token::Null, span) => emit_event!(cb, Match::Null, span, pointer)?,
                (Token::Comma, span) => {
                    if !expect_value {
                        index += 1
                    } else {
                        return parser_error!(ParserErrorDetails::ValueExpected, span.start);
                    }
                }
                (_token, span) => {
                    return parser_error!(ParserErrorDetails::InvalidArray, span.start);
                }
            }
            first_pass = false;
            expect_value = !expect_value;
            pointer.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::time::Instant;
    use std::{env, fs};

    use bytesize::ByteSize;
    use chisel_common::char::coords::Coords;

    use chisel_common::relative_file;

    use crate::json::sax::Parser;
    use crate::json::specs;
    use crate::ParserErrorDetails;

    #[test]
    fn should_puke_on_empty_input() {
        let input = "";
        let parser = Parser::default();
        let parsed = parser.parse_str(input, &mut |_e| Ok(()));
        assert!(parsed.is_err());
        assert_eq!(
            parsed.err().unwrap().details,
            ParserErrorDetails::ZeroLengthInput
        );
    }

    #[test]
    fn should_parse_successfully() {
        let mut counter = 0;
        let path = relative_file!("fixtures/json/valid/events.json");
        let parser = Parser::default();
        let parsed = parser.parse_file(&path, &mut |_e| {
            counter += 1;
            Ok(())
        });
        println!("{} SAX events processed", counter);
        assert!(parsed.is_ok());
    }

    #[test]
    fn should_successfully_handle_basic_invalid_inputs() {
        for spec in specs::invalid_json_specs() {
            let mut counter = 0;
            let path = relative_file!(spec.filename);
            let parser = Parser::default();
            let parse_result = parser.parse_file(&path, &mut |_e| {
                counter += 1;
                Ok(())
            });
            println!("Parse result = {:?}", parse_result);
            assert!(&parse_result.is_err());

            let err = parse_result.err().unwrap();
            let err_coords = Coords::from_coords(&err.coords.unwrap());
            assert_eq!(err_coords.line, spec.expected.coords.line);
            assert_eq!(err_coords.column, spec.expected.coords.column)
        }
    }

    #[test]
    fn should_allow_for_parsing_of_a_buffer() {
        let input = "{ \"test\" : 2123232323}".as_bytes();
        let mut buffer = BufReader::new(input);
        let parser = Parser::default();
        let _parsed = parser.parse_buffer(&mut buffer, &mut |_e| Ok(()));
    }

    #[test]
    fn should_parse_basic_test_files() {
        for f in fs::read_dir("fixtures/json/valid").unwrap() {
            let path = f.unwrap().path();
            println!("Parsing {:?}", &path);
            if path.is_file() {
                let mut counter = 0;
                let len = fs::metadata(&path).unwrap().len();
                let start = Instant::now();
                let path = relative_file!(path.to_str().unwrap());
                let parser = Parser::default();
                let parsed = parser.parse_file(&path, &mut |_e| {
                    counter += 1;
                    Ok(())
                });
                if parsed.is_err() {
                    println!("Parse of {:?} failed!", &path);
                    println!("Parse failed with errors: {:?}", &parsed)
                }
                assert!(parsed.is_ok());
                println!(
                    "Parsed {} in {:?} [{:?}], {} SAX events processed",
                    ByteSize(len),
                    start.elapsed(),
                    path,
                    counter
                );
            }
        }
    }
}
