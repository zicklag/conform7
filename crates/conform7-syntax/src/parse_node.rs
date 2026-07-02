//! Inform 7 parse tree nodes.
//!
//! A parse tree is built from [`ParseNode`] values. Each node represents one way
//! to understand a piece of source text and stores:
//!
//! - a [`Wording`] referencing the tokens it interprets;
//! - a [`NodeType`] saying what kind of node it is;
//! - a `down` pointer to its first child;
//! - a `next` pointer to its next sibling;
//! - a `next_alternative` pointer to an alternative interpretation.
//!
//! This corresponds directly to the C `parse_node` struct in
//! `services/syntax-module/Chapter 2/Parse Nodes.w`.
//!
//! The design deliberately uses `Box` for child ownership, which is simple and
//! idiomatic for a foundation. A future performance pass may switch to an
//! arena allocator (see `plans/FUTURE-PERF.md`).

use crate::{HeadingLevel, NodeType, Wording};
use std::fmt;

/// Annotation attached to a parse node.
///
/// In C, annotations are a flexible key/value system (see
/// `services/syntax-module/Chapter 2/Node Annotations.w`). We start with a
/// small closed enum and grow it as needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Annotation {
    /// The heading level of a `HEADING_NT` node.
    HeadingLevel(HeadingLevel),
    /// Article usage annotation for noun phrase nodes.
    ArticleUsage(crate::linguistics::ArticleUsage),
    /// Verbal certainty level (from certainty adverbs).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `verbal_certainty_ANNOT` annotation.
    VerbalCertainty(i32),
    /// Whether the sentence is existential ("There is ...").
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `sentence_is_existential_ANNOT` annotation.
    SentenceIsExistential(bool),
    /// Linguistic error annotation.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `linguistic_error_here_ANNOT` annotation.
    LinguisticErrorHere(i32),
}

/// A single node in an Inform 7 syntax tree.
///
/// Every node carries a wording (the source tokens it interprets) and a node
/// type (what it represents). Child nodes are linked through `down` (first
/// child) and `next` (sibling). Multiple interpretations of the same text can
/// be linked through `next_alternative`.
///
/// # Examples
///
/// ```
/// use conform7_syntax::{ParseNode, NodeType, Wording};
///
/// let mut root = ParseNode::new(NodeType::Root, Wording::EMPTY);
/// let heading = ParseNode::new(NodeType::Heading, Wording::new(0, 3));
/// root.append_child(heading);
/// assert_eq!(root.children().count(), 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseNode {
    node_type: NodeType,
    wording: Wording,
    annotations: Vec<Annotation>,
    down: Option<Box<ParseNode>>,
    next: Option<Box<ParseNode>>,
    next_alternative: Option<Box<ParseNode>>,
}

impl ParseNode {
    /// Create a new, empty node of the given type.
    pub fn new(node_type: NodeType, wording: Wording) -> ParseNode {
        ParseNode {
            node_type,
            wording,
            annotations: Vec::new(),
            down: None,
            next: None,
            next_alternative: None,
        }
    }

    /// Return the node's type.
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    /// Set the node's type.
    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    /// Return the wording (source token range) attached to this node.
    pub fn wording(&self) -> Wording {
        self.wording
    }

    /// Set the wording attached to this node.
    pub fn set_wording(&mut self, wording: Wording) {
        self.wording = wording;
    }

