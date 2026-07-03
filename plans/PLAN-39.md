# Plan 39: Equality Details — Typechecking, Assertion, and Schema for the Equality and Empty Families

**Status**: Complete
**Target**: 1 day

## Goal

Implement the Equality Details system — adding typecheck, assert, and schema methods to the equality and empty binary predicate families created by `EqualityRelation::start()` (PLAN-22, Complete). This creates the `EqualityDetails` module with simplified typechecking (accept any kinds), assertion (no-op), and schema compilation (no-op) for the equality and never-holding relations.

This is the next module in the assertions-module startup sequence
(`inform7/assertions-module/Chapter 1/Assertions Module.w`, lines 25-37):
`ExplicitRelations::start()` (PLAN-38, Complete) is followed by
`EqualityDetails::start()` at line 31. It depends only on the binary
predicate infrastructure already built and is independently testable
without grammar parsing, the knowledge module, or run-time compilation.

## Background

### C reference architecture

The Equality Details system adds methods to the families created by
`EqualityRelation::start()` (PLAN-22):

- `inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w` —
  `EqualityDetails::start`, `typecheck`, `assert`, `schema`,
  `typecheck_empty`, `assert_empty`, `schema_empty` (~340 lines).
- `services/calculus-module/Chapter 3/The Equality Relation.w` — creates
  `equality_bp_family` (EQUALITY_FAMILY = 0), `spatial_bp_family`
  (SPATIAL_FAMILY = 1), and `empty_bp_family` (EMPTY_FAMILY = 2).
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` —
  `BpFamily`, `BpFamilyMethods`, method dispatch.

In the C reference:

```c
void EqualityDetails::start(void) {
    METHOD_ADD(equality_bp_family, TYPECHECK_BPF_MTID, EqualityDetails::typecheck);
    METHOD_ADD(equality_bp_family, ASSERT_BPF_MTID, EqualityDetails::assert);
    METHOD_ADD(equality_bp_family, SCHEMA_BPF_MTID, EqualityDetails::schema);

    METHOD_ADD(empty_bp_family, TYPECHECK_BPF_MTID, EqualityDetails::typecheck_empty);
    METHOD_ADD(empty_bp_family, ASSERT_BPF_MTID, EqualityDetails::assert_empty);
    METHOD_ADD(empty_bp_family, SCHEMA_BPF_MTID, EqualityDetails::schema_empty);
}
```

The full `typecheck` method is polymorphic and handles text/topic mismatches,
object/value comparisons, rule/rulebook/activity comparisons, and kind
compatibility. The full `assert` method can set global variables. The full
`schema` method compiles equality to I6 code. The empty-family methods are
no-ops.

### Current Rust state

- `crates/conform7-semantics/src/calculus/equality_relation.rs` —
  `EqualityRelation::start()` returns `(Vec<BpFamily>, Vec<BinaryPredicate>)`
  with the equality, spatial, and empty families.
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` —
  `BpFamilyMethods` provides `typecheck`, `assert`, `schema`, and dispatch
  helpers.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` —
  `BinaryPredicate`, `BinaryPredicates`, `BinaryPredicateFamilies::first_stock()`.

## What's needed

1. **`EqualityDetails` module** — a new module
   `crates/conform7-semantics/src/calculus/equality_details.rs` with:
   - `EqualityDetails::start(&mut [BpFamily])` — wire the six methods into the
     equality and empty families.
   - `EqualityDetails::typecheck(...)` — simplified: return `1` (`ALWAYS_MATCH`).
   - `EqualityDetails::assert(...)` — simplified: return `false`.
   - `EqualityDetails::schema(...)` — simplified: return `false`.
   - `EqualityDetails::typecheck_empty(...)` — return `1` (`ALWAYS_MATCH`).
   - `EqualityDetails::assert_empty(...)` — return `false`.
   - `EqualityDetails::schema_empty(...)` — return `false`.

2. **Integration** — declare `pub mod equality_details;` in
   `crates/conform7-semantics/src/calculus/mod.rs` and update the module map.

3. **Unit tests** — verify that `start()` wires methods to the correct
   families, and that each method returns the expected simplified value,
   both directly and via `BinaryPredicateFamilies` dispatch.

## Tasks

### 1. Create the `EqualityDetails` module

Create `crates/conform7-semantics/src/calculus/equality_details.rs`:

```rust
/// The Equality Details system — typechecking, assertion, and schema for
/// the equality and empty binary predicate families.
///
/// Corresponds to `EqualityDetails` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`).
///
/// Adds methods to the families created by `EqualityRelation::start()` (PLAN-22):
/// - equality_bp_family (EQUALITY_FAMILY = 0) — gets typecheck, assert, schema
/// - empty_bp_family (EMPTY_FAMILY = 2) — gets typecheck_empty, assert_empty, schema_empty
///
/// The spatial family (SPATIAL_FAMILY = 1) does not get methods here.
///
/// Simplified:
/// - No `StandardProblems::tcp_problem` (no problem messages)
/// - No `PluginCalls::typecheck_equality` (no plugin typechecking)
/// - No `Properties::can_name_coincide_with_kind` (no property name checking)
/// - No `Kinds::Behaviour::is_object` (no object kind checking)
/// - No `Kinds::compatible` (no kind compatibility checking)
/// - No `Lvalues::is_actual_NONLOCAL_VARIABLE` (no variable assignment)
/// - No `PropertyInferences::draw` (no property inference drawing)
/// - No `Calculus::Schemas::modify` (no schema modification)
/// - No `CompileLvalues::interpret_store` (no I6 store compilation)
/// - No `Kinds::get_construct` (no kind constructor checking)
/// - No `Cinders::kind_of_term` (no term kind resolution)
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::BinaryPredicate;
use crate::calculus::equality_relation::{EMPTY_FAMILY, EQUALITY_FAMILY};

