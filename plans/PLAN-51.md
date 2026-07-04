# Plan 51: BinaryPredicateFamilies::first_stock — Wire Up Built-in Relations

**Status**: In progress
**Target**: 1 day

## Goal

Create `BinaryPredicateFamilies::start_all()` that wires up all calculus-module BP families (equality, spatial, empty, quasinumeric, universal, explicit, by-function) and calls `first_stock` to create the 16 built-in binary predicates (8 pairs). This is step 3 of the `BUILT_IN_STUFF_CSEQ` compilation bench.

## Background

### C reference

From `gitignore/inform/inform7/core-module/Chapter 1/How To Compile.w`, lines 126-132:

```c
@<Build a rudimentary set of kinds, relations, verbs and inference subjects@> =
    Task::advance_stage_to(BUILT_IN_STUFF_CSEQ, I"Making built in infrastructure",
        -1, debugging, sequence_timer);
    BENCH(InferenceSubjects::make_built_in);            // PLAN-50
    BENCH(Task::make_built_in_kind_constructors);        // PLAN-50
    BENCH(BinaryPredicateFamilies::first_stock)          // ← THIS PLAN
    BENCH(BootVerbs::make_built_in)                      // PLAN-52+
```

`BinaryPredicateFamilies::first_stock` (C: `services/calculus-module/Chapter 3/Binary Predicate Families.w`) iterates all registered BP families and calls `stock(family, 1)` on each. Stage 1 creates the built-in relations.

### Current Rust state

- `BinaryPredicateFamilies` has `first_stock` and `second_stock` dispatch methods
- Each BP family module has its own `start()` function and `stock()` method
- Families use different patterns: `EqualityRelation::start()` returns new Vecs, others append
- Knowledge module families (provision, same-property, setting-property) use their own Vecs with index 0

### What first_stock creates (calculus module only)

| Family | Index | BPs Created | BP Indices |
|--------|-------|-------------|------------|
| equality | 0 | R_equality (self-reversal) | 0 |
| spatial | 1 | a_has_b_predicate + reversal | 1-2 |
| empty | 2 | R_empty (self-reversal) | 3 |
| quasinumeric | 3 | 4 inequality pairs | 4-11 |
| universal | 4 | R_universal + R_meaning pairs | 12-15 |
| explicit | 5 | nothing (source text creates) | — |
| by-function | 6 | nothing (source text creates) | — |

**Total**: 16 BPs (8 pairs)

## Decision

### 1. Is PLAN-51 the correct next step?

**Yes.** It's step 3 of the `BUILT_IN_STUFF_CSEQ` bench, directly after PLAN-50. The BP families already exist with `stock` methods; we just need a unified `start_all()` and integration tests.

### 2. Is it independently testable?

**Yes.** Create `start_all()`, call `first_stock`, verify 7 families and 16 BPs with correct names, families, reversals, and index details.

### 3. What is the smallest independently testable subset?

1. `BinaryPredicateFamilies::start_all()` — creates 7 families, returns `(Vec<BpFamily>, Vec<BinaryPredicate>)`
2. `first_stock` on the result — creates 16 BPs
3. Verify each BP's family, name, reversal, term details

### 4. What simplifications are appropriate?

- **No knowledge module families** — provision, same-property, setting-property use their own Vecs and index 0. Integrating them requires updating constants. Deferred.
- **No `UnaryPredicateFamilies::stock(1)`** — the C code calls this first, but UP families are a separate concern. Deferred.
- **No global registry** — use local Vecs passed through function calls.
- **No `PluginCalls`** — deferred.
- **No `second_stock`** — deferred (needs property system).

## Tasks

### Task 1: Add `BinaryPredicateFamilies::start_all()`

Edit `crates/conform7-semantics/src/calculus/binary_predicate_families.rs`.

Add a function that creates all calculus-module BP families:

```rust
/// Create all calculus-module BP families and their built-in BPs.
///
/// Corresponds to the sequence of `start()` calls in the C reference:
/// - `EqualityRelation::start()` (families 0-2)
/// - `QuasinumericRelations::start()` (family 3)
/// - `UniversalRelation::start()` (family 4)
/// - `ExplicitRelations::start()` (families 5-6)
///
/// Does NOT include knowledge-module families (provision, same-property,
/// setting-property) — those are created separately.
///
/// Returns (families, bp_registry) with first_stock already called.
pub fn start_all() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
    let (mut families, mut bp_registry) = crate::calculus::equality_relation::EqualityRelation::start();
    crate::calculus::quasinumeric_relations::QuasinumericRelations::start(&mut families, &mut bp_registry);
    crate::calculus::universal_relation::UniversalRelation::start(&mut families, &mut bp_registry);
    crate::calculus::explicit_relations::ExplicitRelations::start(&mut families, &mut bp_registry);
    Self::first_stock(&mut families, &mut bp_registry);
    (families, bp_registry)
}
```

