# Plan 55: Assertion Matrix — Pass 1 Dispatch and Case Skeleton

**Status**: In progress
**Target**: 2 days

## Goal

Implement the assertion matrix dispatch skeleton (`Assertions::which_assertion_case`, `Assertions::make_coupling`) and wire up `MajorNodes::pass_1`. This is the first part of the `ASSERTIONS_PASS_1_CSEQ` bench.

## Background

### C pipeline

After `MajorNodes::pre_pass` (PLAN-53) and the Refiner (PLAN-54), the next step is `MajorNodes::pass_1` which processes assertions by dispatching to the 42-case assertion matrix.

### The assertion matrix

The matrix is a 12×12 grid mapping node type pairs to case numbers:

```
                   AND  WITH  XoY  KIND  ALLOW  PLIST  ADJ  ACTN  REL  EVRY  CN   PN
AND_NT           {  1,   2,   1,   1,   1,   1,   1,   1,   1,  16,   1,   1 }
WITH_NT          {  3,   4,   3,   3,   3,   3,   3,   3,  14,  16,   3,   3 }
X_OF_Y_NT        {  5,   2,   6,   7,   9,   7,   7,   7,  20,  16,  23,   7 }
KIND_NT          {  5,   2,   8,   8,   9,   8,   8,   8,   8,  16,   8,   8 }
ALLOWED_NT       {  5,   2,  10,  10,  9,  10,  10,  10,  10,  25,  25,  25 }
PROPERTY_LIST_NT {  5,   2,  11,  12,  9,  18,  22,  19,  20,  16,  18,  18 }
ADJECTIVE_NT     {  5,   2,  13,  12,  9,  22,  22,  24,  20,  16,  29,  29 }
ACTION_NT        {  5,   2,  11,  19,  9,  19,  19,  27,  20,  16,  32,  32 }
RELATIONSHIP_NT  {  5,  15,  21,  20,  9,  20,  42,  20,  28,  31,  34,  36 }
EVERY_NT         { 17,  17,  17,  17, 17,  17,  17,  17,  17,  33,  17,  17 }
COMMON_NOUN_NT   {  5,   2,  11,  12,  9,  18,  30,  19,  35,  16,  38,  39 }
PROPER_NOUN_NT   {  5,   2,  26,  12,  9,  18,  30,  19,  37,  16,  40,  41 }
```

### Current Rust state

- `MajorNodes::pre_pass` — complete
- `MajorNodes::pass_1` — stub
- `Refiner` — complete (PLAN-54)
- `Creator` — stub
- All 1501 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-55 the correct next step?

**Yes.** The Refiner is complete, so the next step is the assertion matrix dispatch that pass_1 uses.

### 2. Is it independently testable?

**Yes.** The matrix lookup is a pure function. Cases 1, 3, 5 (recursive splitting) can be tested with synthetic parse trees.

### 3. What is the smallest independently testable subset?

1. `Assertions::which_assertion_case(px, py)` — pure function, 144 matrix entries
2. `Assertions::make_coupling(px, py)` — entry point
3. Cases 1, 3, 5 implemented (recursive splitting)
4. All other cases as stubs
5. `MajorNodes::pass_1` wired up

### 4. What simplifications are appropriate?

- **All complex cases are stubs** — only cases 1, 3, 5 (recursive splitting) are real
- **All plugin-dependent paths are stubs** — `PluginCalls::intervene_in_assertion` returns false
- **All IF-model-specific paths are stubs** — map directions, action patterns
- **All complex modules are stubs** — PropertyKnowledge, Relational, Implications, Assemblies, NewPropertyAssertions, SpecialMeanings

## Tasks

### Task 1: Create `Assertions` module

Create `crates/conform7-semantics/src/assertions/assertions.rs` with:
- `Assertions::which_assertion_case(px, py)` — the 12×12 matrix
- `Assertions::make_coupling(px, py)` — entry point
- `Assertions::make_assertion_recursive_inner(px, py)` — dispatch
- `Assertions::allow_node_type(p)` — check ASSERT_NFLAG
- All 42 case handlers (cases 1, 3, 5 real; rest stubs)

### Task 2: Create stub modules

Create stub modules for:
- `property_knowledge.rs`
- `relational.rs`
- `new_property_assertions.rs`
- `implications.rs`
- `assemblies.rs`
- `special_meanings.rs`

### Task 3: Wire up `MajorNodes::pass_1`

Modify `major_nodes.rs` to implement `pass_1`:
- Call `Refiner::refine_coupling(px, py, false)` for each SENTENCE_NT
- Dispatch to `Assertions::make_coupling(px, py)` or `try_special_meaning`

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-55: Assertion Matrix — Pass 1 Dispatch and Case Skeleton"`

## Success Criteria

- [ ] `Assertions::which_assertion_case` correctly maps all 144 matrix entries
- [ ] `Assertions::make_coupling` dispatches to the correct case
- [ ] Case 1 splits AND on py into sub-assertions
- [ ] Case 5 splits AND on px into sub-assertions
- [ ] Case 3 splits WITH on py into sub-assertions
- [ ] All 42 cases are handled (most as stubs)
- [ ] `MajorNodes::pass_1` traverses the tree and refines/dispatches sentences
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
- **Tables::traverse_to_stock** — deferred
- **MajorNodes::pass_2** — deferred
- **Cases 2, 4, 6-42 (non-trivial implementations)** — deferred
