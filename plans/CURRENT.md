# Plan 2: I7 Lexer Foundation

**Status**: In progress
**Started**: 2026-06-27
**Target**: 3-5 days

## Goal

Create the `conform7-syntax` crate with a lexer that tokenizes Inform 7 source
text into a flat sequence of tokens. This is the first piece of the I7
frontend and a prerequisite for all subsequent parsing, semantic analysis, and
LSP work.

## Why this is the right next step

The `conform7-inter` crate is complete — it can read, write, and round-trip
textual Inter with 100% fidelity. The next logical piece is the I7 frontend.

The lexer is the smallest independently testable piece of that frontend:
- It has a well-defined input (source text) and output (token stream).
- The C reference (`services/words-module/Chapter 3/Lexer.w`) is clear and
  self-contained.
- It can be built and tested without any parser, world model, or Salsa
  infrastructure.
- It unblocks all subsequent work on the parser and LSP.

## What we build

### Crate: `conform7-syntax`

```
crates/conform7-syntax/
├── Cargo.toml
└── src/
    ├── lib.rs           # Public API, re-exports
    ├── syntax_kind.rs   # SyntaxKind enum for all I7 token/node types
    ├── lexer.rs         # I7 source lexer (tokenizer)
    └── token.rs         # Token type with source location
```

### Capabilities

1. **`SyntaxKind` enum** — Covers all I7 token types:
   - `Word` — any natural language word (case-preserved)
   - `QuotedString` — `"text"` (including text substitutions inside)
   - `I6Bracketed` — `(- ... -)` embedded Inform 6 code
   - `TextSubstitution` — `[...]` inside quoted strings
   - `Comment` — `[...]` outside quoted strings
   - `HeadingMarker` — Volume, Book, Part, Chapter, Section
   - `Punctuation` — `. , : ; ? ! ( ) { }`
   - `ParagraphBreak` — blank line between paragraphs
   - `Number` — integer literal
   - `Whitespace` — spaces, tabs (preserved for round-trip)
   - `Newline` — line endings
   - `Error` — malformed input

2. **Lexer** — State machine that reads I7 source text and produces tokens:
   - Handles quoted strings with text substitutions
   - Handles I6 escape blocks `(- ... -)`
   - Handles comments `[...]` outside strings
   - Handles text substitutions `[...]` inside strings
   - Handles paragraph breaks (blank lines)
   - Handles punctuation marks
   - Handles regular words (case-preserved)
   - Tracks source locations (file, line, column)
   - Reports errors for malformed input (unclosed quotes, etc.)

3. **Tests** — Comprehensive test suite:
   - Basic word tokenization
   - Quoted strings with escapes
   - Text substitutions inside strings
   - I6 escape blocks
   - Comments
   - Paragraph breaks
   - Punctuation
   - Error cases (unclosed quotes, unclosed I6 blocks)
   - Real I7 source snippets from the Standard Rules

## Reference implementation

The C lexer is in `services/words-module/Chapter 3/Lexer.w`. Key design points:

- Words are numbered 0, 1, 2, ... in order of reading.
- Text references throughout I7's data structures are `(w1, w2)` pairs.
- The lexer is a simple state machine, not a parser combinator.
- Case is preserved but the lexer notes whether a word starts with uppercase.
- Paragraph breaks (blank lines) are significant — they end sentences.
- Comments `[...]` are stripped (not stored as words).
- I6 escape blocks `(- ... -)` are stored as single "words" with the full text.

Our Rust lexer follows the same logic but produces a `Token` stream instead of
numbered words, and preserves comments for round-trip fidelity (matching the
approach we took in `conform7-inter`).

## Out of scope

- **Rowan CST/AST integration** — comes after the lexer is stable.
- **Chumsky parser** — comes in the next plan.
- **Salsa database** — comes when we have ASTs to query.
- **Heading parsing** — detecting headings is a parser-level concern
  (though the lexer provides the `HeadingMarker` token type).
- **Sentence classification** — comes in the parser.
- **I6 sub-parser** — `(- ... -)` blocks will be stored as raw text for now.

## Success criteria

- [x] `SyntaxKind` enum covers all I7 token types.
- [x] Lexer correctly tokenizes basic I7 source (words, strings, punctuation).
- [x] Lexer handles quoted strings with text substitutions.
- [x] Lexer handles I6 escape blocks `(- ... -)`.
- [x] Lexer handles comments `[...]` outside strings.
- [x] Lexer handles paragraph breaks.
- [x] Lexer reports errors for malformed input.
- [x] Lexer tokenizes real I7 source snippets (from Standard Rules).
- [x] All tests pass.
- [x] `cargo clippy --all-targets` is clean.
