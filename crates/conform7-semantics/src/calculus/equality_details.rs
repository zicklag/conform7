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
use crate::calculus::binary_predicate_families::BpFamily;
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
        let (families, bp_registry) = setup_stocked();
        let result = EqualityDetails::typecheck(
            &families[EQUALITY_FAMILY],
            &bp_registry[R_EQUALITY],
            &[],
            &[],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test that `assert` returns `false`.
    #[test]
    fn test_assert_returns_false() {
        let (families, bp_registry) = setup_stocked();
        let _result = EqualityDetails::assert(
            &families[EQUALITY_FAMILY],
            &bp_registry[R_EQUALITY],
            0, None, 0, None,
            &mut [], &mut vec![], &[], &mut vec![], &mut vec![], &[],
        );
        assert!(!_result);
    }

    /// Test that `schema` returns `false`.
    #[test]
    fn test_schema_returns_false() {
        let (families, bp_registry) = setup_stocked();
        let result = EqualityDetails::schema(
            &families[EQUALITY_FAMILY],
            0,
            &bp_registry[R_EQUALITY],
        );
        assert!(!result);
    }

    /// Test that `typecheck_empty` returns `ALWAYS_MATCH`.
    #[test]
    fn test_typecheck_empty_returns_always_match() {
        let (families, bp_registry) = setup_stocked();
        let result = EqualityDetails::typecheck_empty(
            &families[EMPTY_FAMILY],
            &bp_registry[R_EMPTY],
            &[],
            &[],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test that `assert_empty` returns `false`.
    #[test]
    fn test_assert_empty_returns_false() {
        let (families, bp_registry) = setup_stocked();
        let _result = EqualityDetails::assert_empty(
            &families[EMPTY_FAMILY],
            &bp_registry[R_EMPTY],
            0, None, 0, None,
            &mut [], &mut vec![], &[], &mut vec![], &mut vec![], &[],
        );
        assert!(!_result);
    }

    /// Test that `schema_empty` returns `false`.
    #[test]
    fn test_schema_empty_returns_false() {
        let (families, bp_registry) = setup_stocked();
        let result = EqualityDetails::schema_empty(
            &families[EMPTY_FAMILY],
            0,
            &bp_registry[R_EMPTY],
        );
        assert!(!result);
    }

    /// Test dispatch via `BinaryPredicateFamilies::typecheck`.
    #[test]
    fn test_dispatch_typecheck_via_families() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);
        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[R_EQUALITY], &[], &[], &families,
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test dispatch via `BinaryPredicateFamilies::assert`.
    #[test]
    fn test_dispatch_assert_via_families() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);
        let _result = BinaryPredicateFamilies::assert(
            &bp_registry[R_EQUALITY],
            0, None, 0, None,
            &families,
            &mut [], &mut vec![], &[], &mut vec![], &mut vec![], &[],
        );
        assert!(!_result);
    }

    /// Test dispatch via `BinaryPredicateFamilies::get_schema`.
    #[test]
    fn test_dispatch_schema_via_families() {
        let (mut families, bp_registry) = setup_stocked();
        EqualityDetails::start(&mut families);
        let result = BinaryPredicateFamilies::get_schema(
            0, &bp_registry[R_EQUALITY], &families,
        );
        assert!(!result);
    }
}
