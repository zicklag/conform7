# Plan 56: MajorNodes::pass_2 — Second Pass Through Major Nodes

**Status**: In progress
**Target**: 1 day

## Goal

Implement `MajorNodes::pass_2` — the second pass through major nodes (ASSERTIONS_PASS_2_CSEQ). This is the last of the three passes, handling textual sentences, INFORM6CODE_NT, BIBLIOGRAPHIC_NT, and pass-2-specific assertion matrix guards.

## Background

### C pipeline

After `MajorNodes::pass_1` (PLAN-55), the next bench is `ASSERTIONS_PASS_2_CSEQ`:

```c
BENCH(MajorNodes::pre_pass)       // PLAN-53
BENCH(Task::verify)                // deferred
BENCH(MajorNodes::pass_1)         // PLAN-55
BENCH(Tables::traverse_to_stock)  // deferred
BENCH(MajorNodes::pass_2)         // ← THIS PLAN
```

### What pass_2 does differently from pass_1

| Aspect | pass_1 | pass_2 |
|--------|--------|--------|
| SENTENCE_NT (textual) | ignored | `Assertions::make_appearance(p)` |
| SENTENCE_NT (non-textual, special meaning) | `PASS_1_SMFT` | `PASS_2_SMFT` |
| SENTENCE_NT (non-textual, regular) | refine + dispatch | skip refinement, dispatch to matrix |
| INFORM6CODE_NT | ignored | `InterventionRequests::make(p)` |
| BIBLIOGRAPHIC_NT | ignored | `BibliographicData::bibliographic_data(p)` |
| Post-traversal | nothing | `World::deduce_object_instance_kinds()` |

### Current Rust state

- `MajorNodes::pre_pass` — complete
- `MajorNodes::pass_1` — complete (PLAN-55)
- `MajorNodes::pass_2` — stub
- `Assertions` module — complete with 42-case matrix
- All 1520 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-56 the correct next step?

**Yes.** It's the last of the three passes, directly after pass_1. All prerequisites exist.

### 2. Is it independently testable?

**Yes.** Create a syntax tree with various node types, run pass_2, verify correct dispatch.

### 3. What is the smallest independently testable subset?

1. Wire up pass_2 traversal in `MajorNodes::visit`
2. Implement `process_sentence_pass_2` with textual/non-textual dispatch
3. Add pass-2-specific assertion matrix guards (reject three forms, case 8 early return)
4. Stub `Assertions::make_appearance`, `InterventionRequests::make`, `BibliographicData`

### 4. What simplifications are appropriate?

- All pass-2-specific problem messages are stubs
- `World::deduce_object_instance_kinds` is a no-op
- `InterventionRequests::make` stores nothing
- `BibliographicData::bibliographic_data` is a no-op
- `Assertions::make_appearance` is a stub
- All assertion matrix cases remain stubs (only pass-2 guards added)

## Tasks

### Task 1: Wire up `MajorNodes::pass_2`

Modify `major_nodes.rs` to add pass=2 handling in `visit`:
- SENTENCE_NT → `process_sentence_pass_2(node)`
- INFORM6CODE_NT → `InterventionRequests::make(node)`
- BIBLIOGRAPHIC_NT → stub

### Task 2: Add pass-2-specific assertion matrix guards

Modify `assertions.rs` to add:
- `@<Reject three forms of assertion@>` logic
- Case 8 early return on pass 2

### Task 3: Add stub modules

- `intervention_requests.rs` — `InterventionRequests::make` stub
- `bibliographic_data.rs` — `BibliographicData::bibliographic_data` stub

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-56: MajorNodes::pass_2 — Second Pass Through Major Nodes"`

## Success Criteria

- [ ] `MajorNodes::pass_2` traverses the tree and dispatches by node type
- [ ] Textual sentences call `Assertions::make_appearance`
- [ ] Non-textual sentences dispatch to assertion matrix (skip refinement)
- [ ] INFORM6CODE_NT calls `InterventionRequests::make`
- [ ] BIBLIOGRAPHIC_NT is handled (no panic)
- [ ] Pass-2-specific assertion matrix guards are in place
- [ ] Case 8 returns early on pass 2
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **Real Creator implementation** — deferred
- **Real PropertyKnowledge** — deferred
- **Real Relational assertions** — deferred
- **Real Implications** — deferred
- **Real Assemblies** — deferred
- **Real NewPropertyAssertions** — deferred
- **Real SpecialMeanings** — deferred
- **Real InterventionRequests storage** — deferred
- **Real BibliographicData** — deferred
- **Real World::deduce_object_instance_kinds** — deferred
- **Real make_appearance** — deferred
- **MODEL_CSEQ** — deferred
- **Cases 2, 4, 6-42 (non-trivial implementations)** — deferred
