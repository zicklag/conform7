# Plan 45: Adjectives by Inter Condition — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Implement the foundation of the Adjectives by Inter Condition system — creating the Inter condition adjective meaning family with `claim_definition` method. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 37), immediately after `AdjectivesByInterFunction::start()` (PLAN-44, Complete).

## Decision

### 1. Is `AdjectivesByInterCondition` the correct next step?

**Yes.** In the C assertions-module startup sequence, after `AdjectivesByInterFunction::start()` comes `AdjectivesByInterCondition::start()`. The Rust assertions module already has `AdjectivesByPhrase` (PLAN-42), `AdjectivesByCondition` (PLAN-43), and `AdjectivesByInterFunction` (PLAN-44), so the Inter-condition family is the natural next foundation piece.

### 2. Is it independently testable?

**Yes.** The foundation consists of pure data-structure operations:
- creating an `adjective_meaning_family` with priority 4;
- registering only the `claim_definition` method (no deferred support-function method);
- `claim_definition` deciding whether to claim a definition by matching the `<inform6-condition-adjective-definition>` template, checking `sense == 1` and empty `CALLW`, and if so creating an `Adjective`, an `AdjectiveMeaning`, linking them, setting the domain, and marking `TEST_ATOM_TASK` as via-support-function.

These operations require no Preform/Salsa grammar engine, no I6 schema generation, and no full `AdjectiveMeanings::claim_definition` dispatcher.

### 3. What is the smallest independently testable subset?

1. `AdjectivesByInterCondition::start(&mut families) -> usize` creates the Inter-condition family at priority 4 and returns its index.
2. `AdjectivesByInterCondition::claim_definition(...)` declines (`None`) when the condition text does not match the Inter condition template.
3. `claim_definition(...)` declines when `sense != 1`.
4. `claim_definition(...)` declines when `calling_text` is non-empty.
5. `claim_definition(...)` accepts for a matching `i6/inter condition "C" says so` template and the resulting meaning:
   - belongs to the Inter-condition family,
   - is attached to a declared adjective whose name is the headword,
   - has a text-based domain,
   - has `TEST_ATOM_TASK` marked as via-support-function,
   - has `NOW_ATOM_TRUE_TASK` and `NOW_ATOM_FALSE_TASK` still `NO_TASKMODE`.
6. `is_by_inter_condition` returns true only for Inter-condition-family meanings.

### 4. What simplifications are appropriate?

- **No `Definition` struct yet.** The C `claim_definition` creates a `definition` via `AdjectivalDefinitionFamily::new_definition`. The full `AdjectivalDefinitionFamily` is out of scope (Chapter 5), so the foundation stores `None` in `family_specific_data` and defers the `Definition` struct.
- **No real Preform/Salsa grammar engine.** We take the condition text as a `&'static str` and use a small Rust helper that recognizes the single legal template and extracts the condition text. The full `<inform6-condition-adjective-definition>` grammar is deferred.
- **No `parse_node` handling.** The source sentence location `q` is represented by an optional string tag; the foundation ignores it.
- **No `RTAdjectives::set_schemas_for_raw_Inter_condition` schema generation.** The C function builds an Inform 6 schema and then calls `perform_task_via_function`. The foundation skips schema generation but performs the same task-mode marking: only `TEST_ATOM_TASK` is marked as via-support-function.
- **No deferred `prepare_schemas` method.** The C family does not register `GENERATE_IN_SUPPORT_FUNCTION_ADJM_MTID`; schema setup happens inline during `claim_definition`. Therefore the Rust family also installs only `claim_definition`; `prepare_schemas` is left `None`.
- **No generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the Inter-condition family's method is installed.
- **Use the existing measurement-shaped `ClaimDefinitionFn` signature.** The current Rust type alias does not include the family index or calling wording. We install a thin wrapper that reads the Inter-condition family index from a static and assumes an empty calling text.

## Background

### C reference architecture

Adjectives defined by a raw Inter condition look like:

```inform7
Definition: a text is empty if i6/inter condition "TEXT_TY_Empty" says so.
```

The C implementation lives in `inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w` (lines 17-43). Key functions:

