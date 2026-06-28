# Plan 7: Preform Grammar Parser

**Status**: Complete
**Started**: 2026-06-27
**Completed**: 2026-06-27

## Goal

Build a parser for the Preform grammar format used in `Syntax.preform` files.
This is the first piece of the Preform infrastructure — the pattern-matching
engine that drives all I7 parsing.

The Preform grammar is a text-based format defining ~720 nonterminals for
English alone. Each nonterminal has one or more productions, and each production
is a sequence of tokens (fixed words, wildcards, or sub-nonterminals).

This plan covers **parsing only** — loading the grammar into in-memory data
structures. The matching engine (which takes a nonterminal and a wording and
tries to match it) comes in a later plan.

## Background

The C implementation loads Preform grammar from `Syntax.preform` files at
runtime via `LoadPreform::load()` (see `services/words-module/Chapter 4/Preform.w`).
The grammar format is:

```
<nonterminal-name> internal

<nonterminal-name> ::=
    production1 |
    production2 |
    ...
```

Where productions contain:
- **Fixed words**: literal text like `to`, `is`, `a`, `room`
- **Ellipsis wildcards**: `...` (matches any number of words) or `.....` (matches exactly N words)
- **Sub-nonterminals**: `<quoted-text>`, `<if-start-of-paragraph>`, etc.
- **Internal**: keyword meaning the matching is done by a Rust/C function, not grammar rules

## Tasks

- [x] Add a `preform` module to `conform7-syntax` with:
  - `Nonterminal` struct (name, productions, internal flag)
  - `Production` struct (sequence of production tokens)
  - `ProductionToken` enum (FixedWord, Wildcard, SubNonterminal)
  - `Grammar` struct (collection of nonterminals)
- [x] Implement a `parse_grammar(source: &str) -> Result<Grammar, String>` function
      that parses the Preform text format.
- [x] Handle: nonterminal declarations, `internal` keyword, `::=` separator,
      `|` production separator, fixed words, `...` wildcards, `<name>` sub-nonterminals.
- [x] Add unit tests with small grammar snippets.
- [x] Add a test that loads the real `Syntax.preform` file and verifies the
      nonterminal count matches expectations.
- [x] Update module docs and plan status.

## Success criteria

- [x] A small grammar snippet can be parsed and the resulting data structures
      are correct.
- [x] The real `Syntax.preform` file can be loaded without errors.
- [x] All tests pass.
- [x] `cargo clippy --all-targets` is clean.

## Out of scope

- The Preform matching engine (backtracking, word range matching).
- Internal nonterminal implementations (Rust functions for `internal` NTs).
- Grammar optimization (NT incidence bits, word range extremes).
- The `balanced-text` production format (`.` wildcards).
