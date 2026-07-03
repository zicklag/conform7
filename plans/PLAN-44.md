# Plan 44: Adjectives by Inter Function — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Implement the foundation of the Adjectives by Inter Function system — creating the Inter routine adjective meaning family with `claim_definition` method. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 36), immediately after `AdjectivesByCondition::start()` (PLAN-43, Complete).

## Decision

### 1. Is `AdjectivesByInterFunction` the correct next step?

**Yes.** In the C assertions-module startup sequence, after `AdjectivesByCondition::start()` comes `AdjectivesByInterFunction::start()`. The Rust assertions module already has `AdjectivesByPhrase` (PLAN-42) and `AdjectivesByCondition` (PLAN-43), so the Inter-routine family is the natural next foundation piece.

### 2. Is it independently testable?

**Yes.** The foundation consists of pure data-structure operations:
- creating an `adjective_meaning_family` with priority 5;
- registering only the `claim_definition` method (no deferred support-function method);
- `claim_definition` deciding whether to claim a definition by matching the `<inform6-routine-adjective-definition>` template, checking `sense == 1` and empty `CALLW`, and if so creating an `Adjective`, an `AdjectiveMeaning`, linking them, setting the domain, and marking the appropriate atom tasks as via-support-function.

These operations require no Preform/Salsa grammar engine, no I6 schema generation, and no full `AdjectiveMeanings::claim_definition` dispatcher.

### 3. What is the smallest independently testable subset?

1. `AdjectivesByInterFunction::start(&mut families) -> usize` creates the inter-routine family at priority 5 and returns its index.
2. `AdjectivesByInterFunction::claim_definition(...)` declines (`None`) when the condition text does not match the Inter routine template.
3. `claim_definition(...)` declines when `sense != 1`.
4. `claim_definition(...)` declines when `calling_text` is non-empty.
5. `claim_definition(...)` accepts for a matching "says so" template (`setting == false`) and the resulting meaning:
   - belongs to the inter-routine family,
   - is attached to a declared adjective whose name is the headword,
   - has a text-based domain,
   - has `TEST_ATOM_TASK` marked as via-support-function,
   - has `NOW_ATOM_TRUE_TASK` and `NOW_ATOM_FALSE_TASK` still `NO_TASKMODE`.
6. `claim_definition(...)` accepts for a matching "makes it so" template (`setting == true`) and marks `TEST_ATOM_TASK`, `NOW_ATOM_TRUE_TASK`, and `NOW_ATOM_FALSE_TASK` as via-support-function.
7. `is_by_inter_function` returns true only for inter-routine-family meanings.

### 4. What simplifications are appropriate?

- **No `Definition` struct yet.** The C `claim_definition` creates a `definition` via `AdjectivalDefinitionFamily::new_definition`. The full `AdjectivalDefinitionFamily` is out of scope (Chapter 5), so the foundation stores `None` in `family_specific_data` and defers the `Definition` struct.
- **No real Preform/Salsa grammar engine.** We take the condition text as a `&'static str` and use a small Rust helper that recognizes the two templates and extracts the routine name, the setting flag, and the remaining wording. The full `<inform6-routine-adjective-definition>` grammar is deferred.
- **No `parse_node` handling.** The source sentence location `q` is represented by an optional string tag; the foundation ignores it.
- **No `RTAdjectives::set_schemas_for_raw_Inter_function` schema generation.** The C function builds Inform 6 schemas and then calls `perform_task_via_function`. The foundation skips schema generation but performs the same task-mode marking: for "test" templates only `TEST_ATOM_TASK`; for "makes it so" templates also `NOW_ATOM_TRUE_TASK` and `NOW_ATOM_FALSE_TASK`.
- **No deferred `prepare_schemas` method.** The C family does not register `GENERATE_IN_SUPPORT_FUNCTION_ADJM_MTID`; schema setup happens inline during `claim_definition`. Therefore the Rust family also installs only `claim_definition`; `prepare_schemas` is left `None`.
- **No generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the inter-routine family's method is installed.
- **Use the existing measurement-shaped `ClaimDefinitionFn` signature.** The current Rust type alias does not include the family index or calling wording. We install a thin wrapper that reads the inter-routine family index from a static and assumes an empty calling text.

