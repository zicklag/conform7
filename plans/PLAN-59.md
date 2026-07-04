# Plan 59: AUGMENT_CSEQ — Augment Model World

**Status**: In progress
**Target**: 1 day

## Goal

Wire up the AUGMENT_CSEQ bench: `World::stage_V` with stage ordering validation, and a `MappingHints::traverse_for_map_parameters` stub.

## Background

### C pipeline

From `How To Compile.w` lines 165-169:

```c
@<Augment model world with low-level properties@> =
    BENCH(World::stage_V)                              // add stage ordering check
    BENCH(MappingHints::traverse_for_map_parameters)   // stub (deferred)
```

### Current Rust state

- `World::stage_V` exists as a stub — sets stage and calls `ask_plugins_at_stage` (no-op)
- No `MappingHints` module exists
- All 1541 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-59 the correct next step?

**Yes.** It's the next bench in the C pipeline. It's very small — just stage ordering validation and a stub.

### 2. Is it independently testable?

**Yes.** Test stage ordering validation with synthetic stage transitions.

### 3. What is the smallest independently testable subset?

1. Add stage ordering check to `World::ask_plugins_at_stage`
2. Create `MappingHints` stub
3. Create AUGMENT_CSEQ dispatch

### 4. What simplifications are appropriate?

- `PluginCalls::complete_model` remains a no-op
- `MappingHints::traverse_for_map_parameters` is a no-op stub
- No `mapping_hint` struct, no Preform grammar, no map plugin

## Tasks

### Task 1: Add stage ordering validation to `World`

Modify `world.rs` to add `CURRENT_STAGE + 1 == stage` check in `ask_plugins_at_stage`.

### Task 2: Create `MappingHints` stub

Create `assertions/mapping_hints.rs` with `traverse_for_map_parameters` no-op.

### Task 3: Create AUGMENT_CSEQ dispatch

Create `assertions/augment_cseq.rs` with `run_augment_cseq()`.

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-59: AUGMENT_CSEQ — Augment Model World"`

## Success Criteria

- [ ] `World::stage_V` validates stage ordering
- [ ] `MappingHints::traverse_for_map_parameters` is a no-op stub
- [ ] `run_augment_cseq` calls both benches without panic
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **Real `MappingHints` implementation** — deferred (needs syntax tree traversal, special meanings, Preform grammar, map plugin)
- **`PluginCalls::complete_model`** — deferred
- **PHRASES_CSEQ** — deferred
- **Inter generation** — deferred
