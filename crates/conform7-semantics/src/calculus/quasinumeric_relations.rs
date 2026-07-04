/// The Quasinumeric Relations system — binary predicates for numerical comparisons.
///
/// Corresponds to `Calculus::QuasinumericRelations` in the C reference
/// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`).
///
/// Creates a quasinumeric bp_family with four binary predicates:
/// - R_numerically_greater_than (>)
/// - R_numerically_less_than (<)
/// - R_numerically_greater_than_or_equal_to (>=)
/// - R_numerically_less_than_or_equal_to (<=)
///
/// These relations can be applied not only to numbers but also to units
/// (height, length, etc.). The inequality relations are used throughout
/// Inform for numerical comparisons.
///
/// Simplified:
/// - No PreformUtilities::wording (uses string names)
/// - No Kinds::compatible (typecheck always returns ALWAYS_MATCH)
/// - No Kinds::FloatingPoint::uses_floating_point (schema returns FALSE)
/// - No Kinds::Behaviour::get_comparison_routine (schema returns FALSE)
/// - No Problems::quote_kind (no problem messages)
/// - No StandardProblems::tcp_problem (no problem messages)
/// - No BinaryPredicates::set_index_details (deferred)
/// - No Calculus::Schemas::new (uses Option<&str> in make_pair)
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;
use crate::knowledge::properties::Property;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the quasinumeric family in the family registry.
///
/// This assumes the equality relation (PLAN-22) has already created families 0-2.
/// The quasinumeric family is family 3.
pub const QUASINUMERIC_FAMILY: usize = 3;

/// Index of the greater-than predicate in the BP registry.
///
/// Created by `QuasinumericRelations::stock()` during first stock.
/// This assumes the equality relation (PLAN-22) has already created BPs 0-3.
pub const R_NUMERICALLY_GREATER_THAN: usize = 4;

/// Index of the less-than predicate in the BP registry.
///
/// Created by `QuasinumericRelations::stock()` during first stock.
pub const R_NUMERICALLY_LESS_THAN: usize = 6;

/// Index of the greater-than-or-equal-to predicate in the BP registry.
///
/// Created by `QuasinumericRelations::stock()` during first stock.
pub const R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO: usize = 8;

/// Index of the less-than-or-equal-to predicate in the BP registry.
///
/// Created by `QuasinumericRelations::stock()` during first stock.
pub const R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO: usize = 10;

/// The quasinumeric relations module.
///
/// Corresponds to `Calculus::QuasinumericRelations` in the C reference
/// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`).
pub struct QuasinumericRelations;

impl QuasinumericRelations {
    /// Create the quasinumeric bp_family with its methods.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 29-37).
    ///
    /// Appends the quasinumeric family to `families` and returns its index.
    /// When called after `EqualityRelation::start()`, the family will be at index 3.
    /// The bp_registry is left empty (stocking fills it via `first_stock`).
    #[allow(clippy::ptr_arg)]
    pub fn start(
        families: &mut Vec<BpFamily>,
        _bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        let family_idx = families.len();
        let family = BpFamily {
            name: "quasinumeric",
            methods: BpFamilyMethods {
                stock: Some(QuasinumericRelations::stock),
                typecheck: Some(QuasinumericRelations::typecheck),
                assert: Some(QuasinumericRelations::assert),
                schema: Some(QuasinumericRelations::schema),
                describe_for_problems: Some(QuasinumericRelations::describe_for_problems),
                describe_for_index: Some(QuasinumericRelations::describe_for_index),
            },
        };
        families.push(family);
        family_idx
    }