## Background

### C reference architecture

Adjectives defined by a named Inter/I6 routine look like:

```inform7
Definition: a container is see-through if I6 routine/function "MapScope" says so.
Definition: a container is opaque if I6 routine/function "MakeOpaque" makes it so.
```

The C implementation lives in `inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w` (lines 17-54). Key functions:

```c
adjective_meaning_family *inter_routine_amf = NULL;

void AdjectivesByInterFunction::start(void) {
    inter_routine_amf = AdjectiveMeanings::new_family(5);
    METHOD_ADD(inter_routine_amf, CLAIM_DEFINITION_SENTENCE_ADJM_MTID,
        AdjectivesByInterFunction::claim_definition);
}

int AdjectivesByInterFunction::is_by_Inter_function(adjective_meaning *am) {
    if ((am) && (am->family == inter_routine_amf)) return TRUE;
    return FALSE;
}

int AdjectivesByInterFunction::claim_definition(adjective_meaning_family *f,
    adjective_meaning **result, parse_node *q,
    int sense, wording AW, wording DNW, wording CONW, wording CALLW) {
    int setting = FALSE;
    wording EW = EMPTY_WORDING, RW = EMPTY_WORDING;
    if (<inform6-routine-adjective-definition>(CONW)) {
        setting = <<r>>;
        RW = GET_RW(<inform6-routine-adjective-definition>, 1);
        EW = GET_RW(<inform6-routine-adjective-definition>, 2);
    } else return FALSE;

    if (sense != 1) return FALSE;
    if (Wordings::nonempty(CALLW)) return FALSE;

    definition *def = AdjectivalDefinitionFamily::new_definition(q);
    adjective_meaning *am =
        AdjectiveMeanings::new(inter_routine_amf, STORE_POINTER_definition(def), EW);
    def->am_of_def = am;
    adjective *adj = Adjectives::declare(AW, NULL);
    AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    AdjectiveMeaningDomains::set_from_text(am, DNW);
    RTAdjectives::set_schemas_for_raw_Inter_function(am, RW, setting);
    *result = am;
    return TRUE;
}
```

The grammar `<inform6-routine-adjective-definition>` (lines 10-12) matches two forms:

```c
<inform6-routine-adjective-definition> ::=
    i6/inter routine/function {<quoted-text-without-subs>} says so ( ... ) |   ==> { FALSE, - }
    i6/inter routine/function {<quoted-text-without-subs>} makes it so ( ... ) ==> { TRUE, - }
```

The production value `<<r>>` is `FALSE` for a test routine and `TRUE` for a setting routine. Capture 1 (`RW`) is the quoted routine name; capture 2 (`EW`) is used as the meaning's indexing text.

The runtime schema setup is in `inform7/runtime-module/Chapter 5/Adjectives.w`, lines 441-460:

```c
void RTAdjectives::set_schemas_for_raw_Inter_function(adjective_meaning *am, wording RW,
    int setting) {
    int wn = Wordings::first_wn(RW);
    Word::dequote(wn);
    if (setting) {
        i6_schema *sch = AdjectiveMeanings::make_schema(am, TEST_ATOM_TASK);
        Calculus::Schemas::modify(sch, "*=-(%N(*1, -1))", wn);
        AdjectiveMeanings::perform_task_via_function(am, TEST_ATOM_TASK);
        sch = AdjectiveMeanings::make_schema(am, NOW_ATOM_TRUE_TASK);
        Calculus::Schemas::modify(sch, "*=-(%N(*1, true))", wn);
        AdjectiveMeanings::perform_task_via_function(am, NOW_ATOM_TRUE_TASK);
        sch = AdjectiveMeanings::make_schema(am, NOW_ATOM_FALSE_TASK);
        Calculus::Schemas::modify(sch, "*=-(%N(*1, false))", wn);
        AdjectiveMeanings::perform_task_via_function(am, NOW_ATOM_FALSE_TASK);
    } else {
        i6_schema *sch = AdjectiveMeanings::make_schema(am, TEST_ATOM_TASK);
        Calculus::Schemas::modify(sch, "*=-(%N(*1))", wn);
        AdjectiveMeanings::perform_task_via_function(am, TEST_ATOM_TASK);
    }
}
```

