# Plan 42: Adjectives by Phrase — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Implement the foundation of the Adjectives by Phrase system in the assertions module. This corresponds to `AdjectivesByPhrase::start()` in `inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`, which creates the phrase family (priority 6) and installs the `claim_definition` method. The family method handles adjectives defined by an explicit I7 phrase (sense == 0, i.e. `DEFINED_PHRASALLY`).

This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 34), immediately after `ImperativeDefinitionFamilies::create()` (PLAN-41, Complete).

## Decision

### 1. Is `AdjectivesByPhrase` the correct next step?

**Yes.** In the C assertions-module startup sequence, after `ImperativeDefinitionFamilies::create()` comes `AdjectivesByPhrase::start()`. The Rust assertions module already has `ImperativeDefinitionFamilies` (PLAN-41, Complete), so the phrase-adjective family is the natural next foundation piece.

### 2. Is it independently testable?

**Yes.** The foundation consists of pure data-structure operations:
- creating an `adjective_meaning_family` with priority 6;
- registering a `claim_definition` method;
- `claim_definition` deciding whether to claim a definition, and if so, creating an `Adjective`, an `AdjectiveMeaning`, linking them, setting the domain, and marking `TEST_ATOM_TASK` as via-support-function;
- `is_defined_by_phrase` checking whether a meaning belongs to the phrase family.

These operations require no Preform grammar, no I6 compilation, and no full `AdjectiveMeanings::claim_definition` dispatcher.

### 3. What is the smallest independently testable subset?

The smallest testable subset is:
1. `AdjectivesByPhrase::start(&mut families) -> usize` creates the phrase family at priority 6 and returns its index.
2. `AdjectivesByPhrase::claim_definition(...)` declines (`None`) when `sense != 0`.
3. `AdjectivesByPhrase::claim_definition(...)` accepts (`Some(am_idx)`) when `sense == 0`, and the resulting meaning:
   - belongs to the phrase family,
   - is attached to a declared adjective whose name is the headword,
   - has a text-based domain,
   - has `TEST_ATOM_TASK` marked as via-support-function.

### 4. What simplifications are appropriate?

- **No `Definition` struct yet.** The C `AdjectivesByPhrase::claim_definition` creates a `definition` via `AdjectivalDefinitionFamily::new_definition`. That `definition` is needed later by `define_adjective_by_phrase` when `AdjectivalDefinitionFamily::given_body` sets up run-time schemas. The full `AdjectivalDefinitionFamily` is out of scope (Chapter 5), so the foundation stores `None` in `family_specific_data` and defers the `Definition` struct.
- **No Preform grammar.** We take already-split wording parameters (`headword`, `domain_text`, etc.) instead of parsing `<adjective-definition>`.
- **No `parse_node` handling.** The source sentence location `q` is represented by an optional string tag; the foundation ignores it.
- **No `RTAdjectives::set_schemas_for_I7_phrase`.** Run-time schema preparation is deferred.
- **No generic `AdjectiveMeanings::claim_definition` dispatcher.** We implement only the phrase family method; the loop over priorities that calls each family will be added later.
- **Use the existing measurement-shaped `ClaimDefinitionFn` signature.** The current Rust type alias does not include the family index or calling wording. We install a thin wrapper that reads the phrase-family index from a static and forwards to the testable public function. The signature will be aligned with the C reference when the generic dispatcher is implemented.

## Background

### C reference architecture

Adjectives defined by an explicit phrase look like:

```inform7
Definition: A container is possessed by the Devil:
    if its carrying capacity is 666, decide yes;
    decide no.
```

The C implementation lives in `inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`. Key functions:

