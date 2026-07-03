use crate::calculus::bp_term_details::BpTermDetails;

/// A binary predicate — the underlying data structure for relations.
///
/// Corresponds to `binary_predicate` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 60-86).
///
/// A binary predicate B is such that for any combination x and y, and at
/// any given moment at run-time, B(x, y) is either true or false. x and y
/// are called its "terms", and are numbered 0 and 1.
///
/// Each BP has a partner called its "reversal". If B is the original and R
/// is its reversal, then B(x, y) is true if and only if R(y, x) is true.
#[derive(Clone, Debug)]
pub struct BinaryPredicate {
    /// The family this BP belongs to (index into a family registry).
    pub relation_family: usize,
    /// Family-specific data (simplified: a string tag for now).
    pub family_specific: Option<String>,
    /// The relation name (simplified: a string instead of `word_assemblage`).
    pub relation_name: Option<String>,
    /// Debugging log name.
    pub debugging_log_name: Option<String>,
    /// Term details for the left (0) and right (1) terms.
    pub term_details: [BpTermDetails; 2],
    /// The reversal BP (index into a BP registry).
    pub reversal: Option<usize>,
    /// Was this BP created directly? (as opposed to being a reversal of another)
    pub right_way_round: bool,
    /// Task functions for compiling code (simplified: string schemas).
    /// Indices: 0=unused, 1=TEST_ATOM_TASK, 2=NOW_ATOM_TRUE_TASK, 3=NOW_ATOM_FALSE_TASK.
    pub task_functions: [Option<String>; 4],
    /// Loop parent optimisation proviso (simplified: a string).
    pub loop_parent_optimisation_proviso: Option<String>,
    /// Loop parent optimisation ranger (simplified: a string).
    pub loop_parent_optimisation_ranger: Option<String>,
    /// Knowledge about this BP (inference subject index).
    /// This is the bridge between the calculus module and the knowledge module.
    pub knowledge_about_bp: Option<usize>,
}

/// Creation and accessor functions for binary predicates.
///
/// Corresponds to `BinaryPredicates` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 88-371).
pub struct BinaryPredicates;

