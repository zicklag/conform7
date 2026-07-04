/// The Universal Relation system — binary predicates for the universal
/// and meaning relations.
///
/// Corresponds to `Relations::Universal` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`).
///
/// Creates a universal bp_family with two binary predicates:
/// - R_universal — the universal relation ("relates"), which can apply
///   to any two things and subsumes all other relations
/// - R_meaning — the meaning relation ("means"), which relates a verb
///   to its meaning (a relation)
///
/// The universal relation is the most general relation in Inform. It is
/// used for relation-level operations: testing whether a relation applies
/// between two values, and asserting/retracting relation facts at run-time.
///
/// The meaning relation is used to associate verbs with their semantic
/// meaning (a relation). For example, the verb "to love" might mean the
/// "loves" relation.
///
/// Simplified:
/// - No PreformUtilities::wording (uses string names)
/// - No Kinds::eq (typecheck always returns ALWAYS_MATCH)
/// - No Kinds::get_construct (typecheck always returns ALWAYS_MATCH)
/// - No Kinds::binary_construction_material (typecheck always returns ALWAYS_MATCH)
/// - No Kinds::compatible (typecheck always returns ALWAYS_MATCH)
/// - No Problems::quote_kind (no problem messages)
/// - No StandardProblems::tcp_problem (no problem messages)
/// - No Calculus::Schemas::modify (schema returns FALSE)
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;
use crate::knowledge::properties::Property;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the universal family in the family registry.
///
/// This assumes the equality relation (PLAN-22) has already created families 0-2,
/// and the quasinumeric relation (PLAN-36) has created family 3.
/// The universal family is family 4.
pub const UNIVERSAL_FAMILY: usize = 4;

/// Index of the universal relation predicate in the BP registry.
///
/// Created by `UniversalRelation::stock()` during first stock.
/// This assumes the equality relation (PLAN-22) has already created BPs 0-3,
/// and the quasinumeric relation (PLAN-36) has created BPs 4-11.
pub const R_UNIVERSAL: usize = 12;

/// Index of the meaning relation predicate in the BP registry.
///
/// Created by `UniversalRelation::stock()` during first stock.
pub const R_MEANING: usize = 14;

/// The universal relation module.
///
/// Corresponds to `Relations::Universal` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`).
pub struct UniversalRelation;

impl UniversalRelation {
    /// Create the universal bp_family with its methods.
    ///
    /// Corresponds to `Relations::Universal::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 19-26).
    ///
    /// # Arguments
    ///
    /// * `families` - The family registry to add to.
    /// * `bp_registry` - The BP registry to add to.
    ///
    /// # Returns
    ///
    /// The index of the created family in the registry.
    #[allow(clippy::ptr_arg)]
    pub fn start(
        families: &mut Vec<BpFamily>,
        _bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        let family_idx = families.len();
        let family = BpFamily {
            name: "universal",
            methods: BpFamilyMethods {
                stock: Some(UniversalRelation::stock),
                typecheck: Some(UniversalRelation::typecheck),
                assert: Some(UniversalRelation::assert),
                schema: Some(UniversalRelation::schema),
                describe_for_problems: Some(UniversalRelation::describe_for_problems),
                describe_for_index: None,
            },
        };
        families.push(family);
        family_idx
    }

    /// Stock the universal family (stage 1): create two binary predicates.
    ///
    /// Corresponds to `Relations::Universal::stock` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 32-45).
    ///
    /// Creates two binary predicates at stock stage 1:
    /// - R_universal — the universal relation ("relates")
    /// - R_meaning — the meaning relation ("means")
    ///
    /// Both use open term details (BPTerms::new(None)) — no kind restriction.
    ///
    /// Simplified:
    /// - No PreformUtilities::wording (uses string names)
    /// - No Calculus::Schemas::new (uses None for test schemas)
    pub fn stock(
        _family: &BpFamily,
        n: u8,
        bp_registry: &mut Vec<BinaryPredicate>,
        _property_registry: &[Property],
    ) {
        if n == 1 {
            // Create open term details (no kind restriction).
            // Corresponds to BPTerms::new(NULL) in the C reference.
            let open_term = BPTerms::new(None);

            // R_universal: the universal relation ("relates")
            // Corresponds to lines 34-38 of the C reference.
            BinaryPredicates::make_pair(
                UNIVERSAL_FAMILY,
                open_term.clone(),
                open_term.clone(),
                "relates",
                "relates-rev",
                None,
                None,
                Some("relates"),
                bp_registry,
            );

            // R_meaning: the meaning relation ("means")
            // Corresponds to lines 39-43 of the C reference.
            BinaryPredicates::make_pair(
                UNIVERSAL_FAMILY,
                open_term.clone(),
                open_term,
                "means",
                "means-rev",
                None,
                None,
                Some("means"),
                bp_registry,
            );
        }
    }

