# Plan 43: Adjectives by Condition — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Implement the foundation of the Adjectives by Condition system — creating the condition adjective meaning family with `claim_definition` and `prepare_schemas` methods. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 35), immediately after `AdjectivesByPhrase::start()` (PLAN-42, Complete).

## Decision

### 1. Is `AdjectivesByCondition` the correct next step?

**Yes.** In the C assertions-module startup sequence, after `AdjectivesByPhrase::start()` comes `AdjectivesByCondition::start()`. The Rust assertions module already has `AdjectivesByPhrase` (PLAN-42, Complete), so the condition-adjective family is the natural next foundation piece.

### 2. Is it independently testable?

**Yes.** The foundation consists of pure data-structure operations:
- creating an `adjective_meaning_family` with priority 7;
- registering `claim_definition` and `prepare_schemas` methods;
- `claim_definition` deciding whether to claim a definition, and if so, creating an `Adjective`, an `AdjectiveMeaning`, linking them, setting the domain, and marking `TEST_ATOM_TASK` as via-support-function;
- `is_defined_by_condition` checking whether a meaning belongs to the condition family.

These operations require no Preform grammar, no I6 compilation, and no full `AdjectiveMeanings::claim_definition` dispatcher.

### 3. What is the smallest independently testable subset?

The smallest testable subset is:
1. `AdjectivesByCondition::start(&mut families) -> usize` creates the condition family at priority 7 and returns its index.
2. `AdjectivesByCondition::claim_definition(...)` declines (`None`) when `sense == 0`.
3. `AdjectivesByCondition::claim_definition(...)` accepts (`Some(am_idx)`) when `sense != 0` (e.g. `sense == 1` or `sense == -1`), and the resulting meaning:
   - belongs to the condition family,
   - is attached to a declared adjective whose name is the headword,
   - has a text-based domain,
   - has `TEST_ATOM_TASK` marked as via-support-function.
4. `AdjectivesByCondition::prepare_schemas` is installed as a no-op placeholder.

### 4. What simplifications are appropriate?

- **No `Definition` struct yet.** The C `AdjectivesByCondition::claim_definition` creates a `definition` via `AdjectivalDefinitionFamily::new_definition`. That `definition` is needed later by `RTAdjectives::support_for_I7_condition`. The full `AdjectivalDefinitionFamily` is out of scope (Chapter 5), so the foundation stores `None` in `family_specific_data` and defers the `Definition` struct.
- **No Preform grammar.** We take already-split wording parameters (`headword`, `domain_text`, `condition_text`) instead of parsing `<adjective-definition>`.
- **No `parse_node` handling.** The source sentence location `q` is represented by an optional string tag; the foundation ignores it.
- **No `RTAdjectives::support_for_I7_condition`.** Run-time support-function generation that emits the condition as Inform 6 code is deferred. The `prepare_schemas` method is installed as a no-op placeholder.
- **No generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the condition family's method is installed.
- **Use the existing measurement-shaped `ClaimDefinitionFn` signature.** The current Rust type alias does not include the family index or calling wording. We install a thin wrapper that reads the condition-family index from a static and forwards to the testable public function. The signature will be aligned with the C reference when the generic dispatcher is implemented.

## Background

### C reference architecture

Adjectives defined by a one-line Inform 7 condition look like:

```inform7
Definition: A container is roomy if its carrying capacity is greater than 10.
```

The C implementation lives in `inform7/assertions-module/Chapter 8/Adjectives by Condition.w`. Key functions:

