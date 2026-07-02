//! Linguistics module for Inform 7 sentence diagram parsing.
//!
//! Provides the types and functions for parsing noun phrases, articles,
//! and other linguistic constructs. This is the bridge between the Preform
//! matching engine and the world model — the first step that turns raw source
//! text into structured parse trees with linguistic annotations.
//!
//! # References
//!
//! - C reference: `services/linguistics-module/Chapter 1/Diagrams.w` —
//!   sentence diagram node types and constructors.
//! - C reference: `services/linguistics-module/Chapter 2/Articles.w` —
//!   article types, internal NTs, and tables.
//! - C reference: `services/linguistics-module/Chapter 4/Noun Phrases.w` —
//!   noun phrase parsing at 4 levels.
//! - C reference: `services/linguistics-module/Chapter 1/Stock Control.w` —
//!   linguistic stock and small word sets.

use std::collections::HashMap;

use crate::parse_node::{Annotation, ParseNode};
use crate::preform::{InternalRegistry, PreformContext, match_nonterminal_impl, Grammar};
use crate::{NodeType, Wording};

// ---------------------------------------------------------------------------
// Certainty level constants
// ---------------------------------------------------------------------------

/// Impossible certainty level.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const IMPOSSIBLE_CE: i32 = -2;

/// Unlikely certainty level.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const UNLIKELY_CE: i32 = -1;

/// Unknown certainty level.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const UNKNOWN_CE: i32 = 0;

/// Likely certainty level.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const LIKELY_CE: i32 = 1;

/// Certain certainty level.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const CERTAIN_CE: i32 = 2;

/// Initially certainty level (for initial state).
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub const INITIALLY_CE: i32 = 3;

/// Write a certainty level as a human-readable string.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
pub fn certainty_write(level: i32) -> &'static str {
    match level {
        IMPOSSIBLE_CE => "impossible",
        UNLIKELY_CE => "unlikely",
        UNKNOWN_CE => "unknown",
        LIKELY_CE => "likely",
        CERTAIN_CE => "certain",
        INITIALLY_CE => "initially",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Article types
// ---------------------------------------------------------------------------

/// An article word in the Inform 7 language.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 2/Articles.w`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Article {
    /// The name of the article (e.g., "definite", "indefinite").
    pub name: &'static str,
}

/// The usage of an article with a specific word.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 2/Articles.w`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArticleUsage {
    /// The article that was matched (e.g., definite, indefinite).
    pub article: Article,
    /// The actual word that was matched (e.g., "the", "a").
    pub word: String,
}

/// A small word set for fast word lookup.
///
/// Wraps a `HashMap<String, T>` to provide the fast word-lookup mechanism
/// used by articles, verbs, and nouns in the C reference.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Stock Control.w`
pub type SmallWordSet<T> = HashMap<String, T>;