The family does **not** register a `GENERATE_IN_SUPPORT_FUNCTION` method; `RTAdjectives::set_schemas_for_raw_Inter_function` is called directly from `claim_definition`.

### Current Rust state

- `crates/conform7-semantics/src/assertions/mod.rs` exposes `imperative_definition_families`, `adjectives_by_condition`, and `adjectives_by_phrase`.
- `crates/conform7-semantics/src/knowledge/adjectives.rs` contains `Adjective`, `AdjectiveMeaning`, `AdjectiveMeaningFamily`, `AdjectiveMeaningFamilyMethods`, `AdjectiveMeanings`, `AdjectiveAmbiguity`, `AdjectiveMeaningDomains`, `perform_task_via_function`, and the `ClaimDefinitionFn` type alias.
- `AdjectiveMeanings::new_family` takes `name`, `priority`, `methods`, and `families`.
- `AdjectiveMeanings::new` creates a meaning with a family index and optional family-specific data.
- `AdjectiveMeanings::perform_task_via_function` already exists and sets the requested task mode to `VIA_SUPPORT_FUNCTION_TASKMODE`.
- `AdjectiveMeaningFamilyMethods.claim_definition` currently uses the measurement-shaped function signature. We will install a thin wrapper rather than changing the signature in this plan.

## Tasks

### Task 1: Create the `AdjectivesByInterFunction` module

Create `crates/conform7-semantics/src/assertions/adjectives_by_inter_function.rs`.

Module-level doc comment:

```rust
//! Adjectives by Inter Function — adjectives defined by a named Inter/I6 routine.
//!
//! Corresponds to `AdjectivesByInterFunction` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`).
//!
//! This module creates the `inter_routine_amf` family and claims adjective
//! definitions whose body names an Inter routine/function
//! (`Definition: a ... is ... if i6/inter routine/function "R" says so`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No real Preform/Salsa grammar parsing; a small string helper recognizes
//!   the two legal templates and extracts the routine name, setting flag, and
//!   remaining text.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_raw_Inter_function` schema generation;
//!   task-mode marking is performed so that TEST (and NOW, for setting
//!   routines) are flagged via-support-function.
//! - The family method wrapper uses a static family index to fit the existing
//!   measurement-shaped `ClaimDefinitionFn` signature.
```

Imports:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
    NOW_ATOM_FALSE_TASK, NOW_ATOM_TRUE_TASK, TEST_ATOM_TASK,
};
use crate::knowledge::measurements::MeasurementDefinition;
use crate::knowledge::properties::Property;
```

Static family index:

```rust
/// Global family index for the Inter routine family.
///
/// Mirrors the C static `inter_routine_amf`. Set by `AdjectivesByInterFunction::start`.
static INTER_ROUTINE_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Inter Function module.
///
/// Corresponds to `AdjectivesByInterFunction` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`).
pub struct AdjectivesByInterFunction;
```

Internal parsed representation:

```rust
/// Result of matching `<inform6-routine-adjective-definition>`.
///
/// `setting == false` corresponds to `says so` (a test routine).
/// `setting == true` corresponds to `makes it so` (a now routine).
#[derive(Clone, Copy, Debug, PartialEq)]
struct InterRoutineDefinition {
    routine_name: &'static str,
    extra_text: &'static str,
    setting: bool,
}
```

Implement `AdjectivesByInterFunction`:

```rust
impl AdjectivesByInterFunction {
    /// Priority of the Inter routine family in the definition-claim order.
    ///
    /// Corresponds to the `5` passed to `AdjectiveMeanings::new_family` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, line 20).
    pub const INTER_ROUTINE_FAMILY_PRIORITY: u8 = 5;

