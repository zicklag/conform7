# Plan 49: Adjectival Predicates — The `adjectival_up_family` Unary Predicate Family

**Status**: Complete
**Target**: 1 day

## Goal

Implement the Adjectival Predicates system — a unary predicate family for adjectives in the Inform 7 calculus. This creates the `adjectival_up_family` with seven methods (`typecheck`, `infer_kind`, `assert`, `testable`, `test`, `schema`, `log`) and helper functions (`new_up`, `new_atom`, `new_atom_on_x`, `to_adjective`, `parity`, `flip_parity`).

This is the **first item** in the assertions module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 26) and the **only remaining unimplemented item** from the assertions module startup sequence (items 1-6). All other startup items (CreationPredicates, QuasinumericRelations, Universal, ExplicitRelations, EqualityDetails, KindPredicatesRevisited, ImperativeDefinitionFamilies, AdjectivesByPhrase/Condition/InterFunction/InterCondition) are already complete.

## Background

### C reference architecture

The C reference (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 1-165) defines a single unary predicate family for adjectives:

```c
up_family *adjectival_up_family = NULL;

void AdjectivalPredicates::start(void) {
    adjectival_up_family = UnaryPredicateFamilies::new();
    METHOD_ADD(adjectival_up_family, TYPECHECK_UPF_MTID, AdjectivalPredicates::typecheck);
    METHOD_ADD(adjectival_up_family, INFER_KIND_UPF_MTID, AdjectivalPredicates::infer_kind);
    METHOD_ADD(adjectival_up_family, ASSERT_UPF_MTID, AdjectivalPredicates::assert);
    METHOD_ADD(adjectival_up_family, TESTABLE_UPF_MTID, AdjectivalPredicates::testable);
    METHOD_ADD(adjectival_up_family, TEST_UPF_MTID, AdjectivalPredicates::test);
    METHOD_ADD(adjectival_up_family, SCHEMA_UPF_MTID, AdjectivalPredicates::get_schema);
    METHOD_ADD(adjectival_up_family, LOG_UPF_MTID, AdjectivalPredicates::log);
}
```
(lines 13-22)

**`new_up`** (lines 24-30): Creates a new adjectival unary predicate from an `adjective *` and a parity (positive/negative sense). Uses `UnaryPredicates::new(adjectival_up_family)`, converts the adjective's stock index to an `lcon`, and sets the sense.

```c
unary_predicate *AdjectivalPredicates::new_up(adjective *adj, int pos) {
    unary_predicate *au = UnaryPredicates::new(adjectival_up_family);
    au->lcon = Stock::to_lcon(adj->in_stock);
    if (pos) au->lcon = Lcon::set_sense(au->lcon, POSITIVE_SENSE);
    else au->lcon = Lcon::set_sense(au->lcon, NEGATIVE_SENSE);
    return au;
}
```

**`new_atom`** (lines 32-35): Creates a proposition atom from an adjective and term.

**`new_atom_on_x`** (lines 37-39): Creates an atom on variable 0 (the default bound variable).

**`log`** (lines 41-45): Logs the adjective with optional "not-" prefix for negative parity.

**`infer_kind`** (lines 47-52): Infers the kind from the adjective's first meaning's domain.

**`typecheck`** (lines 54-66): Checks if the adjective can be applied to the term's kind. Uses `AdjectiveAmbiguity::can_be_applied_to`. Returns `NEVER_MATCH` if misapplied, `ALWAYS_MATCH` otherwise.

**`assert`** (lines 75-102): Asserts the adjective as a true fact about the model world. Uses `AdjectiveAmbiguity::assert` and `Assert::subject_of_term`/`Assert::spec_of_term`.

**`testable`** (lines 104-109): Returns `TRUE` if the adjective has an either-or property meaning.

**`test`** (lines 111-126): Tests the adjective at compile-time using the either-or property's possession marker.

**`schema`** (lines 128-135): Compiles run-time code for the adjective. Uses `AdjectiveAmbiguity::schema_for_task`.

**`to_adjective`** (lines 140-144): Extracts the `adjective *` from a unary predicate by checking the family and converting the `lcon` back.

**`parity`** (lines 146-151): Returns the positive/negative sense of the predicate.

**`flip_parity`** (lines 157-164): Reverses the sense of the predicate.

### Current Rust state