```c
adjective_meaning_family *phrase_amf = NULL;

void AdjectivesByPhrase::start(void) {
    phrase_amf = AdjectiveMeanings::new_family(6);
    METHOD_ADD(phrase_amf, CLAIM_DEFINITION_SENTENCE_ADJM_MTID,
        AdjectivesByPhrase::claim_definition);
}

int AdjectivesByPhrase::is_defined_by_phrase(adjective_meaning *am) {
    if ((am) && (am->family == phrase_amf)) return TRUE;
    return FALSE;
}

void AdjectivesByPhrase::define_adjective_by_phrase(parse_node *p, id_body *idb,
    wording *CW, kind **K) {
    ...
}

int AdjectivesByPhrase::claim_definition(adjective_meaning_family *f,
    adjective_meaning **result, parse_node *q,
    int sense, wording AW, wording DNW, wording CONW, wording CALLW) {
    if (sense != 0) return FALSE;
    definition *def = AdjectivalDefinitionFamily::new_definition(q);
    adjective_meaning *am = AdjectiveMeanings::new(phrase_amf,
        STORE_POINTER_definition(def), Node::get_text(q));
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

The family is offered definitions in ascending priority order by `AdjectiveMeanings::claim_definition`. The phrase family priority is 6, so it claims phrasal definitions after earlier families (e.g. measurement at priority 3) have declined. The method only claims `sense == 0` (`DEFINED_PHRASALLY`); other senses are handled by condition-based or measurement families.

### Current Rust state

- `crates/conform7-semantics/src/assertions/mod.rs` exposes `imperative_definition_families`.
- `crates/conform7-semantics/src/knowledge/adjectives.rs` contains `Adjective`, `AdjectiveMeaning`, `AdjectiveMeaningFamily`, `AdjectiveMeaningFamilyMethods`, `AdjectiveMeanings`, `AdjectiveAmbiguity`, `AdjectiveMeaningDomains`.
- `AdjectiveMeanings::new_family` takes `name`, `priority`, `methods`, and `families`.
- `AdjectiveMeanings::new` creates a meaning with a family index and optional family-specific data.
- `AdjectiveMeanings::perform_task_via_function` does not yet exist.
- `AdjectiveMeaningFamilyMethods.claim_definition` currently uses a measurement-shaped function signature. We will install a thin wrapper rather than changing the signature in this plan.

## Tasks

### Task 1: Add task-mode constants and `perform_task_via_function`

Edit `crates/conform7-semantics/src/knowledge/adjectives.rs`.

Add constants (near the `AdjectiveMeaning` struct or in the `AdjectiveMeanings` impl):

```rust
pub const TEST_ATOM_TASK: usize = 0;
pub const NOW_ATOM_TRUE_TASK: usize = 1;
pub const NOW_ATOM_FALSE_TASK: usize = 2;

pub const NO_TASKMODE: i8 = 0;
pub const DIRECT_TASKMODE: i8 = 1;
pub const VIA_SUPPORT_FUNCTION_TASKMODE: i8 = 2;
```

Add:

```rust
impl AdjectiveMeanings {
    /// Mark an atom task as needing to be performed via a support function.
    ///
    /// Corresponds to `AdjectiveMeanings::perform_task_via_function` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 165-167).
    pub fn perform_task_via_function(am_idx: usize, task: usize, meanings: &mut [AdjectiveMeaning]) {
        if let Some(am) = meanings.get_mut(am_idx) {
            if task < am.task_modes.len() {
                am.task_modes[task] = VIA_SUPPORT_FUNCTION_TASKMODE;
            }
        }
    }
}
```

### Task 2: Create the `AdjectivesByPhrase` module

Create `crates/conform7-semantics/src/assertions/adjectives_by_phrase.rs`.

Module-level doc comment:

```rust
//! Adjectives by Phrase — adjectives defined by an explicit Inform 7 phrase.
//!
//! Corresponds to `AdjectivesByPhrase` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`).
//!
//! This module creates the `phrase_amf` family and claims adjective definitions
//! whose body is an explicit phrase (`Definition: a ... is ...: ...`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No Preform grammar parsing.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_I7_phrase`.
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
/// Global family index for the phrase family.
///
/// Mirrors the C static `phrase_amf`. Set by `AdjectivesByPhrase::start`.
static PHRASE_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Phrase module.
///
/// Corresponds to `AdjectivesByPhrase` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`).
pub struct AdjectivesByPhrase;

impl AdjectivesByPhrase {
    /// Priority of the phrase family in the definition-claim order.
    ///
    /// Corresponds to the `6` passed to `AdjectiveMeanings::new_family` in the C reference.
    pub const PHRASE_FAMILY_PRIORITY: u8 = 6;