    /// Create the Inter routine adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByInterFunction::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, lines 19-23).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByInterFunction::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "inter_routine",
            Self::INTER_ROUTINE_FAMILY_PRIORITY,
            methods,
            families,
        );
        INTER_ROUTINE_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the Inter routine family set by the most recent `start`.
    fn inter_routine_family_idx() -> usize {
        INTER_ROUTINE_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the Inter routine family.
    ///
    /// Corresponds to `AdjectivesByInterFunction::is_by_Inter_function` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, lines 25-28).
    pub fn is_by_inter_function(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        inter_routine_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == inter_routine_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only Inter-routine definitions that match the expected template,
    /// have `sense == 1`, and have no calling wording. Creates the adjective
    /// meaning, declares the adjective, links them, stores the domain, and
    /// marks the appropriate tasks as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` is ignored.
    /// - The routine name `RW` is extracted but not stored in the meaning
    ///   (the `Definition` struct that will hold it is deferred).
    /// - No Inform 6 schemas are generated; only the task-mode side effects of
    ///   `RTAdjectives::set_schemas_for_raw_Inter_function` are reproduced.
    pub fn claim_definition(
        inter_routine_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        calling_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        let parsed = Self::parse_inter_routine_definition(condition_text?)?;

        if sense != 1 {
            return None;
        }
        if calling_text.is_some_and(|s| !s.is_empty()) {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            inter_routine_family_idx,
            None,
            Some(parsed.extra_text),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);

        Self::mark_inter_routine_tasks(am_idx, parsed.setting, meanings);

        Some(am_idx)
    }

    /// Mark the atom tasks that are implemented by the raw Inter function.
    ///
    /// Mirrors the `perform_task_via_function` calls made by
    /// `RTAdjectives::set_schemas_for_raw_Inter_function`
    /// (`inform7/runtime-module/Chapter 5/Adjectives.w`, lines 441-460).
    fn mark_inter_routine_tasks(am_idx: usize, setting: bool, meanings: &mut [AdjectiveMeaning]) {
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);
        if setting {
            AdjectiveMeanings::perform_task_via_function(am_idx, NOW_ATOM_TRUE_TASK, meanings);
            AdjectiveMeanings::perform_task_via_function(am_idx, NOW_ATOM_FALSE_TASK, meanings);
        }
    }

    /// A minimal parser for the two `<inform6-routine-adjective-definition>` templates.
    ///
    /// This is a foundation stand-in for the full Preform grammar engine. It
    /// accepts strings of the form:
    ///
    /// - `i6/inter routine/function "Name" says so (...)` -> `setting = false`
    /// - `i6/inter routine/function "Name" makes it so (...)` -> `setting = true`
    ///
    /// The `routine_name` returned is the text inside the quotes (without the
    /// quotes). The `extra_text` returned is the original `condition_text`.
    fn parse_inter_routine_definition(
        text: &'static str,
    ) -> Option<InterRoutineDefinition> {
        let trimmed = text.trim_start();

        // Strip the required prefix; accept the exact Inform 7 keyword phrase.
        let after_prefix = trimmed
            .strip_prefix("i6/inter routine/function")
            .or_else(|| trimmed.strip_prefix("inter routine/function"))?
            .trim_start();

        // The routine name must be a quoted string.
        if !after_prefix.starts_with('"') {
            return None;
        }
        let inner = &after_prefix[1..];
        let end = inner.find('"')?;
        let routine_name = &inner[..end];
        let tail = &inner[end + 1..];

        // Decide which of the two templates matched.
        let tail_trimmed = tail.trim_start();
        let setting = if tail_trimmed.starts_with("makes it so") {
            true
        } else if tail_trimmed.starts_with("says so") {
            false
        } else {
            return None;
        };

        Some(InterRoutineDefinition {
            routine_name,
            extra_text: text,
            setting,
        })
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific parameters and the calling wording (assumed empty),
    /// and reads the Inter routine family index from the static set by `start`.
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
            Self::inter_routine_family_idx(),
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
- It returns the original `condition_text` as `extra_text`; this is a foundation simplification. The C reference stores capture 2 (`EW`) in the meaning's indexing text, but without the Preform capture machinery we keep the whole condition text. A later plan can refine the capture mapping.

### Task 2: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`.

Add the module declaration:

```rust
pub mod adjectives_by_inter_function;
```

Add a module-map row:

```markdown
| [`adjectives_by_inter_function`] | `Chapter 8/Adjectives by Inter Function.w` | Inter-routine-defined adjectives |
```

Add the C reference to the References list:

```markdown
- C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`
```

Update the startup-sequence comment to show the new module after condition:

```markdown
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44), etc.
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
//! then `AdjectivesByInterFunction::start()` (PLAN-44), etc.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
//! | [`adjectives_by_phrase`] | `Chapter 8/Adjectives by Phrase.w` | Phrase-defined adjectives |
//! | [`adjectives_by_condition`] | `Chapter 8/Adjectives by Condition.w` | Condition-defined adjectives |
//! | [`adjectives_by_inter_function`] | `Chapter 8/Adjectives by Inter Function.w` | Inter-routine-defined adjectives |
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 1/Assertions Module.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
//! - C reference: `inform7/assertions-module/Chapter 5/To Phrase Family.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Condition.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Rule Family.w`

pub mod imperative_definition_families;
pub mod adjectives_by_condition;
pub mod adjectives_by_inter_function;
pub mod adjectives_by_phrase;
```

### Task 3: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/adjectives_by_inter_function.rs`.

Required imports in tests:

```rust
use super::*;
use crate::knowledge::adjectives::{NO_TASKMODE, VIA_SUPPORT_FUNCTION_TASKMODE};
```

Tests:

1. `start_creates_inter_routine_family_with_priority_5`

```rust
#[test]
fn start_creates_inter_routine_family_with_priority_5() {
    let mut families = Vec::new();
    let idx = AdjectivesByInterFunction::start(&mut families);
    assert_eq!(families[idx].name, "inter_routine");
    assert_eq!(families[idx].definition_claim_priority, 5);
    assert!(families[idx].methods.claim_definition.is_some());
    assert!(families[idx].methods.prepare_schemas.is_none());
}
```

2. `claim_definition_declines_unrecognised_condition_text`

```rust
#[test]
fn claim_definition_declines_unrecognised_condition_text() {
    let mut families = Vec::new();
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByInterFunction::claim_definition(
        family,
        "shiny",
        1,
        Some("thing"),
        Some("it is shiny"),
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
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    for sense in [0, -1, 2] {
        let result = AdjectivesByInterFunction::claim_definition(
            family,
            "shiny",
            sense,
            Some("thing"),
            Some("i6/inter routine/function \"IsShiny\" says so"),
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
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let result = AdjectivesByInterFunction::claim_definition(
        family,
        "shiny",
        1,
        Some("thing"),
        Some("i6/inter routine/function \"IsShiny\" says so"),
        Some("call it here"),
        &mut adjectives,
        &mut meanings,
    );
    assert!(result.is_none());
    assert!(adjectives.is_empty());
    assert!(meanings.is_empty());
}
```

5. `claim_definition_creates_test_routine_meaning`

```rust
#[test]
fn claim_definition_creates_test_routine_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByInterFunction::claim_definition(
        family,
        "see-through",
        1,
        Some("container"),
        Some("i6/inter routine/function \"MapScope\" says so"),
        None,
        &mut adjectives,
        &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, family);
    assert!(meanings[am_idx].family_specific_data.is_none());
    assert_eq!(
        meanings[am_idx].indexing_text,
        Some("i6/inter routine/function \"MapScope\" says so")
    );
    assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
    assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_TRUE_TASK], NO_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_FALSE_TASK], NO_TASKMODE);

    let adj_idx = meanings[am_idx].owning_adjective.unwrap();
    assert_eq!(adjectives[adj_idx].name, "see-through");
    assert!(adjectives[adj_idx].meanings.contains(&am_idx));
}
```

6. `claim_definition_creates_setting_routine_meaning`

```rust
#[test]
fn claim_definition_creates_setting_routine_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByInterFunction::claim_definition(
        family,
        "opaque",
        1,
        Some("container"),
        Some("i6/inter routine/function \"MakeOpaque\" makes it so"),
        None,
        &mut adjectives,
        &mut meanings,
    ).unwrap();

    assert_eq!(meanings[am_idx].family, family);
    assert!(meanings[am_idx].family_specific_data.is_none());
    assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
    assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_TRUE_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);
    assert_eq!(meanings[am_idx].task_modes[NOW_ATOM_FALSE_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

    let adj_idx = meanings[am_idx].owning_adjective.unwrap();
    assert_eq!(adjectives[adj_idx].name, "opaque");
    assert!(adjectives[adj_idx].meanings.contains(&am_idx));
}
```

7. `is_by_inter_function_true_for_inter_routine_meaning`

```rust
#[test]
fn is_by_inter_function_true_for_inter_routine_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterFunction::start(&mut families);
    let mut adjectives = Vec::new();
    let mut meanings = Vec::new();

    let am_idx = AdjectivesByInterFunction::claim_definition(
        family,
        "see-through",
        1,
        None,
        Some("i6/inter routine/function \"MapScope\" says so"),
        None,
        &mut adjectives,
        &mut meanings,
    ).unwrap();

    assert!(AdjectivesByInterFunction::is_by_inter_function(
        am_idx, &meanings, family
    ));
}
```

8. `is_by_inter_function_false_for_other_meaning`

```rust
#[test]
fn is_by_inter_function_false_for_other_meaning() {
    let mut families = Vec::new();
    let family = AdjectivesByInterFunction::start(&mut families);
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

    assert!(!AdjectivesByInterFunction::is_by_inter_function(
        am_idx, &meanings, family
    ));
}
```

### Task 4: Verify

- [ ] `cargo build`
- [ ] `cargo test -- assertions::adjectives_by_inter_function`
- [ ] `cargo test` (all existing tests still pass)
- [ ] `cargo clippy --all-targets`

## Success Criteria

- [ ] `assertions::adjectives_by_inter_function` module exists and compiles.
- [ ] `AdjectivesByInterFunction::start` creates a family named `"inter_routine"` with priority 5 and only the `claim_definition` method installed (`prepare_schemas` is `None`).
- [ ] `AdjectivesByInterFunction::claim_definition` returns `None` when the condition text does not match the Inter routine template.
- [ ] `AdjectivesByInterFunction::claim_definition` returns `None` for `sense != 1`.
- [ ] `AdjectivesByInterFunction::claim_definition` returns `None` when `calling_text` is non-empty.
- [ ] `AdjectivesByInterFunction::claim_definition` returns `Some(am_idx)` for matching `"says so"` templates.
- [ ] The created meaning belongs to the inter-routine family and has `family_specific_data == None`.
- [ ] The created adjective has the given headword and contains the new meaning.
- [ ] The meaning's domain text is set from `domain_text`.
- [ ] For `"says so"` templates, only `TEST_ATOM_TASK` is marked `VIA_SUPPORT_FUNCTION_TASKMODE`.
- [ ] For `"makes it so"` templates, `TEST_ATOM_TASK`, `NOW_ATOM_TRUE_TASK`, and `NOW_ATOM_FALSE_TASK` are all marked `VIA_SUPPORT_FUNCTION_TASKMODE`.
- [ ] `AdjectivesByInterFunction::is_by_inter_function` returns true only for inter-routine-family meanings.
- [ ] `cargo clippy --all-targets` is clean.
- [ ] The total test count remains at least 1386 (existing) plus the new tests.

## Out of Scope

- **Full `AdjectivalDefinitionFamily` (Chapter 5).** The `definition` struct, `new_definition`, node rewriting, and `am_of_def` back-link are deferred.
- **`RTAdjectives::set_schemas_for_raw_Inter_function` schema generation.** Run-time Inform 6 schema construction is deferred. The foundation reproduces only the task-mode side effects inline in `claim_definition`.
- **Generic `AdjectiveMeanings::claim_definition` dispatcher.** The loop over families by priority is deferred; only the inter-routine family's method is installed.
- **Full Preform / Salsa grammar integration.** The small string parser in this plan is a stand-in for `<inform6-routine-adjective-definition>`. The real grammar engine, capture numbering, and word-range handling are deferred.
- **Problem messages.** No `StandardProblems` calls for malformed Inter routine definitions.
- **Other adjective definition families.** `AdjectivesByInterCondition` and later families are out of scope.
- **Storing the routine name `RW` and the exact `EW` capture.** These will be persisted when the `Definition` struct and full grammar integration are introduced.