- **`crates/conform7-semantics/src/calculus/unary_predicate_families.rs`** defines `UpFamily` and `UpFamilyMethods` with all seven method slots:
  - `log: fn(&UpFamily, &UnaryPredicate) -> String` (required)
  - `infer_kind: fn(&UpFamily, &UnaryPredicate) -> Option<&'static str>` (required)
  - `testable: fn(&UpFamily, &UnaryPredicate) -> bool` (required)
  - `test: fn(&UpFamily, &UnaryPredicate) -> bool` (required)
  - `typecheck: Option<fn(&UpFamily, &UnaryPredicate, &[Option<usize>], &[Option<usize>]) -> i8>` (optional)
  - `assert: Option<fn(&UpFamily, &UnaryPredicate, bool, &PcalcProp) -> bool>` (optional)
  - `schema: Option<fn(&UpFamily, u8, &UnaryPredicate) -> bool>` (optional)

- **`crates/conform7-semantics/src/calculus/unary_predicates.rs`** defines `UnaryPredicate` with fields: `family`, `assert_kind`, `composited`, `unarticled`, `calling_name`. Note: there is no `lcon` field — the parity (positive/negative sense) is not stored on the predicate. This is a simplification that needs to be addressed: the adjectival predicates need to track parity.

- **`crates/conform7-semantics/src/knowledge/adjectives.rs`** defines:
  - `Adjective` struct with `name`, `meanings`, `sorted_meanings`, `compilation_data`
  - `Adjectives::declare`, `Adjectives::find`, `Adjectives::get_nominative_singular`
  - `AdjectiveMeaning` struct with `owning_adjective`, `domain`, `family`, `task`, `task_mode`, `domain_data`
  - `AdjectiveMeaningFamily` struct with `name`, `priority`, `methods`
  - `AdjectiveMeaningFamilyMethods` with `assert`, `claim_definition`, `domain`, `index`, `compare_for_sorting`
  - `AdjectiveAmbiguity` struct with `first_meaning`, `can_be_applied_to`, `assert`, `has_either_or_property_meaning`, `schema_for_task`
  - `AdjectiveMeaningDomains` struct with `get_kind`

- **`crates/conform7-semantics/src/calculus/mod.rs`** already lists all calculus modules. No `adjectival_predicates` module exists yet.

- **`crates/conform7-semantics/src/assertions/mod.rs`** lists the assertions module startup sequence but does not mention `AdjectivalPredicates` — it's a calculus-level module (like `CreationPredicates`).

- **No `AdjectivalPredicates` module exists anywhere in the Rust codebase.** Confirmed by grep: zero matches for `adjectival_predicates` or `AdjectivalPredicates` in any Rust file.

### Key gap: parity tracking

The C `UnaryPredicate` stores parity in an `lcon` field (a combination of stock index + sense). The Rust `UnaryPredicate` has no `lcon` or parity field. For the adjectival predicates, we need to track whether the predicate is positive or negative sense.

**Simplification**: Add a `parity: bool` field to `UnaryPredicate` (true = positive sense, false = negative sense). This is a minimal addition that enables the adjectival predicates without introducing the full `lcon`/`Stock` system. The field defaults to `true` (positive sense) and is only used by the adjectival family.

## Decision

### 1. Is AdjectivalPredicates the correct next step?

**Yes.** It is the **first item** in the assertions module startup sequence (line 26) and the **only remaining unimplemented item** from items 1-6. All other startup items are complete:

| # | Item | Status |
|---|------|--------|
| 1 | `AdjectivalPredicates::start()` | **NOT YET STARTED** |
| 2 | `CreationPredicates::start()` | PLAN-35 Complete |
| 3 | `Calculus::QuasinumericRelations::start()` | PLAN-36 Complete |
| 4 | `Relations::Universal::start()` | PLAN-37 Complete |
| 5 | `ExplicitRelations::start()` | PLAN-38 Complete |
| 6 | `EqualityDetails::start()` | PLAN-39 Complete |
| 7 | `KindPredicatesRevisited::start()` | PLAN-40 Complete |
| 8 | `ImperativeDefinitionFamilies::create()` | PLAN-46/47/48 Complete |
| 9-12 | AdjectivesByPhrase/Condition/InterFunction/InterCondition | PLAN-42/43/44/45 Complete |

### 2. Why AdjectivalPredicates over other candidates?

- **Startup order**: It's literally the first call in `AssertionsModule::start()` (line 26). Every other startup item is already implemented.
- **Foundation**: The adjectival predicate family is used by the assertion pipeline when processing adjective atoms in propositions. Without it, adjectives cannot be used in the calculus.
- **Depends on existing infrastructure**: The adjective meaning system (PLAN-28/29/30/31/32/33/34), the unary predicate family system (PLAN-16), and the unary predicate system (PLAN-15) are all complete. The adjectival predicates module is the final piece that connects adjectives to the calculus.
- **Small scope**: At ~165 lines of C, this is a compact, well-defined module. It creates one family with seven methods and six helper functions.