```c
adjective_meaning_family *condition_amf = NULL;

void AdjectivesByCondition::start(void) {
    condition_amf = AdjectiveMeanings::new_family(7);
    METHOD_ADD(condition_amf, GENERATE_IN_SUPPORT_FUNCTION_ADJM_MTID,
        RTAdjectives::support_for_I7_condition);
    METHOD_ADD(condition_amf, CLAIM_DEFINITION_SENTENCE_ADJM_MTID,
        AdjectivesByCondition::claim_definition);
}

int AdjectivesByCondition::claim_definition(adjective_meaning_family *f,
    adjective_meaning **result, parse_node *q,
    int sense, wording AW, wording DNW, wording CONW, wording CALLW) {
    if (sense == 0) return FALSE;
    definition *def = AdjectivalDefinitionFamily::new_definition(q);
    adjective_meaning *am = AdjectiveMeanings::new(condition_amf,
        STORE_POINTER_definition(def), Node::get_text(q));
    def->condition_to_match = CONW;
    def->format = sense;
    def->domain_calling = CALLW;
    def->am_of_def = am;
    adjective *adj = Adjectives::declare(AW, NULL);
    AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    AdjectiveMeanings::perform_task_via_function(am, TEST_ATOM_TASK);
    AdjectiveMeaningDomains::set_from_text(am, DNW);
    *result = am;
    return TRUE;
}
```

The family is offered definitions in ascending priority order by `AdjectiveMeanings::claim_definition`. The condition family priority is 7, so it claims condition-based definitions after earlier families (e.g. phrase at priority 6) have declined. The method only claims `sense != 0`; `sense == 0` (`DEFINED_PHRASALLY`) is handled by the phrase family.

`GENERATE_IN_SUPPORT_FUNCTION_ADJM_MTID` is called when the runtime module compiles a support function for the adjective. It is installed as `RTAdjectives::support_for_I7_condition`, which retrieves the `definition` from the meaning and emits the condition as Inform 6 code. This is deferred in the foundation.

### Current Rust state

- `crates/conform7-semantics/src/assertions/mod.rs` exposes `imperative_definition_families` and `adjectives_by_phrase`.
- `crates/conform7-semantics/src/knowledge/adjectives.rs` contains `Adjective`, `AdjectiveMeaning`, `AdjectiveMeaningFamily`, `AdjectiveMeaningFamilyMethods`, `AdjectiveMeanings`, `AdjectiveAmbiguity`, `AdjectiveMeaningDomains`, and `perform_task_via_function`.
- `AdjectiveMeanings::new_family` takes `name`, `priority`, `methods`, and `families`.
- `AdjectiveMeanings::new` creates a meaning with a family index and optional family-specific data.
- `AdjectiveMeanings::perform_task_via_function` already exists.
- `AdjectiveMeaningFamilyMethods.claim_definition` currently uses a measurement-shaped function signature. We will install a thin wrapper rather than changing the signature in this plan.
- `AdjectiveMeaningFamilyMethods.prepare_schemas` is the placeholder for `GENERATE_IN_SUPPORT_FUNCTION_ADJM_MTID` and has the simplified signature `fn(usize, i32)`.

## Tasks

### Task 1: Create the `AdjectivesByCondition` module

Create `crates/conform7-semantics/src/assertions/adjectives_by_condition.rs`.

Module-level doc comment:

```rust
//! Adjectives by Condition — adjectives defined by a one-line Inform 7 condition.
//!
//! Corresponds to `AdjectivesByCondition` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`).
//!
//! This module creates the `condition_amf` family and claims adjective
//! definitions whose body is a single I7 condition
//! (`Definition: a ... is ... if ...`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No Preform grammar parsing.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::support_for_I7_condition`.
//! - The family method wrapper uses a static family index to fit the existing
//!   measurement-shaped `ClaimDefinitionFn` signature.
```

Imports:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
    TEST_ATOM_TASK, VIA_SUPPORT_FUNCTION_TASKMODE,
};
use crate::knowledge::measurements::MeasurementDefinition;
use crate::knowledge::properties::Property;
```

Public API:

```rust
/// Global family index for the condition family.
///
/// Mirrors the C static `condition_amf`. Set by `AdjectivesByCondition::start`.
static CONDITION_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Condition module.
///
/// Corresponds to `AdjectivesByCondition` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`).
pub struct AdjectivesByCondition;

impl AdjectivesByCondition {
    /// Priority of the condition family in the definition-claim order.
    ///
    /// Corresponds to the `7` passed to `AdjectiveMeanings::new_family` in the C reference.
    pub const CONDITION_FAMILY_PRIORITY: u8 = 7;

