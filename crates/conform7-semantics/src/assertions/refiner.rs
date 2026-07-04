//! The Refiner — refine parse tree nodes for assertion processing.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`. It is the
//! tree-annotation step that refines `UNPARSED_NOUN_NT` nodes into typed
//! noun phrase nodes (`PROPER_NOUN_NT`, `COMMON_NOUN_NT`, `KIND_NT`, etc.)
//! with proper annotations (subject, evaluation, creation proposition).
//!
//! The Refiner is called from `MajorNodes::pass_1` on each coupling in the
//! syntax tree. It is the critical prerequisite for assertion processing.

use conform7_syntax::node_type::NodeType;
use conform7_syntax::parse_node::ParseNode;

/// The Refiner — refines parse tree nodes for assertion processing.
///
/// # References
///
/// - C reference: `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
pub struct Refiner;

impl Refiner {
    /// Refine a coupling (px, py) — the two sides of a verb phrase.
    ///
    /// This is the entry point called from `MajorNodes::pass_1`. It refines
    /// both sides of a coupling, removing "with" wrappers, performing
    /// with-surgery and and-surgery.
    ///
    /// # References
    ///
    /// - C reference: `Refiner::refine_coupling` in
    ///   `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
    pub fn refine_coupling(px: &mut ParseNode, py: &mut ParseNode, _now_negated: bool) {
        Self::un_with(px);
        Self::un_with(py);
        Self::refine(px);
        Self::refine(py);
        Self::with_surgery(px, py);
        Self::and_surgery(px, py);
    }

    /// Remove "with" wrappers from a node.
    ///
    /// If this node is a `WITH_NT` with a single child, replace it with the
    /// child. This unwraps trivial "with" nodes that don't represent a
    /// possessive relationship.
    ///
    /// # References
    ///
    /// - C reference: `Refiner::un_with` in
    ///   `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
    fn un_with(node: &mut ParseNode) {
        if node.node_type() == NodeType::With
            && node.child_count() == 1
        {
            node.replace_with_first_child();
        }
    }

    /// Merge possessive "with" into the parent.
    ///
    /// Deferred: full with-surgery is not yet implemented. This is a stub
    /// that will merge possessive "with" relationships into the parent node.
    ///
    /// # References
    ///
    /// - C reference: `Refiner::with_surgery` in
    ///   `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
    fn with_surgery(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: full with-surgery
    }

    /// Split "and" conjunctions.
    ///
    /// Deferred: full and-surgery is not yet implemented. This is a stub
    /// that will split "and" conjunctions into separate assertions.
    ///
    /// # References
    ///
    /// - C reference: `Refiner::and_surgery` in
    ///   `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
    fn and_surgery(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: full and-surgery
    }

