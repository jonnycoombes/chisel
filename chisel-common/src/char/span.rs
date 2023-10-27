use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use crate::char::coords::Coords;

/// A [Span] represents a linear interval within the parser input, between to different [Coords]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Span {
    /// Start [Coords] for the span
    pub start: Coords,
    /// End [Coords] for the span
    pub end: Coords,
}

impl Span {}

impl Eq for Span {}

impl PartialOrd<Self> for Span {
    /// The partial order is based on the start coordinates for a span
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.start.cmp(&other.start) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
}

impl Ord for Span {
    /// The total order for a span is based on the start coordinates of a span
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "start: {}, end: {}", self.start, self.end,)
    }
}
#[cfg(test)]
mod test {
    use crate::char::coords::Coords;
    use crate::char::span::Span;

    #[test]
    fn equality_between_defaults() {
        let s1 = Span::default();
        let s2 = Span::default();
        assert_eq!(s1, s2)
    }

    #[test]
    fn basic_ordering_should_work() {
        let mut s1 = Span::default();
        let s2 = Span::default();
        s1.start = Coords {
            line: 2,
            column: 1,
            absolute: 3,
        };
        assert!(s1 > s2)
    }
}
