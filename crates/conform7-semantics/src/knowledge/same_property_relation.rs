/// The same property relation — compares a property value between two owners.
///
/// Corresponds to `SameAsRelations` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`).
///
/// Creates one bp_family instance:
/// - same_property_bp_family — for the same-property-value-as relation
///
/// Each valued property gets one make_pair in this family. For example,
/// if there is a valued property "height", then a relation "same-height-as"
/// is created to serve as the meaning of "the same height as".
///
/// Simplified:
/// - No Preform grammar for relation name construction
/// - No preposition registration (SameAsRelations::register_same_property_as)
/// - No RTProperties::iname (run-time compilation)
/// - No Calculus::Schemas (simplified string schemas)
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods, DECLINE_TO_MATCH};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;
use crate::knowledge::properties::Property;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the same property family in the family registry.
pub const SAME_PROPERTY_FAMILY: usize = 0;

/// The same property relation module.
///
/// Corresponds to `SameAsRelations` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`).
pub struct SameAsRelations;

impl SameAsRelations {
    /// Create the same property family with its methods.
    ///
    /// Corresponds to `SameAsRelations::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 11-15).
    ///
    /// Returns (families, bp_registry) where:
    /// - families[0] = same_property_bp_family
    /// - bp_registry is empty (stocking fills it)
    pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
        let same_property_family = BpFamily {
            name: "same_property",
            methods: BpFamilyMethods {
                stock: Some(SameAsRelations::stock),
                typecheck: Some(SameAsRelations::typecheck),
                ..BpFamilyMethods::default()
            },
        };