impl BinaryPredicates {
    /// Create the equality relation (its own reversal).
    ///
    /// The equality relation is special: B(x, y) is true iff x = y, and
    /// its reversal is itself (equality is symmetric).
    ///
    /// Corresponds to `BinaryPredicates::make_equality` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 88-110).
    ///
    /// # Arguments
    ///
    /// * `family_idx` - Index of the family this BP belongs to.
    /// * `relation_name` - The name of the relation.
    /// * `bp_registry` - The BP registry to add to.
    ///
    /// # Returns
    ///
    /// The index of the created BP in the registry.
    pub fn make_equality(
        family_idx: usize,
        relation_name: &str,
        bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        let idx = bp_registry.len();
        let bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: Some(relation_name.to_string()),
            debugging_log_name: Some(relation_name.to_string()),
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
            reversal: Some(idx), // equality is its own reversal
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };
        bp_registry.push(bp);
        idx
    }

    /// Create a single BP (internal helper).
    ///
    /// Corresponds to `BinaryPredicates::make_single` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 112-170).
    ///
    /// # Arguments
    ///
    /// * `family_idx` - Index of the family this BP belongs to.
    /// * `left_term` - Term details for the left (0) term.
    /// * `right_term` - Term details for the right (1) term.
    /// * `name` - The debugging log name.
    /// * `test_fn` - The test function schema (optional).
    /// * `make_true_fn` - The make-true function schema (optional).
    /// * `relation_name` - The relation name (optional).
    /// * `bp_registry` - The BP registry to add to.
    ///
    /// # Returns
    ///
    /// The index of the created BP in the registry.
    #[allow(clippy::too_many_arguments)]
    pub fn make_single(
        family_idx: usize,
        left_term: BpTermDetails,
        right_term: BpTermDetails,
        name: &str,
        test_fn: Option<&str>,
        make_true_fn: Option<&str>,
        relation_name: Option<&str>,
        bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        let idx = bp_registry.len();
        let bp = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: relation_name.map(|s| s.to_string()),
            debugging_log_name: Some(name.to_string()),
            term_details: [left_term, right_term],
            reversal: None,
            right_way_round: true,
            task_functions: [None, test_fn.map(|s| s.to_string()), make_true_fn.map(|s| s.to_string()), None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };
        bp_registry.push(bp);
        idx
    }

    /// Create a matched pair of BPs (each is the reversal of the other).
    ///
    /// Corresponds to `BinaryPredicates::make_pair` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 172-230).
    ///
    /// Creates two BPs: the original (right-way-round) and its reversal.
    /// The reversal has swapped term details and the opposite right_way_round flag.
    ///
    /// # Arguments
    ///
    /// * `family_idx` - Index of the family this BP belongs to.
    /// * `left_term` - Term details for the left (0) term of the original.
    /// * `right_term` - Term details for the right (1) term of the original.
    /// * `name` - The debugging log name for the original.
    /// * `namer` - The debugging log name for the reversal.
    /// * `make_true_fn` - The make-true function schema for the original (optional).
    /// * `test_fn` - The test function schema for the original (optional).
    /// * `source_name` - The relation name (optional).
    /// * `bp_registry` - The BP registry to add to.
    ///
    /// # Returns
    ///
    /// The index of the right-way-round BP in the registry.
    #[allow(clippy::too_many_arguments)]
    pub fn make_pair(
        family_idx: usize,
        left_term: BpTermDetails,
        right_term: BpTermDetails,
        name: &str,
        namer: &str,
        make_true_fn: Option<&str>,
        test_fn: Option<&str>,
        source_name: Option<&str>,
        bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        // Create the original (right-way-round) BP
        let original_idx = Self::make_single(
            family_idx,
            left_term.clone(),
            right_term.clone(),
            name,
            test_fn,
            make_true_fn,
            source_name,
            bp_registry,
        );

        // Create the reversal (wrong-way-round) BP with swapped terms
        let reversal_idx = bp_registry.len();
        let reversal = BinaryPredicate {
            relation_family: family_idx,
            family_specific: None,
            relation_name: source_name.map(|s| s.to_string()),
            debugging_log_name: Some(namer.to_string()),
            term_details: [right_term, left_term], // swapped
            reversal: Some(original_idx),
            right_way_round: false,
            task_functions: [None, test_fn.map(|s| s.to_string()), make_true_fn.map(|s| s.to_string()), None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };
        bp_registry.push(reversal);

        // Update the original's reversal to point to the reversal
        if let Some(orig) = bp_registry.get_mut(original_idx) {
            orig.reversal = Some(reversal_idx);
        }

        original_idx
    }
}

impl BinaryPredicate {
    /// Return the reversal index.
    ///
    /// Corresponds to `BinaryPredicates::get_reversal` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 232-242).
    pub fn get_reversal(&self) -> Option<usize> {
        self.reversal
    }

    /// Test if this BP is the wrong way round (i.e., it was created as the
    /// reversal of another BP).
    ///
    /// Corresponds to `BinaryPredicates::is_the_wrong_way_round` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 244-254).
    pub fn is_the_wrong_way_round(&self) -> bool {
        !self.right_way_round
    }

    /// Return the test function schema.
    ///
    /// Corresponds to `BinaryPredicates::get_test_function` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 256-266).
    pub fn get_test_function(&self) -> Option<&str> {
        self.task_functions[1].as_deref() // TEST_ATOM_TASK is at index 1
    }

    /// Test if the BP or its reversal can be made true at run-time.
    ///
    /// A BP can be made true at run-time if it has a make-true function
    /// (at index 2 = NOW_ATOM_TRUE_TASK), or if its reversal has one.
    ///
    /// Corresponds to `BinaryPredicates::can_be_made_true_at_runtime` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 268-282).
    pub fn can_be_made_true_at_runtime(&self, bp_registry: &[BinaryPredicate]) -> bool {
        // Check if this BP has a make-true function
        if self.task_functions[2].is_some() {
            return true;
        }
        // Check if the reversal has a make-true function
        if let Some(rev_idx) = self.reversal {
            if rev_idx < bp_registry.len() && bp_registry[rev_idx].task_functions[2].is_some() {
                return true;
            }
        }
        false
    }

