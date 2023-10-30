use chisel_lexers::json::numerics::LazyNumeric;
use std::borrow::Cow;
use std::fmt::Debug;

/// The JSON DOM parser
pub mod dom;

pub mod events;
/// The JSON SAX parser
pub mod sax;
#[cfg(test)]
pub(crate) mod specs;

/// Enumeration of possible numeric types. Lazy numerics will be returned by the lexer backend if
/// the associated feature is enabled, otherwise either floats or integer numerics are spat out
#[derive(Debug, Clone)]
pub enum JsonNumeric {
    Float(f64),
    Integer(i64),
    Lazy(LazyNumeric),
}

/// Structure representing a JSON key value pair
#[derive(Debug, Clone)]
pub struct JsonKeyValue<'a> {
    /// The key for the pair
    pub key: String,
    /// The JSON value
    pub value: JsonValue<'a>,
}

/// Basic enumeration of different Json values
#[derive(Debug, Clone)]
pub enum JsonValue<'a> {
    /// Map of values
    Object(Vec<JsonKeyValue<'a>>),
    /// Array of values
    Array(Vec<JsonValue<'a>>),
    /// Canonical string value
    String(Cow<'a, str>),
    /// Number value which will be a member of the union [JsonNumeric]
    Number(JsonNumeric),
    /// Floating point numeric value
    Float(f64),
    /// Integer numeric value
    Integer(i64),
    /// Canonical boolean value
    Boolean(bool),
    /// Canonical null value
    Null,
}
