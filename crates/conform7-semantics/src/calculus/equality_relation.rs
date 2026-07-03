/// The equality relation and related families.
///
/// Corresponds to `Calculus::Equality` in the C reference
/// (`services/calculus-module/Chapter 3/The Equality Relation.w`).
///
/// Creates three bp_family instances:
/// - equality_bp_family — for the equality relation (R_equality)
/// - spatial_bp_family — for the "has" / "is-had-by" pair (a_has_b_predicate)
/// - empty_bp_family — for the "never-holding" relation (R_empty)
///
/// These are the first concrete uses of the binary predicate system.
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the equality family in the family registry.
pub const EQUALITY_FAMILY: usize = 0;
/// Index of the spatial family in the family registry.
pub const SPATIAL_FAMILY: usize = 1;
/// Index of the empty family in the family registry.
pub const EMPTY_FAMILY: usize = 2;

/// Index of the equality predicate in the BP registry.
///
/// Created by `EqualityRelation::stock()` during first_stock.
pub const R_EQUALITY: usize = 0;
/// Index of the "has" predicate in the BP registry (right-way-round).
///
/// Created by `EqualityRelation::stock_spatial()` during first_stock.
/// Its reversal (is-had-by) is at index 2.
pub const A_HAS_B_PREDICATE: usize = 1;
/// Index of the "never-holding" predicate in the BP registry.
///
/// Created by `EqualityRelation::stock_empty()` during first_stock.
pub const R_EMPTY: usize = 3;
/// The equality relation module.
///
/// Corresponds to `Calculus::Equality` in the C reference
/// (`services/calculus-module/Chapter 3/The Equality Relation.w`).
pub struct EqualityRelation;

impl EqualityRelation {
    /// Create the three families with their methods.
    ///
    /// Corresponds to `Calculus::Equality::start` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 24-46).
    ///
    /// Returns (families, bp_registry) where:
    /// - families[0] = equality_bp_family
    /// - families[1] = spatial_bp_family
    /// - families[2] = empty_bp_family
    /// - bp_registry is empty (stocking fills it)
    pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
        let equality_family = BpFamily {
            name: "equality",
            methods: BpFamilyMethods {
                stock: Some(EqualityRelation::stock),
                describe_for_problems: Some(EqualityRelation::describe_for_problems),
                describe_for_index: Some(EqualityRelation::describe_for_index),
                ..BpFamilyMethods::default()
            },
        };

        let spatial_family = BpFamily {
            name: "spatial",
            methods: BpFamilyMethods {
                stock: Some(EqualityRelation::stock_spatial),
                ..BpFamilyMethods::default()
            },
        };

        let empty_family = BpFamily {
            name: "empty",
            methods: BpFamilyMethods {
                stock: Some(EqualityRelation::stock_empty),
                describe_for_problems: Some(EqualityRelation::describe_empty_for_problems),
                describe_for_index: Some(EqualityRelation::describe_empty_for_index),
                ..BpFamilyMethods::default()
            },
        };

