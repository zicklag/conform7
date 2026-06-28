# Plan 3: I7 Sentence Splitter

**Status**: Complete
**Completed**: 2026-06-27

## Goal

Build a sentence splitter that takes the token stream from the lexer and
breaks it into sentences, classifying each sentence by type (heading,
structural, or regular). This is the second stage of the I7 frontend and
the foundation for all subsequent semantic analysis.

## Why this is the right next step

The lexer is complete — it produces a flat `Vec<Token>`. The next step in
the C pipeline (`services/syntax-module/Chapter 3/Sentences.w`) is to break
that token stream into sentences. This is:

- **Independently testable**: takes tokens in, produces sentences out.
  No world model, no Salsa, no Chumsky needed.
- **Well-defined**: the C reference is clear and self-contained (~400 lines).
- **Builds on the lexer**: uses the same `conform7-syntax` crate.
- **Unblocks everything**: the parser, semantic analysis, and LSP all
  consume sentences, not raw tokens.

## What we build

### New module: `crates/conform7-syntax/src/sentence.rs`

A `SentenceBreaker` struct that implements the sentence-breaking FSM:

```rust
/// A classified sentence from an I7 source file.
pub struct Sentence {
    /// The range of tokens this sentence covers.
    pub token_range: Range<usize>,
    /// The classification of this sentence.
    pub classification: SentenceClassification,
}

/// How a sentence is classified.
pub enum SentenceClassification {
    /// A heading: "Chapter 1 - The Beginning"
    Heading { level: HeadingLevel },
    /// A structural sentence: "Include ...", "Table ...", "Equation ..."
    Structural(StructuralType),
    /// A regular sentence (assertion, phrase, rule, etc.)
    Regular,
    /// A rule preamble ending with a colon
    RulePreamble,
    /// A rule body phrase ending with a semicolon
    RulePhrase,
}

/// Heading levels matching the C hierarchy.
pub enum HeadingLevel {
    Volume = 1,
    Book = 2,
    Part = 3,
    Chapter = 4,
    Section = 5,
}

/// Types of structural sentences.
pub enum StructuralType {
    Include,
    Table,
    Equation,
    Use,
    // ... more as discovered
}
```

### Capabilities

1. **Sentence breaking** — Scan through tokens and split on:
   - Full stops `.` (standard sentence end)
   - Semicolons `;` (rule phrase separator)
   - Colons `:` (rule preamble end)
   - Paragraph breaks (blank lines)
   - Quoted text ending with punctuation followed by a capital letter

2. **Heading detection** — Classify sentences starting with `Volume`,
   `Book`, `Part`, `Chapter`, or `Section` at paragraph start.

3. **Structural sentence detection** — Classify sentences starting with
   structural keywords (`Include`, `Table`, `Equation`, `Use`, etc.).

4. **Rule mode** — Track when we're inside a rule definition (preamble
   ending with colon, followed by phrases ending with semicolons).

5. **Error handling** — Report malformed sentences (unexpected semicolons,
   headings with line breaks, etc.).

### Tests

- Basic sentence splitting (simple assertions)
- Heading detection and level assignment
- Rule preamble + phrase splitting
- Structural sentence detection
- Quoted text sentence boundaries
- Paragraph break sentence boundaries
- Error cases (unexpected semicolons, etc.)
- Real I7 source snippets (Standard Rules excerpts)

## Reference implementation

The C sentence breaker is in `services/syntax-module/Chapter 3/Sentences.w`.
Key design points:

- A finite state machine (`syntax_fsm_state`) tracks:
  - Current source file
  - Whether we're inside a rule (`inside_rule_mode`)
  - Whether we're inside a table (`inside_table_mode`)
  - Extension position state
  - Skipping material level (for conditional compilation by heading)

- Sentence-ending punctuation:
  - `.` (FULLSTOP_V) — always ends a sentence
  - `;` (SEMICOLON_V) — ends a phrase within a rule
  - `:` (COLON_V) — ends a rule preamble (with exceptions for time notation)
  - `|` (PARBREAK_V) — paragraph break, always ends a sentence
  - `X` break — quoted text ending with `?!.` followed by a capital letter

- Heading detection uses the `<dividing-sentence>` Preform nonterminal
  which returns a heading level (1-10) or extension begin/end markers.

- Structural sentences are detected by the `<structural-sentence>` nonterminal.

- Rule mode: after a colon, subsequent sentences ending with `;` are rule
  phrases; a sentence ending with `.` or paragraph break exits rule mode.

## Out of scope

- **Chumsky parser integration** — comes after sentence splitting is stable.
- **Salsa database** — comes when we have ASTs to query.
- **Semantic analysis** — classifying sentences into assertions, phrases,
  rules, etc. is a later concern.
- **I6 sub-parser** — `(- ... -)` blocks are stored as raw text for now.
- **Extension handling** — `begins here` / `ends here` detection is deferred.
- **Conditional compilation** — heading-level skipping is deferred.

## Known limitations

- **Consecutive stops with whitespace**: The C lexer numbers words (whitespace
  is not a word), so `. ;` counts as two consecutive stops. Our token stream
  includes whitespace tokens, so we stop counting at the whitespace. The
  stop character used for classification may differ from C in rare cases
  (first stop vs. last), but the number of non-empty sentences is the same.
- **COMMENT tokens between X break components**: The C lexer strips comments,
  so they never appear between quoted text and the next word. Our lexer
  preserves comments, so a comment between `"Look out!"` and `The` would
  prevent X break detection. This is rare in practice.
- **Table mode only checks for `"Table"`**: The C `<structural-sentence>`
  nonterminal detects multiple tabbed structural types, but only `Table`
  uses tabbed rows in I7.

## Success criteria

- [x] Sentence breaker correctly splits simple I7 source into sentences.
- [x] Heading detection works (Volume, Book, Part, Chapter, Section).
- [x] Rule preamble + phrase splitting works.
- [x] Structural sentence detection works (Include, Table, etc.).
- [x] Quoted text sentence boundaries work.
- [x] Paragraph breaks end sentences.
- [x] Error cases are handled gracefully.
- [x] All tests pass.
- [x] `cargo clippy --all-targets` is clean.