    /// Refine a single node by its type.
    ///
    /// Dispatches on the node type to perform type-specific refinement:
    ///
    /// | Node Type | Action |
    /// |-----------|--------|
    /// | `With` | Recursive refine on children |
    /// | `And` | Recursive refine on children |
    /// | `XOfY` | Recursive refine on children |
    /// | `Relationship` | Recursive refine on children |
    /// | `Called` | Convert to `ProperNoun` |
    /// | `Kind` | Look up kind subject (stub) |
    /// | `Pronoun` | Anaphora lookup (stub) |
    /// | `UnparsedNoun` / `ProperNoun` / `CommonNoun` | Noun phrase resolution (stub) |
    ///
    /// # References
    ///
    /// - C reference: `Refiner::refine` in
    ///   `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
    pub fn refine(node: &mut ParseNode) {
        match node.node_type() {
            NodeType::With
            | NodeType::And
            | NodeType::XOfY
            | NodeType::Relationship => {
                // Recursive refine on children
                let mut children = node.take_children();
                for child in &mut children {
                    Self::refine(child.as_mut());
                }
                node.set_children(children);
            }
            NodeType::Called => {
                // Convert CALLED_NT to PROPER_NOUN_NT
                node.set_node_type(NodeType::ProperNoun);
            }
            NodeType::Kind => {
                // Look up kind subject — stub for now
                // Deferred: KindSubjects::kind_to_subject
            }
            NodeType::Pronoun => {
                // Anaphora lookup — stub for now
                // Deferred: Anaphora::get_current_subject
            }
            NodeType::UnparsedNoun | NodeType::ProperNoun | NodeType::CommonNoun => {
                // Noun phrase resolution — stub for now
                // Deferred: full noun phrase resolution
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::Wording;

    #[test]
    fn refine_coupling_does_not_panic() {
        let mut px = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        let mut py = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        Refiner::refine_coupling(&mut px, &mut py, false);
        // Should not panic
    }

    #[test]
    fn un_with_removes_single_child_with() {
        let mut node = ParseNode::new(NodeType::With, Wording::EMPTY);
        let child = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);
        node.append_child(child);
        Refiner::un_with(&mut node);
        assert_eq!(node.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn un_with_does_not_remove_multi_child_with() {
        let mut node = ParseNode::new(NodeType::With, Wording::EMPTY);
        let child1 = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);
        let child2 = ParseNode::new(NodeType::CommonNoun, Wording::EMPTY);
        node.append_child(child1);
        node.append_child(child2);
        Refiner::un_with(&mut node);
        // Should remain a With node since it has multiple children
        assert_eq!(node.node_type(), NodeType::With);
    }

    #[test]
    fn un_with_does_not_remove_non_with() {
        let mut node = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);
        Refiner::un_with(&mut node);
        assert_eq!(node.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_converts_called_to_proper_noun() {
        let mut node = ParseNode::new(NodeType::Called, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_does_not_panic_on_unknown_type() {
        let mut node = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        Refiner::refine(&mut node);
        // Should not panic
    }

    #[test]
    fn refine_recurses_into_with_children() {
        let mut node = ParseNode::new(NodeType::With, Wording::EMPTY);
        let mut child = ParseNode::new(NodeType::Called, Wording::EMPTY);
        child.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        node.append_child(child);
        Refiner::refine(&mut node);
        // The Called child should be converted to ProperNoun
        let child = node.children().next().expect("should have a child");
        assert_eq!(child.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_recurses_into_and_children() {
        let mut node = ParseNode::new(NodeType::And, Wording::EMPTY);
        let child1 = ParseNode::new(NodeType::Called, Wording::EMPTY);
        let child2 = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        node.append_child(child1);
        node.append_child(child2);
        Refiner::refine(&mut node);
        let mut children = node.children();
        let first = children.next().expect("should have first child");
        assert_eq!(first.node_type(), NodeType::ProperNoun);
        let second = children.next().expect("should have second child");
        assert_eq!(second.node_type(), NodeType::UnparsedNoun);
    }

    #[test]
    fn refine_recurses_into_x_of_y_children() {
        let mut node = ParseNode::new(NodeType::XOfY, Wording::EMPTY);
        let child = ParseNode::new(NodeType::Called, Wording::EMPTY);
        node.append_child(child);
        Refiner::refine(&mut node);
        let child = node.children().next().expect("should have a child");
        assert_eq!(child.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_recurses_into_relationship_children() {
        let mut node = ParseNode::new(NodeType::Relationship, Wording::EMPTY);
        let child = ParseNode::new(NodeType::Called, Wording::EMPTY);
        node.append_child(child);
        Refiner::refine(&mut node);
        let child = node.children().next().expect("should have a child");
        assert_eq!(child.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_kind_does_not_panic() {
        let mut node = ParseNode::new(NodeType::Kind, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::Kind);
    }

    #[test]
    fn refine_pronoun_does_not_panic() {
        let mut node = ParseNode::new(NodeType::Pronoun, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::Pronoun);
    }

    #[test]
    fn refine_unparsed_noun_does_not_panic() {
        let mut node = ParseNode::new(NodeType::UnparsedNoun, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
    }

    #[test]
    fn refine_proper_noun_does_not_panic() {
        let mut node = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::ProperNoun);
    }

    #[test]
    fn refine_common_noun_does_not_panic() {
        let mut node = ParseNode::new(NodeType::CommonNoun, Wording::EMPTY);
        Refiner::refine(&mut node);
        assert_eq!(node.node_type(), NodeType::CommonNoun);
    }

    #[test]
    fn refine_coupling_un_with_both_sides() {
        // Test that refine_coupling calls un_with on both sides
        let mut px = ParseNode::new(NodeType::With, Wording::EMPTY);
        px.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        let mut py = ParseNode::new(NodeType::With, Wording::EMPTY);
        py.append_child(ParseNode::new(NodeType::CommonNoun, Wording::EMPTY));
        Refiner::refine_coupling(&mut px, &mut py, false);
        assert_eq!(px.node_type(), NodeType::ProperNoun);
        assert_eq!(py.node_type(), NodeType::CommonNoun);
    }
}