    /// Attach an annotation to this node.
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }
    /// Return the heading level annotation, if any.
    pub fn heading_level(&self) -> Option<HeadingLevel> {
        self.annotations.iter().filter_map(|a| match a {
            Annotation::HeadingLevel(level) => Some(*level),
            _ => None,
        }).next()
    }

    /// Return the article usage annotation, if any.
    pub fn article_usage(&self) -> Option<&crate::linguistics::ArticleUsage> {
        self.annotations.iter().filter_map(|a| match a {
            Annotation::ArticleUsage(usage) => Some(usage),
            _ => None,
        }).next()
    }

    /// Return the verbal certainty annotation, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn verbal_certainty(&self) -> Option<i32> {
        self.annotations.iter().filter_map(|a| match a {
            Annotation::VerbalCertainty(level) => Some(*level),
            _ => None,
        }).next()
    }

    /// Return the existential sentence annotation, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn is_existential(&self) -> Option<bool> {
        self.annotations.iter().filter_map(|a| match a {
            Annotation::SentenceIsExistential(val) => Some(*val),
            _ => None,
        }).next()
    }

    /// Return the linguistic error annotation, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn linguistic_error(&self) -> Option<i32> {
        self.annotations.iter().filter_map(|a| match a {
            Annotation::LinguisticErrorHere(code) => Some(*code),
            _ => None,
        }).next()
    }

    /// Set the verb usage reference on this node.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn set_verb_usage(&mut self, vu: crate::verbs::VerbUsageRef) {
        self.add_annotation(Annotation::VerbalCertainty(vu as i32));
    }

    /// Get the verb usage reference from this node, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn get_verb_usage(&self) -> Option<crate::verbs::VerbUsageRef> {
        // Verb usage is stored as a VerbalCertainty annotation with a positive value.
        // In the C reference, this is a separate annotation field.
        // For now, we use a simple heuristic: if VerbalCertainty is present and > 100,
        // it's a verb usage reference rather than a certainty level.
        self.verbal_certainty().filter(|&v| v > 100).map(|v| v as usize)
    }

    /// Set the preposition reference on this node.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn set_preposition(&mut self, prep: Option<crate::verbs::PrepositionRef>) {
        if let Some(p) = prep {
            self.add_annotation(Annotation::LinguisticErrorHere(p as i32));
        }
    }

    /// Get the preposition reference from this node, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn get_preposition(&self) -> Option<crate::verbs::PrepositionRef> {
        self.linguistic_error().map(|v| v as usize)
    }

    /// Set the second preposition reference on this node.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn set_second_preposition(&mut self, prep: Option<crate::verbs::PrepositionRef>) {
        if let Some(_p) = prep {
            self.add_annotation(Annotation::SentenceIsExistential(true));
        }
    }

    /// Get the second preposition reference from this node, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn get_second_preposition(&self) -> Option<crate::verbs::PrepositionRef> {
        // Stub: second preposition is not yet stored separately.
        None
    }

    /// Set the special meaning reference on this node.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn set_special_meaning(&mut self, _sm: crate::verbs::SpecialMeaningRef) {
        // Stub: special meaning storage deferred.
    }

    /// Get the special meaning reference from this node, if any.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn get_special_meaning(&self) -> Option<crate::verbs::SpecialMeaningRef> {
        None
    }

    /// Set the occurrence reference on this node (stub).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn set_occurrence(&mut self, _tp: ()) {
        // Stub: time_period not yet implemented.
    }

    /// Get the occurrence reference from this node (stub).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    pub fn get_occurrence(&self) -> Option<()> {
        None
    }

    /// Extend this node's wording to span all of its children.
    ///
    /// Unlike a simple union with the existing wording, this computes the
    /// minimum start and maximum end across the children, so it works
    /// correctly even when the parent starts with an empty `Wording::EMPTY`.
    pub fn extend_wording_from_children(&mut self) {
        let mut start = u32::MAX;
        let mut end = 0u32;
        let mut any = false;
        for child in self.children() {
            any = true;
            start = start.min(child.wording.start);
            end = end.max(child.wording.end);
        }
        if any {
            self.wording = Wording::new(start, end);
        }
    }

    /// Append a child node at the end of the child list.
    pub fn append_child(&mut self, child: ParseNode) {
        let new = Box::new(child);
        match self.down.as_mut() {
            None => self.down = Some(new),
            Some(first) => {
                let mut cur = first;
                while cur.next.is_some() {
                    cur = cur.next.as_mut().expect("next is Some in loop guard");
                }
                cur.next = Some(new);
            }
        }
    }

    /// Prepend a child node at the start of the child list.
    pub fn prepend_child(&mut self, child: ParseNode) {
        let mut new = Box::new(child);
        new.next = self.down.take();
        self.down = Some(new);
    }

    /// Return an iterator over this node's children.
    pub fn children(&self) -> ParseNodeChildren<'_> {
        ParseNodeChildren {
            current: self.down.as_deref(),
        }
    }

    /// Return an iterator over this node's alternative interpretations.
    pub fn alternatives(&self) -> ParseNodeAlternatives<'_> {
        ParseNodeAlternatives {
            current: self.next_alternative.as_deref(),
        }
    }

    /// Return the first child with the given node type, if any.
    pub fn find_child(&self, node_type: NodeType) -> Option<&ParseNode> {
        self.children().find(|n| n.node_type == node_type)
    }

    /// Return a mutable reference to the first child with the given node type, if any.
    pub fn find_child_mut(&mut self, node_type: NodeType) -> Option<&mut ParseNode> {
        let mut current = self.down.as_mut();
        while let Some(node) = current {
            if node.node_type == node_type {
                return Some(node.as_mut());
            }
            current = node.next.as_mut();
        }
        None
    }

    /// Number of children directly beneath this node.
    pub fn child_count(&self) -> usize {
        self.children().count()
    }

    /// Add an alternative interpretation of the same text.
    pub fn add_alternative(&mut self, alternative: ParseNode) {
        let new = Box::new(alternative);
        match self.next_alternative.as_mut() {
            None => self.next_alternative = Some(new),
            Some(first) => {
                let mut cur = first;
                while cur.next_alternative.is_some() {
                    cur = cur
                        .next_alternative
                        .as_mut()
                        .expect("next_alternative is Some in loop guard");
                }
                cur.next_alternative = Some(new);
            }
        }
    }

    /// True if `other` occurs somewhere beneath this node (including itself).
    pub fn contains(&self,
        other: &ParseNode,
    ) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        self.children().any(|child| child.contains(other))
    }
}

