//! Major nodes — the three-pass traversal through the syntax tree.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Major Nodes.w`. It is the central
//! dispatch for processing assertion sentences — the sentences that describe
//! the model world in Inform 7 source text.
//!
//! The three passes are:
//!
//! 1. **pre_pass** — diagram sentences via Preform matching (this plan).
//! 2. **pass_1** — process assertions (deferred).
//! 3. **pass_2** — process remaining assertions (deferred).
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Major Nodes.w`
//! - C reference: `inform7/core-module/Chapter 1/Pass 1 of 3.w`
//! - C reference: `inform7/core-module/Chapter 1/Pass 2 of 3.w`
//! - C reference: `inform7/core-module/Chapter 1/Pass 3 of 3.w`

use conform7_syntax::node_type::NodeType;
use conform7_syntax::parse_node::ParseNode;

use crate::assertions::anaphora::Anaphora;
use crate::assertions::classifying::Classifying;
use crate::assertions::equations::Equations;
use crate::assertions::imperative_subtrees::ImperativeSubtrees;
use crate::assertions::plugin_calls::PluginCalls;
use crate::assertions::property_sentences::PropertySentences;
use crate::assertions::tables::Tables;

/// The major nodes orchestrator.
///
/// Traverses the syntax tree and dispatches each node to the appropriate
/// handler based on its node type.
pub struct MajorNodes;

impl MajorNodes {
    /// Pre-pass through major nodes — diagram sentences via Preform.
    ///
    /// This is the first of three passes through the syntax tree. It
    /// corresponds to `MajorNodes::pre_pass` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 1 of 3.w`).
    pub fn pre_pass(tree: &mut ParseNode) {
        tree.traverse_mut(&mut |node| {
            Self::visit(node, 0);
        });
    }

    /// Pass 1 — process assertions (stub).
    ///
    /// Deferred: needs Refiner, Assertions matrix.
    pub fn pass_1(_tree: &mut ParseNode) {
        // Deferred: assertion processing
    }

    /// Pass 2 — process remaining assertions (stub).
    ///
    /// Deferred: needs Assertions matrix, World model.
    pub fn pass_2(_tree: &mut ParseNode) {
        // Deferred: remaining assertion processing
    }

    fn visit(node: &mut ParseNode, _pass: i32) {
        match node.node_type() {
            NodeType::Root => {}
            NodeType::Heading => {
                Anaphora::new_discussion();
            }
            NodeType::BeginHere => {
                Anaphora::new_discussion();
                // Extension boundary tracking deferred
            }
            NodeType::EndHere => {
                Anaphora::new_discussion();
                // Extension boundary tracking deferred
            }
            NodeType::DefnCont => {}
            NodeType::Sentence => {
                Classifying::sentence(node);
                PropertySentences::look_for_property_creation(node);
                PluginCalls::new_assertion_notify(node);
            }
            NodeType::Imperative => {
                ImperativeSubtrees::accept(node);
            }
            NodeType::Table => {
                Tables::create_table(node);
            }
            NodeType::Equation => {
                Equations::new_at(node, false);
            }
            NodeType::Trace => {
                // Toggle trace — deferred
            }
            NodeType::Inform6Code => {}
            NodeType::Bibliographic => {}
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn pre_pass_does_not_panic() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        MajorNodes::pre_pass(&mut tree);
        // Should not panic
    }

    #[test]
    fn pre_pass_visits_sentence_nodes() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let sentence = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        tree.append_child(sentence);
        MajorNodes::pre_pass(&mut tree);
        // Sentence should be marked as classified
        let classified = tree.children().any(|child| {
            child.annotations().iter().any(|a| matches!(a, conform7_syntax::parse_node::Annotation::Classified))
        });
        assert!(classified, "Sentence should be marked as classified after pre_pass");
    }

    #[test]
    fn pre_pass_handles_heading() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let heading = ParseNode::new(NodeType::Heading, Wording::EMPTY);
        tree.append_child(heading);
        MajorNodes::pre_pass(&mut tree);
        // Should not panic
    }

    #[test]
    fn pre_pass_handles_imperative() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let imperative = ParseNode::new(NodeType::Imperative, Wording::EMPTY);
        tree.append_child(imperative);
        MajorNodes::pre_pass(&mut tree);
        // Should not panic
    }

    #[test]
    fn pre_pass_handles_table() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let table = ParseNode::new(NodeType::Table, Wording::EMPTY);
        tree.append_child(table);
        MajorNodes::pre_pass(&mut tree);
        // Should not panic
    }

    #[test]
    fn pre_pass_handles_equation() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let equation = ParseNode::new(NodeType::Equation, Wording::EMPTY);
        tree.append_child(equation);
        MajorNodes::pre_pass(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_1_stub_does_not_panic() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        MajorNodes::pass_1(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_stub_does_not_panic() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }
}
