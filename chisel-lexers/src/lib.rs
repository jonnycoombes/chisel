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
//! The scanner operates through maintaining an internal state:
//!
//! - A transient buffer of characters, paired with their coordinates in the input
//! - A transient pushback buffer, which can be used to temporarily push characters back into the input
//! - A single lookahead character which can be used for LA(1) operations by a lexer  
//! - A current position in the input
//! - A source of `char`s (typically a byte decoder of some description)
//!
//! At any point, a lexer can take a look at the front and the back of the buffer in order to decide
//! whether to either *advance*, *pushback* or *lookahead*:
//!
//! - *advance*  - add another character to the scanning buffer
//! - *pushback* - take the last read character from the buffer and add to the pushback buffer
//! - *lookahead* - read one character from the input, but don't add it to the buffer (yet)
//!
//! If a fault occurs during any of these operations, the scanner returns a `Err` containing
//! information about the nature of the fault and the location in the input where it occurred.
//!
//! After several advances, pushbacks lookaheads, a lexer can retrieve the contents of the buffer
//! in order to form a new token.
//!
//! ## Lexers
//!
//! ### JSON Lexer
//!
//!

pub mod json;
pub mod scanner;
