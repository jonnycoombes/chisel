//! # Overview
//! This crate contains the lexical analysis backends used by *chisel*. The basic architecture
//! is very simple - multiple lexers using a common scanning implementation.
//!
//! A *scanner* consumes characters from an underlying source of characters, and keeps track of the
//! position where the character was read. It also provides some basic buffering and lookahead/pushback
//! functionality.
//!
//! It's always assumed that input is read linearly and can only be read once, from start to finish.
//!
//! A *lexer* consumes from a scanner, and attempts to construct *tokens* which may be
//! consumed by *parsers* further up the stack.
//!
//! A lexer defines and is capable of producing its own set of distinct tokens
//! specific to the parsing task in hand.  (For example, the JSON lexer produces JSON-specific
//! tokens only).
//!
//! ## Scanning the input
//! The scanner operates through maintaining a simple internal state:
//!
//! - A current position in the input
//! - An input buffer used to control pushbacks and lookaheads
//! - An accumulation buffer for gathering up characters
//!
//! A lexer simply pulls characters through the scanner (which adds positional information to each
//! one) and gathers them up within the accumulation buffer until it sees something that triggers
//! the parse of a valid token.
//!
//! Once the lexer is ready to consume all the content in the accumulation buffer, functions are
//! provided to extract the contents of the buffer in a number of formats (e.g. a string or char
//! array) and to then clear the buffer without resetting all the internal scanner state.
//!
//! A simple example of using the scanner is shown below:
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
//! // consume the first character from the scanner...
//! let first = scanner.advance(true);
//! assert!(first.is_ok());
//! assert_eq!(scanner.front().unwrap().ch, 'l');
//! assert_eq!(scanner.front().unwrap().coords.column, 1);
//!
//! // consume a second character
//! assert!(scanner.advance(true).is_ok());
//!
//! // ...and then pushback onto the buffer
//! scanner.pushback();
//!
//! // front of the buffer should still be 'l'
//! assert_eq!(scanner.front().unwrap().ch, 'l');
//!
//! // advance again - this time char will be taken from the pushback buffer
//! let _ = scanner.advance(true);
//! assert_eq!(scanner.front().unwrap().ch, 'e');
//!
//! // grab the contents of the buffer as a string
//! let buffer_contents= scanner.buffer_as_string_with_span();
//! assert_eq!(buffer_contents.str, String::from("le"));
//!
//! // reset the scanner and empty the buffer
//! scanner.clear();
//!
//! // buffer should now be empty
//! assert!(scanner.buffer_as_string_with_span().str.is_empty());
//!
//! // advance yet again
//! assert!(scanner.advance(true).is_ok());
//!
//! // the third character read will be from the 3rd column in the input
//! assert_eq!(scanner.front().unwrap().ch, 't');
//! assert_eq!(scanner.front().unwrap().coords.column, 3);
//!
//!
//! ```
//!
//! ## Lexers
//!
//! Within the current release, only a single lexer backend is implemented within this crate:
//!
//! ### JSON Lexer
//!
//!

//!

pub mod json;
pub mod scanner;
