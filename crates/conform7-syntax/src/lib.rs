//! Syntax definitions for Inform 7 source text.
//!
//! This crate defines the `SyntaxKind` enum — the canonical set of token and
//! node types used throughout the Conform7 compiler — and provides a lexer
//! that tokenizes I7 source text into a flat sequence of tokens.
//!
//! The lexer is the first stage of the I7 frontend. It reads raw source
//! characters and produces a token stream that the parser (in a later plan)
//! will consume to build a Rowan CST/AST.
//!
//! # Architecture
//!
//! The lexer is a simple state machine, mirroring the C implementation in
//! `services/words-module/Chapter 3/Lexer.w`. It handles:
//!
//! - **Ordinary words**: natural language text, numbers, punctuation
//! - **Quoted strings**: `"text"` with text substitutions `[...]` inside
//! - **I6 escape blocks**: `(- ... -)` embedded Inform 6 code
//! - **Comments**: `[...]` outside strings (stripped, not stored as tokens)
//! - **Paragraph breaks**: blank lines (semantically significant in I7)
//! - **Headings**: lines beginning with Volume/Book/Part/Chapter/Section
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 3/Lexer.w`
//! - C reference: `services/words-module/Chapter 3/Feeds.w`

pub mod lexer;
pub mod syntax_kind;
pub mod token;

pub use lexer::Lexer;
pub use syntax_kind::SyntaxKind;
pub use token::Token;
