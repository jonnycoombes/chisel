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
    /// Create a new coordinate based on a coordinate reference
    pub fn from_coords(value: &Coords) -> Self {
        Coords {
            line: value.line,
            column: value.column,
            absolute: value.absolute,
        }
    }

    /// Create a new coordinate starting on a specific line
    pub fn from_line(line: usize) -> Self {
        Coords {
            absolute: 0,
            line,
            column: 0,
        }
    }

    /// Check whether this coordinate is strictly *before* another coordinate
    pub fn is_before(&self, other: &Coords) -> bool {
        self < other
    }

    /// Check whether this coordinate is strictly *after* another coordinate
    pub fn is_after(&self, other: &Coords) -> bool {
        self > other
    }

    /// Take all the values of the supplied [Coords] instance
    #[inline]
    pub fn copy_from(&mut self, other: &Coords) {
        self.line = other.line;
        self.column = other.column;
        self.absolute = other.absolute;
    }

    /// In place increment
    #[inline]
    pub fn increment(&mut self) {
        self.column += 1;
        self.absolute += 1;
    }

    /// In place decrement
    #[inline]
    pub fn decrement(&mut self) {
        self.column -= 1;
        self.absolute -= 1;
    }

    /// In place increment with a line return
    #[inline]
    pub fn increment_newline(&mut self) {
        self.column = 0;
        self.line += 1;
        self.absolute += 1;
    }

    /// Increment the coordinate within the current line and return new struct
    #[inline]
    pub fn copy_increment(&self) -> Self {
        Coords {
            line: self.line,
            column: self.column + 1,
            absolute: self.absolute + 1,
        }
    }

    /// Increment the coordinates and bump the line number (assumes that the new line will start
    /// in column zero) and return new struct
    #[inline]
    pub fn copy_increment_newline(&self) -> Self {
        Coords {
            line: self.line + 1,
            column: 1,
            absolute: self.absolute + 1,
        }
    }

    /// Decrement the coordinate within the current line, but panic if we try and decrement
    /// column or absolute below zero, return a new struct
    #[inline]
    pub fn copy_decrement(&mut self) -> Self {
        Coords {
            line: self.line,
            column: self.column - 1,
            absolute: self.absolute - 1,
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
        let c1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let c2 = Coords {
            line: 1,
            column: 12,
            absolute: 12,
        };
        assert!(c1.is_before(&c2));
        assert!(!c2.is_before(&c1));
    }

    #[test]
    fn is_after_behaves_correctly() {
        let c1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let c2 = Coords {
            line: 1,
            column: 12,
            absolute: 12,
        };
        assert!(!c1.is_after(&c2));
        assert!(c2.is_after(&c1))
    }

    #[test]
    fn equality_between_coords() {
        let c1 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        let c2 = Coords {
            line: 1,
            column: 1,
            absolute: 1,
        };
        assert_eq!(c1, c2)
    }

    #[test]
    fn inc_works() {
        let mut c1 = Coords::from_line(1);
        for _ in 1..=5 {
            c1 = c1.copy_increment();
        }
        assert_eq!(
            c1,
            Coords {
                line: 1,
                column: 5,
                absolute: 5
            }
        )
    }

    #[test]
    fn inc_newline_works() {
        let mut c1 = Coords::default();
        let mut c2 = Coords::default();
        assert_eq!(c1, c2);
        c1 = c1.copy_increment_newline();
        assert!(c1 > c2);
        c2 = c2.copy_increment_newline();
        assert_eq!(c1, c2)
    }

    #[test]
    fn dec_works() {
        let mut c1 = Coords::from_line(1);
        for _ in 1..=5 {
            c1.increment();
        }
        for _ in 1..=3 {
            c1.decrement()
        }
        assert_eq!(
            c1,
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
        let mut c1 = Coords::from_line(1);
        for _ in 1..=5 {
            c1.increment();
        }
        for _ in 1..=6 {
            c1.decrement()
        }
    }

    #[test]
    fn merge_should_take_all_values_from_source() {
        let c1 = Coords::default().copy_increment();
        let mut c2 = Coords::default();
        c2.copy_from(&c1);
        assert_eq!(c1, c2)
    }
}
