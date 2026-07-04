/// The Explicit Relations system — binary predicate families for relations
/// created explicitly by the source text.
///
/// Corresponds to `ExplicitRelations` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`).
///
/// Creates two bp_families:
/// - `explicit_bp_family` — for relations defined explicitly in source text
///   (e.g., "X is adjacent to Y", "X contains Y", "X loves Y")
/// - `by_function_bp_family` — for relations defined by a function
///
/// The `ExplicitBpData` struct stores family-specific data for each BP:
/// - `form_of_relation` — one of the `Relation_*` constants
/// - `i6_storage_property` — run-time storage property
/// - `store_dynamically` — whether to store dynamically
///
/// Simplified:
/// - No `RelationInferences::draw_spec` (assert returns FALSE)
/// - No `RelationInferences::draw` (assert returns FALSE)
/// - No `PropertyInferences::draw` (infer_property_based_relation is a stub)
/// - No `StandardProblems::sentence_problem` (no problem messages)
/// - No `BinaryPredicates::kind` (relates_values_not_objects uses simplified logic)
/// - No `PreformUtilities::wording` (uses string names)
/// - No `WordAssemblages::first_word` (uses string name directly)
use crate::calculus::binary_predicate_families::BpFamily;
use crate::calculus::binary_predicate_families::BpFamilyMethods;
use crate::calculus::binary_predicates::BinaryPredicate;
use crate::calculus::binary_predicates::BinaryPredicates;
use crate::knowledge::properties::Property;
use crate::calculus::bp_term_details::BPTerms;

// ---------------------------------------------------------------------------
// Relation_* constants — form of a relation
// ---------------------------------------------------------------------------

/// None of the below — no explicit form.
///
/// Corresponds to `Relation_Implicit` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 70).
pub const RELATION_IMPLICIT: i8 = -1;

/// One to one: "R relates one K to one K".
///
/// Corresponds to `Relation_OtoO` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 71).
pub const RELATION_OTO_O: i8 = 1;

/// One to various: "R relates one K to various K".
///
/// Corresponds to `Relation_OtoV` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 72).
pub const RELATION_OTO_V: i8 = 2;

/// Various to one: "R relates various K to one K".
///
/// Corresponds to `Relation_VtoO` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 73).
pub const RELATION_VTO_O: i8 = 3;

/// Various to various: "R relates various K to various K".
///
/// Corresponds to `Relation_VtoV` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 74).
pub const RELATION_VTO_V: i8 = 4;

/// Symmetric one to one: "R relates one K to another".
///
/// Corresponds to `Relation_Sym_OtoO` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 75).
pub const RELATION_SYM_OTO_O: i8 = 5;

/// Symmetric various to various: "R relates K to each other".
///
/// Corresponds to `Relation_Sym_VtoV` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 76).
pub const RELATION_SYM_VTO_V: i8 = 6;

/// Equivalence relation: "R relates K to each other in groups".
///
/// Corresponds to `Relation_Equiv` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, line 77).
pub const RELATION_EQUIV: i8 = 7;

// ---------------------------------------------------------------------------
// Global constants for family indices
// ---------------------------------------------------------------------------

/// Index of the explicit family in the family registry.
///
/// This assumes the equality relation (PLAN-22) has already created families 0-2,
/// the quasinumeric relation (PLAN-36) has created family 3,
/// and the universal relation (PLAN-37) has created family 4.
/// The explicit family is family 5.
pub const EXPLICIT_FAMILY: usize = 5;

/// Index of the by-function family in the family registry.
///
/// This assumes the equality relation (PLAN-22) has already created families 0-2,
/// the quasinumeric relation (PLAN-36) has created family 3,
/// the universal relation (PLAN-37) has created family 4,
/// and the explicit family is family 5.
/// The by-function family is family 6.
pub const BY_FUNCTION_FAMILY: usize = 6;

// ---------------------------------------------------------------------------
// ExplicitBpData — family-specific data for each BP
// ---------------------------------------------------------------------------

