//! Common types used throughout the rest of the crate
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

/// General result type used by a decoder instance
pub type DecoderResult<T> = Result<T, DecoderError>;

/// Enumeration of different decoder errors
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DecoderErrorCode {
    /// Something went pear-shaped in the underlying stream
    StreamFailure,
    /// Detected an invalid byte sequence
    InvalidByteSequence,
    /// Out of range error
    OutOfRange,
    /// The end of the input has been reached
    EndOfInput,
}

/// Structure for encoding errors
#[derive(Debug, Clone)]
pub struct DecoderError {
    /// The error code
    pub code: DecoderErrorCode,

    /// Associated error message
    pub message: Cow<'static, str>,
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Code: {:?}, Message: {}", self.code, self.message)
    }
}

/// Helper macro for generating errors
#[macro_export]
macro_rules! decoder_error {
    ($code : expr, $msg : expr) => {
        DecoderError {
            code: $code,
            message: $msg.into(),
        }
    };
}

#[macro_export]
macro_rules! end_of_input {
    () => {
        Err(DecoderError {
            code: DecoderErrorCode::EndOfInput,
            message: "end of input reached".into(),
        })
    };
}

#[macro_export]
macro_rules! invalid_byte_sequence {
    () => {
        Err(DecoderError {
            code: DecoderErrorCode::InvalidByteSequence,
            message: "invalid byte sequence".into(),
        })
    };
}
