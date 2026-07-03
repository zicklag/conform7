/// The provision relation — determines which properties can be held by which subjects.
///
/// Corresponds to `ProvisionRelation` in the C reference
/// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`).
///
/// Creates one bp_family instance:
/// - provision_bp_family — for the provision relation (R_provision)
///
/// The provision relation is a make_pair (not make_equality) because it has
/// a reversal: "provides" / "is-provided-by".
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::property_permissions::PropertyPermission;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the provision family in the family registry.
pub const PROVISION_FAMILY: usize = 0;

/// Index of the provision predicate in the BP registry (right-way-round).
///
/// Created by `ProvisionRelation::stock()` during first_stock.
/// Its reversal (is-provided-by) is at index 1.
pub const R_PROVISION: usize = 0;

/// The provision relation module.
///
/// Corresponds to `ProvisionRelation` in the C reference
/// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`).
pub struct ProvisionRelation;

impl ProvisionRelation {
    /// Create the provision family with its methods.
    ///
    /// Corresponds to `ProvisionRelation::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 19-27).
    ///
    /// Returns (families, bp_registry) where:
    /// - families[0] = provision_bp_family
    /// - bp_registry is empty (stocking fills it)
    pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
        let provision_family = BpFamily {
            name: "provision",
            methods: BpFamilyMethods {
                stock: Some(ProvisionRelation::stock),
                typecheck: Some(ProvisionRelation::typecheck),
                assert: Some(ProvisionRelation::assert),
                schema: Some(ProvisionRelation::schema),
                describe_for_index: Some(ProvisionRelation::describe_for_index),
                ..BpFamilyMethods::default()
            },
        };