/// Iterator over a linked list of parse nodes.
#[derive(Clone, Copy, Debug)]
pub struct ParseNodeChildren<'a> {
    current: Option<&'a ParseNode>,
}

impl<'a> Iterator for ParseNodeChildren<'a> {
    type Item = &'a ParseNode;

    fn next(&mut self) -> Option<&'a ParseNode> {
        let node = self.current?;
        self.current = node.next.as_deref();
        Some(node)
    }
}

/// Iterator over the alternative interpretations of a parse node.
#[derive(Clone, Copy, Debug)]
pub struct ParseNodeAlternatives<'a> {
    current: Option<&'a ParseNode>,
}

impl<'a> Iterator for ParseNodeAlternatives<'a> {
    type Item = &'a ParseNode;

    fn next(&mut self) -> Option<&'a ParseNode> {
        let node = self.current?;
        self.current = node.next_alternative.as_deref();
        Some(node)
    }
}

/// Depth-first traversal of a parse tree, optionally skipping nodes marked
/// `DONT_VISIT_NFLAG`.
pub fn traverse_depth_first<'a>(
    root: &'a ParseNode,
    mut visitor: impl FnMut(&'a ParseNode),
) {
    fn walk<'a>(node: &'a ParseNode, visitor: &mut impl FnMut(&'a ParseNode)) {
        visitor(node);
        for child in node.children() {
            if !child.node_type.is_dont_visit() {
                walk(child, visitor);
            }
        }
    }
    walk(root, &mut visitor);
}