### 3. Is it independently testable?

**Yes.** The foundation consists of:
- Creating the `adjectival_predicates` module with the `AdjectivalPredicates` struct
- Implementing `AdjectivalPredicates::start()` which creates one `UpFamily` with seven methods wired
- Implementing helper functions (`new_up`, `new_atom`, `new_atom_on_x`, `to_adjective`, `parity`, `flip_parity`)
- Adding a `parity: bool` field to `UnaryPredicate` (default `true`)
- Testing that the family is created with correct methods, that helper functions work, and that parity tracking works

### 4. What is the smallest independently testable subset?

1. Add `parity: bool` field to `UnaryPredicate` (default `true`).
2. Create `AdjectivalPredicates::start()` returning one `&'static UpFamily` with seven methods wired.
3. Implement `AdjectivalPredicates::new_up(adj_idx, pos)` creating a predicate with correct parity.
4. Implement `AdjectivalPredicates::new_atom(adj_idx, negated, term)` creating a proposition atom.
5. Implement `AdjectivalPredicates::new_atom_on_x(adj_idx, negated)` creating an atom on variable 0.
6. Implement `AdjectivalPredicates::to_adjective(up)` extracting the adjective index from a predicate.
7. Implement `AdjectivalPredicates::parity(up)` returning the positive/negative sense.
8. Implement `AdjectivalPredicates::flip_parity(up)` reversing the sense.
9. Implement the seven methods with appropriate simplifications.
10. All existing tests continue to pass.

### 5. What simplifications are appropriate?

- **No `lcon`/`Stock` system.** The C uses `lcon` (logical constant) to encode both the adjective's stock index and the parity sense. We use a simple `parity: bool` field on `UnaryPredicate` and store the adjective index directly.
- **`typecheck` always returns `ALWAYS_MATCH` (1).** The full implementation uses `TypecheckPropositions::kind_of_term`, `AdjectiveAmbiguity::can_be_applied_to`, and `TypecheckPropositions::problem`. We defer the kind-checking logic.
- **`infer_kind` uses existing `AdjectiveAmbiguity::first_meaning` and `AdjectiveMeaningDomains::get_kind`.** This can be implemented with existing infrastructure.
- **`assert` returns `false` (not handled).** The full implementation uses `AdjectiveAmbiguity::assert`, `Assert::subject_of_term`, `Assert::spec_of_term`, and `Assert::issue_couldnt_problem`. We defer the assertion logic.
- **`testable` uses existing `AdjectiveAmbiguity::has_either_or_property_meaning`.** This can be implemented with existing infrastructure.
- **`test` returns `false`.** The full implementation uses `Properties::get_possession_marker`. We defer the compile-time testing logic.
- **`schema` returns `false`.** The full implementation uses `AdjectiveAmbiguity::schema_for_task`. We defer the schema compilation logic.
- **`log` uses existing `Adjectives::get_nominative_singular` and parity.** This can be implemented with existing infrastructure.
- **No `wording` type.** Uses `&'static str` for adjective names.
- **No problem messages.** No `StandardProblems` calls for typecheck failures.
- **No `parse_node` handling.** Source-location and wording manipulation is deferred.
- **No `inference_subject` handling.** The `assert` method takes simplified parameters.

## Tasks

### Task 1: Add `parity` field to `UnaryPredicate`

Edit `crates/conform7-semantics/src/calculus/unary_predicates.rs`.

**1a. Add `parity` field to the struct (after `calling_name`, around line 19):**

```rust
    /// Calling name (for calling predicates only).
    pub calling_name: Option<&'static str>,
    /// Parity — whether this predicate is in positive sense (true) or
    /// negative sense (false).
    ///
    /// Used by the adjectival predicate family to track whether the
    /// adjective is applied positively (e.g., "open") or negatively
    /// (e.g., "not open").
    ///
    /// Corresponds to the sense stored in `lcon` in the C reference
    /// (`services/calculus-module/Chapter 2/Unary Predicates.w`).
    /// Simplified: a bool instead of Lcon sense.
    pub parity: bool,
```

**1b. Update `new` to initialize `parity: true` (around line 30):**

```rust
    pub fn new(family: &'static UpFamily) -> Self {
        UnaryPredicate {
            family,
            assert_kind: None,
            composited: false,
            unarticled: false,
            calling_name: None,
            parity: true,
        }
    }
```

**1c. Update `copy` to include `parity` (line 42):**

The `copy` method uses `self.clone()`, which will automatically include the new field since `Clone` is derived. No change needed.