```c
adjective_meaning_family *inter_condition_amf = NULL;

void AdjectivesByInterCondition::start(void) {
    inter_condition_amf = AdjectiveMeanings::new_family(4);
    METHOD_ADD(inter_condition_amf, CLAIM_DEFINITION_SENTENCE_ADJM_MTID,
        AdjectivesByInterCondition::claim_definition);
}

int AdjectivesByInterCondition::claim_definition(adjective_meaning_family *f,
    adjective_meaning **result, parse_node *q,
    int sense, wording AW, wording DNW, wording CONW, wording CALLW) {
    if (sense != 1) return FALSE;
    if (Wordings::nonempty(CALLW)) return FALSE;
    if (!(<inform6-condition-adjective-definition>(CONW))) return FALSE;
    int text_wn = <<r>>;
    wording IN = GET_RW(<inform6-condition-adjective-definition>, 1);

    definition *def = AdjectivalDefinitionFamily::new_definition(q);
    adjective_meaning *am =
        AdjectiveMeanings::new(inter_condition_amf,
            STORE_POINTER_definition(def), IN);
    def->am_of_def = am;
    adjective *adj = Adjectives::declare(AW, NULL);
    AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    AdjectiveMeaningDomains::set_from_text(am, DNW);
    RTAdjectives::set_schemas_for_raw_Inter_condition(am, text_wn);
    *result = am;
    return TRUE;
}
```

The grammar `<inform6-condition-adjective-definition>` (line 14) matches one form:

```c
<inform6-condition-adjective-definition> ::=
    i6/inter condition <quoted-text-without-subs> says so ( ... ) ==> { pass 1 }
```

The production value `<<r>>` is the word number of the quoted condition text, used only for schema generation. Capture 1 (`IN`) is the quoted condition text wording, stored as the meaning's `indexing_text`.

The runtime schema setup is in `inform7/runtime-module/Chapter 5/Adjectives.w`, lines 431-434:

```c
void RTAdjectives::set_schemas_for_raw_Inter_condition(adjective_meaning *am, int wn) {
    i6_schema *sch = AdjectiveMeanings::make_schema(am, TEST_ATOM_TASK);
    Word::dequote(wn);
    Calculus::Schemas::modify(sch, "(%N)", wn);
}
```

The family does **not** register a `GENERATE_IN_SUPPORT_FUNCTION` method; `RTAdjectives::set_schemas_for_raw_Inter_condition` is called directly from `claim_definition`, and it only prepares a test schema (no NOW tasks).

### Current Rust state

- `crates/conform7-semantics/src/assertions/mod.rs` exposes `imperative_definition_families`, `adjectives_by_condition`, `adjectives_by_phrase`, and `adjectives_by_inter_function`.
- `crates/conform7-semantics/src/knowledge/adjectives.rs` contains `Adjective`, `AdjectiveMeaning`, `AdjectiveMeaningFamily`, `AdjectiveMeaningFamilyMethods`, `AdjectiveMeanings`, `AdjectiveAmbiguity`, `AdjectiveMeaningDomains`, `perform_task_via_function`, and the `ClaimDefinitionFn` type alias.
- `AdjectiveMeanings::new_family` takes `name`, `priority`, `methods`, and `families`.
- `AdjectiveMeanings::new` creates a meaning with a family index and optional family-specific data.
- `AdjectiveMeanings::perform_task_via_function` already exists and sets the requested task mode to `VIA_SUPPORT_FUNCTION_TASKMODE`.
- `AdjectiveMeaningFamilyMethods.claim_definition` currently uses the measurement-shaped function signature. We will install a thin wrapper rather than changing the signature in this plan.

## Tasks

### Task 1: Create the `AdjectivesByInterCondition` module

Create `crates/conform7-semantics/src/assertions/adjectives_by_inter_condition.rs`.

Module-level doc comment:

```rust
//! Adjectives by Inter Condition — adjectives defined by a raw Inter condition.
//!
//! Corresponds to `AdjectivesByInterCondition` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`).
//!
//! This module creates the `inter_condition_amf` family and claims adjective
//! definitions whose body is a raw Inter condition
//! (`Definition: a ... is ... if i6/inter condition "C" says so`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No real Preform/Salsa grammar parsing; a small string helper recognizes
//!   the single legal template and extracts the condition text.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_raw_Inter_condition` schema generation;
//!   only `TEST_ATOM_TASK` is marked via-support-function.
//! - The family method wrapper uses a static family index to fit the existing
//!   measurement-shaped `ClaimDefinitionFn` signature.
```

Imports:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
    TEST_ATOM_TASK,
};
use crate::knowledge::measurements::MeasurementDefinition;
use crate::knowledge::properties::Property;
```

