# Plan 52: BootVerbs::make_built_in — Bootstrap Verbs and Special Meanings

**Status**: In progress
**Target**: 1 day

## Goal

Implement `BootVerbs::make_built_in` — the fourth and final step of the `BUILT_IN_STUFF_CSEQ` compilation bench. This creates the bootstrap verbs "to be" and "to mean", declares ~20 special meaning functions, registers all verb usages, and defines the built-in relation name constants.

## Background

### C reference

From `gitignore/inform/inform7/core-module/Chapter 1/How To Compile.w`, lines 126-132:

```c
@<Build a rudimentary set of kinds, relations, verbs and inference subjects@> =
    BENCH(InferenceSubjects::make_built_in);            // PLAN-50
    BENCH(Task::make_built_in_kind_constructors);        // PLAN-50
    BENCH(BinaryPredicateFamilies::first_stock);          // PLAN-51
    BENCH(BootVerbs::make_built_in)                      // ← THIS PLAN
```

`BootVerbs::make_built_in` (C: `inform7/assertions-module/Chapter 2/Booting Verbs.w`) performs four tasks:

1. **Create special meanings** (lines 73-106): ~20 special meaning functions with priorities 1-4
2. **Create verbs** (lines 124-135): Conjugate "be" (copular) and "mean" (non-copular)
3. **Give meaning to mean** (lines 140-142): Attach "verb-means" special meaning to "to mean"
4. **Define relation names** (lines 147-201): 23 relation name constants (0-22)

### Current Rust state

**In `conform7-syntax`:**
- `Verbs` registry: `new_verb`, `new_operator_verb`, `add_form`, `declare_special_meaning`, `new_usage`, `new_tier`, `add_usage_to_tier`, `make_preposition`, `register_conjugation`, `find_conjugation`
- `VerbMeaning`: `meaninglessness()`, `regular()`, `special()`, `indirected()`
- `SpecialMeaningFn` / `generic_smf`: function pointer type and default handler
- `Conjugation::conjugate`: handles "be", "have", regular verbs
- `WordAssemblage`, `Stock`, `Lcon`, `VerbUsage`, `VerbUsageTier`: all complete

**In `conform7-semantics`:**
- `BinaryPredicateFamilies::start_all()`: creates 7 families, 16 BPs
- `BinaryPredicate` struct: `relation_name`, `relation_family`, etc.

### What needs to be built

1. **`register_all_usages_of_verb`** — registers all conjugated forms as verb usages in tiers
2. **`BootVerbs::make_built_in`** — orchestrates special meanings, verb creation, usage registration
3. **Relation name constants** — 23 `pub const usize` values

## Decision

### 1. Is PLAN-52 the correct next step?

**Yes.** It's step 4 of the `BUILT_IN_STUFF_CSEQ` bench, directly after PLAN-51. All dependencies (conjugation, verb registry, special meanings) exist in the syntax crate.

### 2. Is it independently testable?

**Yes.** Entirely within `conform7-syntax` (verbs, conjugations, special meanings) and `conform7-semantics` (relation name constants). No cross-crate dependencies.

### 3. What is the smallest independently testable subset?

1. `Verbs::register_all_usages_of_verb(verb, unexpected_upper_casing, priority)` — registers all conjugated forms
2. `Verbs::make_built_in()` — orchestrates everything, returns `(to_be_ref, to_mean_ref)`
3. Relation name constants in `conform7-semantics`

### 4. What simplifications are appropriate?

- **All SMF functions use `generic_smf`** — real implementations deferred
- **No `PluginCalls::make_special_meanings()`** — deferred
- **Simplified `register_all_usages_of_verb`** — registers key forms without full deduplication
- **No Preform grammar** — use `WordAssemblage::lit_1` directly
- **No `current_sentence` tracking** — use `None`

## Tasks

### Task 1: Add `register_all_usages_of_verb` to `Verbs`

Edit `crates/conform7-syntax/src/verbs.rs`.

Add a method that registers all conjugated forms of a verb as usages in tiers.

### Task 2: Add `BootVerbs::make_built_in` to `Verbs`

Edit `crates/conform7-syntax/src/verbs.rs`.

Add a method that:
1. Declares ~20 special meanings with `generic_smf`
2. Conjugates "be" and "mean"
3. Creates the two verbs
4. Registers all usages
5. Attaches "verb-means" to "to mean"
6. Returns `(to_be_ref, to_mean_ref)`

### Task 3: Add relation name constants

Create `crates/conform7-semantics/src/calculus/relation_names.rs` with 23 constants.

### Task 4: Add unit tests

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy --all-targets` — no new warnings
- [ ] `git add -A && git commit -m "PLAN-52: BootVerbs::make_built_in"`

## Success Criteria

- [ ] `Verbs::make_built_in()` creates 2 verbs ("to be" copular, "to mean" non-copular)
- [ ] ~20 special meanings declared with correct names and priorities
- [ ] Verb usages registered for "is", "are", "is not", "are not", "means", "mean"
- [ ] Usages in correct tiers (priority 2 for "to be", priority 3 for "to mean")
- [ ] "verb-means" special meaning attached to "to mean"
- [ ] 23 relation name constants defined
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **Real SMF implementations** — deferred
- **`PluginCalls::make_special_meanings()`** — deferred
- **`UnaryPredicateFamilies::stock(1)`** — deferred
- **`second_stock`** — deferred
- **Three passes through major nodes** — deferred
- **Model world creation** — deferred
