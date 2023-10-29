use std::borrow::Cow;

/// The JSON DOM parser
pub mod dom;

pub mod events;
/// The JSON SAX parser
pub mod sax;
#[cfg(test)]
pub(crate) mod specs;

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