### Task 2: Create the `AdjectivalPredicates` module

Create `crates/conform7-semantics/src/calculus/adjectival_predicates.rs`.

Module-level doc comment:

```rust
//! The Adjectival Predicates system — a unary predicate family for adjectives.
//!
//! Corresponds to `AdjectivalPredicates` in the C reference
//! (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`).
//!
//! Creates a single unary predicate family (`adjectival_up_family`) with seven
//! methods:
//!
//! - `typecheck` — check if the adjective can be applied to the term's kind
//! - `infer_kind` — infer the kind from the adjective's first meaning's domain
//! - `assert` — assert the adjective as a true fact about the model world
//! - `testable` — whether the adjective can be tested at compile-time
//! - `test` — test the adjective at compile-time
//! - `schema` — compile run-time code for the adjective
//! - `log` — log the adjective for debugging
//!
//! Also provides helper functions:
//! - `new_up` — create a new adjectival unary predicate
//! - `new_atom` — create a proposition atom from an adjective
//! - `new_atom_on_x` — create an atom on variable 0
//! - `to_adjective` — extract the adjective index from a predicate
//! - `parity` — get the positive/negative sense
//! - `flip_parity` — reverse the sense
//!
//! Simplified:
//! - No `lcon`/`Stock` system — uses a `parity: bool` field on `UnaryPredicate`
//!   and stores the adjective index directly in `assert_kind`.
//! - `typecheck` always returns `ALWAYS_MATCH` (1) — full kind-checking deferred.
//! - `assert` returns `false` (not handled) — full assertion logic deferred.
//! - `test` returns `false` — compile-time testing deferred.
//! - `schema` returns `false` — schema compilation deferred.
//! - No `wording` type — uses `&'static str` for adjective names.
//! - No problem messages.
//! - No `parse_node` or `inference_subject` handling.
```

Imports:

```rust
use crate::calculus::atoms::{AtomElement, PcalcProp};
use crate::calculus::terms::PcalcTerm;
use crate::calculus::unary_predicate_families::{UpFamily, UpFamilyMethods};
use crate::calculus::unary_predicates::UnaryPredicate;
use crate::knowledge::adjectives::{Adjective, AdjectiveAmbiguity, AdjectiveMeaningDomains, Adjectives};
```

The module struct:

```rust
/// The Adjectival Predicates module.
///
/// Corresponds to `AdjectivalPredicates` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`).
pub struct AdjectivalPredicates;
```

Global static for the family:

```rust
use std::sync::LazyLock;

