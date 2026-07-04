# Plan 58: TABLES_CSEQ — Tables and Grammar

**Status**: In progress
**Target**: 1 day

## Goal

Wire up the TABLES_CSEQ bench: `Measurements::validate_definitions`, `BinaryPredicateFamilies::second_stock`, and `Tables::check_tables_for_kind_clashes`. Two of three benches are already implemented; the third needs a minimal `Table` struct and clash-checking logic.

## Background

### C pipeline

From `How To Compile.w` lines 158-163:

```c
@<Tables and grammar@> =
    BENCH(Measurements::validate_definitions)          // already implemented
    BENCH(BinaryPredicateFamilies::second_stock)       // already implemented
    BENCH(Tables::check_tables_for_kind_clashes)       // ← THIS PLAN
```

### Current Rust state

- `Measurements::validate_definitions` — fully implemented in `knowledge/measurements.rs`
- `BinaryPredicateFamilies::second_stock` — fully implemented in `calculus/binary_predicate_families.rs`
- `Tables` module — stub with only `create_table` (no-op)
- All 1537 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-58 the correct next step?

**Yes.** TABLES_CSEQ is the next bench in the C pipeline. Two of three benches are already done; the third is small and independently testable.

### 2. Is it independently testable?

**Yes.** `check_tables_for_kind_clashes` takes a `&[Table]` and kind-checking function, returns clash descriptions. Testable with synthetic data.

### 3. What is the smallest independently testable subset?

1. Add minimal `Table` struct with `table_name_text`
2. Implement `Tables::check_tables_for_kind_clashes`
3. Wire up TABLES_CSEQ dispatch

### 4. What simplifications are appropriate?

- Minimal `Table` struct (only `table_name_text` needed)
- No Preform grammar — string match against known kind names
- No problem message system — return `Vec<String>` of clash descriptions
- `create_table` remains a stub

## Tasks

### Task 1: Add minimal `Table` struct

Modify `tables.rs` to add `Table` struct and `check_tables_for_kind_clashes`.

### Task 2: Wire up TABLES_CSEQ dispatch

Create a dispatch function that calls all three benches.

### Task 3: Add unit tests

### Task 4: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-58: TABLES_CSEQ — Tables and Grammar"`

## Success Criteria

- [ ] `Tables::check_tables_for_kind_clashes` detects table names matching kind names
- [ ] Only subkinds of object trigger clash reports
- [ ] `Measurements::validate_definitions` is called from dispatch
- [ ] `BinaryPredicateFamilies::second_stock` is called from dispatch
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **`Tables::traverse_to_stock`** — deferred
- **`Tables::create_table`** — remains a stub
- **`MappingHints::traverse_for_map_parameters`** — deferred
- **`LiteralPatterns::define_named_phrases`** — deferred
- **`ImperativeDefinitions::assess_all`** — deferred
- **Inter generation** — deferred
