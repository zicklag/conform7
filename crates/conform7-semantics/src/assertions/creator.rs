//! The Creator — consult the creator for object/kind creation.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 4/The Creator.w`. It is responsible
//! for creating objects and kinds when an assertion sentence describes
//! something that doesn't yet exist in the model world.
//!
//! Currently a stub — full implementation is deferred.

use conform7_syntax::parse_node::ParseNode;

/// The Creator — consults the creator for object/kind creation.
///
/// # References
///
/// - C reference: `inform7/assertions-module/Chapter 4/The Creator.w`
pub struct Creator;

impl Creator {
    /// Consult the creator for a coupling (stub).
    ///
    /// In the full implementation, this creates objects or kinds when the
    /// assertion describes something that doesn't yet exist. For now, it is
    /// a no-op.
    ///
    /// # References
    ///
    /// - C reference: `Creator::consult_the_creator` in
    ///   `inform7/assertions-module/Chapter 4/The Creator.w`
    pub fn consult_the_creator(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: object/kind creation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::Wording;

    #[test]
    fn consult_the_creator_does_not_panic() {
        let mut px = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        let mut py = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        Creator::consult_the_creator(&mut px, &mut py);
        // Should not panic — stub is a no-op
    }
}