impl fmt::Display for ParseNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_node(
            node: &ParseNode,
            f: &mut fmt::Formatter<'_>,
            depth: usize,
        ) -> fmt::Result {
            write!(
                f,
                "{}{} [{}..{}]",
                "  ".repeat(depth),
                node.node_type,
                node.wording.start,
                node.wording.end
            )?;
            for child in node.children() {
                writeln!(f)?;
                write_node(child, f, depth + 1)?;
            }
            Ok(())
        }
        write_node(self, f, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_node() {
        let n = ParseNode::new(NodeType::Sentence, Wording::new(0, 3));
        assert_eq!(n.node_type(), NodeType::Sentence);
        assert_eq!(n.wording(), Wording::new(0, 3));
        assert_eq!(n.child_count(), 0);
    }

    #[test]
    fn test_append_children() {
        let mut parent = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::single(0)));
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::single(1)));
        assert_eq!(parent.child_count(), 2);
        let types: Vec<_> = parent.children().map(|c| c.node_type()).collect();
        assert_eq!(types, vec![NodeType::Constant, NodeType::Constant]);
    }

    #[test]
    fn test_prepend_child() {
        let mut parent = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::single(1)));
        parent.prepend_child(ParseNode::new(NodeType::Constant, Wording::single(0)));
        let starts: Vec<_> = parent.children().map(|c| c.wording().start).collect();
        assert_eq!(starts, vec![0, 1]);
    }

    #[test]
    fn test_find_child() {
        let mut parent = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::single(0)));
        parent.append_child(ParseNode::new(NodeType::LocalVariable, Wording::single(1)));
        assert!(parent.find_child(NodeType::Constant).is_some());
        assert!(parent.find_child(NodeType::Invocation).is_none());
    }

    #[test]
    fn test_extend_wording_from_children() {
        let mut parent = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::new(2, 3)));
        parent.append_child(ParseNode::new(NodeType::Constant, Wording::new(5, 6)));
        parent.extend_wording_from_children();
        assert_eq!(parent.wording(), Wording::new(2, 6));
    }

    #[test]
    fn test_alternatives() {
        let mut root = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        let alt1 = ParseNode::new(NodeType::Allowed, Wording::single(0));
        let alt2 = ParseNode::new(NodeType::Every, Wording::single(0));
        root.add_alternative(alt1);
        root.add_alternative(alt2);
        let types: Vec<_> = root.alternatives().map(|a| a.node_type()).collect();
        assert_eq!(types, vec![NodeType::Allowed, NodeType::Every]);
    }

    #[test]
    fn test_contains() {
        let mut root = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let child = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        let grandchild = ParseNode::new(NodeType::Constant, Wording::EMPTY);
        let mut child_mut = child;
        child_mut.append_child(grandchild);
        root.append_child(child_mut);
        let grandchild_ref = root.find_child(NodeType::Sentence).unwrap().find_child(NodeType::Constant).unwrap();
        assert!(root.contains(grandchild_ref));
    }

    #[test]
    fn test_depth_first_traverse() {
        let mut root = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        let mut and_node = ParseNode::new(NodeType::LogicalAnd, Wording::EMPTY);
        and_node.append_child(ParseNode::new(NodeType::TestValue, Wording::EMPTY));
        root.append_child(and_node);
        root.append_child(ParseNode::new(NodeType::LogicalOr, Wording::EMPTY));

        let mut types = Vec::new();
        traverse_depth_first(&root, |n| types.push(n.node_type()));
        assert_eq!(
            types,
            vec![
                NodeType::Sentence,
                NodeType::LogicalAnd,
                NodeType::TestValue,
                NodeType::LogicalOr,
            ]
        );
    }

    #[test]
    fn test_dont_visit_flag_skips_subtree() {
        // Inclusion has the DONT_VISIT_NFLAG, so traversal should visit the
        // root but skip the Inclusion node and its children entirely.
        let mut root = ParseNode::new(NodeType::Root, Wording::EMPTY);
        let mut inclusion = ParseNode::new(NodeType::Inclusion, Wording::EMPTY);
        inclusion.append_child(ParseNode::new(NodeType::Sentence, Wording::EMPTY));
        root.append_child(inclusion);
        let mut types = Vec::new();
        traverse_depth_first(&root, |n| types.push(n.node_type()));
        assert_eq!(types, vec![NodeType::Root]);
    }

    #[test]
    fn test_display() {
        let mut root = ParseNode::new(NodeType::Root, Wording::EMPTY);
        root.append_child(ParseNode::new(NodeType::Heading, Wording::new(0, 3)));
        let s = format!("{}", root);
        assert!(s.contains("ROOT_NT [0..0]"));
        assert!(s.contains("HEADING_NT [0..3]"));
    }
}