/// Look up a word in the article tables and return its article usage.
///
/// Checks the word (case-insensitively) against the known definite and
/// indefinite article words. This is used by [`NounPhrases::parse_np_articled`]
/// to attach article annotations after the matching engine matches `<np-articled>`.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 2/Articles.w`
pub fn lookup_article(word: &str) -> Option<ArticleUsage> {
    let word_lower = word.to_lowercase();
    match word_lower.as_str() {
        "the" => Some(ArticleUsage {
            article: Article { name: "definite" },
            word: word_lower,
        }),
        "a" | "an" | "some" => Some(ArticleUsage {
            article: Article { name: "indefinite" },
            word: word_lower,
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Diagram constructor functions
// ---------------------------------------------------------------------------

/// Diagram constructor functions for sentence diagram node types.
///
/// Each function creates a [`ParseNode`] of the corresponding type, with the
/// given wording and optional children.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 1/Diagrams.w`
pub struct Diagrams;

impl Diagrams {
    /// Create a new `UNPARSED_NOUN_NT` node.
    ///
    /// Corresponds to `Diagrams::new_unparsed_noun` in the C reference.
    pub fn new_unparsed_noun(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::UnparsedNoun, wording)
    }

    /// Create a new `PROPER_NOUN_NT` node.
    ///
    /// Corresponds to `Diagrams::new_proper_noun` in the C reference.
    pub fn new_proper_noun(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::ProperNoun, wording)
    }

    /// Create a new `COMMON_NOUN_NT` node.
    ///
    /// Corresponds to `Diagrams::new_common_noun` in the C reference.
    pub fn new_common_noun(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::CommonNoun, wording)
    }

    /// Create a new `PRONOUN_NT` node.
    ///
    /// Corresponds to `Diagrams::new_pronoun` in the C reference.
    pub fn new_pronoun(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::Pronoun, wording)
    }

    /// Create a new `DEFECTIVE_NOUN_NT` node.
    ///
    /// Corresponds to `Diagrams::new_defective` in the C reference.
    pub fn new_defective(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::DefectiveNoun, wording)
    }

    /// Create a new `KIND_NT` node with one child.
    ///
    /// Corresponds to `Diagrams::new_kind` in the C reference.
    pub fn new_kind(wording: Wording, child: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::Kind, wording);
        node.append_child(child);
        node
    }

    /// Create a new `RELATIONSHIP_NT` node with one child.
    ///
    /// Corresponds to `Diagrams::new_relationship` in the C reference.
    pub fn new_relationship(wording: Wording, child: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::Relationship, wording);
        node.append_child(child);
        node
    }

    /// Create a new `CALLED_NT` node with two children.
    ///
    /// Corresponds to `Diagrams::new_called` in the C reference.
    pub fn new_called(wording: Wording, child1: ParseNode, child2: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::Called, wording);
        node.append_child(child1);
        node.append_child(child2);
        node
    }

    /// Create a new `WITH_NT` node with two children.
    ///
    /// Corresponds to `Diagrams::new_with` in the C reference.
    pub fn new_with(wording: Wording, child1: ParseNode, child2: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::With, wording);
        node.append_child(child1);
        node.append_child(child2);
        node
    }

    /// Create a new `AND_NT` node with two children.
    ///
    /// Corresponds to `Diagrams::new_and` in the C reference.
    pub fn new_and(wording: Wording, child1: ParseNode, child2: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::And, wording);
        node.append_child(child1);
        node.append_child(child2);
        node
    }

    /// Create a new `PROPERTY_LIST_NT` node.
    ///
    /// Corresponds to `Diagrams::new_property_list` in the C reference.
    pub fn new_property_list(wording: Wording) -> ParseNode {
        ParseNode::new(NodeType::PropertyList, wording)
    }

    /// Create a new `X_OF_Y_NT` node with two children.
    ///
    /// Corresponds to `Diagrams::new_x_of_y` in the C reference.
    pub fn new_x_of_y(wording: Wording, child1: ParseNode, child2: ParseNode) -> ParseNode {
        let mut node = ParseNode::new(NodeType::XOfY, wording);
        node.append_child(child1);
        node.append_child(child2);
        node
    }
}

// ---------------------------------------------------------------------------
// Noun phrase parsing
// ---------------------------------------------------------------------------

/// Noun phrase parsing functions.
///
/// Provides the NP1 (`<np-unparsed>`) and NP2 (`<np-articled>`) levels of
/// noun phrase parsing.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 4/Noun Phrases.w`
pub struct NounPhrases;

impl NounPhrases {
    /// Parse an unparsed noun phrase (NP1).
    ///
    /// Matches `<np-unparsed>` against the wording and creates an
    /// `UNPARSED_NOUN_NT` node.
    ///
    /// # References
    ///
    /// - C reference: `NounPhrases::parse_np_unparsed` in
    ///   `services/linguistics-module/Chapter 4/Noun Phrases.w`
    pub fn parse_np_unparsed(
        ctx: &PreformContext,
        registry: &InternalRegistry,
        wording: Wording,
    ) -> Option<ParseNode> {
        let m = match_nonterminal_impl(ctx, registry, "np-unparsed", wording)?;
        let node_wording = Wording::new(m.word_range.start, m.word_range.end);
        Some(Diagrams::new_unparsed_noun(node_wording))
    }

    /// Parse an articled noun phrase (NP2).
    ///
    /// Matches `<np-articled>` against the wording, extracts the article
    /// annotation, and creates an annotated `UNPARSED_NOUN_NT` node.
    ///
    /// # References
    ///
    /// - C reference: `NounPhrases::parse_np_articled` in
    ///   `services/linguistics-module/Chapter 4/Noun Phrases.w`
    pub fn parse_np_articled(
        ctx: &PreformContext,
        registry: &InternalRegistry,
        wording: Wording,
    ) -> Option<ParseNode> {
        let m = match_nonterminal_impl(ctx, registry, "np-articled", wording)?;
        let node_wording = Wording::new(m.word_range.start, m.word_range.end);
        let mut node = Diagrams::new_unparsed_noun(node_wording);

        // Extract the first word from the matched wording and look it up
        // in the article tables. The matching engine does not propagate
        // internal payloads from sub-nonterminal matches up to the parent,
        // so we re-parse the article from the matched wording here.
        let first_word_idx = m.word_range.start as usize;
        if let Some(word) = ctx.word_text.get(first_word_idx) {
            if let Some(usage) = lookup_article(word) {
                node.add_annotation(Annotation::ArticleUsage(usage));
            }
        }

        Some(node)
    }