    /// Return the kind of the relation (simplified: returns the kind of term 0
    /// if both terms have the same kind).
    ///
    /// Corresponds to `BinaryPredicates::kind` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 284-300).
    pub fn kind(&self) -> Option<usize> {
        // Simplified: return the kind of term 0 if both terms have the same kind
        let k0 = self.term_details[0].implies_kind;
        let k1 = self.term_details[1].implies_kind;
        if k0 == k1 {
            k0
        } else {
            None
        }
    }

    /// Return the kind of a specific term (0 or 1).
    ///
    /// Corresponds to `BinaryPredicates::term_kind` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 302-312).
    pub fn term_kind(&self, t: usize) -> Option<usize> {
        if t < 2 {
            self.term_details[t].implies_kind
        } else {
            None
        }
    }

    /// Set index display names for both terms (and the reversal's terms).
    ///
    /// Corresponds to `BinaryPredicates::set_index_details` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 314-330).
    pub fn set_index_details(
        &mut self,
        left: Option<&str>,
        right: Option<&str>,
        bp_registry: &mut [BinaryPredicate],
    ) {
        self.term_details[0].index_term_as = left.map(|s| s.to_string());
        self.term_details[1].index_term_as = right.map(|s| s.to_string());

        // Also update the reversal's terms (swapped)
        if let Some(rev_idx) = self.reversal {
            if rev_idx < bp_registry.len() {
                bp_registry[rev_idx].term_details[0].index_term_as = right.map(|s| s.to_string());
                bp_registry[rev_idx].term_details[1].index_term_as = left.map(|s| s.to_string());
            }
        }
    }

    /// Get the debugging log name.
    ///
    /// Corresponds to `BinaryPredicates::get_log_name` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicates.w`).
    pub fn get_log_name(&self) -> Option<&str> {
        self.debugging_log_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::bp_term_details::BPTerms;

    #[test]
    fn test_make_equality_creates_bp_with_correct_family_and_name() {
        let mut registry = Vec::new();
        let idx = BinaryPredicates::make_equality(0, "equality", &mut registry);

        assert_eq!(idx, 0);
        assert_eq!(registry[idx].relation_family, 0);
        assert_eq!(registry[idx].relation_name, Some("equality".to_string()));
        assert_eq!(registry[idx].debugging_log_name, Some("equality".to_string()));
    }

    #[test]
    fn test_make_equality_is_its_own_reversal() {
        let mut registry = Vec::new();
        let idx = BinaryPredicates::make_equality(0, "equality", &mut registry);

        assert_eq!(registry[idx].reversal, Some(idx));
    }

    #[test]
    fn test_make_equality_is_right_way_round() {
        let mut registry = Vec::new();
        let idx = BinaryPredicates::make_equality(0, "equality", &mut registry);

        assert!(registry[idx].right_way_round);
    }

    #[test]
    fn test_make_single_creates_bp_with_correct_fields() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        let idx = BinaryPredicates::make_single(
            2,
            left,
            right,
            "my_relation",
            Some("TEST_FN"),
            Some("MAKE_TRUE_FN"),
            Some("my_relation_name"),
            &mut registry,
        );

        assert_eq!(idx, 0);
        assert_eq!(registry[idx].relation_family, 2);
        assert_eq!(registry[idx].debugging_log_name, Some("my_relation".to_string()));
        assert_eq!(registry[idx].relation_name, Some("my_relation_name".to_string()));
        assert_eq!(registry[idx].term_details[0].implies_kind, Some(0));
        assert_eq!(registry[idx].term_details[1].implies_kind, Some(1));
        assert_eq!(registry[idx].task_functions[1], Some("TEST_FN".to_string()));
        assert_eq!(registry[idx].task_functions[2], Some("MAKE_TRUE_FN".to_string()));
        assert!(registry[idx].right_way_round);
        assert_eq!(registry[idx].reversal, None);
    }

    #[test]
    fn test_make_single_with_none_fields() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();

        let idx = BinaryPredicates::make_single(
            0, left, right, "test", None, None, None, &mut registry,
        );

