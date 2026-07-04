# Plan 61: Inter Emission Infrastructure — Core Primitives

**Status**: In progress
**Target**: 2 days

## Goal

Create the core Inter emission infrastructure: an `emit` module in `conform7-inter` with primitive emission functions for constants, pragmas, and child packages, along with a `conform7-runtime` crate for I7-level emission.

## Background

### C pipeline

After PHRASES_CSEQ, the next bench is INTER1_CSEQ: 5 parts of Inter generation. The emission infrastructure is foundational — no Inter can be generated without it.

### Current Rust state

- `conform7-inter` crate: `inter::tree` (InterTree, Package, Symbol), `inter::instruction` (Instruction), `inter::textual` (reader/writer), `inter::value` (value pairs)
- No emission primitives exist (no way to add instructions to the tree)
- All 1549 tests pass, clippy clean, no unsafe code

### What needs to be built

1. **Emit primitives** in `conform7-inter`: functions to add instructions to an InterTree
2. **`conform7-runtime` crate**: I7-level emission wrapping the primitives
3. **First simple bench**: `RTCommandGrammars::compile_non_generic_constants` — 2 numeric constants

## Decision

### 1. Is PLAN-61 the correct next step?

**Yes.** Inter emission is the next bench in the C pipeline. The core primitives are independently testable — they manipulate the InterTree directly without needing any world model state.

### 2. Is it independently testable?

**Yes.** Create an InterTree, emit a constant, write it as text, verify the output. Can round-trip through textual reader/writer.

### 3. What is the smallest independently testable subset?

1. `emit_numeric_constant` — adds a `constant` instruction with INT32 value
2. `emit_text_constant` — adds a `constant` instruction with TEXT value
3. `emit_pragma` — adds a `pragma` instruction
4. `emit_child_package` — creates a new child package
5. `conform7-runtime` crate with hierarchy and emit modules
6. `RTCommandGrammars::compile_non_generic_constants` — 2 numeric constants

### 4. What simplifications are appropriate?

- No `inter_name` struct — use `(symbol_name: &str, package)` directly
- No `inter_bookmark` — append instructions at end of package
- No `packaging_state` push/pop — always know which package to emit into
- No `Hierarchy::find` / `make_available` — pass explicit paths
- No metadata annotations — deferred

## Tasks

### Task 1: Add `emit` module to `conform7-inter`

Create `crates/conform7-inter/src/emit.rs` with `Emit` struct and primitive functions.

### Task 2: Create `conform7-runtime` crate

Create `crates/conform7-runtime/` with `Cargo.toml`, `src/lib.rs`, and modules for hierarchy and emit.

### Task 3: Implement `RTCommandGrammars::compile_non_generic_constants`

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-61: Inter Emission Infrastructure — Core Primitives"`

## Success Criteria

- [ ] `emit_numeric_constant` adds a `constant` instruction to the specified package
- [ ] `emit_text_constant` adds a `constant` instruction with TEXT value
- [ ] `emit_pragma` adds a `pragma` instruction
- [ ] `emit_child_package` creates a new child package
- [ ] `conform7-runtime` crate compiles with hierarchy and emit modules
- [ ] `compile_non_generic_constants` emits 2 numeric constants
- [ ] All emitted Inter can be round-tripped through textual reader/writer
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **`RTUseOptions::compile`** — deferred (needs use-option state)
- **`Interventions::make_all`** — deferred
- **`RTKindConstructors::compile`** — deferred (needs package creation)
- **`RTLiteralPatterns::compile`** — deferred (needs code-level emission)
- **`inter_name` abstraction** — deferred
- **`inter_bookmark`** — deferred
- **`Hierarchy::find` / `make_available`** — deferred
- **Metadata annotations** — deferred