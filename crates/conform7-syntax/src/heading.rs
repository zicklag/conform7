//! Heading sentence parser.
//!
//! Converts a classified heading sentence from the sentence breaker into a
//! `HEADING_NT` parse node. This is the first end-to-end bridge between the
//! sentence breaker and the parse node model.
//!
//! In the C implementation, headings are created in
//! `services/syntax-module/Chapter 3/Sentences.w`:
//!
//! ```c
//! new = Node::new(HEADING_NT);
//! Node::set_text(new, W);
//! Annotations::write_int(new, heading_level_ANNOT, heading_level);
//! ```
//!
//! We mirror this by setting the node's wording to the sentence's token range
//! and attaching a `HeadingLevel` annotation.

use crate::parse_node::{Annotation, ParseNode};
use crate::{NodeType, Sentence, SentenceClassification, Wording};

/// Convert a heading sentence into a `HEADING_NT` parse node.
///
/// # Panics
///
/// Panics if the sentence is not classified as a heading.
///
/// # Examples
///
/// ```
/// use conform7_syntax::{break_sentences, parse_heading, HeadingLevel, Lexer, NodeType};
///
/// let source = "Chapter 1 - The Beginning";
/// let tokens = Lexer::tokenize(source).unwrap();
/// let sentences = break_sentences(&tokens);
/// let node = parse_heading(&sentences[0], &tokens);
/// assert_eq!(node.node_type(), NodeType::Heading);
/// assert_eq!(node.heading_level(), Some(HeadingLevel::Chapter));
/// ```
pub fn parse_heading(sentence: &Sentence, _tokens: &[crate::Token]) -> ParseNode {
    let level = match sentence.classification {
        SentenceClassification::Heading { level } => level,
        _ => panic!("parse_heading called on non-heading sentence"),
    };

    let wording = Wording::new(
        sentence.token_range.start as u32,
        sentence.token_range.end as u32,
    );

    let mut node = ParseNode::new(NodeType::Heading, wording);
    node.add_annotation(Annotation::HeadingLevel(level));
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{break_sentences, HeadingLevel, Lexer};

    #[test]
    fn test_parse_chapter_heading() {
        let source = "Chapter 1 - The Beginning";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);

        let node = parse_heading(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Heading);
        assert_eq!(node.heading_level(), Some(HeadingLevel::Chapter));
        assert_eq!(node.wording().len(), 9); // Chapter/ ws / 1 / ws / - / ws / The / ws / Beginning
    }

    #[test]
    fn test_all_heading_levels() {
        let source = "Volume 1\n\nBook 1\n\nPart 1\n\nChapter 1\n\nSection 1";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 5);

        let expected = [
            HeadingLevel::Volume,
            HeadingLevel::Book,
            HeadingLevel::Part,
            HeadingLevel::Chapter,
            HeadingLevel::Section,
        ];

        for (sentence, expected_level) in sentences.iter().zip(expected.iter()) {
            let node = parse_heading(sentence, &tokens);
            assert_eq!(node.heading_level(), Some(*expected_level));
        }
    }

    #[test]
    fn test_heading_followed_by_regular_sentence() {
        let source = "Chapter 1 - The Beginning\n\nThe Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);

        let heading = parse_heading(&sentences[0], &tokens);
        assert_eq!(heading.node_type(), NodeType::Heading);
        assert_eq!(heading.heading_level(), Some(HeadingLevel::Chapter));
    }

    #[test]
    #[should_panic(expected = "parse_heading called on non-heading sentence")]
    fn test_panic_on_non_heading() {
        let source = "The Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        parse_heading(&sentences[0], &tokens);
    }
}
