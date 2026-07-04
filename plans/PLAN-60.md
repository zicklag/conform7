# Plan 60: PHRASES_CSEQ — Phrases and Rules (ImperativeDefinitions::assess_all)

**Status**: In progress
**Target**: 1 day

## Goal

Implement `ImperativeDefinitions::assess_all` — the 4-step assessment of imperative definitions — and wire up the PHRASES_CSEQ dispatch. The other 4 benches (LiteralPatterns, Equations, Tables, RTRulebooks) are stubs.

## Background

### C pipeline

From `How To Compile.w` lines 171-178:

```c
@<Phrases and rules@> =
    BENCH(LiteralPatterns::define_named_phrases)          // stub (deferred)
    BENCH(ImperativeDefinitions::assess_all)              // ← THIS PLAN
    BENCH(Equations::traverse_to_stock)                   // stub (deferred)
    BENCH(Tables::traverse_to_stock)                      // stub (deferred)
    BENCH(RTRulebooks::RulebookOutcomePrintingRule)       // stub (deferred)
```

### What assess_all does

4-step process:
1. **Assess**: Loop over `imperative_defn` instances, call family `assess`, create `id_body`, call `given_body`
2. **Register**: Loop over families, call `register`
3. **Make runtime context data**: Loop over defns, call `to_phrcd`
4. **Complete**: Loop over families, call `assessment_complete`

### Current Rust state

- `ImperativeDefinitionFamilies` — fully implemented with all method slots
- `AdjectivalDefinitionFamily`, `ToPhraseFamily`, `RuleFamily` — wired with stub methods
- `ImperativeSubtrees` — only `accept` stub exists
- All 1542 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-60 the correct next step?

**Yes.** PHRASES_CSEQ is the next bench. `ImperativeDefinitions::assess_all` is the most tractable — the family dispatch layer already exists.

### 2. Is it independently testable?

**Yes.** Create synthetic `ImperativeDefn` instances, call `assess_all`, verify all 4 steps execute.

### 3. What is the smallest independently testable subset?

1. `ImperativeDefn` struct with family reference
2. `IdBody` struct (minimal)
3. `ImperativeDefinitions::assess_all` — 4-step loop
4. PHRASES_CSEQ dispatch with 4 stubs

### 4. What simplifications are appropriate?

- No linked list — use `Vec<ImperativeDefn>`
- No `CompileImperativeDefn::initialise_stack_frame` — no-op
- No problem reporting
- All other benches are no-op stubs

## Tasks

### Task 1: Create `ImperativeDefinitions` module

Create `assertions/imperative_definitions.rs` with `ImperativeDefn`, `IdBody`, and `assess_all`.

### Task 2: Create PHRASES_CSEQ dispatch

Create `assertions/phrases_cseq.rs` with `run_phrases_cseq()`.

### Task 3: Add `accept_all` stub to `ImperativeSubtrees`

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-60: PHRASES_CSEQ — Phrases and Rules (ImperativeDefinitions::assess_all)"`

## Success Criteria

- [ ] `ImperativeDefinitions::assess_all` executes all 4 steps
- [ ] Step 1 calls family `assess` and `given_body` for each defn
- [ ] Step 2 calls family `register` for each family
- [ ] Step 3 calls family `to_phrcd` for each defn
- [ ] Step 4 calls family `assessment_complete` for each family
- [ ] `run_phrases_cseq` calls all 5 benches without panic
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **`LiteralPatterns::define_named_phrases`** — deferred
- **`Equations::traverse_to_stock`** — deferred
- **`Tables::traverse_to_stock`** — deferred
- **`RTRulebooks::RulebookOutcomePrintingRule`** — deferred
- **Real `ImperativeSubtrees::accept_all`** — deferred
- **Inter generation** — deferred
