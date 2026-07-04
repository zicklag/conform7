#![forbid(unsafe_code)]

//! Syntax definitions for Inform 7 source text.
//!
//! This crate defines the `SyntaxKind` enum — the canonical set of token and
//! node types used throughout the Conform7 compiler — and provides a lexer
//! that tokenizes I7 source text into a flat sequence of tokens, a sentence
//! breaker that splits the token stream into classified sentences, and the
//! foundational parse tree data structures used by the I7 parser.
//!
//! # Pipeline
//!
//! 1. **Lexer** (`lexer`): characters → tokens
//! 2. **Sentence breaker** (`sentence`): tokens → classified sentences
//! 3. **Parse tree** (`parse_node`, `node_type`, `wording`): sentences → AST
//!    (this stage is the data model; the grammar parser comes later)
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 3/Lexer.w`
//! - C reference: `services/words-module/Chapter 3/Feeds.w`
//! - C reference: `services/syntax-module/Chapter 3/Sentences.w`
//! - C reference: `services/syntax-module/Chapter 2/Parse Nodes.w`
//! - C reference: `services/syntax-module/Chapter 2/Node Types.w`

pub mod heading;
pub mod linguistics;
pub mod lexer;
pub mod node_type;
pub mod parse_node;
pub(crate) mod preform_internal;
pub mod preform;
pub mod linguistic_constants;
pub mod stock_control;
pub mod verb_conjugation;
pub mod verbs;
pub mod word_assemblage;
pub mod verb_phrases;
pub mod sentence;
pub mod structural;
pub mod syntax_kind;
pub mod token;
pub mod wording;

pub use heading::parse_heading;
pub use lexer::Lexer;
pub use preform::{
    match_nonterminal_impl, InternalNonterminal, InternalPayload,
    InternalRegistry, InternalResult, Match, Nonterminal, PreformContext, Production,
    ProductionToken, ProductionTokenCategory, Grammar, parse_preform_grammar,
};

pub use structural::parse_structural;
pub use node_type::{NodeCategory, NodeFlags, NodeType, NodeTypeMetadata};
pub use parse_node::{traverse_depth_first, ParseNode, ParseNodeAlternatives, ParseNodeChildren};
pub use sentence::{break_sentences, HeadingLevel, Sentence, SentenceClassification, StructuralType};
pub use syntax_kind::SyntaxKind;
pub use token::Token;
pub use linguistics::{Article, ArticleUsage, Diagrams, NounPhrases, parse_noun_phrase};
pub use wording::Wording;
