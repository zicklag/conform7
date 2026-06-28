# Plan 6: I7 Sentence-to-AST Bridge — Structural Sentences

**Status**: In progress
**Started**: 2026-06-27
**Target**: 1-2 days

## Goal

Extend the sentence-to-AST bridge to handle structural sentences already
classified by the sentence breaker: `Include ...`, `Use ...`, `Table ...`, and
`Equation ...`.

These sentence types do not require the full Preform grammar — their structure is
detectable from classification and token ranges, making them a natural next step
after headings.

## Background

The sentence breaker classifies structural sentences with a `StructuralType`:

- `Include` — include an extension or kit.
- `Use` — enable a compiler option or feature.
- `Table` — declare a table.
- `Equation` — declare an equation.

In the C implementation these map to node types like `INCLUDE_NT`, `BEGINHERE_NT`,
`ENDHERE_NT`, `TABLE_NT`, and `EQUATION_NT` (see
`services/syntax-module/Chapter 3/Sentences.w`).

## Tasks

- [ ] Add a `parse_structural` helper that converts a structural sentence into
      the appropriate `ParseNode`.
- [ ] Map each `StructuralType` to the correct `NodeType`.
- [ ] Preserve the sentence wording on the resulting node.
- [ ] Add unit tests for each structural sentence type.
- [ ] Add an integration test: source text → lexer → sentence breaker →
      structural AST nodes.
- [ ] Update `plans/CURRENT.md` and module docs.

## Success criteria

- [ ] Each structural sentence classification produces a `ParseNode` of the
      corresponding type.
- [ ] Node wording covers the structural sentence body.
- [ ] All tests pass.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- Parsing regular assertion sentences (e.g., "The Lab is a room.").
- Parsing rule preambles and phrase bodies.
- Preform grammar loading or matching.
- Bibliographic sentence handling (the quoted title sentence at the start of a
  source file).
