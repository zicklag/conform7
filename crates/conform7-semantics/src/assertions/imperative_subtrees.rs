//! Imperative subtrees — processing imperative sentences.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Imperative Subtrees.w`. It handles
//! imperative sentences — sentences that describe actions or rules rather
//! than declarative assertions.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Imperative Subtrees.w`

use conform7_syntax::parse_node::ParseNode;

/// Imperative subtree processing.
pub struct ImperativeSubtrees;

impl ImperativeSubtrees {
    /// Accept an imperative subtree (stub).
    ///
    /// Called when an IMPERATIVE_NT node is encountered during the pre-pass.
    /// Processes the imperative block and registers it for later compilation.
    ///
    /// # References
    ///
    /// - C reference: `inform7/assertions-module/Chapter 2/Imperative Subtrees.w` —
    ///   `ImperativeSubtrees::accept`
    pub fn accept(_node: &mut ParseNode) {
        // Deferred: imperative block parsing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn accept_does_not_panic() {
        let mut node = ParseNode::new(NodeType::Imperative, Wording::EMPTY);
        ImperativeSubtrees::accept(&mut node);
        // Should not panic
    }
}