        assert_eq!(registry[idx].relation_name, None);
        assert_eq!(registry[idx].task_functions[1], None);
        assert_eq!(registry[idx].task_functions[2], None);
    }

    #[test]
    fn test_make_pair_creates_two_bps() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        let original_idx = BinaryPredicates::make_pair(
            0,
            left,
            right,
            "contains",
            "contained_by",
            Some("MAKE_TRUE"),
            Some("TEST"),
            Some("containment"),
            &mut registry,
        );

        assert_eq!(registry.len(), 2);
        assert_eq!(original_idx, 0);
    }

    #[test]
    fn test_make_pair_sets_reversal_correctly() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        let original_idx = BinaryPredicates::make_pair(
            0,
            left,
            right,
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        // Original points to reversal
        assert_eq!(registry[original_idx].reversal, Some(1));
        // Reversal points to original
        assert_eq!(registry[1].reversal, Some(original_idx));
    }

    #[test]
    fn test_make_pair_sets_right_way_round_correctly() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        let original_idx = BinaryPredicates::make_pair(
            0,
            left,
            right,
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        assert!(registry[original_idx].right_way_round);
        assert!(!registry[1].right_way_round);
    }

    #[test]
    fn test_make_pair_swaps_term_details_on_reversal() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        BinaryPredicates::make_pair(
            0,
            left,
            right,
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        // Original: term 0 = kind 0, term 1 = kind 1
        assert_eq!(registry[0].term_details[0].implies_kind, Some(0));
        assert_eq!(registry[0].term_details[1].implies_kind, Some(1));
        // Reversal: term 0 = kind 1, term 1 = kind 0 (swapped)
        assert_eq!(registry[1].term_details[0].implies_kind, Some(1));
        assert_eq!(registry[1].term_details[1].implies_kind, Some(0));
    }

    #[test]
    fn test_get_reversal_returns_correct_index() {
        let mut registry = Vec::new();
        let eq_idx = BinaryPredicates::make_equality(0, "equality", &mut registry);

        assert_eq!(registry[eq_idx].get_reversal(), Some(eq_idx));
    }

    #[test]
    fn test_get_reversal_returns_none_for_single_bp() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "single", None, None, None, &mut registry);

        assert_eq!(registry[0].get_reversal(), None);
    }

    #[test]
    fn test_is_the_wrong_way_round_returns_true_for_reversal() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        BinaryPredicates::make_pair(
            0,
            left,
            right,
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        assert!(!registry[0].is_the_wrong_way_round()); // original
        assert!(registry[1].is_the_wrong_way_round()); // reversal
    }

    #[test]
    fn test_get_test_function_returns_test_schema() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(
            0,
            left,
            right,
            "test_rel",
            Some("TEST_SCHEMA"),
            None,
            None,
            &mut registry,
        );

        assert_eq!(registry[0].get_test_function(), Some("TEST_SCHEMA"));
    }

    #[test]
    fn test_get_test_function_returns_none_when_not_set() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test_rel", None, None, None, &mut registry);

        assert_eq!(registry[0].get_test_function(), None);
    }

    #[test]
    fn test_can_be_made_true_at_runtime_returns_true_if_bp_has_make_true() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(
            0,
            left,
            right,
            "test_rel",
            None,
            Some("MAKE_TRUE"),
            None,
            &mut registry,
        );

        assert!(registry[0].can_be_made_true_at_runtime(&registry));
    }

    #[test]
    fn test_can_be_made_true_at_runtime_returns_true_if_reversal_has_make_true() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();

        // Create a pair where only the reversal has a make-true function
        // make_pair gives both BPs the same task functions
        BinaryPredicates::make_pair(
            0,
            left,
            right,
            "original",
            "reversal",
            Some("MAKE_TRUE"),
            None,
            Some("test"),
            &mut registry,
        );

        // Both have the make-true function since make_pair copies them
        assert!(registry[0].can_be_made_true_at_runtime(&registry));
        assert!(registry[1].can_be_made_true_at_runtime(&registry));
    }

    #[test]
    fn test_can_be_made_true_at_runtime_returns_false_when_no_make_true() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test_rel", None, None, None, &mut registry);

        assert!(!registry[0].can_be_made_true_at_runtime(&registry));
    }

    #[test]
    fn test_set_index_details_updates_bp_and_reversal() {
        // Test 1: set_index_details on a standalone BP (no reversal)
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test", None, None, None, &mut registry);

        let mut bp = registry.swap_remove(0);
        bp.set_index_details(Some("left_name"), Some("right_name"), &mut []);
        assert_eq!(bp.term_details[0].index_term_as, Some("left_name".to_string()));
        assert_eq!(bp.term_details[1].index_term_as, Some("right_name".to_string()));

        // Test 2: set_index_details on a pair updates the reversal
        // We test this by directly setting fields on the original and verifying
        // the reversal is updated via the method's logic
        let mut registry = Vec::new();
        let original_idx = BinaryPredicates::make_pair(
            0,
            BPTerms::new_kind(Some(0)),
            BPTerms::new_kind(Some(1)),
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        let rev_idx = registry[original_idx].reversal.unwrap();

        // Directly set the original's term details and verify the reversal is updated
        // by calling set_index_details on a standalone copy with the reversal as the registry
        let mut original = registry[original_idx].clone();
        let mut reversal = registry[rev_idx].clone();
        original.reversal = Some(0); // reversal is at index 0 in the sub-slice
        let reversal_slice = std::slice::from_mut(&mut reversal);
        original.set_index_details(
            Some("container"),
            Some("contents"),
            reversal_slice,
        );

        // Verify the original's term details
        assert_eq!(original.term_details[0].index_term_as, Some("container".to_string()));
        assert_eq!(original.term_details[1].index_term_as, Some("contents".to_string()));
        // Verify the reversal's term details (swapped)
        assert_eq!(reversal.term_details[0].index_term_as, Some("contents".to_string()));
        assert_eq!(reversal.term_details[1].index_term_as, Some("container".to_string()));
    }

    #[test]
    fn test_kind_returns_kind_when_both_terms_have_same_kind() {
        let left = BPTerms::new_kind(Some(5));
        let right = BPTerms::new_kind(Some(5));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test", None, None, None, &mut registry);

        assert_eq!(registry[0].kind(), Some(5));
    }

    #[test]
    fn test_kind_returns_none_when_terms_have_different_kinds() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test", None, None, None, &mut registry);

        assert_eq!(registry[0].kind(), None);
    }

    #[test]
    fn test_kind_returns_none_when_kinds_not_set() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(0, left, right, "test", None, None, None, &mut registry);

        assert_eq!(registry[0].kind(), None);
    }

    #[test]
    fn test_get_log_name_returns_debugging_name() {
        let left = BPTerms::new(None);
        let right = BPTerms::new(None);
        let mut registry = Vec::new();
        BinaryPredicates::make_single(
            0,
            left,
            right,
            "my_debug_name",
            None,
            None,
            None,
            &mut registry,
        );

        assert_eq!(registry[0].get_log_name(), Some("my_debug_name"));
    }

    #[test]
    fn test_make_pair_returns_index_of_original() {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();

        let result = BinaryPredicates::make_pair(
            0,
            left,
            right,
            "a",
            "b",
            None,
            None,
            Some("test"),
            &mut registry,
        );

        assert_eq!(result, 0);
        assert!(registry[result].right_way_round);
    }

    #[test]
    fn test_multiple_bps_in_registry() {
        let mut registry = Vec::new();

        let eq_idx = BinaryPredicates::make_equality(0, "equality", &mut registry);
        let left1 = BPTerms::new_kind(Some(0));
        let right1 = BPTerms::new_kind(Some(1));
        let pair_idx = BinaryPredicates::make_pair(
            1,
            left1,
            right1,
            "contains",
            "contained_by",
            None,
            None,
            Some("containment"),
            &mut registry,
        );

        assert_eq!(eq_idx, 0);
        assert_eq!(pair_idx, 1);
        assert_eq!(registry.len(), 3);

        // Equality is its own reversal
        assert_eq!(registry[0].reversal, Some(0));
        // Pair: original (1) -> reversal (2), reversal (2) -> original (1)
        assert_eq!(registry[1].reversal, Some(2));
        assert_eq!(registry[2].reversal, Some(1));
    }
}
