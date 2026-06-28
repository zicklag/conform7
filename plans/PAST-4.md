# Plan 4: I7 Parse Node Foundation

**Status**: Complete
**Started**: 2026-06-27
**Completed**: 2026-06-27

## Goal

Build the in-memory representation of Inform 7 syntax trees: `ParseNode`,
`wording` (source text ranges), and the enumerated node type system. This is
the data model that will later be produced by the Preform-driven parser and
consumed by semantic analysis.

This plan does **not** implement the grammar parser itself. We are building the
**tree data model and infrastructure** first, following principle #8 (data
model before I/O) and principle #10 (start small, ship early).

## Background

The C implementation defines parse nodes in:

- `services/syntax-module/Chapter 2/Parse Nodes.w` — the `parse_node` struct,
  tree composition, copying, and traversal.
- `services/syntax-module/Chapter 2/Node Types.w` — enumerated node types,
  metadata (name, min/max children, category, flags), and hierarchy rules.
- `inform7/core-module/Chapter 1/Core Preform.w` — Inform-only node types and
  annotations (to be consulted for the full set of node types used by I7).

Key concepts to port:

- `parse_node` has a `wording` (word range), a `node_type_t`, annotations,
  `down` (first child), `next` (sibling), and `next_alternative` (fork to
  alternative interpretation).
- Enumerated node types have bit 32 set and metadata in a table; other node
  types are meaning codes from the Preform vocabulary.
- Node type categories (`L1_NCAT`, `L2_NCAT`, etc.) enforce which nodes can be
  children of which others.
- Word ranges (`wording`) are lightweight references into the source text.

## Tasks

- [x] Add a `wording` module to `conform7-syntax` with source text range types.
- [x] Add a `parse_node` module with `ParseNode` struct, child/sibling links,
      and basic constructors.
- [x] Add a `node_type` module with an enumerated node type enum and metadata
      (name, min/max children, category, flags) for the core syntax node types.
- [x] Implement tree traversal helpers: children iterator, find by type,
      contains, depth-first walk.
- [x] Implement `Debug`/`Display` printing of parse trees for tests.
- [x] Add unit tests for construction, composition, traversal, and metadata.
- [x] Update `plans/CURRENT.md` and `conform7-syntax` module docs.

## Success criteria

- [x] `ParseNode` tree can represent a simple sentence (e.g., a heading node
      with its text range as a child).
- [x] Enumerated node type metadata is correct for the core set.
- [x] Tree traversal helpers are tested.
- [x] All tests pass.
- [x] `cargo clippy --all-targets` is clean.

## Out of scope

- Preform grammar rules.
- Parsing actual I7 sentences into ASTs.
- Annotations (to be added once the basic tree is solid).
- Alternative interpretations / `next_alternative` parsing logic.
