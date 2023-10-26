#![allow(dead_code)]
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// A [Coords] represents a single location within the parser input
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coords {
    /// The absolute character position
    pub absolute: usize,
    /// The row position
    pub line: usize,
    /// The column position
    pub column: usize,
}

/// Associated functions for the [Coords] struct
impl Coords {
    /// Create a new coordinate starting on a specific line
    fn from_line(line: usize) -> Self {
        Coords {
            absolute: 0,
            line,
            column: 0,
        }
    }

    /// Check whether this coordinate is strictly *before* another coordinate
    fn is_before(&self, other: &Coords) -> bool {
        self < other
    }

    /// Check whether this coordinate is strictly *after* another coordinate
    fn is_after(&self, other: &Coords) -> bool {
        self > other
    }

    /// Increment the coordinate within the current line
    fn inc(&mut self) {
        self.column += 1;
        self.absolute += 1;
    }

    /// Decrement the coordinate within the current line, but panic if we try and decrement
    /// column or absolute below zero
    fn dec(&mut self) {
        self.column -= 1;
        self.absolute -= 1;
        if self.column < 0 || self.absolute < 0 {
            panic!("column out of bounds")
        }
    }
}

impl Display for Coords {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[abs: {}, line: {}, column: {}]",
            self.absolute, self.line, self.column
        )
    }
}

impl Default for Coords {
    /// The default set of coordinates are positioned at the start of the first row
    fn default() -> Self {
        Coords {
            absolute: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Eq for Coords {}

impl PartialOrd<Self> for Coords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.absolute.cmp(&other.absolute) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => Some(Ordering::Equal),
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
}

impl Ord for Coords {
    fn cmp(&self, other: &Self) -> Ordering {
        self.absolute.cmp(&other.absolute)
    }
}

#[cfg(test)]
mod test {
    use crate::char::coords::Coords;

    #[test]
    fn is_before_behaves_as_you_would_expect() {
        let coord1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let coord2 = Coords {
            line: 1,
            column: 12,
            absolute: 12,
        };
        assert!(coord1.is_before(&coord2));
        assert!(!coord2.is_before(&coord1));
    }

    #[test]
    fn is_after_behaves_correctly() {
        let coord1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let coord2 = Coords {
            line: 1,
            column: 12,
            absolute: 12,
        };
        assert!(!coord1.is_after(&coord2));
        assert!(coord2.is_after(&coord1))
    }

    #[test]
    fn equality_between_coords() {
        let coord1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let coord2 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        assert_eq!(coord1, coord2)
    }

    #[test]
    fn inc_works() {
        let mut coord1 = Coords::from_line(1);
        for _ in 1..=5 {
            coord1.inc();
        }
        assert_eq!(
            coord1,
            Coords {
                line: 1,
                column: 5,
                absolute: 5
            }
        )
    }

    #[test]
    fn dec_works() {
        let mut coord1 = Coords::from_line(1);
        for _ in 1..=5 {
            coord1.inc();
        }
        for _ in 1..=3 {
            coord1.dec()
        }
        assert_eq!(
            coord1,
            Coords {
                line: 1,
                column: 2,
                absolute: 2
            }
        )
    }

    #[test]
    #[should_panic]
    fn dec_panics() {
        let mut coord1 = Coords::from_line(1);
        for _ in 1..=5 {
            coord1.inc();
        }
        for _ in 1..=6 {
            coord1.dec()
        }
    }
}