/// The adjectival unary predicate family.
///
/// Corresponds to `adjectival_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, line 8).
///
/// This family handles predicates derived from linguistic adjectives
/// (e.g., "open", "closed", "empty", "red").
pub static ADJECTIVAL_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "adjectival",
        UpFamilyMethods {
            log: AdjectivalPredicates::log,
            infer_kind: AdjectivalPredicates::infer_kind,
            testable: AdjectivalPredicates::testable,
            test: AdjectivalPredicates::test,
            typecheck: Some(AdjectivalPredicates::typecheck),
            assert: Some(AdjectivalPredicates::assert),
            schema: Some(AdjectivalPredicates::schema),
        },
    )
});
```

Implementation:

```rust
impl AdjectivalPredicates {
    /// Create the adjectival predicate family.
    ///
    /// Corresponds to `AdjectivalPredicates::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 13-22).
    ///
    /// Returns a reference to the singleton `ADJECTIVAL_UP_FAMILY`.
    pub fn start() -> &'static UpFamily {
        &ADJECTIVAL_UP_FAMILY
    }

    /// Create a new adjectival unary predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::new_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 24-30).
    ///
    /// `adj_idx` is the index of the adjective in the adjective registry.
    /// `pos` is `true` for positive sense, `false` for negative sense.
    ///
    /// Simplified: stores the adjective index in `assert_kind` as a string
    /// representation, and parity in the `parity` field, instead of using
    /// the `lcon`/`Stock` system.
    pub fn new_up(adj_idx: usize, pos: bool) -> UnaryPredicate {
        let mut up = UnaryPredicate::new(&ADJECTIVAL_UP_FAMILY);
        up.assert_kind = Some(Box::leak(format!("adj:{}", adj_idx).into_boxed_str()));
        up.parity = pos;
        up
    }

    /// Create a proposition atom from an adjective and term.
    ///
    /// Corresponds to `AdjectivalPredicates::new_atom` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 32-35).
    ///
    /// `negated` is `true` to create a negative-sense atom (equivalent to
    /// "not adjective"), `false` for positive sense.
    pub fn new_atom(adj_idx: usize, negated: bool, term: PcalcTerm) -> PcalcProp {
        let up = Self::new_up(adj_idx, !negated);
        PcalcProp::unary_predicate_new_from_up(up, term)
    }

    /// Create an adjectival atom on variable 0 (the default bound variable).
    ///
    /// Corresponds to `AdjectivalPredicates::new_atom_on_x` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 37-39).
    pub fn new_atom_on_x(adj_idx: usize, negated: bool) -> PcalcProp {
        Self::new_atom(adj_idx, negated, PcalcTerm::new_variable(0))
    }

    /// Extract the adjective index from a unary predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::to_adjective` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 140-144).
    ///
    /// Returns `None` if the predicate is not from the adjectival family.
    ///
    /// Simplified: extracts the index from the `assert_kind` string
    /// (format: `"adj:<index>"`) instead of using `lcon`/`Stock`.
    pub fn to_adjective(up: &UnaryPredicate) -> Option<usize> {
        if up.family.name != "adjectival" {
            return None;
        }
        up.assert_kind.and_then(|s| {
            s.strip_prefix("adj:").and_then(|n| n.parse::<usize>().ok())
        })
    }

    /// Get the parity (positive/negative sense) of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::parity` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 146-151).
    ///
    /// Returns `true` for positive sense, `false` for negative sense.
    /// If the predicate is not from the adjectival family, returns `true`
    /// (default positive sense, matching the C behavior).
    pub fn parity(up: &UnaryPredicate) -> bool {
        if up.family.name != "adjectival" {
            return true;
        }
        up.parity
    }

    /// Flip the parity of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::flip_parity` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 157-164).
    ///
    /// Does nothing if the predicate is not from the adjectival family.
    pub fn flip_parity(up: &mut UnaryPredicate) {
        if up.family.name == "adjectival" {
            up.parity = !up.parity;
        }
    }

    // -----------------------------------------------------------------------
    // Family methods
    // -----------------------------------------------------------------------

    /// Log an adjectival predicate to the debug log.
    ///
    /// Corresponds to `AdjectivalPredicates::log` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 41-45).
    ///
    /// Simplified: uses `Adjectives::get_nominative_singular` and the
    /// adjective registry to format the output.
    fn log(_family: &UpFamily, up: &UnaryPredicate) -> String {
        let prefix = if Self::parity(up) { "" } else { "not-" };
        match Self::to_adjective(up) {
            Some(adj_idx) => {
                // We need access to the adjective registry. Since this is a
                // static method without registry access, we use the
                // assert_kind string as a fallback.
                format!("{}{}", prefix, up.assert_kind.unwrap_or("?"))
            }
            None => format!("{}?", prefix),
        }
    }

    /// Infer the kind from an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::infer_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 47-52).
    ///
    /// Simplified: returns `None` since we don't have access to the
    /// adjective registry from this static context. The full implementation
    /// will use `AdjectiveAmbiguity::first_meaning` and
    /// `AdjectiveMeaningDomains::get_kind`.
    fn infer_kind(_family: &UpFamily, _up: &UnaryPredicate) -> Option<&'static str> {
        // Deferred: requires access to the adjective registry and
        // AdjectiveAmbiguity::first_meaning.
        None
    }

    /// Whether an adjectival predicate can be tested at compile-time.
    ///
    /// Corresponds to `AdjectivalPredicates::testable` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 104-109).
    ///
    /// Simplified: returns `false`. The full implementation will use
    /// `AdjectiveAmbiguity::has_either_or_property_meaning`.
    fn testable(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
        // Deferred: requires access to the adjective registry and
        // AdjectiveAmbiguity::has_either_or_property_meaning.
        false
    }

    /// Test an adjectival predicate at compile-time.
    ///
    /// Corresponds to `AdjectivalPredicates::test` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 111-126).
    ///
    /// Simplified: returns `false`. The full implementation will use
    /// `Properties::get_possession_marker`.
    fn test(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
        // Deferred: requires access to the property system and
        // possession markers.
        false
    }

    /// Typecheck the terms of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 54-66).
    ///
    /// Simplified: always returns `1` (ALWAYS_MATCH). The full implementation
    /// will use `TypecheckPropositions::kind_of_term` and
    /// `AdjectiveAmbiguity::can_be_applied_to`.
    fn typecheck(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        // Deferred: full kind-checking logic.
        1 // ALWAYS_MATCH
    }

    /// Assert an adjectival predicate as a true fact about the model world.
    ///
    /// Corresponds to `AdjectivalPredicates::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 75-102).
    ///
    /// Simplified: returns `false` (not handled). The full implementation
    /// will use `AdjectiveAmbiguity::assert`, `Assert::subject_of_term`,
    /// and `Assert::spec_of_term`.
    fn assert(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _now_negated: bool,
        _prop: &PcalcProp,
    ) -> bool {
        // Deferred: full assertion logic.
        false
    }

    /// Compile run-time code for an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::get_schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 128-135).
    ///
    /// Simplified: returns `false` (not handled). The full implementation
    /// will use `AdjectiveAmbiguity::schema_for_task`.
    fn schema(
        _family: &UpFamily,
        _task: u8,
        _up: &UnaryPredicate,
    ) -> bool {
        // Deferred: full schema compilation logic.
        false
    }
}
```

**Note on `PcalcProp::unary_predicate_new_from_up`**: The existing `PcalcProp` API uses `unary_predicate_new` which takes a `&'static str` predicate name. We need a variant that takes an owned `UnaryPredicate`. Check the existing API and add if needed. If the existing API only supports string-based predicate references, we may need to add a `UnaryPredicate` variant to `PredicateRef` or use a different approach. The implementer should verify the `PcalcProp` API and adapt accordingly.

