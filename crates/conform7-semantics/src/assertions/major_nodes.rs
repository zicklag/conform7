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
use crate::assertions::assertions::Assertions;
use crate::assertions::bibliographic_data::BibliographicData;
use crate::assertions::classifying::Classifying;
use crate::assertions::equations::Equations;
use crate::assertions::imperative_subtrees::ImperativeSubtrees;
use crate::assertions::intervention_requests::InterventionRequests;
use crate::assertions::plugin_calls::PluginCalls;
use crate::assertions::property_sentences::PropertySentences;
use crate::assertions::refiner::Refiner;
use crate::assertions::special_meanings::SpecialMeanings;
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

    /// Pass 1 — process assertions.
    ///
    /// This is the second of three passes through the syntax tree. It
    /// corresponds to `MajorNodes::pass_1` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 2 of 3.w`).
    ///
    /// For each SENTENCE_NT, this pass:
    /// 1. Finds the VERB_NT child (the sentence diagram)
    /// 2. Extracts px (subject) and py (object) from the verb node
    /// 3. Calls Refiner::refine_coupling to refine both sides
    /// 4. Tries special meanings
    /// 5. Dispatches to Assertions::make_coupling
    pub fn pass_1(tree: &mut ParseNode) {
        tree.traverse_mut(&mut |node| {
            Self::visit(node, 1);
        });
    }

    /// Pass 2 — process remaining assertions.
    ///
    /// This is the third of three passes through the syntax tree. It
    /// corresponds to `MajorNodes::pass_2` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 3 of 3.w`).
    ///
    /// For each SENTENCE_NT, this pass:
    /// 1. Checks if the sentence is textual (just quoted text)
    /// 2. Textual → calls Assertions::make_appearance (stub)
    /// 3. Non-textual → extracts VERB_NT child, dispatches to assertion matrix
    ///    (skips refinement)
    ///
    /// Also handles INFORM6CODE_NT and BIBLIOGRAPHIC_NT nodes.
    pub fn pass_2(tree: &mut ParseNode) {
        tree.traverse_mut(&mut |node| {
            Self::visit(node, 2);
        });
        // Post-traversal: deduce object instance kinds (stub)
        // World::deduce_object_instance_kinds() — deferred
    }

    #[allow(clippy::collapsible_match)]
    fn visit(node: &mut ParseNode, pass: i32) {
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
                if pass == 0 {
                    // Pre-pass: diagram the sentence
                    Classifying::sentence(node);
                    PropertySentences::look_for_property_creation(node);
                    PluginCalls::new_assertion_notify(node);
                } else if pass == 1 {
                    // Pass 1: process assertions
                    Self::process_sentence(node);
                } else if pass == 2 {
                    // Pass 2: process remaining assertions
                    Self::process_sentence_pass_2(node);
                }
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
            NodeType::Inform6Code => {
                if pass == 2 {
                    InterventionRequests::make(node);
                }
            }
            NodeType::Bibliographic => {
                if pass == 2 {
                    BibliographicData::bibliographic_data(node);
                }
            }
            _ => {}
        }
    }

    /// Process a sentence node during pass 1.
    ///
    /// Extracts the verb phrase (VERB_NT child), refines the coupling,
    /// tries special meanings, and dispatches to the assertion matrix.
    fn process_sentence(node: &mut ParseNode) {
        // Find the VERB_NT child (the sentence diagram from pre_pass)
        let verb_node = match node.find_child_mut(NodeType::Verb) {
            Some(v) => v,
            None => return, // No verb diagram — not an assertion sentence
        };

        // Extract px (subject) and py (object) from the verb node's children.
        // The VERB_NT has children: [subject, object] where each is an
        // UNPARSED_NOUN_NT (or a more specific noun type after refinement).
        let mut children = verb_node.take_children();
        if children.len() < 2 {
            // Not enough children — restore and skip
            verb_node.set_children(children);
            return;
        }

        // Take px (first child = subject) and py (second child = object)
        let mut px = children.remove(0);
        let mut py = children.remove(0);

        // Restore remaining children (if any) to the verb node
        verb_node.set_children(children);

        // Refine the coupling
        Refiner::refine_coupling(&mut px, &mut py, false);

        // Try special meanings first
        if !SpecialMeanings::try_special_meaning(&mut px, &mut py) {
            // Dispatch to the assertion matrix
            Assertions::make_coupling(&mut px, &mut py);
        }
    }

    /// Process a sentence node during pass 2.
    ///
    /// Textual sentences call `Assertions::make_appearance` (stub).
    /// Non-textual sentences extract the VERB_NT child and dispatch to the
    /// assertion matrix without refinement.
    fn process_sentence_pass_2(node: &mut ParseNode) {
        if Classifying::sentence_is_textual(node) {
            // Textual sentence — make appearance assertion (stub)
            Assertions::make_appearance(node);
            return;
        }

        // Non-textual sentence: extract verb phrase and dispatch to matrix
        // (skip refinement — already done in pass 1)
        let verb_node = match node.find_child_mut(NodeType::Verb) {
            Some(v) => v,
            None => return,
        };

        let mut children = verb_node.take_children();
        if children.len() < 2 {
            verb_node.set_children(children);
            return;
        }

        let mut px = children.remove(0);
        let mut py = children.remove(0);
        verb_node.set_children(children);

        // Dispatch directly to the assertion matrix (skip refinement)
        Assertions::make_coupling_pass_2(&mut px, &mut py);
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
    fn pass_2_does_not_panic() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_sentence_node() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let sentence = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        tree.append_child(sentence);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_inform6_code() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let code = ParseNode::new(NodeType::Inform6Code, Wording::EMPTY);
        tree.append_child(code);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_bibliographic() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let bib = ParseNode::new(NodeType::Bibliographic, Wording::EMPTY);
        tree.append_child(bib);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_textual_sentence() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        // A textual sentence has no children and non-empty wording
        let sentence = ParseNode::new(NodeType::Sentence, Wording::new(0, 5));
        tree.append_child(sentence);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_non_textual_sentence_with_verb() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let mut sentence = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        // Add a verb child with subject and object
        let mut verb = ParseNode::new(NodeType::Verb, Wording::EMPTY);
        verb.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        verb.append_child(ParseNode::new(NodeType::CommonNoun, Wording::EMPTY));
        sentence.append_child(verb);
        tree.append_child(sentence);
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }

    #[test]
    fn pass_2_handles_all_node_types() {
        let mut tree = ParseNode::new(NodeType::Root, Wording::EMPTY);
        tree.append_child(ParseNode::new(NodeType::Heading, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::BeginHere, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::EndHere, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::DefnCont, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Imperative, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Table, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Equation, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Trace, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Inform6Code, Wording::EMPTY));
        tree.append_child(ParseNode::new(NodeType::Bibliographic, Wording::EMPTY));
        MajorNodes::pass_2(&mut tree);
        // Should not panic
    }
}