Static family index:

```rust
/// Global family index for the Inter condition family.
///
/// Mirrors the C static `inter_condition_amf`. Set by `AdjectivesByInterCondition::start`.
static INTER_CONDITION_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Inter Condition module.
///
/// Corresponds to `AdjectivesByInterCondition` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`).
pub struct AdjectivesByInterCondition;
```

Internal parsed representation:

```rust
/// Result of matching `<inform6-condition-adjective-definition>`.
///
/// Inter-condition definitions are always test-only (`says so`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct InterConditionDefinition {
    condition_text: &'static str,
    quoted_text: &'static str,
}
```

Implement `AdjectivesByInterCondition`:

```rust
impl AdjectivesByInterCondition {
    /// Priority of the Inter condition family in the definition-claim order.
    ///
    /// Corresponds to the `4` passed to `AdjectiveMeanings::new_family` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`, line 22).
    pub const INTER_CONDITION_FAMILY_PRIORITY: u8 = 4;

    /// Create the Inter condition adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByInterCondition::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`, lines 21-25).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByInterCondition::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "inter_condition",
            Self::INTER_CONDITION_FAMILY_PRIORITY,
            methods,
            families,
        );
        INTER_CONDITION_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the Inter condition family set by the most recent `start`.
    fn inter_condition_family_idx() -> usize {
        INTER_CONDITION_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the Inter condition family.
    pub fn is_by_inter_condition(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        inter_condition_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == inter_condition_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only Inter-condition definitions that match the expected template,
    /// have `sense == 1`, and have no calling wording. Creates the adjective
    /// meaning, declares the adjective, links them, stores the domain, and
    /// marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` is ignored.
    /// - The quoted condition text is extracted and stored as `indexing_text`;
    ///   The raw word number and full `Definition` context that the C reference stores are deferred.
    ///
    /// - No Inform 6 schemas are generated; only the task-mode side effect of
    ///   `RTAdjectives::set_schemas_for_raw_Inter_condition` is reproduced.
    #[allow(clippy::too_many_arguments)]
    pub fn claim_definition(
        inter_condition_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        calling_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        let parsed = Self::parse_inter_condition_definition(condition_text?)?;

        if sense != 1 {
            return None;
        }
        if calling_text.is_some_and(|s| !s.is_empty()) {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            inter_condition_family_idx,
            None,
            Some(parsed.quoted_text),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);

        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// A minimal parser for the single `<inform6-condition-adjective-definition>` template.
    ///
    /// This is a foundation stand-in for the full Preform grammar engine. It
    /// accepts strings of the form:
    ///
    /// - `i6/inter condition "Condition" says so (...)` -> returns the quoted text
    ///
    /// The `condition_text` returned is the text inside the quotes (without the
    /// quotes). The `quoted_text` returned is the original quoted segment.
    fn parse_inter_condition_definition(
        text: &'static str,
    ) -> Option<InterConditionDefinition> {
        let trimmed = text.trim_start();

        // Strip the required prefix; accept the exact Inform 7 keyword phrase.
        let after_prefix = trimmed
            .strip_prefix("i6/inter condition")
            .or_else(|| trimmed.strip_prefix("inter condition"))?
            .trim_start();

        // The condition must be a quoted string.
        if !after_prefix.starts_with('"') {
            return None;
        }
        let inner = &after_prefix[1..];
        let end = inner.find('"')?;
        let condition_text = &inner[..end];
        let tail = &inner[end + 1..];

        // The template must end with "says so"; ignore any parenthesised tail.
        if !tail.trim_start().starts_with("says so") {
            return None;
        }

        Some(InterConditionDefinition {
            condition_text,
            quoted_text: condition_text,
        })
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific parameters and the calling wording (assumed empty),
    /// and reads the Inter condition family index from the static set by `start`.
    #[allow(clippy::too_many_arguments)]
    fn claim_definition_family_method(
        headword: &'static str,
        _prop: Option<usize>,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        _definitions: &mut Vec<MeasurementDefinition>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        _properties: &[Property],
    ) -> Option<usize> {
        Self::claim_definition(
            Self::inter_condition_family_idx(),
            headword,
            sense,
            domain_text,
            condition_text,
            None,
            adjectives,
            meanings,
        )
    }
}
```

Notes on the parser:
- It deliberately ignores the parenthesised argument list, matching the `(...)` wildcard in the Preform grammar.
- It returns the quoted text as both `condition_text` and `quoted_text`; this is a foundation simplification. The C reference stores capture 1 (`IN`) in the meaning's indexing text, but without the Preform capture machinery we keep the quoted segment. A later plan can refine the capture mapping.

### Task 2: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`.

Add the module declaration:

```rust
pub mod adjectives_by_inter_condition;
```

Add a module-map row:

```markdown
| [`adjectives_by_inter_condition`] | `Chapter 8/Adjectives by Inter Condition.w` | Inter-condition-defined adjectives |
```

Add the C reference to the References list:

```markdown
- C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`
```

Update the startup-sequence comment to show the new module after inter function:

```markdown
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
```

Full expected `mod.rs`:

```rust
//! Assertions module — the assertion-processing engine for Inform 7.
//!
//! This module corresponds to the `assertions-module` in the C reference
//! (`inform7/assertions-module/Chapter 1/Assertions Module.w`). It is the
//! central dispatch for processing assertion sentences — the sentences that
//! describe the model world in Inform 7 source text.
//!
//! The assertions module is initialized as part of the startup sequence:
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
//! | [`adjectives_by_phrase`] | `Chapter 8/Adjectives by Phrase.w` | Phrase-defined adjectives |
//! | [`adjectives_by_condition`] | `Chapter 8/Adjectives by Condition.w` | Condition-defined adjectives |
//! | [`adjectives_by_inter_function`] | `Chapter 8/Adjectives by Inter Function.w` | Inter-routine-defined adjectives |
//! | [`adjectives_by_inter_condition`] | `Chapter 8/Adjectives by Inter Condition.w` | Inter-condition-defined adjectives |
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 1/Assertions Module.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
//! - C reference: `inform7/assertions-module/Chapter 5/To Phrase Family.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Condition.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Rule Family.w`

pub mod imperative_definition_families;
pub mod adjectives_by_condition;
pub mod adjectives_by_inter_condition;
pub mod adjectives_by_inter_function;
pub mod adjectives_by_phrase;
```

### Task 3: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/adjectives_by_inter_condition.rs`.

Required imports in tests:

```rust
use super::*;
use crate::knowledge::adjectives::{
    NO_TASKMODE, NOW_ATOM_FALSE_TASK, NOW_ATOM_TRUE_TASK, VIA_SUPPORT_FUNCTION_TASKMODE,
};
```

Tests:

1. `start_creates_inter_condition_family_with_priority_4`

```rust
#[test]
fn start_creates_inter_condition_family_with_priority_4() {
    let mut families = Vec::new();
    let idx = AdjectivesByInterCondition::start(&mut families);
    assert_eq!(families[idx].name, "inter_condition");
    assert_eq!(families[idx].definition_claim_priority, 4);
    assert!(families[idx].methods.claim_definition.is_some());
    assert!(families[idx].methods.prepare_schemas.is_none());
}
```

2. `claim_definition_declines_unrecognised_condition_text`

```rust
#[test]
fn claim_definition_declines_unrecognised_condition_text() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByInterCondition::claim_definition(
        family,
        "empty",
        1,
        Some("text"),
        Some("it is empty"),
        None,
        &mut adjectives,
        &mut meanings,
    );
    assert!(result.is_none());
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

3. `claim_definition_declines_non_one_sense`

```rust
#[test]
fn claim_definition_declines_non_one_sense() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    for sense in [0, -1, 2] {
        let result = AdjectivesByInterCondition::claim_definition(
            family,
            "empty",
            sense,
            Some("text"),
            Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
            None,
            &mut adjectives,
            &mut meanings,
        );
        assert!(result.is_none(), "sense {} should be declined", sense);
    }
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

4. `claim_definition_declines_non_empty_calling_text`

```rust
#[test]
fn claim_definition_declines_non_empty_calling_text() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByInterCondition::claim_definition(
        family,
        "empty",
        1,
        Some("text"),
        Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
        Some("call it here"),
        &mut adjectives,
        &mut meanings,
    );
    assert!(result.is_none());
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

5. `claim_definition_creates_inter_condition_meaning`

```rust
#[test]
fn claim_definition_creates_inter_condition_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByInterCondition::claim_definition(
        family,
        "empty",
        1,
        Some("text"),
        Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
        None,
        &mut adjectives,
        &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, family);
    assert!(meanings[am_idx].family_specific_data.is_none());
    assert_eq!(
        meanings[am_idx].indexing_text,
        Some("TEXT_TY_Empty")
    );
    assert_eq!(meanings[am_idx].domain.domain_text, Some("text"));
    assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_TRUE_TASK], NO_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_FALSE_TASK], NO_TASKMODE);

    let adj_idx = meanings[am_idx].owning_adjective.unwrap();
    assert_eq!(adjectives[adj_idx].name, "empty");
    assert!(adjectives[adj_idx].meanings.contains(&am_idx));
}
```

6. `is_by_inter_condition_true_for_inter_condition_meaning`

```rust
#[test]
fn is_by_inter_condition_true_for_inter_condition_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByInterCondition::claim_definition(
        family,
        "empty",
        1,
        None,
        Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
        None,
        &mut adjectives,
        &mut meanings,
    ).unwrap();

    assert!(AdjectivesByInterCondition::is_by_inter_condition(
        am_idx, &meanings, family
    ));
}
```

7. `is_by_inter_condition_false_for_other_meaning`

```rust
#[test]
fn is_by_inter_condition_false_for_other_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterCondition::start(&mut families);
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

    assert!(!AdjectivesByInterCondition::is_by_inter_condition(
        am_idx, &meanings, family
    ));
}
```

### Task 4: Verify

- [ ] `cargo build`
- [ ] `cargo test -- assertions::adjectives_by_inter_condition`
- [ ] `cargo test` (all existing tests still pass)
- [ ] `cargo clippy --all-targets`

## Success Criteria

- [ ] `assertions::adjectives_by_inter_condition` module exists and compiles.
- [ ] `AdjectivesByInterCondition::start` creates a family named `"inter_condition"` with priority 4 and only the `claim_definition` method installed (`prepare_schemas` is `None`).
- [ ] `AdjectivesByInterCondition::claim_definition` returns `None` when the condition text does not match the Inter condition template.
- [ ] `AdjectivesByInterCondition::claim_definition` returns `None` for `sense != 1`.
- [ ] `AdjectivesByInterCondition::claim_definition` returns `None` when `calling_text` is non-empty.
- [ ] `AdjectivesByInterCondition::claim_definition` returns `Some(am_idx)` for matching `i6/inter condition "..." says so` templates.
- [ ] The created meaning belongs to the Inter condition family and has `family_specific_data == None`.
- [ ] The created adjective has the given headword and contains the new meaning.
- [ ] The meaning's domain text is set from `domain_text`.
- [ ] The meaning's `indexing_text` is set to the quoted condition text.
- [ ] Only `TEST_ATOM_TASK` is marked `VIA_SUPPORT_FUNCTION_TASKMODE`; `NOW_ATOM_TRUE_TASK` and `NOW_ATOM_FALSE_TASK` remain `NO_TASKMODE`.
- [ ] `AdjectivesByInterCondition::is_by_inter_condition` returns true only for Inter-condition-family meanings.
- [ ] `cargo clippy --all-targets` is clean.
- [ ] The total test count remains at least 1394 (existing) plus the new tests.

## Out of Scope

- **Full `AdjectivalDefinitionFamily` (Chapter 5).** The `definition` struct, `new_definition`, node rewriting, and `am_of_def` back-link are deferred.
- **`RTAdjectives::set_schemas_for_raw_Inter_condition` schema generation.** Run-time Inform 6 schema construction is deferred. The foundation reproduces only the task-mode side effect inline in `claim_definition`.
- **Generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the Inter-condition family's method is installed.
- **Full Preform / Salsa grammar integration.** The small string parser in this plan is a stand-in for `<inform6-condition-adjective-definition>`. The real grammar engine, capture numbering, and word-range handling are deferred.
- **Problem messages.** No `StandardProblems` calls for malformed Inter condition definitions.
- **Other adjective definition families.** Families after Inter condition are out of scope.
- **Storing the raw condition word number and exact capture wording.** These will be persisted when the `Definition` struct and full grammar integration are introduced.