    /// Create the phrase adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByPhrase::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`, lines 12-15).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByPhrase::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family("phrase", Self::PHRASE_FAMILY_PRIORITY, methods, families);
        PHRASE_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the phrase family set by the most recent `start`.
    fn phrase_family_idx() -> usize {
        PHRASE_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the phrase family.
    ///
    /// Corresponds to `AdjectivesByPhrase::is_defined_by_phrase` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`, lines 17-20).
    pub fn is_defined_by_phrase(am_idx: usize, meanings: &[AdjectiveMeaning], phrase_family_idx: usize) -> bool {
        meanings.get(am_idx).is_some_and(|am| am.family == phrase_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only phrasally-defined adjectives (`sense == 0`). Creates the
    /// adjective meaning, declares the adjective, links them, stores the domain,
    /// and marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` and calling wording `CALLW` are ignored.
    pub fn claim_definition(
        phrase_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        if sense != 0 {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(phrase_family_idx, None, Some(headword), meanings);
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific and calling parameters and reads the phrase family
    /// index from the static set by `start`.
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
        Self::claim_definition(Self::phrase_family_idx(), headword, sense, domain_text, adjectives, meanings)
    }
}
```

### Task 3: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`:

- Add `pub mod adjectives_by_phrase;`.
- Add a module-map row:

```markdown
| [`adjectives_by_phrase`] | `Chapter 8/Adjectives by Phrase.w` | Phrase-defined adjectives |
```

- Add the C reference:
  - `inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`
- Update the startup sequence comment to show `AdjectivesByPhrase::start()` after `ImperativeDefinitionFamilies::start()`.

### Task 4: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/adjectives_by_phrase.rs`.

Tests:

1. `start_creates_phrase_family_with_priority_6`

```rust
#[test]
fn start_creates_phrase_family_with_priority_6() {
    let mut families = Vec::new();
    let idx = AdjectivesByPhrase::start(&mut families);
    assert_eq!(families[idx].name, "phrase");
    assert_eq!(families[idx].definition_claim_priority, 6);
    assert!(families[idx].methods.claim_definition.is_some());
}
```

2. `claim_definition_declines_non_phrasal_senses`

```rust
#[test]
fn claim_definition_declines_non_phrasal_senses() {
    let mut families = Vec::new();
    let phrase_family = AdjectivesByPhrase::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByPhrase::claim_definition(
        phrase_family, "roomy", 1, Some("container"), &mut adjectives, &mut meanings,
    );
    assert!(result.is_none());

    let result = AdjectivesByPhrase::claim_definition(
        phrase_family, "roomy", -1, Some("container"), &mut adjectives, &mut meanings,
    );
    assert!(result.is_none());
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

3. `claim_definition_creates_phrase_meaning_and_adjective`

```rust
#[test]
fn claim_definition_creates_phrase_meaning_and_adjective() {
    let mut families = Vec::new();
    let phrase_family = AdjectivesByPhrase::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByPhrase::claim_definition(
        phrase_family, "possessed", 0, Some("container"), &mut adjectives, &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, phrase_family);
    assert!(meanings[am_idx].family_specific_data.is_none());
    assert_eq!(meanings[am_idx].indexing_text, Some("possessed"));
    assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
    assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

    let adj_idx = meanings[am_idx].owning_adjective.unwrap();
    assert_eq!(adjectives[adj_idx].name, "possessed");
    assert!(adjectives[adj_idx].meanings.contains(&am_idx));
}
```

4. `is_defined_by_phrase_true_for_phrase_meaning`

```rust
#[test]
fn is_defined_by_phrase_true_for_phrase_meaning() {
    let mut families = Vec::new();
    let phrase_family = AdjectivesByPhrase::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByPhrase::claim_definition(
        phrase_family, "possessed", 0, None, &mut adjectives, &mut meanings,
    ).unwrap();

    assert!(AdjectivesByPhrase::is_defined_by_phrase(am_idx, &meanings, phrase_family));
}
```

5. `is_defined_by_phrase_false_for_other_meaning`

```rust
#[test]
fn is_defined_by_phrase_false_for_other_meaning() {
    let mut families = Vec::new();
    let phrase_family = AdjectivesByPhrase::start(&mut families);
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

    assert!(!AdjectivesByPhrase::is_defined_by_phrase(am_idx, &meanings, phrase_family));
}
```

### Task 5: Verify

- [ ] `cargo build`
- [ ] `cargo test -- assertions::adjectives_by_phrase`
- [ ] `cargo test` (all existing tests still pass)
- [ ] `cargo clippy --all-targets`

## Success Criteria

- [ ] `assertions::adjectives_by_phrase` module exists and compiles.
- [ ] `AdjectivesByPhrase::start` creates a family named `"phrase"` with priority 6 and a `claim_definition` method installed.
- [ ] `AdjectivesByPhrase::claim_definition` returns `None` for `sense != 0`.
- [ ] `AdjectivesByPhrase::claim_definition` returns `Some(am_idx)` for `sense == 0`.
- [ ] The created meaning belongs to the phrase family and has `family_specific_data == None`.
- [ ] The created adjective has the given headword and contains the new meaning.
- [ ] The meaning's domain text is set from `domain_text`.
- [ ] The meaning's `task_modes[TEST_ATOM_TASK]` is `VIA_SUPPORT_FUNCTION_TASKMODE`.
- [ ] `AdjectivesByPhrase::is_defined_by_phrase` returns true only for phrase-family meanings.
- [ ] `AdjectiveMeanings::perform_task_via_function` exists and sets the requested task mode.
- [ ] `cargo clippy --all-targets` is clean.
- [ ] The total test count remains at least 1374 (existing) plus new tests.

## Out of Scope

- **Full `AdjectivalDefinitionFamily` (Chapter 5).** The `definition` struct, `new_definition`, node rewriting, and `given_body` hook are deferred.
- **`define_adjective_by_phrase` body lookup.** The C function scans `definition` objects to connect a compiled I7 phrase body to its adjective meaning. No `id_body`, `Definition` registry, or schema setup yet.
- **Generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the phrase family's method is installed.
- **Run-time adjective compilation.** `RTAdjectives::set_schemas_for_I7_phrase` and support-function schema generation are deferred.
- **Preform grammar / Salsa integration.** No parsing of `Definition:` sentences.
- **Problem messages.** No `StandardProblems` calls for malformed definitions.
- **Negated phrasal definitions.** The C only handles `sense == 0`; `sense == 1`/`-1` are deferred to other families.