/// Family-specific data for explicit and by-function binary predicates.
///
/// Corresponds to `explicit_bp_data` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 39-45).
///
/// Stores the form of relation, the I6 storage property, and whether
/// the relation is stored dynamically.
///
/// Simplified:
/// - No `equiv_data` (equivalence data, only used for Relation_Equiv)
/// - No `v2v_bitmap_iname` (bitmap name, only used for VtoV relations)
#[derive(Clone, Debug, Default)]
pub struct ExplicitBpData {
    /// One of the `Relation_*` constants.
    ///
    /// Corresponds to `form_of_relation` in the C reference.
    pub form_of_relation: i8,
    /// Run-time storage property index (if any).
    ///
    /// Corresponds to `i6_storage_property` in the C reference.
    pub i6_storage_property: Option<usize>,
    /// Whether to store this relation dynamically.
    ///
    /// Corresponds to `store_dynamically` in the C reference.
    pub store_dynamically: bool,
}

impl ExplicitBpData {
    /// Create a new `ExplicitBpData` with default values.
    ///
    /// Defaults:
    /// - `form_of_relation`: `RELATION_OTO_O` (one-to-one)
    /// - `i6_storage_property`: `None`
    /// - `store_dynamically`: `false`
    ///
    /// Corresponds to the initialization in `ExplicitRelations::make_pair_sketchily`
    /// in the C reference (`inform7/assertions-module/Chapter 8/Explicit Relations.w`,
    /// lines 199-206).
    pub fn new() -> Self {
        ExplicitBpData {
            form_of_relation: RELATION_OTO_O,
            i6_storage_property: None,
            store_dynamically: false,
        }
    }
}

// ---------------------------------------------------------------------------
// ExplicitRelations module
// ---------------------------------------------------------------------------

/// The explicit relations module.
///
/// Corresponds to `ExplicitRelations` in the C reference
/// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`).
pub struct ExplicitRelations;

impl ExplicitRelations {
    /// Create the explicit and by-function bp_families with their methods.
    ///
    /// Corresponds to `ExplicitRelations::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 51-64).
    ///
    /// Creates two families:
    /// - `explicit_bp_family` (index `EXPLICIT_FAMILY`) with typecheck, assert,
    ///   schema, describe_for_problems, and describe_for_index methods
    /// - `by_function_bp_family` (index `BY_FUNCTION_FAMILY`) with typecheck, assert,
    ///   schema, describe_for_problems, and a brief describe_for_index method
    ///
    /// # Arguments
    ///
    /// * `families` - The family registry to add to.
    /// * `_bp_registry` - The BP registry to add to (unused by start).
    ///
    /// # Returns
    ///
    /// The index of the explicit family in the registry.
    #[allow(clippy::ptr_arg)]
    pub fn start(
        families: &mut Vec<BpFamily>,
        _bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        // Create the explicit bp_family with all five methods.
        // Corresponds to lines 52-57 of the C reference.
        let explicit_idx = families.len();
        let explicit_family = BpFamily {
            name: "explicit",
            methods: BpFamilyMethods {
                stock: None,
                typecheck: Some(ExplicitRelations::typecheck),
                assert: Some(ExplicitRelations::assert),
                schema: Some(ExplicitRelations::schema),
                describe_for_problems: Some(ExplicitRelations::describe_for_problems),
                describe_for_index: Some(ExplicitRelations::describe_for_index),
            },
        };
        families.push(explicit_family);

        // Create the by-function bp_family with all five methods.
        // The by-function family uses REL_br_describe_briefly for describe_for_index.
        // Corresponds to lines 58-64 of the C reference.
        let _by_function_idx = families.len();
        let by_function_family = BpFamily {
            name: "by-function",
            methods: BpFamilyMethods {
                stock: None,
                typecheck: Some(ExplicitRelations::typecheck),
                assert: Some(ExplicitRelations::assert),
                schema: Some(ExplicitRelations::schema),
                describe_for_problems: Some(ExplicitRelations::describe_for_problems),
                describe_for_index: Some(ExplicitRelations::rel_describe_briefly),
            },
        };
        families.push(by_function_family);

        explicit_idx
    }

