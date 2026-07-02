//! Word assemblage type for multi-word texts.
//!
//! A `WordAssemblage` represents a multi-word text (e.g., "carry out" for a
//! phrasal verb, "in front of" for a preposition). It is used throughout the
//! verb system for verb texts, preposition texts, and reference texts.
//!
//! # References
//!
//! - C reference: `services/linguistics-module/Chapter 3/Verbs.w` — the
//!   `word_assemblage` type used for verb and preposition texts.
//! - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
//!   `word_assemblage` used for preposition texts.

use std::fmt;

/// A multi-word text, such as a verb phrase or preposition.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` — the
///   `word_assemblage` type.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct WordAssemblage {
    /// The words that make up this assemblage.
    pub words: Vec<String>,
}

impl WordAssemblage {
    /// Create a new word assemblage from a vector of words.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::new`.
    pub fn new(words: Vec<String>) -> Self {
        WordAssemblage { words }
    }

    /// Create an empty word assemblage (zero words).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::lit_0`.
    pub fn lit_0() -> Self {
        WordAssemblage { words: Vec::new() }
    }

    /// Create a word assemblage from a single word.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::lit_1`.
    pub fn lit_1(word: &str) -> Self {
        WordAssemblage {
            words: vec![word.to_string()],
        }
    }

    /// Join two word assemblages into one.
    ///
    /// The words of `a` come first, followed by the words of `b`.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::join`.
    pub fn join(a: &WordAssemblage, b: &WordAssemblage) -> Self {
        let mut words = a.words.clone();
        words.extend(b.words.iter().cloned());
        WordAssemblage { words }
    }

    /// Get the first word of this assemblage, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::first_word`.
    pub fn first_word(&self) -> Option<&str> {
        self.words.first().map(|s| s.as_str())
    }

    /// Get the number of words in this assemblage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::length`.
    pub fn length(&self) -> usize {
        self.words.len()
    }

    /// Check if this assemblage is non-empty.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::nonempty`.
    pub fn nonempty(&self) -> bool {
        !self.words.is_empty()
    }

    /// Check if this assemblage equals another by comparing words.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `WordAssemblages::eq`.
    #[allow(clippy::should_implement_trait)]
    pub fn eq(&self, other: &WordAssemblage) -> bool {
        self.words == other.words
    }
}

impl fmt::Display for WordAssemblage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, word) in self.words.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", word)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let wa = WordAssemblage::new(vec![]);
        assert_eq!(wa.length(), 0);
        assert!(!wa.nonempty());
        assert!(wa.first_word().is_none());
    }

    #[test]
    fn test_lit_0() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let wa = WordAssemblage::lit_0();
        assert_eq!(wa.length(), 0);
        assert!(!wa.nonempty());
    }

    #[test]
    fn test_lit_1() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let wa = WordAssemblage::lit_1("hello");
        assert_eq!(wa.length(), 1);
        assert!(wa.nonempty());
        assert_eq!(wa.first_word(), Some("hello"));
    }

    #[test]
    fn test_join() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let a = WordAssemblage::lit_1("carry");
        let b = WordAssemblage::lit_1("out");
        let joined = WordAssemblage::join(&a, &b);
        assert_eq!(joined.length(), 2);
        assert_eq!(joined.first_word(), Some("carry"));
        assert_eq!(joined.words, vec!["carry", "out"]);
    }

    #[test]
    fn test_join_multiple() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let a = WordAssemblage::new(vec!["in".to_string(), "front".to_string()]);
        let b = WordAssemblage::lit_1("of");
        let joined = WordAssemblage::join(&a, &b);
        assert_eq!(joined.length(), 3);
        assert_eq!(joined.words, vec!["in", "front", "of"]);
    }

    #[test]
    fn test_eq() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let a = WordAssemblage::new(vec!["hello".to_string(), "world".to_string()]);
        let b = WordAssemblage::new(vec!["hello".to_string(), "world".to_string()]);
        let c = WordAssemblage::new(vec!["hello".to_string()]);
        assert!(a.eq(&b));
        assert!(!a.eq(&c));
    }

    #[test]
    fn test_display() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let wa = WordAssemblage::new(vec!["carry".to_string(), "out".to_string()]);
        assert_eq!(format!("{}", wa), "carry out");
    }

    #[test]
    fn test_display_single() {
        let wa = WordAssemblage::lit_1("hello");
        assert_eq!(format!("{}", wa), "hello");
    }

    #[test]
    fn test_display_empty() {
        let wa = WordAssemblage::lit_0();
        assert_eq!(format!("{}", wa), "");
    }

    #[test]
    fn test_first_word_empty() {
        let wa = WordAssemblage::lit_0();
        assert_eq!(wa.first_word(), None);
    }

    #[test]
    fn test_nonempty() {
        assert!(!WordAssemblage::lit_0().nonempty());
        assert!(WordAssemblage::lit_1("x").nonempty());
    }
}