    /// Add an article annotation to a parse node.
    ///
    /// Corresponds to `NounPhrases::add_art` in the C reference.
    pub fn add_article(node: &mut ParseNode, article_usage: ArticleUsage) {
        node.add_annotation(Annotation::ArticleUsage(article_usage));
    }
}

/// Parse a noun phrase from raw text.
///
/// This is the main public API for noun phrase parsing. It:
///
/// 1. Tokenizes the input text using the existing lexer.
/// 2. Creates a [`PreformContext`] from the token stream.
/// 3. Calls the matching engine for `<np-articled>` first, then
///    `<np-unparsed>` as a fallback.
/// 4. Creates the appropriate [`ParseNode`] from the match.
/// 5. Returns the parse node.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 4/Noun Phrases.w`
pub fn parse_noun_phrase(
    text: &str,
    grammar: &Grammar,
    registry: &InternalRegistry,
) -> Option<ParseNode> {
    let tokens = crate::Lexer::tokenize(text).ok()?;
    if tokens.is_empty() {
        return None;
    }

    // Extract word text from tokens (skip whitespace tokens).
    let word_text: Vec<&str> = tokens
        .iter()
        .filter(|t| t.kind != crate::SyntaxKind::WHITESPACE)
        .map(|t| t.text.as_str())
        .collect();

    if word_text.is_empty() {
        return None;
    }

    let word_count = word_text.len() as u32;
    let wording = Wording::new(0, word_count);

    let ctx = PreformContext {
        grammar,
        word_text: &word_text,
        is_paragraph_start: false,
    };

    // Try <np-articled> first, then fall back to <np-unparsed>.
    if let Some(node) = NounPhrases::parse_np_articled(&ctx, registry, wording) {
        return Some(node);
    }

    NounPhrases::parse_np_unparsed(&ctx, registry, wording)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preform::parse_preform_grammar;
    use crate::preform::InternalNonterminal;
    use crate::preform_internal::make_article_internals;
    use crate::NodeCategory;

    // -----------------------------------------------------------------------
    // Article tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_article_matches_definite() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, definite, _) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n<definite-article> internal\n").unwrap();
        let words = &["the"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let result = article.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "article should match 'the'");
        assert_eq!(
            result.unwrap().payload,
            crate::preform::InternalPayload::Article("definite".to_string())
        );

        let result = definite.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "definite-article should match 'the'");
    }

    #[test]
    fn test_article_matches_indefinite() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, _, indefinite) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n<indefinite-article> internal\n").unwrap();
        let words = &["a"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let result = article.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "article should match 'a'");

        let result = indefinite.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "indefinite-article should match 'a'");
    }

    #[test]
    fn test_article_matches_an() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, _, indefinite) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n<indefinite-article> internal\n").unwrap();
        let words = &["an"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let result = article.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "article should match 'an'");

        let result = indefinite.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "indefinite-article should match 'an'");
    }

    #[test]
    fn test_article_matches_some() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, _, indefinite) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n<indefinite-article> internal\n").unwrap();
        let words = &["some"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let result = article.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "article should match 'some'");

        let result = indefinite.match_nonterminal(&ctx, Wording::new(0, 1));
        assert!(result.is_some(), "indefinite-article should match 'some'");
    }

    #[test]
    fn test_article_fails_unknown_word() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, definite, indefinite) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n<definite-article> internal\n<indefinite-article> internal\n").unwrap();
        let words = &["xyzzy"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        assert!(article.match_nonterminal(&ctx, Wording::new(0, 1)).is_none());
        assert!(definite.match_nonterminal(&ctx, Wording::new(0, 1)).is_none());
        assert!(indefinite.match_nonterminal(&ctx, Wording::new(0, 1)).is_none());
    }

    #[test]
    fn test_definite_article_fails_indefinite() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (_, definite, _) = make_article_internals();
        let grammar = parse_preform_grammar("<definite-article> internal\n").unwrap();
        let words = &["a"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        assert!(definite.match_nonterminal(&ctx, Wording::new(0, 1)).is_none(),
            "definite-article should fail on 'a'");
    }

    #[test]
    fn test_indefinite_article_fails_definite() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (_, _, indefinite) = make_article_internals();
        let grammar = parse_preform_grammar("<indefinite-article> internal\n").unwrap();
        let words = &["the"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        assert!(indefinite.match_nonterminal(&ctx, Wording::new(0, 1)).is_none(),
            "indefinite-article should fail on 'the'");
    }

    #[test]
    fn test_article_fails_multi_word() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let (article, _, _) = make_article_internals();
        let grammar = parse_preform_grammar("<article> internal\n").unwrap();
        let words = &["the", "cat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        assert!(article.match_nonterminal(&ctx, Wording::new(0, 2)).is_none(),
            "article should fail on multi-word wording");
    }

    // -----------------------------------------------------------------------
    // Diagram constructor tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_unparsed_noun() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_unparsed_noun(Wording::new(0, 3));
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 3));
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn test_new_proper_noun() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_proper_noun(Wording::new(0, 2));
        assert_eq!(node.node_type(), NodeType::ProperNoun);
        assert_eq!(node.wording(), Wording::new(0, 2));
    }

    #[test]
    fn test_new_common_noun() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_common_noun(Wording::new(0, 1));
        assert_eq!(node.node_type(), NodeType::CommonNoun);
    }

    #[test]
    fn test_new_pronoun() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_pronoun(Wording::new(0, 1));
        assert_eq!(node.node_type(), NodeType::Pronoun);
    }

    #[test]
    fn test_new_defective() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_defective(Wording::new(0, 1));
        assert_eq!(node.node_type(), NodeType::DefectiveNoun);
    }

    #[test]
    fn test_new_kind_with_child() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child = Diagrams::new_unparsed_noun(Wording::new(1, 2));
        let node = Diagrams::new_kind(Wording::new(0, 2), child);
        assert_eq!(node.node_type(), NodeType::Kind);
        assert_eq!(node.child_count(), 1);
    }

    #[test]
    fn test_new_relationship_with_child() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child = Diagrams::new_unparsed_noun(Wording::new(1, 2));
        let node = Diagrams::new_relationship(Wording::new(0, 2), child);
        assert_eq!(node.node_type(), NodeType::Relationship);
        assert_eq!(node.child_count(), 1);
    }

    #[test]
    fn test_new_called_with_two_children() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child1 = Diagrams::new_unparsed_noun(Wording::new(0, 1));
        let child2 = Diagrams::new_unparsed_noun(Wording::new(2, 3));
        let node = Diagrams::new_called(Wording::new(0, 3), child1, child2);
        assert_eq!(node.node_type(), NodeType::Called);
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_new_with_two_children() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child1 = Diagrams::new_unparsed_noun(Wording::new(0, 1));
        let child2 = Diagrams::new_unparsed_noun(Wording::new(2, 3));
        let node = Diagrams::new_with(Wording::new(0, 3), child1, child2);
        assert_eq!(node.node_type(), NodeType::With);
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_new_and_two_children() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child1 = Diagrams::new_unparsed_noun(Wording::new(0, 1));
        let child2 = Diagrams::new_unparsed_noun(Wording::new(2, 3));
        let node = Diagrams::new_and(Wording::new(0, 3), child1, child2);
        assert_eq!(node.node_type(), NodeType::And);
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_new_property_list() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let node = Diagrams::new_property_list(Wording::new(0, 2));
        assert_eq!(node.node_type(), NodeType::PropertyList);
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn test_new_x_of_y() {
        // Reference: services/linguistics-module/Chapter 1/Diagrams.w
        let child1 = Diagrams::new_unparsed_noun(Wording::new(0, 1));
        let child2 = Diagrams::new_unparsed_noun(Wording::new(2, 3));
        let node = Diagrams::new_x_of_y(Wording::new(0, 3), child1, child2);
        assert_eq!(node.node_type(), NodeType::XOfY);
        assert_eq!(node.child_count(), 2);
    }

    // -----------------------------------------------------------------------
    // Noun phrase parsing tests (synthetic grammar)
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_np_unparsed_synthetic() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // Create a synthetic grammar with <np-unparsed> matching any text.
        let source = "<np-unparsed> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["hello", "world"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let node = NounPhrases::parse_np_unparsed(&ctx, &registry, Wording::new(0, 2));
        assert!(node.is_some(), "np-unparsed should match any text");
        let node = node.unwrap();
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 2));
    }

    #[test]
    fn test_parse_np_articled_synthetic() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // Create a synthetic grammar with <np-articled> matching article + text.
        let source = concat!(
            "<article> internal\n",
            "<np-articled> ::= <article> ...\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["the", "room"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let node = NounPhrases::parse_np_articled(&ctx, &registry, Wording::new(0, 2));
        assert!(node.is_some(), "np-articled should match 'the room'");
        let node = node.unwrap();
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 2));
    }

    #[test]
    fn test_parse_np_articled_indefinite_synthetic() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let source = concat!(
            "<article> internal\n",
            "<np-articled> ::= <article> ...\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["a", "container"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let node = NounPhrases::parse_np_articled(&ctx, &registry, Wording::new(0, 2));
        assert!(node.is_some(), "np-articled should match 'a container'");
    }

    #[test]
    fn test_parse_np_articled_fails_no_article() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let source = concat!(
            "<article> internal\n",
            "<np-articled> ::= <article> ...\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["xyzzy"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let node = NounPhrases::parse_np_articled(&ctx, &registry, Wording::new(0, 1));
        assert!(node.is_none(), "np-articled should fail on 'xyzzy' (no article)");
    }
    #[test]
    fn test_add_article_annotation() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let mut node = Diagrams::new_unparsed_noun(Wording::new(0, 2));
        let usage = ArticleUsage {
            article: Article { name: "definite" },
            word: "the".to_string(),
        };
        NounPhrases::add_article(&mut node, usage);

        // Verify the annotation was actually added using the getter.
        let retrieved = node.article_usage();
        assert!(retrieved.is_some(), "article annotation should be present");
        assert_eq!(retrieved.unwrap().article.name, "definite");
        assert_eq!(retrieved.unwrap().word, "the");
    }

    // -----------------------------------------------------------------------
    // Integration test: article matching through the matching engine
    // -----------------------------------------------------------------------

    #[test]
    fn test_article_internal_through_matching_engine() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let source = concat!(
            "<article> internal\n",
            "<test> ::= <article> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["the", "cat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_some(), "<article> cat should match 'the cat'");
    }

    #[test]
    fn test_article_internal_fails_through_matching_engine() {
        // Reference: services/linguistics-module/Chapter 2/Articles.w
        let source = concat!(
            "<article> internal\n",
            "<test> ::= <article> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["xyzzy", "cat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_none(), "<article> cat should fail on 'xyzzy cat'");
    }

    // -----------------------------------------------------------------------
    // NodeType metadata tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_verb_metadata() {
        let m = NodeType::Verb.metadata();
        assert_eq!(m.name, "VERB_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 0);
        assert!(!m.flags.assert);
    }

    #[test]
    fn test_unparsed_noun_metadata() {
        let m = NodeType::UnparsedNoun.metadata();
        assert_eq!(m.name, "UNPARSED_NOUN_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 0);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_common_noun_metadata() {
        let m = NodeType::CommonNoun.metadata();
        assert_eq!(m.name, "COMMON_NOUN_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, u32::MAX);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_proper_noun_metadata() {
        let m = NodeType::ProperNoun.metadata();
        assert_eq!(m.name, "PROPER_NOUN_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 0);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_pronoun_metadata() {
        let m = NodeType::Pronoun.metadata();
        assert_eq!(m.name, "PRONOUN_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 0);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_defective_noun_metadata() {
        let m = NodeType::DefectiveNoun.metadata();
        assert_eq!(m.name, "DEFECTIVE_NOUN_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 0);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_relationship_metadata() {
        let m = NodeType::Relationship.metadata();
        assert_eq!(m.name, "RELATIONSHIP_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 2);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_called_metadata() {
        let m = NodeType::Called.metadata();
        assert_eq!(m.name, "CALLED_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 2);
        assert_eq!(m.max_children, 2);
        assert!(!m.flags.assert);
    }

    #[test]
    fn test_with_metadata() {
        let m = NodeType::With.metadata();
        assert_eq!(m.name, "WITH_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 2);
        assert_eq!(m.max_children, 2);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_and_metadata() {
        let m = NodeType::And.metadata();
        assert_eq!(m.name, "AND_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 2);
        assert_eq!(m.max_children, 2);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_kind_metadata() {
        let m = NodeType::Kind.metadata();
        assert_eq!(m.name, "KIND_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, 1);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_property_list_metadata() {
        let m = NodeType::PropertyList.metadata();
        assert_eq!(m.name, "PROPERTY_LIST_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, u32::MAX);
        assert!(m.flags.assert);
    }

    #[test]
    fn test_x_of_y_metadata() {
        let m = NodeType::XOfY.metadata();
        assert_eq!(m.name, "X_OF_Y_NT");
        assert_eq!(m.category, NodeCategory::L3);
        assert_eq!(m.min_children, 2);
        assert_eq!(m.max_children, 2);
        assert!(m.flags.assert);
    }

    // -----------------------------------------------------------------------
    // Real grammar oracle tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_real_syntax_preform_has_linguistics_nts() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"np-unparsed"), "missing np-unparsed");
        assert!(names.contains(&"np-articled"), "missing np-articled");
        assert!(names.contains(&"article"), "missing article");
        assert!(names.contains(&"definite-article"), "missing definite-article");
        assert!(names.contains(&"indefinite-article"), "missing indefinite-article");
    }

    #[test]
    fn test_real_syntax_preform_np_unparsed_match() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let words = &["hello", "world"];
        let registry = InternalRegistry::linguistics();
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let m = match_nonterminal_impl(&ctx, &registry, "np-unparsed", Wording::new(0, 2));
        assert!(m.is_some(), "np-unparsed should match 'hello world' via real grammar");
    }

    #[test]
    fn test_real_syntax_preform_np_articled_match() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let words = &["the", "room"];
        let registry = InternalRegistry::linguistics();
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };

        let m = match_nonterminal_impl(&ctx, &registry, "np-articled", Wording::new(0, 2));
        assert!(m.is_some(), "np-articled should match 'the room' via real grammar");
    }

    // -----------------------------------------------------------------------
    // parse_noun_phrase tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_noun_phrase_unparsed() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // <np-unparsed> matches any non-empty text.
        let source = "<np-unparsed> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();

        let node = parse_noun_phrase("hello world", &grammar, &registry);
        assert!(node.is_some(), "parse_noun_phrase should match 'hello world'");
        let node = node.unwrap();
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 2));
    }

    #[test]
    fn test_parse_noun_phrase_articled() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // <np-articled> matches "the room" and returns annotated node.
        let source = concat!(
            "<article> internal\n",
            "<np-articled> ::= <article> ...\n",
            "<np-unparsed> ::= ...\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();

        let node = parse_noun_phrase("the room", &grammar, &registry);
        assert!(node.is_some(), "parse_noun_phrase should match 'the room'");
        let node = node.unwrap();
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 2));

        // Verify the article annotation was attached.
        let usage = node.article_usage();
        assert!(usage.is_some(), "article annotation should be present on articled NP");
        assert_eq!(usage.unwrap().article.name, "definite");
        assert_eq!(usage.unwrap().word, "the");
    }

    #[test]
    fn test_parse_noun_phrase_fallback() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // "xyzzy" falls through <np-articled> to <np-unparsed>.
        let source = concat!(
            "<article> internal\n",
            "<np-articled> ::= <article> ...\n",
            "<np-unparsed> ::= ...\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();

        let node = parse_noun_phrase("xyzzy", &grammar, &registry);
        assert!(node.is_some(), "parse_noun_phrase should fall back to np-unparsed for 'xyzzy'");
        let node = node.unwrap();
        assert_eq!(node.node_type(), NodeType::UnparsedNoun);
        assert_eq!(node.wording(), Wording::new(0, 1));

        // No article annotation on fallback.
        assert!(node.article_usage().is_none(), "no article annotation on fallback parse");
    }

    #[test]
    fn test_parse_noun_phrase_empty() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // Empty input returns None.
        let source = "<np-unparsed> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();

        let node = parse_noun_phrase("", &grammar, &registry);
        assert!(node.is_none(), "parse_noun_phrase should return None for empty input");
    }

    #[test]
    fn test_parse_noun_phrase_whitespace_only() {
        // Reference: services/linguistics-module/Chapter 4/Noun Phrases.w
        // Whitespace-only input returns None.
        let source = "<np-unparsed> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();

        let node = parse_noun_phrase("   ", &grammar, &registry);
        assert!(node.is_none(), "parse_noun_phrase should return None for whitespace-only input");
    }
}