        (vec![equality_family, spatial_family, empty_family], Vec::new())
    }

    /// Stock the equality family (stage 1): create R_equality.
    ///
    /// Corresponds to `Calculus::Equality::stock` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 52-58).
    pub fn stock(_family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>, _property_registry: &[()]) {
        if n == 1 {
            let family_idx = 0; // equality family is at index 0
            let idx = BinaryPredicates::make_equality(family_idx, "equality", bp_registry);
            // Set index display names: both terms are "value"
            // Corresponds to BinaryPredicates::set_index_details(R_equality, "value", "value")
            // in the C reference (line 57).
            bp_registry[idx].set_index_details(Some("value"), Some("value"), &mut []);
        }
    }

    /// Stock the spatial family (stage 1): create a_has_b_predicate.
    ///
    /// Corresponds to `Calculus::Equality::stock_spatial` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 60-70).
    pub fn stock_spatial(_family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>, _property_registry: &[()]) {
        if n == 1 {
            let family_idx = 1; // spatial family is at index 1
            let left_term = BPTerms::new_full(None, None, None, None);
            let right_term = BPTerms::new(None);
            BinaryPredicates::make_pair(
                family_idx,
                left_term,
                right_term,
                "has",
                "is-had-by",
                None,
                None,
                Some("possession"),
                bp_registry,
            );
        }
    }

    /// Stock the empty family (stage 1): create R_empty.
    ///
    /// Corresponds to `Calculus::Equality::stock_empty` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 72-78).
    pub fn stock_empty(_family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>, _property_registry: &[()]) {
        if n == 1 {
            let family_idx = 2; // empty family is at index 2
            let idx = BinaryPredicates::make_equality(family_idx, "never-holding", bp_registry);
            // Set index display names: both terms are "value"
            // Corresponds to BinaryPredicates::set_index_details(R_empty, "value", "value")
            // in the C reference (line 77).
            bp_registry[idx].set_index_details(Some("value"), Some("value"), &mut []);
        }
    }

    // -----------------------------------------------------------------------
    // Describe methods for the equality family
    // -----------------------------------------------------------------------

    /// Describe the equality relation for problem messages.
    ///
    /// Corresponds to `Calculus::Equality::describe_for_problems` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 83-86).
    ///
    /// Returns an empty string (equivalent to FALSE in C — no special description).
    pub fn describe_for_problems(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
        String::new()
    }

    /// Describe the equality relation for the Phrasebook index.
    ///
    /// Corresponds to `Calculus::Equality::describe_for_index` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 87-90).
    pub fn describe_for_index(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
        "equality".to_string()
    }

    // -----------------------------------------------------------------------
    // Describe methods for the empty family
    // -----------------------------------------------------------------------

    /// Describe the empty relation for problem messages.
    ///
    /// Corresponds to `Calculus::Equality::describe_empty_for_problems` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 91-94).
    ///
    /// Returns an empty string (equivalent to FALSE in C — no special description).
    pub fn describe_empty_for_problems(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
        String::new()
    }

    /// Describe the empty relation for the Phrasebook index.
    ///
    /// Corresponds to `Calculus::Equality::describe_empty_for_index` in the C reference
    /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 95-98).
    pub fn describe_empty_for_index(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
        "never-holding".to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;
    use crate::calculus::bp_term_details::BpTermDetails;

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_three_families() {
        let (families, bp_registry) = EqualityRelation::start();
        assert_eq!(families.len(), 3);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_start_creates_families_with_correct_names() {
        let (families, _) = EqualityRelation::start();
        assert_eq!(families[EQUALITY_FAMILY].name, "equality");
        assert_eq!(families[SPATIAL_FAMILY].name, "spatial");
        assert_eq!(families[EMPTY_FAMILY].name, "empty");
    }

    #[test]
    fn test_equality_family_has_stock_and_describe_methods() {
        let (families, _) = EqualityRelation::start();
        let eq = &families[EQUALITY_FAMILY];
        assert!(eq.methods.stock.is_some());
        assert!(eq.methods.describe_for_problems.is_some());
        assert!(eq.methods.describe_for_index.is_some());
        // Should NOT have typecheck, assert, or schema
        assert!(eq.methods.typecheck.is_none());
        assert!(eq.methods.assert.is_none());
        assert!(eq.methods.schema.is_none());
    }

    #[test]
    fn test_spatial_family_has_only_stock_method() {
        let (families, _) = EqualityRelation::start();
        let sp = &families[SPATIAL_FAMILY];
        assert!(sp.methods.stock.is_some());
        assert!(sp.methods.describe_for_problems.is_none());
        assert!(sp.methods.describe_for_index.is_none());
        assert!(sp.methods.typecheck.is_none());
        assert!(sp.methods.assert.is_none());
        assert!(sp.methods.schema.is_none());
    }

    #[test]
    fn test_empty_family_has_stock_and_describe_methods() {
        let (families, _) = EqualityRelation::start();
        let em = &families[EMPTY_FAMILY];
        assert!(em.methods.stock.is_some());
        assert!(em.methods.describe_for_problems.is_some());
        assert!(em.methods.describe_for_index.is_some());
        assert!(em.methods.typecheck.is_none());
        assert!(em.methods.assert.is_none());
        assert!(em.methods.schema.is_none());
    }

    // -----------------------------------------------------------------------
    // first_stock tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_stock_creates_r_equality_as_own_reversal() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // R_equality is at index 0 (created first by equality family's stock)
        assert!(bp_registry.len() > R_EQUALITY);
        let r_eq = &bp_registry[R_EQUALITY];
        assert_eq!(r_eq.relation_family, EQUALITY_FAMILY);
        assert_eq!(r_eq.relation_name, Some("equality".to_string()));
        assert!(r_eq.right_way_round);
        // Equality is its own reversal
        assert_eq!(r_eq.reversal, Some(R_EQUALITY));
    }

    #[test]
    fn test_first_stock_creates_spatial_pair() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // After first_stock: index 0 = R_equality, index 1 = a_has_b (right-way-round),
        // index 2 = is-had-by (reversal)
        assert!(bp_registry.len() > 2);

        // a_has_b_predicate is at index 1 (second BP created)
        let has = &bp_registry[1];
        assert_eq!(has.relation_family, SPATIAL_FAMILY);
        assert_eq!(has.debugging_log_name, Some("has".to_string()));
        assert!(has.right_way_round);
        // Has a reversal pointing to index 2
        assert_eq!(has.reversal, Some(2));

        // The reversal (is-had-by) is at index 2
        let is_had_by = &bp_registry[2];
        assert_eq!(is_had_by.relation_family, SPATIAL_FAMILY);
        assert_eq!(is_had_by.debugging_log_name, Some("is-had-by".to_string()));
        assert!(!is_had_by.right_way_round);
        // Reversal points back to the original
        assert_eq!(is_had_by.reversal, Some(1));
    }

    #[test]
    fn test_spatial_pair_has_correct_term_details() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // a_has_b_predicate (index 1): left term uses new_full, right term uses new
        let has = &bp_registry[1];
        // Left term: new_full(None, None, None, None)
        assert_eq!(has.term_details[0].implies_infs, None);
        assert_eq!(has.term_details[0].implies_kind, None);
        assert_eq!(has.term_details[0].called_name, None);
        assert_eq!(has.term_details[0].function_of_other, None);
        // Right term: new(None)
        assert_eq!(has.term_details[1].implies_infs, None);
        assert_eq!(has.term_details[1].implies_kind, None);

        // Reversal (index 2): terms are swapped
        let is_had_by = &bp_registry[2];
        assert_eq!(is_had_by.term_details[0].implies_infs, None);
        assert_eq!(is_had_by.term_details[0].implies_kind, None);
        assert_eq!(is_had_by.term_details[1].implies_infs, None);
        assert_eq!(is_had_by.term_details[1].implies_kind, None);
    }

    #[test]
    fn test_first_stock_creates_r_empty_as_own_reversal() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // After first_stock: index 0 = R_equality, index 1 = a_has_b, index 2 = is-had-by,
        // index 3 = R_empty
        assert!(bp_registry.len() > R_EMPTY);
        let r_empty = &bp_registry[R_EMPTY];
        assert_eq!(r_empty.relation_family, EMPTY_FAMILY);
        assert_eq!(r_empty.relation_name, Some("never-holding".to_string()));
        assert!(r_empty.right_way_round);
        // Empty is its own reversal
        assert_eq!(r_empty.reversal, Some(R_EMPTY));
    }

    // -----------------------------------------------------------------------
    // Describe method tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_describe_for_problems_returns_empty_string() {
        let (families, _) = EqualityRelation::start();
        let eq_family = &families[EQUALITY_FAMILY];

        // Create a dummy BP to pass to the describe method
        let dummy_bp = BinaryPredicate {
            relation_family: EQUALITY_FAMILY,
            family_specific: None,
            relation_name: Some("equality".to_string()),
            debugging_log_name: Some("equality".to_string()),
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
            reversal: Some(0),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = EqualityRelation::describe_for_problems(eq_family, &dummy_bp);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_for_index_returns_equality() {
        let (families, _) = EqualityRelation::start();
        let eq_family = &families[EQUALITY_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: EQUALITY_FAMILY,
            family_specific: None,
            relation_name: Some("equality".to_string()),
            debugging_log_name: Some("equality".to_string()),
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
            reversal: Some(0),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = EqualityRelation::describe_for_index(eq_family, &dummy_bp);
        assert_eq!(desc, "equality");
    }

    #[test]
    fn test_describe_empty_for_problems_returns_empty_string() {
        let (families, _) = EqualityRelation::start();
        let em_family = &families[EMPTY_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: EMPTY_FAMILY,
            family_specific: None,
            relation_name: Some("never-holding".to_string()),
            debugging_log_name: Some("never-holding".to_string()),
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
            reversal: Some(0),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = EqualityRelation::describe_empty_for_problems(em_family, &dummy_bp);
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_empty_for_index_returns_never_holding() {
        let (families, _) = EqualityRelation::start();
        let em_family = &families[EMPTY_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: EMPTY_FAMILY,
            family_specific: None,
            relation_name: Some("never-holding".to_string()),
            debugging_log_name: Some("never-holding".to_string()),
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
            reversal: Some(0),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = EqualityRelation::describe_empty_for_index(em_family, &dummy_bp);
        assert_eq!(desc, "never-holding");
    }

    // -----------------------------------------------------------------------
    // set_index_details test
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_index_details_on_r_equality() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // R_equality should have index details set during stock
        let r_eq = &bp_registry[R_EQUALITY];
        assert_eq!(r_eq.term_details[0].index_term_as, Some("value".to_string()));
        assert_eq!(r_eq.term_details[1].index_term_as, Some("value".to_string()));
    }

    // -----------------------------------------------------------------------
    // Family method dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_describe_for_problems_dispatch_via_family() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let desc = BinaryPredicateFamilies::describe_for_problems(
            &bp_registry[R_EQUALITY],
            &families,
        );
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_for_index_dispatch_via_family() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let desc = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[R_EQUALITY],
            &families,
        );
        assert_eq!(desc, "equality");
    }

    #[test]
    fn test_describe_empty_for_problems_dispatch_via_family() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let desc = BinaryPredicateFamilies::describe_for_problems(
            &bp_registry[R_EMPTY],
            &families,
        );
        assert_eq!(desc, "");
    }

    #[test]
    fn test_describe_empty_for_index_dispatch_via_family() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let desc = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[R_EMPTY],
            &families,
        );
        assert_eq!(desc, "never-holding");
    }

    #[test]
    fn test_spatial_family_has_no_describe_methods_falls_back_to_default() {
        let (mut families, mut bp_registry) = EqualityRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // The spatial family has no describe methods, so it falls back to
        // the default: the relation name
        let desc_problems = BinaryPredicateFamilies::describe_for_problems(
            &bp_registry[1], // a_has_b_predicate
            &families,
        );
        assert_eq!(desc_problems, "possession");

        let desc_index = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[1], // a_has_b_predicate
            &families,
        );
        assert_eq!(desc_index, "possession");
    }
}
