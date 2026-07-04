//! Property sentences — detecting property creation in sentences.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Property Sentences.w`. It detects
//! when a sentence creates a new property (e.g., "A thing has a number
//! called the score.").
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Property Sentences.w`

use conform7_syntax::parse_node::ParseNode;

/// Property sentence detection.
pub struct PropertySentences;

impl PropertySentences {
    /// Look for property creation in a sentence (stub).
    ///
    /// Called after a sentence has been classified and diagrammed during the
    /// pre-pass. Checks if the sentence creates a new property and registers
    /// it if so.
    ///
    /// # References
    ///
    /// - C reference: `inform7/assertions-module/Chapter 2/Property Sentences.w` —
    ///   `PropertySentences::look_for_property_creation`
    pub fn look_for_property_creation(_node: &mut ParseNode) {
        // Deferred: property creation detection
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn look_for_property_creation_does_not_panic() {
        let mut node = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        PropertySentences::look_for_property_creation(&mut node);
        // Should not panic
    }
}