### Task 3: Wire the module into the calculus module

Edit `crates/conform7-semantics/src/calculus/mod.rs`.

**3a. Add the module declaration (after `pub mod creation_predicates;`, line 41):**

```rust
pub mod adjectival_predicates;
```

**3b. Add a module-map row in the `# Module Map` table (after the creation_predicates row):**

```
| [`adjectival_predicates`] | `inform7/assertions-module/Chapter 8/The Adjectival Predicates.w` | Adjectival unary predicate family |
```

**3c. Add a reference line in the `# References` list:**

```
//! - C reference: `inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`
```

### Task 4: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/calculus/adjectival_predicates.rs`.

Required imports in tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::unary_predicate_families::UpFamily;
    use crate::calculus::unary_predicates::UnaryPredicate;
    use crate::calculus::terms::PcalcTerm;
    use crate::calculus::atoms::PcalcProp;
}
```

**Test 1: `start_creates_family_with_correct_name`**

```rust
/// Test that `start()` returns a family with the correct name.
#[test]
fn start_creates_family_with_correct_name() {
    let family = AdjectivalPredicates::start();
    assert_eq!(family.name, "adjectival");
}
```

**Test 2: `start_creates_family_with_all_seven_methods`**

```rust
/// Test that the family has all seven methods wired.
#[test]
fn start_creates_family_with_all_seven_methods() {
    let family = AdjectivalPredicates::start();
    // Required methods
    let up = UnaryPredicate::new(family);
    family.methods.log(family, &up);
    family.methods.infer_kind(family, &up);
    family.methods.testable(family, &up);
    family.methods.test(family, &up);
    // Optional methods
    assert!(family.methods.typecheck.is_some(), "typecheck should be installed");
    assert!(family.methods.assert.is_some(), "assert should be installed");
    assert!(family.methods.schema.is_some(), "schema should be installed");
}
```

**Test 3: `new_up_creates_predicate_with_correct_family`**

```rust
/// Test that `new_up` creates a predicate belonging to the adjectival family.
#[test]
fn new_up_creates_predicate_with_correct_family() {
    let up = AdjectivalPredicates::new_up(0, true);
    assert_eq!(up.family.name, "adjectival");
}
```

**Test 4: `new_up_sets_parity_correctly`**

```rust
/// Test that `new_up` sets parity correctly for positive and negative sense.
#[test]
fn new_up_sets_parity_correctly() {
    let pos = AdjectivalPredicates::new_up(0, true);
    assert!(pos.parity, "positive sense should have parity=true");

    let neg = AdjectivalPredicates::new_up(0, false);
    assert!(!neg.parity, "negative sense should have parity=false");
}
```

**Test 5: `new_up_stores_adjective_index`**

```rust
/// Test that `new_up` stores the adjective index correctly.
#[test]
fn new_up_stores_adjective_index() {
    let up = AdjectivalPredicates::new_up(42, true);
    let idx = AdjectivalPredicates::to_adjective(&up);
    assert_eq!(idx, Some(42));
}
```

**Test 6: `to_adjective_returns_none_for_non_adjectival`**

```rust
/// Test that `to_adjective` returns `None` for non-adjectival predicates.
#[test]
fn to_adjective_returns_none_for_non_adjectival() {
    // Create a predicate from a different family
    let other_family = UpFamily::new("other", UpFamilyMethods::default());
    let up = UnaryPredicate::new(&other_family);
    assert_eq!(AdjectivalPredicates::to_adjective(&up), None);
}
```

**Test 7: `parity_returns_true_for_non_adjectival`**

```rust
/// Test that `parity` returns `true` (default positive) for non-adjectival predicates.
#[test]
fn parity_returns_true_for_non_adjectival() {
    let other_family = UpFamily::new("other", UpFamilyMethods::default());
    let up = UnaryPredicate::new(&other_family);
    assert!(AdjectivalPredicates::parity(&up));
}
```

**Test 8: `flip_parity_reverses_sense`**

```rust
/// Test that `flip_parity` reverses the sense of an adjectival predicate.
#[test]
fn flip_parity_reverses_sense() {
    let mut up = AdjectivalPredicates::new_up(0, true);
    assert!(up.parity);

    AdjectivalPredicates::flip_parity(&mut up);
    assert!(!up.parity, "flip_parity should reverse positive to negative");

    AdjectivalPredicates::flip_parity(&mut up);
    assert!(up.parity, "flip_parity should reverse negative back to positive");
}
```

**Test 9: `flip_parity_does_nothing_for_non_adjectival`**

```rust
/// Test that `flip_parity` does nothing for non-adjectival predicates.
#[test]
fn flip_parity_does_nothing_for_non_adjectival() {
    let other_family = UpFamily::new("other", UpFamilyMethods::default());
    let mut up = UnaryPredicate::new(&other_family);
    up.parity = false;

    AdjectivalPredicates::flip_parity(&mut up);
    assert!(!up.parity, "flip_parity should not change non-adjectival predicates");
}
```

**Test 10: `new_atom_creates_proposition_with_correct_parity`**

```rust
/// Test that `new_atom` creates a proposition atom with the correct parity.
#[test]
fn new_atom_creates_proposition_with_correct_parity() {
    let term = PcalcTerm::new_variable(0);
    let prop = AdjectivalPredicates::new_atom(0, false, term);
    // The atom should have the adjectival predicate
    assert_eq!(prop.element, AtomElement::Predicate);
    assert_eq!(prop.arity, 1);
}
```

**Test 11: `new_atom_on_x_creates_atom_on_variable_0`**

```rust
/// Test that `new_atom_on_x` creates an atom on variable 0.
#[test]
fn new_atom_on_x_creates_atom_on_variable_0() {
    let prop = AdjectivalPredicates::new_atom_on_x(0, false);
    assert_eq!(prop.element, AtomElement::Predicate);
    assert_eq!(prop.arity, 1);
    // The term should be variable 0
    if let Some(ref term) = prop.terms.first() {
        assert!(term.is_variable());
        assert_eq!(term.variable_index(), Some(0));
    }
}
```

**Test 12: `typecheck_returns_always_match`**

```rust
/// Test that `typecheck` returns 1 (ALWAYS_MATCH) as a simplified stub.
#[test]
fn typecheck_returns_always_match() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let typecheck_fn = family.methods.typecheck.unwrap();
    let result = typecheck_fn(family, &up, &[], &[]);
    assert_eq!(result, 1, "typecheck should return ALWAYS_MATCH (1)");
}
```

**Test 13: `assert_returns_false`**

```rust
/// Test that `assert` returns `false` as a simplified stub.
#[test]
fn assert_returns_false() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let prop = PcalcProp::new_empty();
    let assert_fn = family.methods.assert.unwrap();
    let result = assert_fn(family, &up, false, &prop);
    assert!(!result, "assert should return false (not handled)");
}
```

**Test 14: `schema_returns_false`**

```rust
/// Test that `schema` returns `false` as a simplified stub.
#[test]
fn schema_returns_false() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let schema_fn = family.methods.schema.unwrap();
    let result = schema_fn(family, 0, &up);
    assert!(!result, "schema should return false (not handled)");
}
```

**Test 15: `testable_returns_false`**

```rust
/// Test that `testable` returns `false` as a simplified stub.
#[test]
fn testable_returns_false() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let result = family.methods.testable(family, &up);
    assert!(!result, "testable should return false (not handled)");
}
```

**Test 16: `test_returns_false`**

```rust
/// Test that `test` returns `false` as a simplified stub.
#[test]
fn test_returns_false() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let result = family.methods.test(family, &up);
    assert!(!result, "test should return false (not handled)");
}
```

**Test 17: `infer_kind_returns_none`**

```rust
/// Test that `infer_kind` returns `None` as a simplified stub.
#[test]
fn infer_kind_returns_none() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let result = family.methods.infer_kind(family, &up);
    assert_eq!(result, None, "infer_kind should return None (not handled)");
}
```

**Test 18: `log_returns_string`**

```rust
/// Test that `log` returns a non-empty string.
#[test]
fn log_returns_string() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, true);
    let result = family.methods.log(family, &up);
    assert!(!result.is_empty(), "log should return a non-empty string");
}
```

**Test 19: `log_negative_parity_includes_not_prefix`**

```rust
/// Test that `log` includes "not-" prefix for negative parity.
#[test]
fn log_negative_parity_includes_not_prefix() {
    let family = AdjectivalPredicates::start();
    let up = AdjectivalPredicates::new_up(0, false);
    let result = family.methods.log(family, &up);
    assert!(result.starts_with("not-"), "negative parity log should start with 'not-'");
}
```

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test -- calculus::adjectival_predicates` — new tests pass
- [ ] `cargo test` — all existing tests still pass
- [ ] `cargo clippy --all-targets` — no new warnings (pre-existing warnings in unrelated files are acceptable)