    /// Stock the quasinumeric family (stage 1): create four binary predicates.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::stock` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 43-75).
    ///
    /// Creates four binary predicates at stock stage 1:
    /// - R_numerically_greater_than with test schema "*1 > *2"
    /// - R_numerically_less_than with test schema "*1 < *2"
    /// - R_numerically_greater_than_or_equal_to with test schema "*1 >= *2"
    /// - R_numerically_less_than_or_equal_to with test schema "*1 <= *2"
    ///
    /// Simplified:
    /// - No PreformUtilities::wording (uses string names)
    /// - No BinaryPredicates::set_index_details (deferred)
    /// - No KindSubjects::from_kind (uses BPTerms::new(None) for term details)
    pub fn stock(
        _family: &BpFamily,
        n: u8,
        bp_registry: &mut Vec<BinaryPredicate>,
        _property_registry: &[Property],
    ) {
        if n == 1 {
            // Create term details for the number kind domain.
            // Simplified: uses BPTerms::new(None) instead of KindSubjects::from_kind.
            // In the full C implementation, this would be:
            //   bp_term_details number_term = BPTerms::new(KindSubjects::from_kind(K_number));
            let number_term = BPTerms::new(None);

            // R_numerically_greater_than: *1 > *2
            // Corresponds to lines 46-49 of the C reference.
            BinaryPredicates::make_pair(
                QUASINUMERIC_FAMILY,
                number_term.clone(),
                number_term.clone(),
                "greater-than",
                "greater-than-rev",
                None,
                Some("*1 > *2"),
                Some("greater-than"),
                bp_registry,
            );

            // R_numerically_less_than: *1 < *2
            // Corresponds to lines 50-53 of the C reference.
            BinaryPredicates::make_pair(
                QUASINUMERIC_FAMILY,
                number_term.clone(),
                number_term.clone(),
                "less-than",
                "less-than-rev",
                None,
                Some("*1 < *2"),
                Some("less-than"),
                bp_registry,
            );

            // R_numerically_greater_than_or_equal_to: *1 >= *2
            // Corresponds to lines 54-57 of the C reference.
            BinaryPredicates::make_pair(
                QUASINUMERIC_FAMILY,
                number_term.clone(),
                number_term.clone(),
                "at-least",
                "at-least-rev",
                None,
                Some("*1 >= *2"),
                Some("at-least"),
                bp_registry,
            );

            BinaryPredicates::make_pair(
                QUASINUMERIC_FAMILY,
                number_term.clone(),
                number_term,
                "at-most",
                "at-most-rev",
                None,
                Some("*1 <= *2"),
                Some("at-most"),
                bp_registry,
            );
        }
    }

