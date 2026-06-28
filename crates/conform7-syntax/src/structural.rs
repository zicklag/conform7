//! Structural sentence parser.
//!
//! Converts a classified structural sentence from the sentence breaker into
//! the appropriate parse node type (`INCLUDE_NT`, `TABLE_NT`, `EQUATION_NT`,
//! or `USE_NT`).
//!
//! Structural sentences are those that demarcate the text or call for other
//! text to be included. In the C implementation they are handled in
//! `services/syntax-module/Chapter 3/Sentences.w` and
//! `inbuild/supervisor-module/Chapter 6/Source Text.w`.

use crate::parse_node::ParseNode;
use crate::{NodeType, Sentence, SentenceClassification, StructuralType, Wording};

/// Convert a structural sentence into the appropriate parse node.
///
/// # Panics
///
/// Panics if the sentence is not classified as a structural sentence.
///
/// # Examples
///
/// ```
/// use conform7_syntax::{break_sentences, parse_structural, Lexer, NodeType};
///
/// let source = "Include Standard Rules by Graham Nelson.";
/// let tokens = Lexer::tokenize(source).unwrap();
/// let sentences = break_sentences(&tokens);
/// let node = parse_structural(&sentences[0], &tokens);
/// assert_eq!(node.node_type(), NodeType::Include);
/// ```
pub fn parse_structural(sentence: &Sentence, _tokens: &[crate::Token]) -> ParseNode {
    let node_type = match sentence.classification {
        SentenceClassification::Structural(StructuralType::Include) => NodeType::Include,
        SentenceClassification::Structural(StructuralType::Table) => NodeType::Table,
        SentenceClassification::Structural(StructuralType::Equation) => NodeType::Equation,
        SentenceClassification::Structural(StructuralType::Use) => NodeType::Use,
        _ => panic!("parse_structural called on non-structural sentence"),
    };

    let wording = Wording::new(
        sentence.token_range.start as u32,
        sentence.token_range.end as u32,
    );

    ParseNode::new(node_type, wording)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{break_sentences, Lexer};

    #[test]
    fn test_include_sentence() {
        let source = "Include Standard Rules by Graham Nelson.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);

        let node = parse_structural(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Include);
        assert_eq!(node.wording().len(), 11); // Include/ws/Standard/ws/Rules/ws/by/ws/Graham/ws/Nelson
    }

    #[test]
    fn test_table_sentence() {
        let source = "Table of Contents";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);

        let node = parse_structural(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Table);
    }

    #[test]
    fn test_equation_sentence() {
        let source = "Equation 2 - Newton's Second Law";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);

        let node = parse_structural(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Equation);
    }

    #[test]
    fn test_use_sentence() {
        let source = "Use American dialect.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);

        let node = parse_structural(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Use);
    }

    #[test]
    fn test_structural_followed_by_regular() {
        let source = "Include Standard Rules by Graham Nelson.\n\nThe Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);

        let node = parse_structural(&sentences[0], &tokens);
        assert_eq!(node.node_type(), NodeType::Include);
    }

    #[test]
    #[should_panic(expected = "parse_structural called on non-structural sentence")]
    fn test_panic_on_non_structural() {
        let source = "The Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        parse_structural(&sentences[0], &tokens);
    }
}
