# Plan 57: World Module — Model Completion Stages

**Status**: In progress
**Target**: 1 day

## Goal

Create the World module (`knowledge/world.rs`) with the five model-building stages, add `complete_model` and `check_model` dispatch to `InferenceSubjectFamilyMethods`, and wire up `World::deduce_object_instance_kinds` in `MajorNodes::pass_2`.

## Background

### C pipeline

After the three passes through major nodes, the next bench is `MODEL_CSEQ`:

```c
BENCH(RTKindDeclarations::declare_base_kinds)     // Inter emission, defer
BENCH(Translations::traverse_for_late_namings)     // depends on SpecialMeanings, defer
BENCH(OrderingInstances::objects_in_definition_sequence)  // already implemented
BENCH(World::stages_II_and_III)                    // ← THIS PLAN
BENCH(World::stage_IV)                             // ← THIS PLAN
```

### Current Rust state

- All three passes through major nodes done (PLAN-53/55/56)
- `OrderingInstances` fully implemented
- `InferenceSubjectFamilyMethods` has `complete_model` and `check_model` fields but no dispatch methods
- `MajorNodes::pass_2` has a stub for `World::deduce_object_instance_kinds`
- All 1526 tests pass, clippy clean, no unsafe code

## Decision

### 1. Is PLAN-57 the correct next step?

**Yes.** The World module is the natural next piece — it's referenced by `MajorNodes::pass_2` and is independently testable.

### 2. Is it independently testable?

**Yes.** Create the World module with stubs for all complex paths, add method dispatch, test with synthetic inference subjects.

### 3. What is the smallest independently testable subset?

1. Create `knowledge/world.rs` with 5 stages and plugin notification
2. Add `complete_model`/`check_model` dispatch to `InferenceSubjects`
3. Wire up `World::deduce_object_instance_kinds` in `MajorNodes::pass_2`

### 4. What simplifications are appropriate?

- All plugin calls are stubs
- `Properties::Appearance::reallocate` is a no-op
- `Assertions::Implications::consider_all` is a no-op
- `complete_model`/`check_model` default implementations are no-ops
- No Inter emission (RTKindDeclarations deferred)
- No Translations traversal (depends on SpecialMeanings, deferred)

## Tasks

### Task 1: Create `knowledge/world.rs`

With `World` struct, stage constants, `deduce_object_instance_kinds`, `stages_II_and_III`, `stage_IV`, `stage_V`, `ask_plugins_at_stage`, `current_building_stage`.

### Task 2: Add `complete_model`/`check_model` dispatch to `InferenceSubjects`

Add dispatch methods to `InferenceSubjectFamilyMethods`.

### Task 3: Wire up `World::deduce_object_instance_kinds` in `MajorNodes::pass_2`

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-57: World Module — Model Completion Stages"`

## Success Criteria

- [ ] `World::deduce_object_instance_kinds` calls plugin mechanism
- [ ] `World::stages_II_and_III` iterates inference subjects and calls `complete_model`
- [ ] `World::stage_IV` iterates inference subjects and calls `check_model`
- [ ] `InferenceSubjects::complete_model` dispatches to family method
- [ ] `InferenceSubjects::check_model` dispatches to family method
- [ ] `World::current_building_stage` tracks stage
- [ ] `MajorNodes::pass_2` calls `World::deduce_object_instance_kinds` after traversal
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **RTKindDeclarations::declare_base_kinds** — Inter emission, deferred
- **Translations::traverse_for_late_namings** — depends on SpecialMeanings, deferred
- **Real plugin system** — deferred
- **Real property permission checks** — deferred
- **Real property contradiction checks** — deferred
- **Real relation permission checks** — deferred
- **Real Implications** — deferred
- **Real Properties::Appearance** — deferred
- **Real SpecialMeanings** — deferred
