use crate::char::coords::Coords;
use std::fmt::{Display, Formatter};

/// A [Span] represents a linear interval within the parser input, between to different [Coords]
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
pub struct Span {
    /// Start [Coords] for the span
    pub start: Coords,
    /// End [Coords] for the span
    pub end: Coords,
}

impl Span {}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "start: {}, end: {}", self.start, self.end,)
    }
}