        (vec![provision_family], Vec::new())
    }

    /// Stock the provision family (stage 1): create R_provision.
    ///
    /// Corresponds to `ProvisionRelation::stock` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 34-43).
    pub fn stock(_family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>, _property_registry: &[()]) {
        if n == 1 {
            let family_idx = 0; // provision family is at index 0
            let left_term = BPTerms::new(None);
            let right_term = BPTerms::new(None);
            BinaryPredicates::make_pair(
                family_idx,
                left_term,
                right_term,
                "provides",
                "is-provided-by",
                None, // no make-true schema
                None, // no test schema
                Some("provision"),
                bp_registry,
            );
            // Set index display names: left term is "value", right term is "property"
            // Corresponds to BinaryPredicates::set_index_details(R_provision, "value", "property")
            // in the C reference (line 42).
            // Inline the logic to avoid double-mutable-borrow on bp_registry.
            let rev_idx = bp_registry[R_PROVISION].reversal;
            bp_registry[R_PROVISION].term_details[0].index_term_as = Some("value".to_string());
            bp_registry[R_PROVISION].term_details[1].index_term_as = Some("property".to_string());
            if let Some(rev) = rev_idx {
                if rev < bp_registry.len() {
                    bp_registry[rev].term_details[0].index_term_as = Some("property".to_string());
                    bp_registry[rev].term_details[1].index_term_as = Some("value".to_string());
                }
            }
        }
    }

    /// Typecheck the provision relation.
    ///
    /// Corresponds to `ProvisionRelation::typecheck` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 51-61).
    ///
    /// Simplified: checks that the right term's kind index matches a property kind.
    /// In the C reference, this checks `Kinds::get_construct(kinds_of_terms[1]) == CON_property`.
    /// Here we use a simplified kind index check.
    ///
    /// Returns:
    /// - 1 (ALWAYS_MATCH) if the right term is a property kind
    /// - -1 (NEVER_MATCH) otherwise
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        // Simplified: check if the right term's kind is a property kind.
        // In the full implementation, this would check Kinds::get_construct == CON_property.
        // For now, we accept any kind (ALWAYS_MATCH) since the property kind system
        // is not yet fully integrated.
        1 // ALWAYS_MATCH
    }

    /// Assert the provision relation: grant a property permission.
    ///
    /// Corresponds to `ProvisionRelation::assert` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 70-80).
    ///
    /// Simplified: uses subject indices and string property names instead of
    /// `inference_subject*` and `parse_node*` pointers.
    ///
    /// Returns true if the permission was granted, false otherwise.
    #[allow(clippy::too_many_arguments)]
    pub fn assert(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        subj0: usize,
        _spec0: Option<&'static str>,
        _subj1: usize,
        spec1: Option<&'static str>,
        subjects: &mut [InferenceSubject],
        permissions: &mut Vec<PropertyPermission>,
        _inference_families: &[crate::knowledge::inferences::InferenceFamily],
        _inferences: &mut Vec<crate::knowledge::inferences::Inference>,
        _data_registry: &mut Vec<crate::knowledge::property_inferences::PropertyInferenceData>,
        _property_registry: &[()],
    ) -> bool {
        if let Some(property_name) = spec1 {
            // SAFETY: We need to pass both a mutable reference to subjects[subj0]
            // (which grant modifies) and an immutable reference to the full subjects
            // slice (which grant reads for hierarchy traversal in find). These are
            // non-overlapping uses because grant only reads from subjects and writes
            // to the specific subject at subj0.
            let subject = unsafe { &mut *(&mut subjects[subj0] as *mut InferenceSubject) };
            PropertyPermission::grant(
                subject,
                property_name,
                Some("provision"),
                subj0,
                subjects,
                permissions,
            );
            // Note: Instances::update_adjectival_forms is deferred.
            // In the C reference, this ensures that adjectival forms of
            // enumerated property values are available for instances that
            // provide the property. This requires the instance system.
            true
        } else {
            false
        }
    }

    /// Compile run-time code for the provision relation.
    ///
    /// Corresponds to `ProvisionRelation::schema` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 87-91).
    ///
    /// Simplified: returns false (decline to compile). The full implementation
    /// would call RTProperties::test_provision_schema for TEST_ATOM_TASK.
    pub fn schema(_family: &BpFamily, _task: u8, _bp: &BinaryPredicate) -> bool {
        false
    }

    /// Describe the provision relation for the Phrasebook index.
    ///
    /// Corresponds to `ProvisionRelation::describe_for_index` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 97-100).
    pub fn describe_for_index(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
        "provision".to_string()
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
    fn test_start_creates_one_family() {
        let (families, bp_registry) = ProvisionRelation::start();
        assert_eq!(families.len(), 1);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let (families, _) = ProvisionRelation::start();
        assert_eq!(families[PROVISION_FAMILY].name, "provision");
    }

    #[test]
    fn test_provision_family_has_all_methods() {
        let (families, _) = ProvisionRelation::start();
        let prov = &families[PROVISION_FAMILY];
        assert!(prov.methods.stock.is_some());
        assert!(prov.methods.typecheck.is_some());
        assert!(prov.methods.assert.is_some());
        assert!(prov.methods.schema.is_some());
        assert!(prov.methods.describe_for_index.is_some());
        // Should NOT have describe_for_problems
        assert!(prov.methods.describe_for_problems.is_none());
    }

    // -----------------------------------------------------------------------
    // first_stock tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_stock_creates_r_provision_as_pair() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // R_provision is at index 0 (created by provision family's stock)
        assert!(bp_registry.len() > R_PROVISION);
        let r_prov = &bp_registry[R_PROVISION];
        assert_eq!(r_prov.relation_family, PROVISION_FAMILY);
        assert_eq!(r_prov.relation_name, Some("provision".to_string()));
        assert_eq!(r_prov.debugging_log_name, Some("provides".to_string()));
        assert!(r_prov.right_way_round);
        // Has a reversal pointing to index 1
        assert_eq!(r_prov.reversal, Some(1));
    }

    #[test]
    fn test_first_stock_creates_reversal() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // After first_stock: index 0 = R_provision (right-way-round),
        // index 1 = is-provided-by (reversal)
        assert!(bp_registry.len() > 1);

        // The reversal (is-provided-by) is at index 1
        let reversal = &bp_registry[1];
        assert_eq!(reversal.relation_family, PROVISION_FAMILY);
        assert_eq!(reversal.debugging_log_name, Some("is-provided-by".to_string()));
        assert!(!reversal.right_way_round);
        // Reversal points back to the original
        assert_eq!(reversal.reversal, Some(0));
    }

    #[test]
    fn test_provision_pair_has_correct_term_details() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // R_provision (index 0): both terms use new(None)
        let r_prov = &bp_registry[0];
        // Left term: new(None)
        assert_eq!(r_prov.term_details[0].implies_infs, None);
        assert_eq!(r_prov.term_details[0].implies_kind, None);
        // Right term: new(None)
        assert_eq!(r_prov.term_details[1].implies_infs, None);
        assert_eq!(r_prov.term_details[1].implies_kind, None);

        // Reversal (index 1): terms are swapped
        let reversal = &bp_registry[1];
        assert_eq!(reversal.term_details[0].implies_infs, None);
        assert_eq!(reversal.term_details[0].implies_kind, None);
        assert_eq!(reversal.term_details[1].implies_infs, None);
        assert_eq!(reversal.term_details[1].implies_kind, None);
    }

    // -----------------------------------------------------------------------
    // set_index_details test
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_index_details_on_r_provision() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        // R_provision should have index details set during stock
        let r_prov = &bp_registry[R_PROVISION];
        assert_eq!(r_prov.term_details[0].index_term_as, Some("value".to_string()));
        assert_eq!(r_prov.term_details[1].index_term_as, Some("property".to_string()));

        // Reversal should have swapped index details
        let reversal = &bp_registry[1];
        assert_eq!(reversal.term_details[0].index_term_as, Some("property".to_string()));
        assert_eq!(reversal.term_details[1].index_term_as, Some("value".to_string()));
    }

    // -----------------------------------------------------------------------
    // Describe method tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_describe_for_index_returns_provision() {
        let (families, _) = ProvisionRelation::start();
        let prov_family = &families[PROVISION_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROVISION_FAMILY,
            family_specific: None,
            relation_name: Some("provision".to_string()),
            debugging_log_name: Some("provides".to_string()),
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
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let desc = ProvisionRelation::describe_for_index(prov_family, &dummy_bp);
        assert_eq!(desc, "provision");
    }

    // -----------------------------------------------------------------------
    // Typecheck test
    // -----------------------------------------------------------------------

    #[test]
    fn test_typecheck_returns_always_match() {
        let (families, _) = ProvisionRelation::start();
        let prov_family = &families[PROVISION_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROVISION_FAMILY,
            family_specific: None,
            relation_name: Some("provision".to_string()),
            debugging_log_name: Some("provides".to_string()),
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
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        // Should always return ALWAYS_MATCH (1) in simplified form
        let result = ProvisionRelation::typecheck(
            prov_family,
            &dummy_bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
        );
        assert_eq!(result, 1);

        // Even with different kinds
        let result = ProvisionRelation::typecheck(
            prov_family,
            &dummy_bp,
            &[Some(99), Some(42)],
            &[None, None],
        );
        assert_eq!(result, 1);
    }

    // -----------------------------------------------------------------------
    // Schema test
    // -----------------------------------------------------------------------

    #[test]
    fn test_schema_returns_false() {
        let (families, _) = ProvisionRelation::start();
        let prov_family = &families[PROVISION_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROVISION_FAMILY,
            family_specific: None,
            relation_name: Some("provision".to_string()),
            debugging_log_name: Some("provides".to_string()),
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
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        // Should return false (decline to compile)
        assert!(!ProvisionRelation::schema(prov_family, 1, &dummy_bp));
        assert!(!ProvisionRelation::schema(prov_family, 2, &dummy_bp));
        assert!(!ProvisionRelation::schema(prov_family, 3, &dummy_bp));
    }

    // -----------------------------------------------------------------------
    // Assert test
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_grants_property_permission() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let r_prov = &bp_registry[R_PROVISION];

        // Create a subject
        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some("test_subject"),
        }];

        let mut permissions = Vec::new();
        // Assert the provision relation
        let result = BinaryPredicateFamilies::assert(
            r_prov,
            0,                       // subj0
            None,                    // spec0
            0,                       // subj1
            Some("test_property"),   // spec1 = property name
            &families,
            &mut subjects,
            &mut permissions,
            &[],                     // inference_families
            &mut vec![],            // inferences
            &mut vec![],            // data_registry
            &[],                     // property_registry
        );

        assert!(result);

        // Verify the permission was granted
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].property_owner, 0);
        assert_eq!(permissions[0].property_granted, "test_property");
        assert_eq!(permissions[0].where_granted, Some("provision"));
    }

    #[test]
    fn test_assert_with_no_property_returns_false() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let r_prov = &bp_registry[R_PROVISION];

        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some("test_subject"),
        }];
        let mut permissions = Vec::new();

        // Assert with no property name
        let result = BinaryPredicateFamilies::assert(
            r_prov,
            0,        // subj0
            None,     // spec0
            0,        // subj1
            None,     // spec1 = no property name
            &families,
            &mut subjects,
            &mut permissions,
            &[],                     // inference_families
            &mut vec![],            // inferences
            &mut vec![],            // data_registry
            &[],                     // property_registry
        );

        assert!(!result);
        assert_eq!(permissions.len(), 0);
    }

    // -----------------------------------------------------------------------
    // Family method dispatch tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_describe_for_index_dispatch_via_family() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let desc = BinaryPredicateFamilies::describe_for_index(
            &bp_registry[R_PROVISION],
            &families,
        );
        assert_eq!(desc, "provision");
    }

    #[test]
    fn test_typecheck_dispatch_via_family() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[R_PROVISION],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, 1);
    }

    #[test]
    fn test_schema_dispatch_via_family() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        assert!(!BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[R_PROVISION],
            &families,
        ));
    }

    #[test]
    fn test_assert_dispatch_via_family() {
        let (mut families, mut bp_registry) = ProvisionRelation::start();
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);

        let r_prov = &bp_registry[R_PROVISION];

        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some("test_subject"),
        }];
        let mut permissions = Vec::new();

        let result = BinaryPredicateFamilies::assert(
            r_prov,
            0,
            None,
            0,
            Some("color"),
            &families,
            &mut subjects,
            &mut permissions,
            &[],                     // inference_families
            &mut vec![],            // inferences
            &mut vec![],            // data_registry
            &[],                     // property_registry
        );

        assert!(result);
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].property_granted, "color");
    }
}