## Success Criteria

- [ ] `calculus::adjectival_predicates` module exists and compiles.
- [ ] `AdjectivalPredicates::start()` returns a `&'static UpFamily` with name `"adjectival"`.
- [ ] The family has all seven methods wired: `log`, `infer_kind`, `testable`, `test`, `typecheck`, `assert`, `schema`.
- [ ] `new_up(adj_idx, pos)` creates a `UnaryPredicate` with correct family, parity, and adjective index.
- [ ] `to_adjective(up)` extracts the adjective index from an adjectival predicate.
- [ ] `parity(up)` returns the correct sense for adjectival predicates, `true` for non-adjectival.
- [ ] `flip_parity(up)` reverses the sense for adjectival predicates, does nothing for non-adjectival.
- [ ] `new_atom(adj_idx, negated, term)` creates a `PcalcProp` atom with correct parity.
- [ ] `new_atom_on_x(adj_idx, negated)` creates an atom on variable 0.
- [ ] `typecheck` returns `1` (ALWAYS_MATCH) — simplified stub.
- [ ] `assert` returns `false` — simplified stub.
- [ ] `schema` returns `false` — simplified stub.
- [ ] `testable` returns `false` — simplified stub.
- [ ] `test` returns `false` — simplified stub.
- [ ] `infer_kind` returns `None` — simplified stub.
- [ ] `log` returns a non-empty string with "not-" prefix for negative parity.
- [ ] `parity: bool` field added to `UnaryPredicate` with default `true`.
- [ ] `cargo clippy --all-targets` introduces no new warnings.
- [ ] All existing tests still pass.

