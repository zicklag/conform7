# Plan 8: Preform Matching Engine

**Status**: Complete
**Target**: 3-5 days

## Goal

Build the Preform matching engine — the runtime that takes a nonterminal and a
wording and tries to match it against all productions, with backtracking.

This is the heart of I7 parsing. Every `<nonterminal>` call in the grammar
resolves through this engine.

## Scope

- Implement `match_nonterminal(grammar, name, wording) -> Option<Match>` that
  tries each production in order and returns the first successful match.
- Handle fixed word matching, wildcard matching (`...`), and sub-nonterminal
  recursion.
- Track matched word ranges for result extraction.
- Handle backtracking when a sub-nonterminal match fails.
- Add unit tests with small grammars.
- Add a test that matches a simple sentence against a real nonterminal from
  the English Syntax.preform file.

## Out of scope

- Internal nonterminal implementations (Rust functions for `internal` NTs).
- Grammar optimization (NT incidence bits, word range extremes).
- Result extraction (the `*_R` result variables in C).
