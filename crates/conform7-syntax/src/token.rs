//! Token type for the I7 lexer.
//!
//! Each `Token` carries its `SyntaxKind`, the source text it covers, and
//! its source location (line, column).

use crate::SyntaxKind;

/// A single token produced by the I7 lexer.
///
/// Tokens are produced in source order. The `text` field contains the exact
/// source text of the token (preserving case, quotes, etc.). The `range`
/// gives the byte offset range in the original source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// The kind of this token.
    pub kind: SyntaxKind,

    /// The exact source text of this token.
    ///
    /// **Note**: Currently a heap-allocated `String` for simplicity.
    /// In a future optimization pass, this could borrow from the source
    /// string via a lifetime parameter to avoid per-token allocations.
    pub text: String,

    /// The byte range of this token in the original source.
    pub range: std::ops::Range<usize>,

    /// The line number (1-indexed) where this token starts.
    pub line: usize,

    /// The column number (1-indexed, in bytes) where this token starts.
    ///
    /// **Note**: Columns are byte-based, not Unicode code-point or
    /// grapheme-cluster based. For ASCII source this is equivalent to
    /// character position. For source with multi-byte characters (e.g.,
    /// em dashes, smart quotes), the column counts raw bytes. The LSP
    /// layer will need to translate to UTF-16 code units for protocol
    /// compliance.
    pub column: usize,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: SyntaxKind, text: impl Into<String>, range: std::ops::Range<usize>, line: usize, column: usize) -> Self {
        Self {
            kind,
            text: text.into(),
            range,
            line,
            column,
        }
    }

    /// Create an error token.
    pub fn error(message: impl Into<String>, range: std::ops::Range<usize>, line: usize, column: usize) -> Self {
        Self {
            kind: SyntaxKind::ERROR,
            text: message.into(),
            range,
            line,
            column,
        }
    }
}