## Out of Scope

- **Full `typecheck` logic.** `TypecheckPropositions::kind_of_term`, `AdjectiveAmbiguity::can_be_applied_to`, and `TypecheckPropositions::problem` (C lines 54-66) are deferred.
- **Full `infer_kind` logic.** `AdjectiveAmbiguity::first_meaning` and `AdjectiveMeaningDomains::get_kind` integration (C lines 47-52) are deferred.
- **Full `assert` logic.** `AdjectiveAmbiguity::assert`, `Assert::subject_of_term`, `Assert::spec_of_term`, `Assert::issue_couldnt_problem` (C lines 75-102) are deferred.
- **Full `testable` logic.** `AdjectiveAmbiguity::has_either_or_property_meaning` integration (C lines 104-109) is deferred.
- **Full `test` logic.** `Properties::get_possession_marker` and compile-time testing (C lines 111-126) are deferred.
- **Full `schema` logic.** `AdjectiveAmbiguity::schema_for_task` and I6 schema compilation (C lines 128-135) are deferred.
- **Full `log` logic.** `Adjectives::get_nominative_singular` with adjective registry access (C lines 41-45) is simplified.
- **`lcon`/`Stock` system.** The C uses `lcon` (logical constant) to encode both stock index and parity. We use a simple `parity: bool` field and store the adjective index in `assert_kind`.
- **`wording` type.** No `PreformUtilities::wording` — uses `&'static str` for names.
- **Problem messages.** No `StandardProblems` calls for typecheck failures.
- **`parse_node` handling.** Source-location and wording manipulation is deferred.
- **`inference_subject` handling.** The `assert` method takes simplified parameters.
- **Adjective registry access.** The `log` and `infer_kind` methods don't have access to the adjective registry from their static context. Full implementation will need to pass registry references.
- **Other assertions-module startup items.** All other startup items (CreationPredicates, QuasinumericRelations, Universal, ExplicitRelations, EqualityDetails, KindPredicatesRevisited, ImperativeDefinitionFamilies, AdjectivesByPhrase/Condition/InterFunction/InterCondition) are already complete.
