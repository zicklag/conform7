//! Classifying sentences — diagramming sentences via Preform matching.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Classifying Sentences.w`. It is
//! responsible for matching sentences against the Preform grammar to produce
//! VERB_NT sentence diagrams.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Classifying Sentences.w`

use conform7_syntax::parse_node::{Annotation, ParseNode};
use conform7_syntax::preform::{match_nonterminal_impl, InternalPayload, InternalRegistry, PreformContext};

/// Sentence classification — diagrams sentences via Preform matching.
pub struct Classifying;

impl Classifying {
    /// Diagram a sentence via Preform matching.
    ///
    /// Corresponds to `Classifying::sentence` in the C reference
    /// (`inform7/assertions-module/Chapter 2/Classifying Sentences.w`).
    ///
    /// This function:
    /// 1. Marks the sentence as classified
    /// 2. Calls `<sentence-without-occurrences>` Preform match
    /// 3. Grafts the resulting VERB_NT subtree onto the SENTENCE_NT
    pub fn sentence(node: &mut ParseNode) {
        // Mark as classified
        node.add_annotation(Annotation::Classified);

        // Match against <sentence-without-occurrences>
        // We need a PreformContext and InternalRegistry to match.
        // For now, this is a best-effort match — if the grammar isn't loaded
        // or the match fails, we simply mark the sentence as classified.
        let wording = node.wording();
        if wording.is_empty() {
            return;
        }

        // Try to match using the linguistics registry if available.
        // The registry is obtained from a global or passed-in context in the
        // full implementation. For now we create a fresh one.
        let registry = InternalRegistry::linguistics();

        // Build a minimal PreformContext. In the full implementation, the
        // grammar and word text would come from the compilation state.
        // For now, we use an empty grammar which will cause the match to
        // gracefully fail — the sentence is still marked as classified.
        let grammar = conform7_syntax::preform::Grammar::default();
        let word_text: Vec<&str> = Vec::new();
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: &word_text,
            is_paragraph_start: false,
            verbs_registry: None,
        };

        if let Some(result) = match_nonterminal_impl(&ctx, &registry, "sentence-without-occurrences", wording) {
            // Graft the VERB_NT subtree onto this SENTENCE_NT
            if let Some(internal) = result.internal {
                if let InternalPayload::ParseNode(verb_node) = internal.payload {
                    node.append_child(*verb_node);
                }
            }
        }
    }

    /// Check if a sentence is textual (just quoted text).
    ///
    /// A textual sentence contains only quoted text with no structural
    /// assertions.
    pub fn sentence_is_textual(node: &ParseNode) -> bool {
        // Simplified: check if the sentence is a single quoted text
        node.is_textual()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn sentence_marks_classified() {
        let mut node = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        Classifying::sentence(&mut node);
        assert!(
            node.annotations().iter().any(|a| matches!(a, Annotation::Classified)),
            "Sentence should be marked as classified"
        );
    }

    #[test]
    fn sentence_is_textual_returns_false_for_empty() {
        let node = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        assert!(!Classifying::sentence_is_textual(&node));
    }

    #[test]
    fn sentence_is_textual_returns_false_for_node_with_children() {
        let mut node = ParseNode::new(NodeType::Sentence, Wording::new(0, 5));
        // Add a child to make it non-textual (has children)
        let child = ParseNode::new(NodeType::Verb, Wording::new(0, 5));
        node.append_child(child);
        assert!(!Classifying::sentence_is_textual(&node));
    }

    #[test]
    fn sentence_is_textual_returns_true_for_plain_text() {
        let node = ParseNode::new(NodeType::Sentence, Wording::new(0, 5));
        // No children, non-empty wording → textual
        assert!(Classifying::sentence_is_textual(&node));
    }
}