    /// Typecheck the terms of a quasinumeric relation.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 80-94).
    ///
    /// Simplified: always returns ALWAYS_MATCH (no kind compatibility checking yet).
    /// The full C implementation checks Kinds::compatible and issues problem messages
    /// for incompatible kind comparisons.
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        1 // ALWAYS_MATCH
    }

    /// Assert a quasinumeric relation.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 100-104).
    ///
    /// These relations cannot be asserted — they are for run-time testing only.
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

    /// Compile run-time code for a quasinumeric relation.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 112-167).
    ///
    /// Simplified: returns FALSE (use default schema). The full implementation
    /// handles floating-point promotion and comparison routines.
    pub fn schema(
        _family: &BpFamily,
        _task: u8,
        _bp: &BinaryPredicate,
    ) -> bool {
        false
    }

    /// Describe the relation in problem messages.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::describe_for_problems` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 172-174).
    ///
    /// Returns an empty string (equivalent to FALSE in C — no special problem description).
    pub fn describe_for_problems(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
    ) -> String {
        String::new()
    }

    /// Describe the relation in the Phrasebook index.
    ///
    /// Corresponds to `Calculus::QuasinumericRelations::describe_for_index` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 175-177).
    pub fn describe_for_index(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
    ) -> String {
        "numeric".to_string()
    }

    /// Check if a binary predicate belongs to the quasinumeric family.
    ///
    /// Corresponds to checking `bp->relation_family == quasinumeric_bp_family`
    /// in the C reference.
    pub fn is_quasinumeric_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
        bp_registry
            .get(bp_idx)
            .map(|bp| bp.relation_family == QUASINUMERIC_FAMILY)
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

    /// Helper: create a quasinumeric setup with equality relation pre-loaded.
    /// This ensures the quasinumeric family is at index 3 (QUASINUMERIC_FAMILY)
    /// and the BP registry has 4 pre-existing BPs (0-3 from equality relation),
    /// matching the real startup sequence.
    fn setup() -> (Vec<BpFamily>, Vec<BinaryPredicate>, usize) {
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        // Pre-populate with 3 dummy families to match equality relation layout
        families.push(BpFamily::new("dummy0"));
        families.push(BpFamily::new("dummy1"));
        families.push(BpFamily::new("dummy2"));
        // Pre-populate with 4 dummy BPs to match equality relation layout
        for i in 0..4 {
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
        let family_idx = QuasinumericRelations::start(&mut families, &mut bp_registry);
        (families, bp_registry, family_idx)
    }

    /// Helper: create and stock a quasinumeric setup.
    fn setup_stocked() -> (Vec<BpFamily>, Vec<BinaryPredicate>, usize) {
        let (mut families, mut bp_registry, family_idx) = setup();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        (families, bp_registry, family_idx)
    }

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

#[test]
    fn test_start_creates_one_family() {
        let (families, bp_registry, _) = setup();
        assert_eq!(families.len(), 4);
        assert_eq!(bp_registry.len(), 4);
    }

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let (families, _, family_idx) = setup();
        assert_eq!(families[family_idx].name, "quasinumeric");
    }

    #[test]
    fn test_quasinumeric_family_has_all_six_methods() {
        let (families, _, family_idx) = setup();
        let qn = &families[family_idx];
        assert!(qn.methods.stock.is_some());
        assert!(qn.methods.typecheck.is_some());
        assert!(qn.methods.assert.is_some());
        assert!(qn.methods.schema.is_some());
        assert!(qn.methods.describe_for_problems.is_some());
        assert!(qn.methods.describe_for_index.is_some());
    }

    // -----------------------------------------------------------------------
    // first_stock tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_stock_creates_four_bps() {
        let (_families, bp_registry, _) = setup_stocked();

        // 4 dummy BPs + (4 BPs × 2 each make_pair creates 2) = 12 BPs
        assert_eq!(bp_registry.len(), 12);
    }

    #[test]
    fn test_first_stock_creates_r_numerically_greater_than() {
        let (_families, bp_registry, _) = setup_stocked();

        // R_numerically_greater_than is at index 4 (first BP created)
        assert!(bp_registry.len() > R_NUMERICALLY_GREATER_THAN);
        let bp = &bp_registry[R_NUMERICALLY_GREATER_THAN];
        assert_eq!(bp.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(bp.relation_name, Some("greater-than".to_string()));
        assert_eq!(bp.debugging_log_name, Some("greater-than".to_string()));
        assert!(bp.right_way_round);
        // Has a reversal pointing to index 5
        assert_eq!(bp.reversal, Some(5));
    }

    #[test]
    fn test_first_stock_creates_r_numerically_less_than() {
        let (_families, bp_registry, _) = setup_stocked();

        // R_numerically_less_than is at index 6
        assert!(bp_registry.len() > R_NUMERICALLY_LESS_THAN);
        let bp = &bp_registry[R_NUMERICALLY_LESS_THAN];
        assert_eq!(bp.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(bp.relation_name, Some("less-than".to_string()));
        assert_eq!(bp.debugging_log_name, Some("less-than".to_string()));
        assert!(bp.right_way_round);
        // Has a reversal pointing to index 7
        assert_eq!(bp.reversal, Some(7));
    }

    #[test]
    fn test_first_stock_creates_r_numerically_greater_than_or_equal_to() {
        let (_families, bp_registry, _) = setup_stocked();

        // R_numerically_greater_than_or_equal_to is at index 8
        assert!(bp_registry.len() > R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO);
        let bp = &bp_registry[R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO];
        assert_eq!(bp.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(bp.relation_name, Some("at-least".to_string()));
        assert_eq!(bp.debugging_log_name, Some("at-least".to_string()));
        assert!(bp.right_way_round);
        // Has a reversal pointing to index 9
        assert_eq!(bp.reversal, Some(9));
    }

    #[test]
    fn test_first_stock_creates_r_numerically_less_than_or_equal_to() {
        let (_families, bp_registry, _) = setup_stocked();

        // R_numerically_less_than_or_equal_to is at index 10
        assert!(bp_registry.len() > R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO);
        let bp = &bp_registry[R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO];
        assert_eq!(bp.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(bp.relation_name, Some("at-most".to_string()));
        assert_eq!(bp.debugging_log_name, Some("at-most".to_string()));
        assert!(bp.right_way_round);
        // Has a reversal pointing to index 11
        assert_eq!(bp.reversal, Some(11));
    }

    // -----------------------------------------------------------------------
    // Test schema tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_greater_than_has_correct_test_schema() {
        let (_families, bp_registry, _) = setup_stocked();

        let bp = &bp_registry[R_NUMERICALLY_GREATER_THAN];
        // task_functions[1] = TEST_ATOM_TASK
        assert_eq!(bp.task_functions[1], Some("*1 > *2".to_string()));
    }

    #[test]
    fn test_less_than_has_correct_test_schema() {
        let (_families, bp_registry, _) = setup_stocked();

        let bp = &bp_registry[R_NUMERICALLY_LESS_THAN];
        assert_eq!(bp.task_functions[1], Some("*1 < *2".to_string()));
    }

    #[test]
    fn test_greater_than_or_equal_has_correct_test_schema() {
        let (_families, bp_registry, _) = setup_stocked();

        let bp = &bp_registry[R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO];
        assert_eq!(bp.task_functions[1], Some("*1 >= *2".to_string()));
    }

    #[test]
    fn test_less_than_or_equal_has_correct_test_schema() {
        let (_families, bp_registry, _) = setup_stocked();

        let bp = &bp_registry[R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO];
        assert_eq!(bp.task_functions[1], Some("*1 <= *2".to_string()));
    }

    // -----------------------------------------------------------------------
    // Term details tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_bps_have_number_term_details() {
        let (_families, bp_registry, _) = setup_stocked();

        // All four BPs should have the same term details (number kind domain)
        for idx in [R_NUMERICALLY_GREATER_THAN, R_NUMERICALLY_LESS_THAN,
                     R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO, R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO]
        {
            let bp = &bp_registry[idx];
            // Both terms should have the same domain (None for now, simplified)
            assert_eq!(bp.term_details[0].implies_infs, None);
            assert_eq!(bp.term_details[1].implies_infs, None);
        }
    }

    // -----------------------------------------------------------------------
    // Reversal tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_greater_than_reversal_has_swapped_terms() {
        let (_families, bp_registry, _) = setup_stocked();

        // greater-than (index 4) reversal is at index 5
        let rev = &bp_registry[5];
        assert_eq!(rev.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(rev.debugging_log_name, Some("greater-than-rev".to_string()));
        assert!(!rev.right_way_round);
        assert_eq!(rev.reversal, Some(R_NUMERICALLY_GREATER_THAN));
    }

    #[test]
    fn test_less_than_reversal_has_swapped_terms() {
        let (_families, bp_registry, _) = setup_stocked();

        // less-than (index 6) reversal is at index 7
        let rev = &bp_registry[7];
        assert_eq!(rev.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(rev.debugging_log_name, Some("less-than-rev".to_string()));
        assert!(!rev.right_way_round);
        assert_eq!(rev.reversal, Some(R_NUMERICALLY_LESS_THAN));
    }

    #[test]
    fn test_at_least_reversal_has_swapped_terms() {
        let (_families, bp_registry, _) = setup_stocked();

        // at-least (index 8) reversal is at index 9
        let rev = &bp_registry[9];
        assert_eq!(rev.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(rev.debugging_log_name, Some("at-least-rev".to_string()));
        assert!(!rev.right_way_round);
        assert_eq!(rev.reversal, Some(R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO));
    }

    #[test]
    fn test_at_most_reversal_has_swapped_terms() {
        let (_families, bp_registry, _) = setup_stocked();

        // at-most (index 10) reversal is at index 11
        let rev = &bp_registry[11];
        assert_eq!(rev.relation_family, QUASINUMERIC_FAMILY);
        assert_eq!(rev.debugging_log_name, Some("at-most-rev".to_string()));
        assert!(!rev.right_way_round);
        assert_eq!(rev.reversal, Some(R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO));
    }

    // -----------------------------------------------------------------------
    // Typecheck test
    // -----------------------------------------------------------------------

    #[test]
    fn test_typecheck_returns_always_match() {
        let (families, _, family_idx) = setup();
        let qn_family = &families[family_idx];

        let dummy_bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some("greater-than".to_string()),
            debugging_log_name: Some("greater-than".to_string()),
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
            reversal: Some(5),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = QuasinumericRelations::typecheck(
            qn_family,
            &dummy_bp,
            &[None, None],
            &[None, None],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    #[test]
    fn test_typecheck_dispatch_via_family() {
        let (_families, bp_registry, _) = setup_stocked();

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[R_NUMERICALLY_GREATER_THAN],
            &[None, None],
            &[None, None],
            &_families,
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    // -----------------------------------------------------------------------
    // Assert test
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_returns_false() {
        let (families, _, family_idx) = setup();
        let qn_family = &families[family_idx];

        let dummy_bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some("greater-than".to_string()),
            debugging_log_name: Some("greater-than".to_string()),
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
            reversal: Some(5),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = QuasinumericRelations::assert(
            qn_family,
            &dummy_bp,
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

    #[test]
    fn test_assert_dispatch_via_family() {
        let (_families, bp_registry, _) = setup_stocked();

        let result = BinaryPredicateFamilies::assert(
            &bp_registry[R_NUMERICALLY_GREATER_THAN],
            0,
            None,
            0,
            None,
            &_families,
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
    // Schema test
    // -----------------------------------------------------------------------

    #[test]
    fn test_schema_returns_false() {
        let (families, _, family_idx) = setup();
        let qn_family = &families[family_idx];

        let dummy_bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some("greater-than".to_string()),
            debugging_log_name: Some("greater-than".to_string()),
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
            reversal: Some(5),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = QuasinumericRelations::schema(qn_family, 1, &dummy_bp);
        assert!(!result);
    }

    #[test]
    fn test_schema_dispatch_via_family() {
        let (_families, bp_registry, _) = setup_stocked();

        // The quasinumeric family's schema returns false, so get_schema
        // should return false even though the BP has a test schema
        let result = BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[R_NUMERICALLY_GREATER_THAN],
            &_families,
        );
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // Describe method tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_describe_for_problems_returns_empty_string() {
        let (families, _, family_idx) = setup();
        let qn_family = &families[family_idx];

        let dummy_bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some("greater-than".to_string()),
            debugging_log_name: Some("greater-than".to_string()),
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
            reversal: Some(5),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = QuasinumericRelations::describe_for_problems(qn_family, &dummy_bp);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_for_problems_dispatch_via_family() {
        let (_families, bp_registry, _) = setup_stocked();

        let desc = BinaryPredicateFamilies::describe_for_problems(
            &bp_registry[R_NUMERICALLY_GREATER_THAN],
            &_families,
        );
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_for_index_returns_numeric() {
        let (families, _, family_idx) = setup();
        let qn_family = &families[family_idx];

        let dummy_bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some("greater-than".to_string()),
            debugging_log_name: Some("greater-than".to_string()),
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
            reversal: Some(5),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = QuasinumericRelations::describe_for_index(qn_family, &dummy_bp);
        assert_eq!(desc, "numeric");
    }

    #[test]
    fn test_describe_for_index_dispatch_via_family() {
        let (_families, bp_registry, _) = setup_stocked();

        let desc = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[R_NUMERICALLY_GREATER_THAN],
            &_families,
        );
        assert_eq!(desc, "numeric");
    }

    // -----------------------------------------------------------------------
    // is_quasinumeric_bp tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_quasinumeric_bp_returns_true_for_quasinumeric_bps() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(QuasinumericRelations::is_quasinumeric_bp(
            R_NUMERICALLY_GREATER_THAN,
            &bp_registry,
        ));
        assert!(QuasinumericRelations::is_quasinumeric_bp(
            R_NUMERICALLY_LESS_THAN,
            &bp_registry,
        ));
        assert!(QuasinumericRelations::is_quasinumeric_bp(
            R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO,
            &bp_registry,
        ));
        assert!(QuasinumericRelations::is_quasinumeric_bp(
            R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO,
            &bp_registry,
        ));
    }

    #[test]
    fn test_is_quasinumeric_bp_returns_true_for_reversals() {
        let (_families, bp_registry, _) = setup_stocked();

        // Reversals are also quasinumeric (same family)
        assert!(QuasinumericRelations::is_quasinumeric_bp(5, &bp_registry)); // greater-than-rev
        assert!(QuasinumericRelations::is_quasinumeric_bp(7, &bp_registry)); // less-than-rev
        assert!(QuasinumericRelations::is_quasinumeric_bp(9, &bp_registry)); // at-least-rev
        assert!(QuasinumericRelations::is_quasinumeric_bp(11, &bp_registry)); // at-most-rev
    }

    #[test]
    fn test_is_quasinumeric_bp_returns_false_for_out_of_range() {
        let (_families, bp_registry, _) = setup_stocked();

        assert!(!QuasinumericRelations::is_quasinumeric_bp(99, &bp_registry));
    }

    // -----------------------------------------------------------------------
    // Stock stage 2 does nothing
    // -----------------------------------------------------------------------

    #[test]
    fn test_stock_at_stage_2_does_nothing() {
        let (mut families, mut bp_registry, _) = setup();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        let len_after_stage1 = bp_registry.len();

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &[]);
        assert_eq!(bp_registry.len(), len_after_stage1);
    }
}