/// The equality details module.
///
/// Corresponds to `EqualityDetails` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`).
pub struct EqualityDetails;

impl EqualityDetails {
    /// Add typecheck, assert, and schema methods to the equality and empty families.
    ///
    /// Corresponds to `EqualityDetails::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 8-16).
    ///
    /// # Arguments
    ///
    /// * `families` - The family registry containing the equality and empty families.
    pub fn start(families: &mut [BpFamily]) {
        // Add methods to the equality family
        if let Some(equality_family) = families.get_mut(EQUALITY_FAMILY) {
            equality_family.methods.typecheck = Some(EqualityDetails::typecheck);
            equality_family.methods.assert = Some(EqualityDetails::assert);
            equality_family.methods.schema = Some(EqualityDetails::schema);
        }

        // Add methods to the empty family
        if let Some(empty_family) = families.get_mut(EMPTY_FAMILY) {
            empty_family.methods.typecheck = Some(EqualityDetails::typecheck_empty);
            empty_family.methods.assert = Some(EqualityDetails::assert_empty);
            empty_family.methods.schema = Some(EqualityDetails::schema_empty);
        }
    }

    /// Typecheck the terms of an equality relation.
    ///
    /// Corresponds to `EqualityDetails::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 23-53).
    ///
    /// Simplified: returns `1` (`ALWAYS_MATCH`), accepting any kinds.
    /// The full implementation handles:
    /// - Text vs. topic mismatch (`NEVER_MATCH`)
    /// - Plugin typechecking
    /// - Object vs. value with coinciding property name
    /// - Understanding vs. snippet
    /// - Text vs. response
    /// - Kind compatibility checks
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        1 // ALWAYS_MATCH
    }

    /// Assert an equality relation.
    ///
    /// Corresponds to `EqualityDetails::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 114-134).
    ///
    /// Simplified: returns `false`. The full implementation handles:
    /// - Setting global variables via `PropertyInferences::draw`
    /// - Checking prevailing mood (`CERTAIN_CE`, `LIKELY_CE`, etc.)
    /// - Checking variable constness
    #[allow(clippy::too_many_arguments)]
    pub fn assert(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _infs0: usize,
        _spec0: Option<&'static str>,
        _infs1: usize,
        _spec1: Option<&'static str>,
        _subjects: &mut [crate::knowledge::inference_subjects::InferenceSubject],
        _permissions: &mut Vec<crate::knowledge::property_permissions::PropertyPermission>,
        _inference_families: &[crate::knowledge::inferences::InferenceFamily],
        _inferences: &mut Vec<crate::knowledge::inferences::Inference>,
        _property_inferences: &mut Vec<crate::knowledge::property_inferences::PropertyInferenceData>,
        _constructors: &[()],
    ) -> bool {
        false
    }

    /// Compile run-time code for an equality relation.
    ///
    /// Corresponds to `EqualityDetails::schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 177-223).
    ///
    /// Simplified: returns `false`. The full implementation handles:
    /// - Property-based equality (e.g., "if the lantern is bright")
    /// - Response text equality
    /// - `TEST_ATOM_TASK` (comparison)
    /// - `NOW_ATOM_TRUE_TASK` (assignment)
    /// - Kind-checking code for run-time
    pub fn schema(
        _family: &BpFamily,
        _task: u8,
        _bp: &BinaryPredicate,
    ) -> bool {
        false
    }

    /// Typecheck the terms of the never-holding (empty) relation.
    ///
    /// Corresponds to `EqualityDetails::typecheck_empty` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 104-107).
    ///
    /// Returns `1` (`ALWAYS_MATCH`) — anything can be hypothetically related to
    /// anything else.
    pub fn typecheck_empty(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        1 // ALWAYS_MATCH
    }

    /// Assert the never-holding (empty) relation.
    ///
    /// Corresponds to `EqualityDetails::assert_empty` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 139-143).
    ///
    /// The never-holding relation cannot be asserted true. Returns `false`.
    #[allow(clippy::too_many_arguments)]
    pub fn assert_empty(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _infs0: usize,
        _spec0: Option<&'static str>,
        _infs1: usize,
        _spec1: Option<&'static str>,
        _subjects: &mut [crate::knowledge::inference_subjects::InferenceSubject],
        _permissions: &mut Vec<crate::knowledge::property_permissions::PropertyPermission>,
        _inference_families: &[crate::knowledge::inferences::InferenceFamily],
        _inferences: &mut Vec<crate::knowledge::inferences::Inference>,
        _property_inferences: &mut Vec<crate::knowledge::property_inferences::PropertyInferenceData>,
        _constructors: &[()],
    ) -> bool {
        false
    }

    /// Compile run-time code for the never-holding (empty) relation.
    ///
    /// Corresponds to `EqualityDetails::schema_empty` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`, lines 336-339).
    ///
    /// The never-holding relation has nothing to compile. Returns `false`.
    pub fn schema_empty(
        _family: &BpFamily,
        _task: u8,
        _bp: &BinaryPredicate,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;
    use crate::calculus::equality_relation::{EqualityRelation, R_EMPTY, R_EQUALITY, SPATIAL_FAMILY};

    /// Helper: create the equality families and stock the BP registry.
    fn setup_stocked() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        (families, bp_registry)
    }

    /// Test that `start()` adds methods to the equality family.
    #[test]
    fn test_start_adds_equality_methods() {
        let (mut families, _bp_registry) = EqualityRelation::start();

        assert!(families[EQUALITY_FAMILY].methods.typecheck.is_none());
        assert!(families[EQUALITY_FAMILY].methods.assert.is_none());
        assert!(families[EQUALITY_FAMILY].methods.schema.is_none());

        EqualityDetails::start(&mut families);

        assert!(families[EQUALITY_FAMILY].methods.typecheck.is_some());
        assert!(families[EQUALITY_FAMILY].methods.assert.is_some());
        assert!(families[EQUALITY_FAMILY].methods.schema.is_some());
    }

    /// Test that `start()` adds methods to the empty family.
    #[test]
    fn test_start_adds_empty_methods() {
        let (mut families, _bp_registry) = EqualityRelation::start();

        assert!(families[EMPTY_FAMILY].methods.typecheck.is_none());
        assert!(families[EMPTY_FAMILY].methods.assert.is_none());
        assert!(families[EMPTY_FAMILY].methods.schema.is_none());

        EqualityDetails::start(&mut families);

        assert!(families[EMPTY_FAMILY].methods.typecheck.is_some());
        assert!(families[EMPTY_FAMILY].methods.assert.is_some());
        assert!(families[EMPTY_FAMILY].methods.schema.is_some());
    }

    /// Test that `start()` does NOT add methods to the spatial family.
    #[test]
    fn test_start_does_not_add_spatial_methods() {
        let (mut families, _bp_registry) = EqualityRelation::start();
        EqualityDetails::start(&mut families);

        assert!(families[SPATIAL_FAMILY].methods.typecheck.is_none());
        assert!(families[SPATIAL_FAMILY].methods.assert.is_none());
        assert!(families[SPATIAL_FAMILY].methods.schema.is_none());
    }

    /// Test that `typecheck` returns `ALWAYS_MATCH`.
    #[test]
    fn test_typecheck_returns_always_match() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[R_EQUALITY],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test that `assert` returns `false`.
    #[test]
    fn test_assert_returns_false() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::assert(
            &bp_registry[R_EQUALITY],
            0,
            None,
            0,
            None,
            &families,
            &mut [],
            &mut vec![],
            &[],
            &mut vec![],
            &mut vec![],
            &[],
        );
        assert!(!result);
    }

    /// Test that `schema` returns `false`.
    #[test]
    fn test_schema_returns_false() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[R_EQUALITY],
            &families,
        );
        assert!(!result);
    }

    /// Test that `typecheck_empty` returns `ALWAYS_MATCH`.
    #[test]
    fn test_typecheck_empty_returns_always_match() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[R_EMPTY],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test that `assert_empty` returns `false`.
    #[test]
    fn test_assert_empty_returns_false() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::assert(
            &bp_registry[R_EMPTY],
            0,
            None,
            0,
            None,
            &families,
            &mut [],
            &mut vec![],
            &[],
            &mut vec![],
            &mut vec![],
            &[],
        );
        assert!(!result);
    }

    /// Test that `schema_empty` returns `false`.
    #[test]
    fn test_schema_empty_returns_false() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);

        let result = BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[R_EMPTY],
            &families,
        );
        assert!(!result);
    }
}
```

### 2. Integrate with the calculus module

- Add `pub mod equality_details;` to
  `crates/conform7-semantics/src/calculus/mod.rs` after `equality_relation`
  (keep declarations alphabetical).
- Add a row for `equality_details` to the module map table in the same file.
- Add a C-reference line for `The Equality Relation Revisited.w` to the
  references list in the same file.

### 3. Verify

- Run `cargo build` to ensure the new module compiles.
- Run `cargo test --lib equality_details` (or `cargo test -- calculus::equality_details`)
  to verify all unit tests pass.
- Run `cargo clippy` to confirm the crate remains clean.

## Success Criteria

- [ ] `EqualityDetails::start()` adds typecheck, assert, and schema methods to the equality family.
- [ ] `EqualityDetails::start()` adds typecheck_empty, assert_empty, and schema_empty methods to the empty family.
- [ ] `EqualityDetails::start()` does NOT add methods to the spatial family.
- [ ] `EqualityDetails::typecheck()` returns `1` (`ALWAYS_MATCH`).
- [ ] `EqualityDetails::assert()` returns `false`.
- [ ] `EqualityDetails::schema()` returns `false`.
- [ ] `EqualityDetails::typecheck_empty()` returns `1` (`ALWAYS_MATCH`).
- [ ] `EqualityDetails::assert_empty()` returns `false`.
- [ ] `EqualityDetails::schema_empty()` returns `false`.
- [ ] The new module compiles and is wired into `calculus/mod.rs`.
- [ ] All unit tests pass and clippy remains clean.

## Out of Scope

- **Problem reporting**: `StandardProblems::tcp_problem` for text-as-topic,
  non-property comparison, and other typechecking errors.
- **Plugin typechecking**: `PluginCalls::typecheck_equality`.
- **Property integration**: `Properties::can_name_coincide_with_kind`,
  `Properties::property_with_same_name_as`.
- **Kind system integration**: `Kinds::Behaviour::is_object`,
  `Kinds::compatible`, `Kinds::get_construct`, `Kinds::dereference_properties`.
- **Lvalues/variables**: `Lvalues::is_actual_NONLOCAL_VARIABLE`,
  `NonlocalVariables::*`, variable assignment, constant-variable checks.
- **Property inferences**: `PropertyInferences::draw`.
- **Schema/I6 compilation**: `Calculus::Schemas::modify`,
  `CompileLvalues::interpret_store`, `EqualitySchemas::interpret_equality`,
  run-time kind checking.
- **Term kind resolution**: `Cinders::kind_of_term`.
- **Stock methods**: the equality and empty families already have stock
  methods from PLAN-22; no new stock methods are added.
- **Preform grammar / Salsa integration**.
