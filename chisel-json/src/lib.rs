//! ## Another JSON Parser?
//!
//! The Chisel JSON parser aims to be a relatively simple DOM and SAX parser for JSON, that does
//! *not include* all the machinery required to support explicit serialisation from, and
//! deserialisation into `structs`/`enums` within Rust.
//!
//! It's a simple little parser that is intended to allow you to choose how you want to parse a lump of *cursed* JSON,
//! and then either build/transform a DOM into a richer AST structure, or alternatively just cherry-pick the useful
//! bits of the payload via closures which are called in response to SAX parsing events.
//!
//! (*Because let's face it, JSON payloads usually come burdened with a whole load of unnecessary crap that
//! you'll never use*).
//!
#![allow(unused_imports)]
#![allow(dead_code)]

extern crate core;

use std::borrow::Cow;
use std::collections::HashMap;

pub mod lexer;
pub mod parsers;
pub mod results;
mod test_macros;

/// Structure representing a JSON key value pair
#[derive(Debug)]
pub struct JsonKeyValue<'a> {
    /// The key for the pair
    pub key: String,
    /// The JSON value
    pub value: JsonValue<'a>,
}

/// Basic enumeration of different Json values
#[derive(Debug)]
pub enum JsonValue<'a> {
    /// Map of values
    Object(Vec<JsonKeyValue<'a>>),
    /// Array of values
    Array(Vec<JsonValue<'a>>),
    /// Canonical string value
    String(Cow<'a, str>),
    /// Floating point numeric value
    Float(f64),
    /// Integer numeric value
    Integer(i64),
    /// Canonical boolean value
    Boolean(bool),
    /// Canonical null value
    Null,
}