        (vec![same_property_family], Vec::new())
    }

    /// Derive the same-property-value-as relation name from a property name.
    ///
    /// Corresponds to the `<same-property-as-construction>` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 74-76).
    ///
    /// For a property named "height", returns "same-height-as".
    /// For a property named "carrying capacity", returns "same-carrying-capacity-as".
    /// For a property named "point of view", returns "same-point-of-view-as".
    fn derive_relation_name(property_name: &str) -> String {
        format!("same-{}-as", property_name.replace(' ', "-"))
    }

    /// Stock the same property family (stage 2): create one make_pair per valued property.
    ///
    /// Corresponds to `SameAsRelations::stock` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 33-59).
    ///
    /// Simplified:
    /// - No Wordings::nonempty check (all properties have names in our simplified model)
    /// - No RTProperties::iname (run-time compilation deferred)
    /// - No Calculus::Schemas (simplified string schemas)
    /// - No SameAsRelations::register_same_property_as (preposition registration deferred)
    ///
    /// Only valued properties (those with `value_data.is_some()`) get a same-property
    /// relation. Either-or properties are skipped.
    ///
    pub fn stock(
        _family: &BpFamily,
        n: u8,
        bp_registry: &mut Vec<BinaryPredicate>,
        property_registry: &[Property],
    ) {
        if n == 2 {
            let family_idx = SAME_PROPERTY_FAMILY;

            for (prn_idx, prn) in property_registry.iter().enumerate() {
                // Only valued properties get a same-property relation.
                if prn.value_data.is_none() {
                    continue;
                }

                // Derive the relation name from the property name.
                let rel_name = Self::derive_relation_name(prn.name);

                // Create a make_pair for this property.
                // In the C reference, the schemas use RTProperties::iname(prn) for
                // run-time property access. Simplified: we use string schemas with
                // the property name as a placeholder.
                let left_term = BPTerms::new(None);
                let right_term = BPTerms::new(None);

                // Build the schema strings. These are simplified versions of the
                // Calculus::Schemas used in the C reference.
                let make_true_schema = format!("*1.{} = *2.{}", prn.name, prn.name);
                let test_schema = format!("*1.{} == *2.{}", prn.name, prn.name);
                let rev_name = format!("{}-reversed", rel_name);

                let bp_idx = BinaryPredicates::make_pair(
                    family_idx,
                    left_term,
                    right_term,
                    &rel_name,
                    &rev_name,
                    Some(&make_true_schema),
                    Some(&test_schema),
                    Some(&rel_name),
                    bp_registry,
                );

                // Store the property index in family-specific data.
                // In the C reference, this is STORE_POINTER_property(prn).
                // Simplified: we store the property index as a string.
                bp_registry[bp_idx].family_specific = Some(format!("property:{}", prn_idx));
            }
        }
    }

    /// Typecheck the same property relation.
    ///
    /// Corresponds to `SameAsRelations::typecheck` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 80-85).
    ///
    /// Returns DECLINE_TO_MATCH, letting the standard machinery handle typechecking.
    pub fn typecheck(
        _family: &BpFamily,
        _bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        DECLINE_TO_MATCH
    }

    /// Retrieve the property index from a binary predicate's family-specific data.
    ///
    /// Corresponds to `SameAsRelations::bp_get_same_as_property` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 88-93).
    ///
    /// Returns `Some(property_index)` if the BP belongs to the same property family
    /// and is right-way-round, `None` otherwise.
    pub fn bp_get_same_as_property(bp: &BinaryPredicate) -> Option<usize> {
        if bp.relation_family != SAME_PROPERTY_FAMILY {
            return None;
        }
        if !bp.right_way_round {
            return None;
        }
        bp.family_specific.as_ref().and_then(|s| {
            s.strip_prefix("property:").and_then(|n| n.parse().ok())
        })
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
    use crate::knowledge::properties::{EitherOrPropertyData, ValuePropertyData};

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_one_family() {
        let (families, bp_registry) = SameAsRelations::start();
        assert_eq!(families.len(), 1);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let (families, _) = SameAsRelations::start();
        assert_eq!(families[SAME_PROPERTY_FAMILY].name, "same_property");
    }

    #[test]
    fn test_same_property_family_has_stock_and_typecheck_methods() {
        let (families, _) = SameAsRelations::start();
        let sp = &families[SAME_PROPERTY_FAMILY];
        assert!(sp.methods.stock.is_some());
        assert!(sp.methods.typecheck.is_some());
        // Should NOT have assert, schema, or describe methods
        assert!(sp.methods.assert.is_none());
        assert!(sp.methods.schema.is_none());
        assert!(sp.methods.describe_for_problems.is_none());
        assert!(sp.methods.describe_for_index.is_none());
    }

    // -----------------------------------------------------------------------
    // derive_relation_name tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_derive_relation_name_single_word() {
        assert_eq!(
            SameAsRelations::derive_relation_name("height"),
            "same-height-as"
        );
    }

    #[test]
    fn test_derive_relation_name_multi_word() {
        assert_eq!(
            SameAsRelations::derive_relation_name("carrying capacity"),
            "same-carrying-capacity-as"
        );
    }

    #[test]
    fn test_derive_relation_name_three_words() {
        assert_eq!(
            SameAsRelations::derive_relation_name("point of view"),
            "same-point-of-view-as"
        );
    }

    // -----------------------------------------------------------------------
    // stock() tests
    // -----------------------------------------------------------------------

    /// Create a valued property for testing.
    fn make_valued_property(name: &'static str) -> Property {
        Property {
            name,
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(ValuePropertyData {
                property_value_kind: None,
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }
    }

    /// Create an either-or property for testing.
    fn make_either_or_property(name: &'static str) -> Property {
        Property {
            name,
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: Some(EitherOrPropertyData {
                negation: None,
                as_adjective: None,
            }),
            value_data: None,
            compilation_data: None,
            possession_marker: false,
        }
    }

    #[test]
    fn test_stock_skips_stage_1() {
        let (families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![make_valued_property("height")];

        // Stock at stage 1 should do nothing
        families[SAME_PROPERTY_FAMILY]
            .methods
            .stock
            .unwrap()(&families[SAME_PROPERTY_FAMILY], 1, &mut bp_registry, &properties);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_stock_creates_one_pair_per_valued_property() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_valued_property("carrying capacity"),
            make_valued_property("point of view"),
        ];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // 3 valued properties → 3 pairs = 6 BPs
        assert_eq!(bp_registry.len(), 6);
    }

    #[test]
    fn test_stock_skips_either_or_properties() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_either_or_property("open"),
            make_valued_property("carrying capacity"),
            make_either_or_property("closed"),
        ];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // 2 valued properties → 2 pairs = 4 BPs
        assert_eq!(bp_registry.len(), 4);
    }

    #[test]
    fn test_stock_empty_property_registry() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties: Vec<Property> = vec![];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_stock_creates_correct_relation_names() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_valued_property("carrying capacity"),
        ];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // First property: "height" → "same-height-as" at index 0
        assert_eq!(
            bp_registry[0].debugging_log_name,
            Some("same-height-as".to_string())
        );
        // Reversal at index 1
        assert_eq!(
            bp_registry[1].debugging_log_name,
            Some("same-height-as-reversed".to_string())
        );

        // Second property: "carrying capacity" → "same-carrying-capacity-as" at index 2
        assert_eq!(
            bp_registry[2].debugging_log_name,
            Some("same-carrying-capacity-as".to_string())
        );
        // Reversal at index 3
        assert_eq!(
            bp_registry[3].debugging_log_name,
            Some("same-carrying-capacity-as-reversed".to_string())
        );
    }

    #[test]
    fn test_stock_sets_family_specific_data() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_valued_property("carrying capacity"),
        ];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // Right-way-round BPs should have family_specific set
        assert_eq!(
            bp_registry[0].family_specific,
            Some("property:0".to_string())
        );
        assert_eq!(
            bp_registry[2].family_specific,
            Some("property:1".to_string())
        );

        // Reversal BPs should NOT have family_specific set (make_pair sets it to None)
        assert_eq!(bp_registry[1].family_specific, None);
        assert_eq!(bp_registry[3].family_specific, None);
    }

    #[test]
    fn test_stock_creates_reversal_relationships() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![make_valued_property("height")];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // Index 0 = right-way-round, index 1 = reversal
        assert!(bp_registry[0].right_way_round);
        assert!(!bp_registry[1].right_way_round);
        assert_eq!(bp_registry[0].reversal, Some(1));
        assert_eq!(bp_registry[1].reversal, Some(0));
    }

    #[test]
    fn test_stock_sets_correct_family() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![make_valued_property("height")];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        assert_eq!(bp_registry[0].relation_family, SAME_PROPERTY_FAMILY);
        assert_eq!(bp_registry[1].relation_family, SAME_PROPERTY_FAMILY);
    }

    // -----------------------------------------------------------------------
    // typecheck() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_typecheck_returns_decline_to_match() {
        let (families, _) = SameAsRelations::start();
        let sp_family = &families[SAME_PROPERTY_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: SAME_PROPERTY_FAMILY,
            family_specific: None,
            relation_name: Some("same-height-as".to_string()),
            debugging_log_name: Some("same-height-as".to_string()),
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

        let result = SameAsRelations::typecheck(
            sp_family,
            &dummy_bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
        );
        assert_eq!(result, DECLINE_TO_MATCH);

        // Even with different kinds
        let result = SameAsRelations::typecheck(
            sp_family,
            &dummy_bp,
            &[Some(99), Some(42)],
            &[None, None],
        );
        assert_eq!(result, DECLINE_TO_MATCH);
    }

    #[test]
    fn test_typecheck_dispatch_via_family() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![make_valued_property("height")];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[0],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, DECLINE_TO_MATCH);
    }

    // -----------------------------------------------------------------------
    // bp_get_same_as_property() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bp_get_same_as_property_returns_correct_index() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_valued_property("carrying capacity"),
        ];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // Right-way-round BPs should return the property index
        assert_eq!(
            SameAsRelations::bp_get_same_as_property(&bp_registry[0]),
            Some(0)
        );
        assert_eq!(
            SameAsRelations::bp_get_same_as_property(&bp_registry[2]),
            Some(1)
        );
    }

    #[test]
    fn test_bp_get_same_as_property_returns_none_for_reversal() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![make_valued_property("height")];

        

        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);

        // Reversal (index 1) should return None
        assert_eq!(
            SameAsRelations::bp_get_same_as_property(&bp_registry[1]),
            None
        );
    }

    #[test]
    fn test_bp_get_same_as_property_returns_none_for_wrong_family() {
        let dummy_bp = BinaryPredicate {
            relation_family: 999, // not SAME_PROPERTY_FAMILY
            family_specific: Some("property:0".to_string()),
            relation_name: None,
            debugging_log_name: None,
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

        assert_eq!(
            SameAsRelations::bp_get_same_as_property(&dummy_bp),
            None
        );
    }

    #[test]
    fn test_bp_get_same_as_property_returns_none_for_no_family_specific() {
        let dummy_bp = BinaryPredicate {
            relation_family: SAME_PROPERTY_FAMILY,
            family_specific: None,
            relation_name: None,
            debugging_log_name: None,
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

        assert_eq!(
            SameAsRelations::bp_get_same_as_property(&dummy_bp),
            None
        );
    }

    // -----------------------------------------------------------------------
    // Integration test: full start + stock + typecheck + bp_get_same_as_property
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_lifecycle() {
        let (mut families, mut bp_registry) = SameAsRelations::start();
        let properties = vec![
            make_valued_property("height"),
            make_either_or_property("open"),
            make_valued_property("carrying capacity"),
            make_valued_property("point of view"),
            make_either_or_property("closed"),
        ];

        

        // Stage 1: nothing happens
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        assert_eq!(bp_registry.len(), 0);

        // Stage 2: 3 valued properties → 3 pairs = 6 BPs
        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, &properties);
        assert_eq!(bp_registry.len(), 6);

        // Verify each right-way-round BP
        for i in (0..6).step_by(2) {
            let bp = &bp_registry[i];
            assert_eq!(bp.relation_family, SAME_PROPERTY_FAMILY);
            assert!(bp.right_way_round);
            assert!(bp.reversal.is_some());

            // Typecheck returns DECLINE_TO_MATCH
            let tc = BinaryPredicateFamilies::typecheck(
                bp,
                &[Some(0), Some(1)],
                &[Some(0), Some(1)],
                &families,
            );
            assert_eq!(tc, DECLINE_TO_MATCH);

            // bp_get_same_as_property returns Some for right-way-round
            assert!(SameAsRelations::bp_get_same_as_property(bp).is_some());
        }

        // Verify reversals
        for i in (1..6).step_by(2) {
            let bp = &bp_registry[i];
            assert!(!bp.right_way_round);
            assert!(bp.reversal.is_some());

            // bp_get_same_as_property returns None for reversals
            assert!(SameAsRelations::bp_get_same_as_property(bp).is_none());
        }
    }
}
