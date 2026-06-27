//! The `SyntaxKind` enum — canonical token and node types for I7 source.
//!
//! This enum covers all token types produced by the lexer, plus node types
//! that the parser will later produce. For now, only the token variants are
//! used; the node variants are reserved for future use.
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 3/Lexer.w` — the eight
//!   categories of text (titling, documentation, heading, quoted, text
//!   substitution, comment, I6 literal, normal)

/// Canonical set of token and node types for Inform 7 source text.
///
/// The naming convention follows rust-analyzer's `SyntaxKind`: token types
/// are `UPPER_CASE` (e.g., `WORD`, `STRING`), while node types are
/// `UpperCamelCase` (e.g., `Heading`, `Sentence`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    // ── Tokens ──────────────────────────────────────────────────────────

    /// A natural language word (e.g., `room`, `Peter`, `take`).
    /// Case is preserved as written.
    WORD,

    /// A quoted string literal: `"Hello, world!"`.
    /// Includes the surrounding quote characters.
    STRING,

    /// An Inform 6 escape block: `(- ... -)`.
    /// Includes the `(-` and `-)` delimiters.
    I6Block,

    /// A text substitution inside a quoted string: `[the noun]`, `[if ...]`.
    /// Includes the surrounding brackets.
    TextSubstitution,

    /// A comment: `[...]` outside strings.
    /// Comments are stripped by the lexer and not emitted as tokens.
    COMMENT,

    /// A heading marker word: `Volume`, `Book`, `Part`, `Chapter`, `Section`.
    /// Only the first word of a heading line gets this kind; subsequent words
    /// are `WORD`.
    HeadingMarker,

    /// A punctuation mark: `.`, `,`, `:`, `;`, `?`, `!`, `(`, `)`, `{`, `}`.
    PUNCTUATION,

    /// A paragraph break (blank line between paragraphs).
    /// Semantically significant in I7 — ends the current sentence.
    ParagraphBreak,

    /// A numeric literal: `0`, `127`, `-3`.
    NUMBER,

    /// A dash or hyphen: `-`, `--`, `—`.
    /// Used in headings (`Chapter 1 - The Beginning`) and other contexts.
    DASH,

    /// An equals sign: `=`.
    EQUALS,

    /// A slash: `/`.
    /// Used in URLs and some I7 syntax.
    SLASH,

    /// A newline character (end of line).
    NEWLINE,

    /// Whitespace (spaces, tabs) — preserved for round-trip fidelity.
    WHITESPACE,

    /// An error token — malformed input that could not be lexed.
    /// Carries an error message.
    ERROR,

    // ── Node types (reserved for parser) ─────────────────────────────────

    /// Root node for a source file.
    SourceFile,

    /// A heading: `Chapter 1 - The Beginning`.
    Heading,

    /// A sentence: `The Lab is a room.`
    Sentence,

    /// An assertion sentence: `Peter is a man in the Lab.`
    Assertion,

    /// A phrase definition: `To expose (X - a value): ...`
    PhraseDefinition,

    /// A rule definition: `Every turn: ...`
    RuleDefinition,

    /// An I6 schema at the sentence level: `(- ... -)`
    I6Schema,

    /// A table definition: `Table of ...`
    TableDefinition,

    /// A use option: `Use American dialect.`
    UseOption,
}

impl SyntaxKind {
    /// Returns `true` if this kind is a token (produced by the lexer).
    pub fn is_token(&self) -> bool {
        matches!(
            self,
            SyntaxKind::WORD
                | SyntaxKind::STRING
                | SyntaxKind::I6Block
                | SyntaxKind::TextSubstitution
                | SyntaxKind::COMMENT
                | SyntaxKind::HeadingMarker
                | SyntaxKind::PUNCTUATION
                | SyntaxKind::ParagraphBreak
                | SyntaxKind::NUMBER
                | SyntaxKind::DASH
                | SyntaxKind::EQUALS
                | SyntaxKind::SLASH
                | SyntaxKind::NEWLINE
                | SyntaxKind::WHITESPACE
                | SyntaxKind::ERROR
        )
    }

    /// Returns `true` if this kind is a node (produced by the parser).
    pub fn is_node(&self) -> bool {
        !self.is_token()
    }
}
