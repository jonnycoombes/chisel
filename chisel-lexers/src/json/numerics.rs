/// A lazy numeric type which allows for raw bytes associated with a floating point value
/// to be stashed away, and then only parsed when needed.  Structs of this type may be returned
/// by the lexer if the associated feature is enabled
#[derive(Debug, Clone, PartialEq)]
pub struct LazyNumeric {
    /// Raw bytes associated with the value
    raw: Vec<u8>,
}

impl LazyNumeric {
    /// Create a new struct given a raw byte slice
    pub fn new(raw: &[u8]) -> Self {
        LazyNumeric {
            raw: Vec::from(raw),
        }
    }

    /// Convenience method for conversion into an [f64]
    pub fn to_float(&self) -> f64 {
        fast_float::parse(self.raw.as_slice()).unwrap()
    }
}

impl From<&LazyNumeric> for f64 {
    fn from(value: &LazyNumeric) -> Self {
        fast_float::parse(value.raw.as_slice()).unwrap()
    }
}

impl From<LazyNumeric> for f64 {
    fn from(value: LazyNumeric) -> Self {
        fast_float::parse(value.raw.as_slice()).unwrap()
    }
}
