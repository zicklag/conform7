//! Lightweight source text ranges.
//!
//! In Inform, a *wording* is a reference to a contiguous range of words in the
//! source text. It is not a copy of the text: it is a pair of indices into the
//! token stream produced by the lexer.
//!
//! Wordings are used throughout the syntax tree: every [`ParseNode`] carries a
//! wording describing which source tokens it interprets. Keeping wordings as
//! small index ranges makes parse nodes cheap to create, move, and copy.
//!
//! This corresponds to the C `wording` type in
//! `services/words-module/Chapter 3/Wordings.w` and the `text_parsed` field of
//! `parse_node` in `services/syntax-module/Chapter 2/Parse Nodes.w`.

use std::ops::Range;

/// A range of token indices into the source text.
///
/// `start` is inclusive and `end` is exclusive, matching Rust's `Range` conventions.
/// An empty wording has `start == end`.
///
/// # Examples
///
/// ```
/// use conform7_syntax::Wording;
///
/// let w = Wording::new(2, 5);
/// assert_eq!(w.len(), 3);
/// assert_eq!(w.as_range(), 2..5);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Wording {
    /// First token index included in the range.
    pub start: u32,
    /// First token index after the range (exclusive).
    pub end: u32,
}

impl Wording {
    /// An empty wording referencing no tokens.
    pub const EMPTY: Wording = Wording { start: 0, end: 0 };

    /// Create a new wording from token index bounds.
    pub const fn new(start: u32, end: u32) -> Wording {
        Wording { start, end }
    }

    /// Create a wording covering a single token index.
    pub const fn single(index: u32) -> Wording {
        Wording { start: index, end: index + 1 }
    }

    /// Convert to a Rust `Range<usize>` for indexing into a token slice.
    pub fn as_range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    /// Number of tokens in the range.
    pub fn len(self) -> usize {
        if self.end >= self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }

    /// True if the range contains no tokens.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// True if this wording contains `other`.
    pub fn contains(self, other: Wording) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Extend this wording to include `other`.
    pub fn union(self, other: Wording) -> Wording {
        Wording {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_wording() {
        assert!(Wording::EMPTY.is_empty());
        assert_eq!(Wording::EMPTY.len(), 0);
    }

    #[test]
    fn test_new_and_len() {
        let w = Wording::new(2, 5);
        assert_eq!(w.len(), 3);
        assert_eq!(w.as_range(), 2..5);
    }

    #[test]
    fn test_single() {
        let w = Wording::single(7);
        assert_eq!(w.len(), 1);
        assert_eq!(w.as_range(), 7..8);
    }

    #[test]
    fn test_contains() {
        let outer = Wording::new(1, 10);
        let inner = Wording::new(3, 5);
        assert!(outer.contains(inner));
        assert!(!inner.contains(outer));
    }

    #[test]
    fn test_union() {
        let a = Wording::new(1, 3);
        let b = Wording::new(5, 7);
        assert_eq!(a.union(b), Wording::new(1, 7));
    }

    #[test]
    fn test_reversed_bounds_give_zero_len() {
        // Defensive: even if somehow created with reversed bounds, len is 0.
        let w = Wording::new(5, 2);
        assert_eq!(w.len(), 0);
    }
}
