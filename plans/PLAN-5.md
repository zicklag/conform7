# Plan 5: I7 Sentence-to-AST Bridge — Headings

**Status**: Complete
**Started**: 2026-06-27
**Completed**: 2026-06-27

## Goal

Build the first end-to-end bridge between the sentence breaker and the parse
node model: take a classified heading sentence and construct a `ParseNode`
subtree.

This is the smallest independently testable piece of actual parsing. It does
not involve the full Preform grammar; it uses the structure that the sentence
breaker has already determined.

## Background

The sentence breaker (`crates/conform7-syntax/src/sentence.rs`) already
classifies sentences, including a `Heading` variant with a `HeadingLevel`
(Volume, Book, Part, Chapter, Section). The parse node model
(`crates/conform7-syntax/src/parse_node.rs`) provides `ParseNode` and
`NodeType::Heading`.

The C implementation stores headings as `HEADING_NT` nodes. The heading text is
the wording attached to the node; children are typically absent. Headings are
L1 category nodes and are used to structure the source text into a tree of
nested chapters.

## Tasks

- [x] Add a `SentenceParser` helper in `conform7-syntax` that converts a
      `Sentence` (with classification `Heading`) into a `ParseNode`.
- [x] Extract the heading body text from the sentence's token range (skipping
      the heading marker word and trailing stop token).
- [x] Preserve the heading level as either a node annotation or a separate
      child/metadata field.
- [x] Add unit tests covering each heading level and malformed headings.
- [x] Add an integration test: source text → lexer → sentence breaker →
      heading AST.
- [x] Update `plans/CURRENT.md` and module docs.

## Success criteria

- [x] A heading sentence produces a `ParseNode` of type `HEADING_NT`.
- [x] The node's wording covers the heading text, not the marker or stop token.
- [x] Heading level (Volume/Book/Part/Chapter/Section) is recoverable from the
      resulting node.
- [x] All tests pass.
- [x] `cargo clippy --all-targets` is clean.

## Out of scope

- Parsing regular assertion sentences (e.g., "The Lab is a room.").
- Parsing rule preambles, structural sentences, or imperative code.
- Preform grammar loading or matching.