### Task 2: Add integration tests

Add to `crates/conform7-semantics/src/calculus/binary_predicate_families.rs`:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::calculus::binary_predicates::BinaryPredicate;

    #[test]
    fn start_all_creates_seven_families() {
        let (families, _) = BinaryPredicateFamilies::start_all();
        assert_eq!(families.len(), 7);
        assert_eq!(families[0].name, "equality");
        assert_eq!(families[1].name, "spatial");
        assert_eq!(families[2].name, "empty");
        assert_eq!(families[3].name, "quasinumeric");
        assert_eq!(families[4].name, "universal");
        assert_eq!(families[5].name, "explicit");
        assert_eq!(families[6].name, "by-function");
    }

    #[test]
    fn first_stock_creates_sixteen_bps() {
        let (families, bp_registry) = BinaryPredicateFamilies::start_all();
        assert_eq!(bp_registry.len(), 16);
    }

    #[test]
    fn first_stock_creates_r_equality() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        let r_eq = &bp_registry[0];
        assert_eq!(r_eq.relation_family, 0);
        assert!(r_eq.right_way_round);
        assert_eq!(r_eq.reversal, Some(0));
    }

    #[test]
    fn first_stock_creates_spatial_pair() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        let has = &bp_registry[1];
        assert_eq!(has.relation_family, 1);
        assert_eq!(has.relation_name.as_deref(), Some("has"));
        assert!(has.right_way_round);
        assert_eq!(has.reversal, Some(2));

        let had = &bp_registry[2];
        assert_eq!(had.relation_family, 1);
        assert_eq!(had.relation_name.as_deref(), Some("is-had-by"));
        assert!(!had.right_way_round);
        assert_eq!(had.reversal, Some(1));
    }

    #[test]
    fn first_stock_creates_r_empty() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        let r_empty = &bp_registry[3];
        assert_eq!(r_empty.relation_family, 2);
        assert_eq!(r_empty.relation_name.as_deref(), Some("never-holding"));
    }

    #[test]
    fn first_stock_creates_quasinumeric_bps() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        assert_eq!(bp_registry[4].relation_name.as_deref(), Some("greater-than"));
        assert_eq!(bp_registry[6].relation_name.as_deref(), Some("less-than"));
        assert_eq!(bp_registry[8].relation_name.as_deref(), Some("at-least"));
        assert_eq!(bp_registry[10].relation_name.as_deref(), Some("at-most"));
        for i in [4, 6, 8, 10] {
            assert_eq!(bp_registry[i].relation_family, 3);
        }
    }

    #[test]
    fn first_stock_creates_universal_bps() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        assert_eq!(bp_registry[12].relation_name.as_deref(), Some("relates"));
        assert_eq!(bp_registry[14].relation_name.as_deref(), Some("means"));
        assert_eq!(bp_registry[12].relation_family, 4);
        assert_eq!(bp_registry[14].relation_family, 4);
    }

    #[test]
    fn first_stock_sets_index_details() {
        let (_, bp_registry) = BinaryPredicateFamilies::start_all();
        assert_eq!(bp_registry[0].term_details[0].index_term_as.as_deref(), Some("value"));
        assert_eq!(bp_registry[0].term_details[1].index_term_as.as_deref(), Some("value"));
        assert_eq!(bp_registry[3].term_details[0].index_term_as.as_deref(), Some("value"));
        assert_eq!(bp_registry[3].term_details[1].index_term_as.as_deref(), Some("value"));
    }
}
```

### Task 3: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy --all-targets` — no new warnings

## Success Criteria

- [ ] `BinaryPredicateFamilies::start_all()` creates 7 families with correct names.
- [ ] `first_stock` creates 16 BPs with correct families, names, reversals, and index details.
- [ ] All existing tests still pass.
- [ ] `cargo clippy --all-targets` introduces no new warnings.

## Out of Scope

- **Knowledge module families** (provision, same-property, setting-property) — deferred.
- **`UnaryPredicateFamilies::stock(1)`** — deferred.
- **`BootVerbs::make_built_in`** — deferred to PLAN-52+.
- **`second_stock`** — deferred (needs property system).
- **Three passes through major nodes** — deferred.
- **Model world creation** — deferred.