    /// Typecheck the terms of an explicit or by-function relation.
    ///
    /// Corresponds to `ExplicitRelations::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 82-87).
    ///
    /// Returns `DECLINE_TO_MATCH` (-1) to use default typechecking.
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        -1 // DECLINE_TO_MATCH
    }

    /// Assert an explicit or by-function relation.
    ///
    /// Corresponds to `ExplicitRelations::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 92-115).
    ///
    /// Simplified: always returns FALSE. The full C implementation handles
    /// dynamic storage, arbitrary assertions, and property-based relations.
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

    /// Compile run-time code for an explicit or by-function relation.
    ///
    /// Corresponds to `ExplicitRelations::schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 120-124).
    ///
    /// Returns FALSE (use default schemas).
    pub fn schema(
        _family: &BpFamily,
        _task: u8,
        _bp: &BinaryPredicate,
    ) -> bool {
        false
    }

    /// Describe the relation in problem messages.
    ///
    /// Corresponds to `ExplicitRelations::describe_for_problems` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 129-131).
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
    /// Corresponds to `ExplicitRelations::describe_for_index` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 132-142).
    ///
    /// Returns a string describing the form of the relation based on the
    /// `Relation_*` constant stored in the BP's family-specific data.
    ///
    /// Note: This is a function pointer used by BpFamilyMethods and cannot
    /// access the explicit data store. It returns a default description for
    /// explicit family BPs.
    pub fn describe_for_index(
        _family: &BpFamily,
        bp: &BinaryPredicate,
    ) -> String {
        if bp.relation_family == EXPLICIT_FAMILY {
            "one-to-one".to_string()
        } else {
            String::new()
        }
    }

    /// Describe the relation briefly in the Phrasebook index (for by-function family).
    ///
    /// Corresponds to `ExplicitRelations::REL_br_describe_briefly` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 143-145).
    pub fn rel_describe_briefly(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
    ) -> String {
        "defined".to_string()
    }

    // -----------------------------------------------------------------------
    // Helper functions
    // -----------------------------------------------------------------------

    /// Check if a binary predicate belongs to the explicit family.
    ///
    /// Corresponds to `ExplicitRelations::is_explicit_with_runtime_storage` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 151-154).
    ///
    /// Simplified: checks if the BP belongs to the explicit family.
    /// The full C implementation also checks for runtime storage.
    pub fn is_explicit_with_runtime_storage(bp: &BinaryPredicate) -> bool {
        bp.relation_family == EXPLICIT_FAMILY
    }

    /// Check if the relation form allows arbitrary assertions.
    ///
    /// Corresponds to `ExplicitRelations::allow_arbitrary_assertions` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 156-162).
    ///
    /// Returns TRUE for equivalence, various-to-various, and symmetric various-to-various
    /// relations.
    pub fn allow_arbitrary_assertions(
        bp_idx: usize,
        bp_registry: &[BinaryPredicate],
        explicit_data: &[ExplicitBpData],
    ) -> bool {
        let f = Self::get_form_of_relation(bp_idx, bp_registry, explicit_data);
        f == RELATION_EQUIV || f == RELATION_VTO_V || f == RELATION_SYM_VTO_V
    }

    /// Set the store_dynamically flag on an explicit BP.
    ///
    /// Corresponds to `ExplicitRelations::store_dynamically` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 164-169).
    ///
    /// # Panics
    ///
    /// Panics if the BP is not in the explicit family (equivalent to `internal_error` in C).
    /// Panics if the BP index is out of range for the explicit data store.
    pub fn store_dynamically(
        bp_idx: usize,
        bp_registry: &mut [BinaryPredicate],
        explicit_data: &mut [ExplicitBpData],
    ) {
        if let Some(bp) = bp_registry.get(bp_idx) {
            if bp.relation_family != EXPLICIT_FAMILY {
                panic!("not explicit");
            }
        }
        let data = explicit_data.get_mut(bp_idx).expect("BP index out of range for explicit data");
        data.store_dynamically = true;
    }

    /// Check if an explicit BP is stored dynamically.
    ///
    /// Corresponds to `ExplicitRelations::stored_dynamically` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 171-177).
    pub fn stored_dynamically(
        bp_idx: usize,
        bp_registry: &[BinaryPredicate],
        explicit_data: &[ExplicitBpData],
    ) -> bool {
        if bp_registry
            .get(bp_idx)
            .map(|bp| bp.relation_family == EXPLICIT_FAMILY)
            .unwrap_or(false)
        {
            explicit_data
                .get(bp_idx)
                .map(|data| data.store_dynamically)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Check if a relation relates values (not objects).
    ///
    /// Corresponds to `ExplicitRelations::relates_values_not_objects` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 179-189).
    ///
    /// Simplified: always returns FALSE. The full C implementation checks
    /// the kinds of the BP's terms to determine if they are object kinds.
    pub fn relates_values_not_objects(
        _bp: &BinaryPredicate,
        _bp_registry: &[BinaryPredicate],
    ) -> bool {
        false
    }

    /// Create a sketchy BP pair with default explicit_bp_data.
    ///
    /// Corresponds to `ExplicitRelations::make_pair_sketchily` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 191-208).
    ///
    /// Creates a pair of BPs (original and reversal) with open term details
    /// and default `ExplicitBpData` (form_of_relation = RELATION_OTO_O,
    /// i6_storage_property = None, store_dynamically = false).
    ///
    /// # Arguments
    ///
    /// * `name` - The relation name.
    /// * `bp_registry` - The BP registry to add to.
    /// * `explicit_data` - The explicit data store to add to.
    ///
    /// # Returns
    ///
    /// The index of the right-way-round BP in the registry.
    pub fn make_pair_sketchily(
        name: &str,
        bp_registry: &mut Vec<BinaryPredicate>,
        explicit_data: &mut Vec<ExplicitBpData>,
    ) -> usize {
        let open_term = BPTerms::new(None);

        let bp_idx = BinaryPredicates::make_pair(
            EXPLICIT_FAMILY,
            open_term.clone(),
            open_term,
            name,
            &format!("{}-rev", name),
            None,
            None,
            Some(name),
            bp_registry,
        );

        // Create default ExplicitBpData for the original and its reversal.
        // Corresponds to lines 199-206 of the C reference.
        let data = ExplicitBpData::new();

        // Ensure the explicit_data store is large enough.
        while explicit_data.len() <= bp_idx + 1 {
            explicit_data.push(ExplicitBpData::new());
        }

        // Set the data for both the original and its reversal.
        // In the C code, both the original and reversal share the same
        // explicit_bp_data pointer.
        explicit_data[bp_idx] = data.clone();
        explicit_data[bp_idx + 1] = data;

        bp_idx
    }
    /// Get the form of relation for a BP.
    ///
    /// Corresponds to `ExplicitRelations::get_form_of_relation` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 216-220).
    ///
    /// Returns `RELATION_IMPLICIT` if the BP is not in the explicit family
    /// or if the index is out of range.
    pub fn get_form_of_relation(
        bp_idx: usize,
        bp_registry: &[BinaryPredicate],
        explicit_data: &[ExplicitBpData],
    ) -> i8 {
        if bp_idx < bp_registry.len()
            && bp_registry[bp_idx].relation_family == EXPLICIT_FAMILY
            && bp_idx < explicit_data.len()
        {
            explicit_data[bp_idx].form_of_relation
        } else if bp_idx < bp_registry.len()
            && bp_registry[bp_idx].relation_family == BY_FUNCTION_FAMILY
        {
            // By-function family BPs default to RELATION_OTO_O
            RELATION_OTO_O
        } else {
            RELATION_IMPLICIT
        }
    }
    /// Convert the form of relation to a text string.
    ///
    /// Corresponds to `ExplicitRelations::form_to_text` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 222-233).
    pub fn form_to_text(
        bp_idx: usize,
        bp_registry: &[BinaryPredicate],
        explicit_data: &[ExplicitBpData],
    ) -> &'static str {
        match Self::get_form_of_relation(bp_idx, bp_registry, explicit_data) {
            RELATION_OTO_O => "Relation_OtoO",
            RELATION_OTO_V => "Relation_OtoV",
            RELATION_VTO_O => "Relation_VtoO",
            RELATION_VTO_V => "Relation_VtoV",
            RELATION_SYM_OTO_O => "Relation_Sym_OtoO",
            RELATION_SYM_VTO_V => "Relation_Sym_VtoV",
            RELATION_EQUIV => "Relation_Equiv",
            _ => "Relation_Implicit",
        }
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

    /// Helper: create an explicit relations setup with equality, quasinumeric,
    /// and universal relations pre-loaded.
    ///
    /// This ensures the explicit family is at index 5 (EXPLICIT_FAMILY)
    /// and the by-function family is at index 6 (BY_FUNCTION_FAMILY).
    fn setup() -> (Vec<BpFamily>, Vec<BinaryPredicate>, Vec<ExplicitBpData>, usize) {
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        let explicit_data = Vec::new();
        // Pre-populate with 5 dummy families to match equality + quasinumeric + universal layout
        families.push(BpFamily::new("dummy0"));
        families.push(BpFamily::new("dummy1"));
        families.push(BpFamily::new("dummy2"));
        families.push(BpFamily::new("dummy3"));
        families.push(BpFamily::new("dummy4"));
        // Pre-populate with 16 dummy BPs to match equality + quasinumeric + universal layout
        for i in 0..16 {
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
        let family_idx = ExplicitRelations::start(&mut families, &mut bp_registry);
        (families, bp_registry, explicit_data, family_idx)
    }

    // -----------------------------------------------------------------------
    // Relation_* constants tests
    // -----------------------------------------------------------------------

    /// Test that all Relation_* constants have the correct values.
    #[test]
    fn test_relation_constants() {
        assert_eq!(RELATION_IMPLICIT, -1);
        assert_eq!(RELATION_OTO_O, 1);
        assert_eq!(RELATION_OTO_V, 2);
        assert_eq!(RELATION_VTO_O, 3);
        assert_eq!(RELATION_VTO_V, 4);
        assert_eq!(RELATION_SYM_OTO_O, 5);
        assert_eq!(RELATION_SYM_VTO_V, 6);
        assert_eq!(RELATION_EQUIV, 7);
    }

    // -----------------------------------------------------------------------
    // Family index constants tests
    // -----------------------------------------------------------------------

    /// Test that family index constants have the correct values.
    #[test]
    fn test_family_index_constants() {
        assert_eq!(EXPLICIT_FAMILY, 5);
        assert_eq!(BY_FUNCTION_FAMILY, 6);
    }

    // -----------------------------------------------------------------------
    // ExplicitBpData tests
    // -----------------------------------------------------------------------

    /// Test that ExplicitBpData::new() creates data with correct defaults.
    #[test]
    fn test_explicit_bp_data_defaults() {
        let data = ExplicitBpData::new();
        assert_eq!(data.form_of_relation, RELATION_OTO_O);
        assert_eq!(data.i6_storage_property, None);
        assert!(!data.store_dynamically);
    }

    /// Test that ExplicitBpData can be customized.
    #[test]
    fn test_explicit_bp_data_custom() {
        let data = ExplicitBpData {
            form_of_relation: RELATION_EQUIV,
            i6_storage_property: Some(42),
            store_dynamically: true,
        };
        assert_eq!(data.form_of_relation, RELATION_EQUIV);
        assert_eq!(data.i6_storage_property, Some(42));
        assert!(data.store_dynamically);
    }

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

    /// Test that `start()` creates both families.
    #[test]
    fn test_start_creates_two_families() {
        let (families, _, _, family_idx) = setup();

        // Should have 7 families (5 dummies + explicit + by-function)
        assert_eq!(families.len(), 7);
        assert_eq!(family_idx, EXPLICIT_FAMILY);
    }

    /// Test that the explicit family has the correct name and methods.
    #[test]
    fn test_explicit_family_has_correct_name_and_methods() {
        let (families, _, _, _) = setup();

        let explicit = &families[EXPLICIT_FAMILY];
        assert_eq!(explicit.name, "explicit");
        assert!(explicit.methods.typecheck.is_some());
        assert!(explicit.methods.assert.is_some());
        assert!(explicit.methods.schema.is_some());
        assert!(explicit.methods.describe_for_problems.is_some());
        assert!(explicit.methods.describe_for_index.is_some());
        // No stock method
        assert!(explicit.methods.stock.is_none());
    }

    /// Test that the by-function family has the correct name and methods.
    #[test]
    fn test_by_function_family_has_correct_name_and_methods() {
        let (families, _, _, _) = setup();

        let by_function = &families[BY_FUNCTION_FAMILY];
        assert_eq!(by_function.name, "by-function");
        assert!(by_function.methods.typecheck.is_some());
        assert!(by_function.methods.assert.is_some());
        assert!(by_function.methods.schema.is_some());
        assert!(by_function.methods.describe_for_problems.is_some());
        assert!(by_function.methods.describe_for_index.is_some());
        // No stock method
        assert!(by_function.methods.stock.is_none());
    }

    // -----------------------------------------------------------------------
    // typecheck() tests
    // -----------------------------------------------------------------------

    /// Test that typecheck returns DECLINE_TO_MATCH.
    #[test]
    fn test_typecheck_returns_decline_to_match() {
        let (families, bp_registry, _, _) = setup();

        let result = ExplicitRelations::typecheck(
            &families[EXPLICIT_FAMILY],
            &bp_registry[0],
            &[],
            &[],
        );
        assert_eq!(result, -1); // DECLINE_TO_MATCH
    }

    /// Test that typecheck dispatch via family returns DECLINE_TO_MATCH.
    #[test]
    fn test_typecheck_dispatch_via_family() {
        let (families, bp_registry, _, _) = setup();

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[0],
            &[],
            &[],
            &families,
        );
        // The dummy BP (family 0) has no typecheck method, so it returns DECLINE_TO_MATCH
        assert_eq!(result, -1);
    }

    // -----------------------------------------------------------------------
    // assert() tests
    // -----------------------------------------------------------------------

    /// Test that assert returns FALSE.
    #[test]
    fn test_assert_returns_false() {
        let (families, bp_registry, _, _) = setup();

        let result = ExplicitRelations::assert(
            &families[EXPLICIT_FAMILY],
            &bp_registry[0],
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
        let (families, bp_registry, _, _) = setup();

        let result = ExplicitRelations::schema(
            &families[EXPLICIT_FAMILY],
            0,
            &bp_registry[0],
        );
        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // describe_for_problems() tests
    // -----------------------------------------------------------------------

    /// Test that describe_for_problems returns an empty string.
    #[test]
    fn test_describe_for_problems_returns_empty() {
        let (families, bp_registry, _, _) = setup();

        let result = ExplicitRelations::describe_for_problems(
            &families[EXPLICIT_FAMILY],
            &bp_registry[0],
        );
        assert_eq!(result, "");
    }

    // -----------------------------------------------------------------------
    // describe_for_index() tests
    // -----------------------------------------------------------------------

    /// Test that describe_for_index returns the form description for the explicit family.
    #[test]
    fn test_describe_for_index_explicit_family() {
        let (families, _, _, _) = setup();

        // Create a BP in the explicit family
        let bp = BinaryPredicate {
            relation_family: EXPLICIT_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = ExplicitRelations::describe_for_index(
            &families[EXPLICIT_FAMILY],
            &bp,
        );
        // Default form is RELATION_OTO_O, so it should return "one-to-one"
        assert_eq!(result, "one-to-one");
    }

    /// Test that rel_describe_briefly returns "defined".
    #[test]
    fn test_rel_describe_briefly() {
        let (families, _, _, _) = setup();

        let bp = BinaryPredicate {
            relation_family: BY_FUNCTION_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = ExplicitRelations::rel_describe_briefly(
            &families[BY_FUNCTION_FAMILY],
            &bp,
        );
        assert_eq!(result, "defined");
    }

    // -----------------------------------------------------------------------
    // Helper function tests
    // -----------------------------------------------------------------------

    /// Test is_explicit_with_runtime_storage for explicit family BPs.
    #[test]
    fn test_is_explicit_with_runtime_storage_true() {
        let bp = BinaryPredicate {
            relation_family: EXPLICIT_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        assert!(ExplicitRelations::is_explicit_with_runtime_storage(&bp));
    }

    /// Test allow_arbitrary_assertions for different relation forms.
    #[test]
    fn test_allow_arbitrary_assertions() {
        let (_, mut bp_registry, mut explicit_data, _) = setup();

        // Create a BP in the explicit family with default form (RELATION_OTO_O)
        let explicit_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: EXPLICIT_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });
        while explicit_data.len() <= explicit_idx {
            explicit_data.push(ExplicitBpData::new());
        }

        // Create a BP in the by-function family
        let by_function_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: BY_FUNCTION_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });

        // Create a non-explicit BP
        let other_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: 0,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });

        // Explicit family BPs with default form (RELATION_OTO_O) should NOT allow arbitrary assertions
        assert!(!ExplicitRelations::allow_arbitrary_assertions(explicit_idx, &bp_registry, &explicit_data));

        // By-function family BPs should NOT allow arbitrary assertions
        assert!(!ExplicitRelations::allow_arbitrary_assertions(by_function_idx, &bp_registry, &explicit_data));

        // Non-explicit BPs should NOT allow arbitrary assertions
        assert!(!ExplicitRelations::allow_arbitrary_assertions(other_idx, &bp_registry, &explicit_data));
    }

    /// Test store_dynamically and stored_dynamically.
    #[test]
    fn test_store_and_stored_dynamically() {
        let (_, mut bp_registry, mut explicit_data, _) = setup();

        // Create a BP in the explicit family
        let bp_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: EXPLICIT_FAMILY,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });

        // Ensure explicit_data is large enough
        while explicit_data.len() <= bp_idx {
            explicit_data.push(ExplicitBpData::new());
        }

        // Initially not stored dynamically
        assert!(!ExplicitRelations::stored_dynamically(bp_idx, &bp_registry, &explicit_data));

        // Set store_dynamically
        ExplicitRelations::store_dynamically(bp_idx, &mut bp_registry, &mut explicit_data);

        // Now it should be stored dynamically
        assert!(ExplicitRelations::stored_dynamically(bp_idx, &bp_registry, &explicit_data));
    }

    /// Test stored_dynamically returns false for non-explicit BPs.
    #[test]
    fn test_stored_dynamically_false_for_non_explicit() {
        let (_, bp_registry, explicit_data, _) = setup();

        // A non-explicit BP should return false
        assert!(!ExplicitRelations::stored_dynamically(0, &bp_registry, &explicit_data));
    }

    /// Test stored_dynamically returns false for out-of-range indices.
    #[test]
    fn test_stored_dynamically_false_for_out_of_range() {
        let (_, bp_registry, explicit_data, _) = setup();

        assert!(!ExplicitRelations::stored_dynamically(99, &bp_registry, &explicit_data));
    }

    /// Test relates_values_not_objects returns false.
    #[test]
    fn test_relates_values_not_objects() {
        let (_, bp_registry, _, _) = setup();

        assert!(!ExplicitRelations::relates_values_not_objects(&bp_registry[0], &bp_registry));
    }

    /// Test make_pair_sketchily creates a BP pair with default data.
    #[test]
    fn test_make_pair_sketchily() {
        let (_, mut bp_registry, mut explicit_data, _) = setup();

        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "adjacent",
            &mut bp_registry,
            &mut explicit_data,
        );

        // Should have created 2 new BPs (original + reversal)
        assert_eq!(bp_registry.len(), 18); // 16 dummies + 2 new

        // Check the original
        let bp = &bp_registry[bp_idx];
        assert_eq!(bp.relation_family, EXPLICIT_FAMILY);
        assert_eq!(bp.relation_name.as_deref(), Some("adjacent"));
        assert_eq!(bp.debugging_log_name.as_deref(), Some("adjacent"));
        assert!(bp.right_way_round);

        // Check the reversal
        let rev_idx = bp_idx + 1;
        let rev = &bp_registry[rev_idx];
        assert_eq!(rev.relation_family, EXPLICIT_FAMILY);
        assert_eq!(rev.debugging_log_name.as_deref(), Some("adjacent-rev"));
        assert!(!rev.right_way_round);
        assert_eq!(rev.reversal, Some(bp_idx));

        // Check explicit data
        assert!(explicit_data.len() > bp_idx);
        assert_eq!(explicit_data[bp_idx].form_of_relation, RELATION_OTO_O);
        assert_eq!(explicit_data[bp_idx].i6_storage_property, None);
        assert!(!explicit_data[bp_idx].store_dynamically);

        // Reversal should share the same data
        assert_eq!(explicit_data[rev_idx].form_of_relation, RELATION_OTO_O);
    }

    /// Test get_form_of_relation for explicit family BPs.
    #[test]
    fn test_get_form_of_relation_explicit() {
        let (_, mut bp_registry, mut explicit_data, _) = setup();

        // Create a BP in the explicit family via make_pair_sketchily
        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        // Default form for explicit family is RELATION_OTO_O
        assert_eq!(
            ExplicitRelations::get_form_of_relation(bp_idx, &bp_registry, &explicit_data),
            RELATION_OTO_O
        );
    }

    /// Test get_form_of_relation for non-explicit BPs.
    #[test]
    fn test_get_form_of_relation_non_explicit() {
        let (_, mut bp_registry, explicit_data, _) = setup();

        // Create a BP not in the explicit family
        let bp_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: 0,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });

        assert_eq!(
            ExplicitRelations::get_form_of_relation(bp_idx, &bp_registry, &explicit_data),
            RELATION_IMPLICIT
        );
    }

    /// Test form_to_text for all relation forms.
    #[test]
    fn test_form_to_text() {
        let (_, mut bp_registry, mut explicit_data, _) = setup();

        // Create a BP in the explicit family via make_pair_sketchily
        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        // Explicit family → "Relation_OtoO" (default)
        assert_eq!(
            ExplicitRelations::form_to_text(bp_idx, &bp_registry, &explicit_data),
            "Relation_OtoO"
        );

        // Create a BP in the by-function family
        let open_term = BPTerms::new(None);
        let by_fn_idx = BinaryPredicates::make_pair(
            BY_FUNCTION_FAMILY,
            open_term.clone(),
            open_term,
            "fn-rel",
            "fn-rel-rev",
            None,
            None,
            Some("fn-rel"),
            &mut bp_registry,
        );

        // By-function family → "Relation_OtoO" (default)
        assert_eq!(
            ExplicitRelations::form_to_text(by_fn_idx, &bp_registry, &explicit_data),
            "Relation_OtoO"
        );

        // Non-explicit BP
        let other_idx = bp_registry.len();
        bp_registry.push(BinaryPredicate {
            relation_family: 0,
            family_specific: None,
            relation_name: Some("test".to_string()),
            debugging_log_name: Some("test".to_string()),
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
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        });

        // Non-explicit → "Relation_Implicit"
        assert_eq!(
            ExplicitRelations::form_to_text(other_idx, &bp_registry, &explicit_data),
            "Relation_Implicit"
        );
    }

    // -----------------------------------------------------------------------
    // Dispatch tests
    // -----------------------------------------------------------------------

    /// Test that typecheck dispatch via the explicit family returns DECLINE_TO_MATCH.
    #[test]
    fn test_typecheck_dispatch_via_explicit_family() {
        let (families, mut bp_registry, mut explicit_data, _) = setup();

        // Create a BP in the explicit family
        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[bp_idx],
            &[],
            &[],
            &families,
        );
        assert_eq!(result, -1); // DECLINE_TO_MATCH
    }

    /// Test that assert dispatch via the explicit family returns FALSE.
    #[test]
    fn test_assert_dispatch_via_explicit_family() {
        let (families, mut bp_registry, mut explicit_data, _) = setup();

        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        let result = BinaryPredicateFamilies::assert(
            &bp_registry[bp_idx],
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

    /// Test that schema dispatch via the explicit family returns FALSE.
    #[test]
    fn test_schema_dispatch_via_explicit_family() {
        let (families, mut bp_registry, mut explicit_data, _) = setup();

        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        let result = BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[bp_idx],
            &families,
        );
        assert!(!result);
    }

    /// Test that describe_for_problems dispatch via the explicit family returns empty string.
    #[test]
    fn test_describe_for_problems_dispatch_via_explicit_family() {
        let (families, mut bp_registry, mut explicit_data, _) = setup();

        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        let result = BinaryPredicateFamilies::describe_for_problems(
            &bp_registry[bp_idx],
            &families,
        );
        assert_eq!(result, "");
    }

    /// Test that describe_for_index dispatch via the explicit family returns form description.
    #[test]
    fn test_describe_for_index_dispatch_via_explicit_family() {
        let (families, mut bp_registry, mut explicit_data, _) = setup();

        let bp_idx = ExplicitRelations::make_pair_sketchily(
            "test-rel",
            &mut bp_registry,
            &mut explicit_data,
        );

        let result = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[bp_idx],
            &families,
        );
        assert_eq!(result, "one-to-one");
    }

    /// Test that describe_for_index dispatch via the by-function family returns "defined".
    #[test]
    fn test_describe_for_index_dispatch_via_by_function_family() {
        let (families, mut bp_registry, _, _) = setup();

        // Create a BP in the by-function family
        let open_term = BPTerms::new(None);
        let bp_idx = BinaryPredicates::make_pair(
            BY_FUNCTION_FAMILY,
            open_term.clone(),
            open_term,
            "fn-rel",
            "fn-rel-rev",
            None,
            None,
            Some("fn-rel"),
            &mut bp_registry,
        );

        let result = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[bp_idx],
            &families,
        );
        assert_eq!(result, "defined");
    }
}