    /// Typecheck the terms of a universal relation.
    ///
    /// Corresponds to `Relations::Universal::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 51-110).
    ///
    /// Simplified: always returns ALWAYS_MATCH. The full C implementation
    /// checks that:
    /// - For R_meaning: first term must be K_verb, second must be CON_relation
    /// - For R_universal: first term must be CON_relation, second must be CON_combination,
    ///   and the relation's domain must match the combination's components
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        1 // ALWAYS_MATCH
    }

    /// Assert a universal relation.
    ///
    /// Corresponds to `Relations::Universal::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 116-120).
    ///
    /// These relations cannot be asserted — they are for run-time use only.
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
        _constructors: &[Property],
    ) -> bool {
        false
    }

    /// Compile run-time code for a universal relation.
    ///
    /// Corresponds to `Relations::Universal::schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 127-149).
    ///
    /// Simplified: returns FALSE (use default schema). The full implementation
    /// handles TEST_ATOM_TASK, NOW_ATOM_TRUE_TASK, and NOW_ATOM_FALSE_TASK
    /// with I6 schema modifications.
    pub fn schema(
        _family: &BpFamily,
        _task: u8,
        _bp: &BinaryPredicate,
    ) -> bool {
        false
    }

    /// Describe the relation in problem messages.
    ///
    /// Corresponds to `Relations::Universal::describe_for_problems` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 155-157).
    ///
    /// Returns an empty string (equivalent to FALSE in C — no special problem description).
    pub fn describe_for_problems(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
    ) -> String {
        String::new()
    }

    /// Check if a binary predicate belongs to the universal family.
    ///
    /// Corresponds to checking `bp->relation_family == universal_bp_family`
    /// in the C reference.
    pub fn is_universal_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
        bp_registry
            .get(bp_idx)
            .map(|bp| bp.relation_family == UNIVERSAL_FAMILY)
            .unwrap_or(false)
    }

    /// Check if a binary predicate is the meaning relation.
    ///
    /// Corresponds to checking `bp == R_meaning` in the C reference.
    pub fn is_meaning_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
        bp_registry
            .get(bp_idx)
            .map(|bp| {
                bp.relation_family == UNIVERSAL_FAMILY
                    && bp.relation_name.as_deref() == Some("means")
            })
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;
    use crate::calculus::bp_term_details::BpTermDetails;

    /// Helper: create a universal relation setup with equality and quasinumeric
    /// relations pre-loaded.
    ///
    /// This ensures the universal family is at index 4 (UNIVERSAL_FAMILY)
    /// and the BP registry has 12 pre-existing BPs (0-3 from equality relation,
    /// 4-11 from quasinumeric relation), matching the real startup sequence.
    fn setup() -> (Vec<BpFamily>, Vec<BinaryPredicate>, usize) {
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        // Pre-populate with 4 dummy families to match equality + quasinumeric layout
        families.push(BpFamily::new("dummy0"));
        families.push(BpFamily::new("dummy1"));
        families.push(BpFamily::new("dummy2"));
        families.push(BpFamily::new("dummy3"));
        // Pre-populate with 12 dummy BPs to match equality + quasinumeric layout
        for i in 0..12 {
            bp_registry.push(BinaryPredicate {
                relation_family: 0,
                family_specific: None,
                relation_name: Some(format!("dummy_bp_{}", i)),
                debugging_log_name: Some(format!("dummy_bp_{}", i)),
                term_details: [
                    BpTermDetails {
                        implies_infs: None,
                        implies_kind: None,
                        called_name: None,
                        function_of_other: None,
                        index_term_as: None,
                    },
                    BpTermDetails {
                        implies_infs: None,
                        implies_kind: None,
                        called_name: None,
                        function_of_other: None,
                        index_term_as: None,
                    },
                ],
                reversal: Some(i),
                right_way_round: true,
                task_functions: [None, None, None, None],
                loop_parent_optimisation_proviso: None,
                loop_parent_optimisation_ranger: None,
                knowledge_about_bp: None,
            });
        }
        let family_idx = UniversalRelation::start(&mut families, &mut bp_registry);
        (families, bp_registry, family_idx)
    }

    /// Helper: create and stock a universal relation setup.
    fn setup_stocked() -> (Vec<BpFamily>, Vec<BinaryPredicate>, usize) {
        let (mut families, mut bp_registry, family_idx) = setup();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        (families, bp_registry, family_idx)
    }

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

    /// Test that `start()` creates the universal family with all five methods.
    #[test]
    fn test_start_creates_family() {
        let (families, _, family_idx) = setup();

        assert_eq!(family_idx, UNIVERSAL_FAMILY);
        assert_eq!(families[family_idx].name, "universal");
        assert!(families[family_idx].methods.stock.is_some());
        assert!(families[family_idx].methods.typecheck.is_some());
        assert!(families[family_idx].methods.assert.is_some());
        assert!(families[family_idx].methods.schema.is_some());
        assert!(families[family_idx].methods.describe_for_problems.is_some());
        // No describe_for_index method
        assert!(families[family_idx].methods.describe_for_index.is_none());
    }

    /// Test that start adds one family to the registry.
    #[test]
    fn test_start_adds_one_family() {
        let (families, _, _) = setup();
        assert_eq!(families.len(), 5);
    }

    // -----------------------------------------------------------------------
    // stock() tests
    // -----------------------------------------------------------------------

    /// Test that stock creates two binary predicates at stage 1.
    #[test]
    fn test_stock_creates_bps() {
        let (_families, bp_registry, _) = setup_stocked();

        // 12 dummy BPs + (2 pairs = 4 BPs) = 16 BPs
        assert_eq!(bp_registry.len(), 16);

        // Check R_universal
        let universal = &bp_registry[R_UNIVERSAL];
        assert_eq!(universal.relation_family, UNIVERSAL_FAMILY);
        assert_eq!(universal.debugging_log_name.as_deref(), Some("relates"));
        assert!(universal.right_way_round);

        // Check R_meaning
        let meaning = &bp_registry[R_MEANING];
        assert_eq!(meaning.relation_family, UNIVERSAL_FAMILY);
        assert_eq!(meaning.debugging_log_name.as_deref(), Some("means"));
        assert!(meaning.right_way_round);
    }

    /// Test that stock at stage 2 does nothing.
    #[test]
    fn test_stock_stage_2_noop() {
        let (families, mut bp_registry, _) = setup();

        // Stock stage 1
        UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);
        let count_after_stage_1 = bp_registry.len();

        // Stock stage 2 should do nothing
        UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 2, &mut bp_registry, &[]);
        assert_eq!(bp_registry.len(), count_after_stage_1);
    }

    // -----------------------------------------------------------------------
    // typecheck() tests
    // -----------------------------------------------------------------------

    /// Test that typecheck always returns ALWAYS_MATCH.
    #[test]
    fn test_typecheck_always_matches() {
        let (families, bp_registry, _) = setup_stocked();

        let result = UniversalRelation::typecheck(
            &families[UNIVERSAL_FAMILY],
            &bp_registry[R_UNIVERSAL],
            &[],
            &[],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    // -----------------------------------------------------------------------
    // assert() tests
    // -----------------------------------------------------------------------

    /// Test that assert returns FALSE.
    #[test]
    fn test_assert_returns_false() {
        let (families, bp_registry, _) = setup_stocked();

        let result = UniversalRelation::assert(
            &families[UNIVERSAL_FAMILY],
            &bp_registry[R_UNIVERSAL],
            0,
            None,
            0,
            None,
            &mut [],
            &mut vec![],
            &[],
            &mut vec![],
            &mut vec![],
            &[],
        );
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // schema() tests
    // -----------------------------------------------------------------------

    /// Test that schema returns FALSE.
    #[test]
    fn test_schema_returns_false() {
        let (families, bp_registry, _) = setup_stocked();

        let result = UniversalRelation::schema(
            &families[UNIVERSAL_FAMILY],
            0,
            &bp_registry[R_UNIVERSAL],
        );
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // describe_for_problems() tests
    // -----------------------------------------------------------------------

    /// Test that describe_for_problems returns an empty string.
    #[test]
    fn test_describe_for_problems_returns_empty() {
        let (families, bp_registry, _) = setup_stocked();

        let result = UniversalRelation::describe_for_problems(
            &families[UNIVERSAL_FAMILY],
            &bp_registry[R_UNIVERSAL],
        );
        assert_eq!(result, "");
    }

    // -----------------------------------------------------------------------
    // is_universal_bp() tests
    // -----------------------------------------------------------------------

    /// Test is_universal_bp returns true for universal BPs.
    #[test]
    fn test_is_universal_bp_true() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(UniversalRelation::is_universal_bp(R_UNIVERSAL, &bp_registry));
        assert!(UniversalRelation::is_universal_bp(R_MEANING, &bp_registry));
    }

    /// Test is_universal_bp returns false for non-universal BPs.
    #[test]
    fn test_is_universal_bp_false() {
        let (_families, bp_registry, _) = setup_stocked();

        // A non-existent BP index should return false
        assert!(!UniversalRelation::is_universal_bp(99, &bp_registry));
    }

    // -----------------------------------------------------------------------
    // is_meaning_bp() tests
    // -----------------------------------------------------------------------

    /// Test is_meaning_bp returns true for the meaning BP.
    #[test]
    fn test_is_meaning_bp_true() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(UniversalRelation::is_meaning_bp(R_MEANING, &bp_registry));
    }

    /// Test is_meaning_bp returns false for the universal BP.
    #[test]
    fn test_is_meaning_bp_false_for_universal() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(!UniversalRelation::is_meaning_bp(R_UNIVERSAL, &bp_registry));
    }

    /// Test is_meaning_bp returns false for non-existent BPs.
    #[test]
    fn test_is_meaning_bp_false_for_nonexistent() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(!UniversalRelation::is_meaning_bp(99, &bp_registry));
    }
}
