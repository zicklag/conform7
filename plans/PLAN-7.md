# Plan 7: I7 Sentence-to-AST Bridge — Rule Preambles

**Status**: Proposed
**Target**: 1-2 days

## Goal

Extend the sentence-to-AST bridge to handle rule preambles — sentences like
`To look upwards:` that define new phrases. The sentence breaker already
classifies these as `RulePreamble`; this plan adds a `parse_rule_preamble()`
function that produces a structured `ParseNode` subtree.

## Background

A rule preamble has the form:

```
To [decide] [whether] phrase-name [(arguments)]: [phrase-options]
```

Examples:
- `To look upwards:`
- `To decide which number is the square root of (N - a number):`
- `To decide whether or not a room is dark:`

In the C implementation, rule preambles are parsed by the Preform grammar
(`<rule-preamble>` nonterminal) and produce `DEFN_CONT_NT` nodes with children
for the phrase name, arguments, and options.

For Plan 7, we do a simplified parse: extract the phrase name (words before
`(` or `:`) and the argument list (parenthesized groups), without the full
Preform engine. This gives us a structured AST that can be refined later.

## Tasks

- [ ] Add a `parse_rule_preamble` helper that converts a `RulePreamble` sentence
      into a `DEFN_CONT_NT` parse node with children.
- [ ] Extract the phrase name as a child node (e.g., `UNKNOWN_NT` or a new
      `PhraseName` node type).
- [ ] Extract each parenthesized argument group as a child node.
- [ ] Detect `to decide` / `to decide whether` modifiers.
- [ ] Add unit tests for various preamble forms.
- [ ] Add an integration test: source text → lexer → sentence breaker →
      preamble AST.
- [ ] Update module docs.

## Success criteria

- [ ] A rule preamble produces a `DEFN_CONT_NT` node.
- [ ] The phrase name is recoverable from the node's children.
- [ ] Arguments (if any) are recoverable from the node's children.
- [ ] All tests pass.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- Parsing rule phrase bodies (the imperative code after the colon).
- Full Preform grammar matching.
- Phrase option detection (e.g., `in the presence of a room`).