    /// Create the condition adjective family and install its methods.
    ///
    /// Corresponds to `AdjectivesByCondition::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`, lines 12-21).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByCondition::claim_definition_family_method),
            prepare_schemas: Some(AdjectivesByCondition::prepare_schemas),
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "condition",
            Self::CONDITION_FAMILY_PRIORITY,
            methods,
            families,
        );
        CONDITION_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the condition family set by the most recent `start`.
    fn condition_family_idx() -> usize {
        CONDITION_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the condition family.
    ///
    /// Corresponds to testing `am->family == condition_amf` in the C reference.
    pub fn is_defined_by_condition(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        condition_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == condition_family_idx)
    }

    /// No-op placeholder for the run-time support function generator.
    ///
    /// Corresponds to `RTAdjectives::support_for_I7_condition` in the C reference
    /// (`inform7/runtime-module/Chapter 5/Adjectives.w`, lines 481ff).
    ///
    /// Simplified: does nothing. The full implementation would retrieve the
    /// `definition` stored in the meaning's `family_specific_data` and emit the
    /// condition as Inform 6 code inside the support function for the task.
    pub fn prepare_schemas(_am_idx: usize, _task: i32) {
        // Run-time compilation deferred.
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only condition-based adjectives (`sense != 0`). Creates the
    /// adjective meaning, declares the adjective, links them, stores the domain,
    /// and marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` and calling wording `CALLW` are ignored.
    /// - The condition text `CONW` and sense value are accepted as parameters
    ///   but not stored; they will be persisted when the `Definition` struct is
    ///   introduced.
    pub fn claim_definition(
        condition_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        if sense == 0 {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            condition_family_idx,
            None,
            Some(headword),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific and calling parameters and reads the condition
    /// family index from the static set by `start`.
    #[allow(clippy::too_many_arguments)]
    fn claim_definition_family_method(
        headword: &'static str,
        _prop: Option<usize>,
        sense: i32,
        domain_text: Option<&'static str>,
        _condition_text: Option<&'static str>,
        _definitions: &mut Vec<MeasurementDefinition>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        _properties: &[Property],
    ) -> Option<usize> {
        Self::claim_definition(
            Self::condition_family_idx(),
            headword,
            sense,
            domain_text,
            adjectives,
            meanings,
        )
    }
}
```

### Task 2: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`:

- Add `pub mod adjectives_by_condition;`.
- Add a module-map row:

```markdown
| [`adjectives_by_condition`] | `Chapter 8/Adjectives by Condition.w` | Condition-defined adjectives |
```

- Add the C reference:
  - `inform7/assertions-module/Chapter 8/Adjectives by Condition.w`
- Update the startup sequence comment to show `AdjectivesByCondition::start()` after `AdjectivesByPhrase::start()`.

### Task 3: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/adjectives_by_condition.rs`.

Tests:

1. `start_creates_condition_family_with_priority_7`

```rust
#[test]
fn start_creates_condition_family_with_priority_7() {
    let mut families = Vec::new();
    let idx = AdjectivesByCondition::start(&mut families);
    assert_eq!(families[idx].name, "condition");
    assert_eq!(families[idx].definition_claim_priority, 7);
    assert!(families[idx].methods.claim_definition.is_some());
    assert!(families[idx].methods.prepare_schemas.is_some());
}
```

2. `claim_definition_declines_phrasal_sense`

```rust
#[test]
fn claim_definition_declines_phrasal_sense() {
    let mut families = Vec::new();
    let condition_family = AdjectivesByCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByCondition::claim_definition(
        condition_family, "roomy", 0, Some("container"), &mut adjectives, &mut meanings,
    );
    assert!(result.is_none());
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

3. `claim_definition_creates_condition_meaning_and_adjective`

```rust
#[test]
fn claim_definition_creates_condition_meaning_and_adjective() {
    let mut families = Vec::new();
    let condition_family = AdjectivesByCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByCondition::claim_definition(
        condition_family, "roomy", 1, Some("container"), &mut adjectives, &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, condition_family);
    assert!(meanings[am_idx].family_specific_data.is_none());
    assert_eq!(meanings[am_idx].indexing_text, Some("roomy"));
    assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
    assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

    let adj_idx = meanings[am_idx].owning_adjective.unwrap();
    assert_eq!(adjectives[adj_idx].name, "roomy");
    assert!(adjectives[adj_idx].meanings.contains(&am_idx));
}
```

4. `claim_definition_accepts_negative_sense`

```rust
#[test]
fn claim_definition_accepts_negative_sense() {
    let mut families = Vec::new();
    let condition_family = AdjectivesByCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByCondition::claim_definition(
        condition_family, "closed", -1, Some("door"), &mut adjectives, &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, condition_family);
    assert_eq!(meanings[am_idx].domain.domain_text, Some("door"));
}
```

5. `is_defined_by_condition_true_for_condition_meaning`

```rust
#[test]
fn is_defined_by_condition_true_for_condition_meaning() {
    let mut families = Vec::new();
    let condition_family = AdjectivesByCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByCondition::claim_definition(
        condition_family, "roomy", 1, None, &mut adjectives, &mut meanings,
    ).unwrap();

    assert!(AdjectivesByCondition::is_defined_by_condition(
        am_idx, &meanings, condition_family
    ));
}
```

6. `is_defined_by_condition_false_for_other_meaning`

```rust
#[test]
fn is_defined_by_condition_false_for_other_meaning() {
    let mut families = Vec::new();
    let condition_family = AdjectivesByCondition::start(&mut families);
    let other_family = AdjectiveMeanings::new_family(
        "other",
        0,
        AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        },
        &mut families,
    );
    let mut meanings = Vec::new();
    let am_idx = AdjectiveMeanings::new(other_family, None, None, &mut meanings);

    assert!(!AdjectivesByCondition::is_defined_by_condition(
        am_idx, &meanings, condition_family
    ));
}
```

7. `prepare_schemas_is_noop`

```rust
#[test]
fn prepare_schemas_is_noop() {
    // Should not panic.
    AdjectivesByCondition::prepare_schemas(0, 0);
}
```

### Task 4: Verify

- [ ] `cargo build`
- [ ] `cargo test -- assertions::adjectives_by_condition`
- [ ] `cargo test` (all existing tests still pass)
- [ ] `cargo clippy --all-targets`

## Success Criteria

- [ ] `assertions::adjectives_by_condition` module exists and compiles.
- [ ] `AdjectivesByCondition::start` creates a family named `"condition"` with priority 7 and both `claim_definition` and `prepare_schemas` methods installed.
- [ ] `AdjectivesByCondition::claim_definition` returns `None` for `sense == 0`.
- [ ] `AdjectivesByCondition::claim_definition` returns `Some(am_idx)` for `sense != 0`.
- [ ] The created meaning belongs to the condition family and has `family_specific_data == None`.
- [ ] The created adjective has the given headword and contains the new meaning.
- [ ] The meaning's domain text is set from `domain_text`.
- [ ] The meaning's `task_modes[TEST_ATOM_TASK]` is `VIA_SUPPORT_FUNCTION_TASKMODE`.
- [ ] `AdjectivesByCondition::is_defined_by_condition` returns true only for condition-family meanings.
- [ ] `AdjectivesByCondition::prepare_schemas` is installed and is a no-op.
- [ ] `cargo clippy --all-targets` is clean.
- [ ] The total test count remains at least 1379 (existing) plus new tests.

## Out of Scope

- **Full `AdjectivalDefinitionFamily` (Chapter 5).** The `definition` struct, `new_definition`, node rewriting, and `given_body` hook are deferred.
- **`RTAdjectives::support_for_I7_condition`.** Run-time support-function generation that emits the condition as Inform 6 code is deferred. The foundation installs a no-op `prepare_schemas` placeholder.
- **Generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the condition family's method is installed.
- **Preform grammar / Salsa integration.** No parsing of `Definition:` sentences.
- **Problem messages.** No `StandardProblems` calls for malformed definitions.
- **Phrasal definitions.** The C only handles `sense != 0`; `sense == 0` is handled by `AdjectivesByPhrase`.
- **Storing `condition_text` and `sense`.** The `Definition` struct needed to persist these fields is deferred.
