/// The Either-Or Property Adjectives system — either-or properties used as adjectives.
///
/// Corresponds to `EitherOrPropertyAdjectives` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`).
///
/// Creates one adjective_meaning_family instance:
/// - either_or_property_amf — for either-or property adjectives
///
/// Each either-or property can be used as an adjective. For example, the
/// either-or property "open" can be used as the adjective "open" in
/// assertions like "the door is open".
///
/// Simplified:
/// - No RTProperties::write_either_or_schemas (run-time compilation deferred)
/// - No RTInferences::index_either_or (index generation deferred)
/// - No Preform grammar for property name resolution
use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
};
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inferences::{Inference, InferenceFamily};
use crate::knowledge::properties::{EitherOrProperties, Property};
use crate::knowledge::property_inferences::{PropertyInferenceData, PropertyInferences};
use crate::knowledge::measurements::MeasurementDefinition;

/// Index of the either-or property family in the family registry.
pub const EITHER_OR_PROPERTY_FAMILY: usize = 0;

/// The either-or property adjectives module.
///
/// Corresponds to `EitherOrPropertyAdjectives` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`).
pub struct EitherOrPropertyAdjectives;

impl EitherOrPropertyAdjectives {
    /// Create the either-or property family with its methods.
    ///
    /// Corresponds to `EitherOrPropertyAdjectives::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 12-20).
    ///
    /// Returns (families, meanings, adjectives) where:
    /// - families[0] = either_or_property_amf
    /// - meanings is empty (create_for_property fills it)
    /// - adjectives is empty (create_for_property fills it)
    pub fn start() -> (Vec<AdjectiveMeaningFamily>, Vec<AdjectiveMeaning>, Vec<Adjective>) {
        let either_or_property_family = AdjectiveMeaningFamily {
            name: "either_or_property",
            definition_claim_priority: 1,
            methods: AdjectiveMeaningFamilyMethods {
                assert: Some(EitherOrPropertyAdjectives::assert),
                claim_definition: None,
                prepare_schemas: Some(EitherOrPropertyAdjectives::prepare_schemas),
                index: Some(EitherOrPropertyAdjectives::index),
            },
        };

        (vec![either_or_property_family], Vec::new(), Vec::new())
    }

    /// Check if an adjective meaning belongs to the either-or property family.
    ///
    /// Corresponds to `EitherOrPropertyAdjectives::is` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 22-25).
    ///
    /// Returns true if the meaning belongs to this family, false otherwise.
    pub fn is(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == EITHER_OR_PROPERTY_FAMILY)
    }

    /// Register an either-or property as an adjective for a given kind.
    ///
    /// Corresponds to `EitherOrPropertyAdjectives::create_for_property` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 35-48).
    ///
    /// This function:
    /// 1. Checks if the property already has an adjective
    /// 2. If it does, checks if the adjective can already be applied to the kind
    /// 3. If it doesn't, declares a new adjective and stores it in the property data
    /// 4. Creates a new adjective meaning for this property-kind pair
    /// 5. Adds the meaning to the adjective
    /// 6. Sets the domain from the kind
    ///
    /// Simplified:
    /// - No internal_error for non-either-or properties (returns silently)
    /// - No Preform grammar for name validation
    ///
    /// Returns the adjective index.
    pub fn create_for_property(
        prn_idx: usize,
        name: &'static str,
        kind_idx: usize,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        properties: &mut [Property],
    ) -> usize {
        // Check if the property is either-or.
        let eo_data = properties[prn_idx].either_or_data.as_ref();
        if eo_data.is_none() {
            // Not an either-or property — return a dummy adjective index.
            // In the C reference, this is an internal_error.
            // Simplified: create a placeholder adjective.
            return Adjectives::declare(name, adjectives);
        }

        // Check if the property already has an adjective.
        let adj_idx = EitherOrProperties::as_adjective(prn_idx, properties);

        let adj_idx = if let Some(adj) = adj_idx {
            // Property already has an adjective — check if it can be applied to this kind.
            if AdjectiveAmbiguity::can_be_applied_to(adj, Some(kind_idx), adjectives, meanings) {
                return adj; // Already registered for this kind — no-op.
            }
            adj
        } else {
            // No adjective yet — declare one and store it in the property data.
            let adj = Adjectives::declare(name, adjectives);
            if let Some(eod) = &mut properties[prn_idx].either_or_data {
                eod.as_adjective = Some(adj);
            }
            adj
        };

        // Create a new adjective meaning for this property-kind pair.
        // The family-specific data stores the property index.
        let am = AdjectiveMeanings::new(
            EITHER_OR_PROPERTY_FAMILY,
            Some(prn_idx),
            Some(name),
            meanings,
        );

        // Add the meaning to the adjective.
        AdjectiveAmbiguity::add_meaning_to_adjective(am, adj_idx, adjectives, meanings);

        // Set the domain from the kind.
        AdjectiveMeaningDomains::set_from_kind(am, kind_idx, meanings);

        adj_idx
    }

    #[allow(clippy::too_many_arguments)]
    /// Assert an either-or property adjective on an inference subject.
    ///
    /// For positive parity (e.g., "the door is open"), calls
    /// `PropertyInferences::draw` with the property name.
    ///
    /// For negative parity (e.g., "the door is not open"), calls
    /// `PropertyInferences::draw_negated` with the property name.
    /// Simplified:
    /// - Property name is retrieved from the meaning's family_specific_data
    ///
    /// Returns true (always handled).
    pub fn assert(
        am_idx: usize,
        subj_idx: usize,
        parity: bool,
        meanings: &mut [AdjectiveMeaning],
        subjects: &mut [InferenceSubject],
        properties: &[Property],
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        data_registry: &mut Vec<PropertyInferenceData>,
        _definitions: &mut [MeasurementDefinition],
    ) -> bool {
        // Retrieve the property index from the meaning's family-specific data.
        let am = &meanings[am_idx];
        let prn_idx = match am.family_specific_data {
            Some(idx) => idx,
            None => return false, // No property data — decline.
        };

        // Look up the property name from the index.
        let prn_name = match properties.get(prn_idx) {
            Some(prn) => prn.name,
            None => return false, // Property not found — decline.
        };

        if parity {
            PropertyInferences::draw(
                subj_idx,
                prn_name,
                None,
                families,
                inferences,
                subjects,
                data_registry,
            );
        } else {
            PropertyInferences::draw_negated(
                subj_idx,
                prn_name,
                None,
                families,
                inferences,
                subjects,
                data_registry,
            );
        }

        true
    }

    /// Prepare I6 schemas for an either-or property adjective.
    ///
    /// Corresponds to `EitherOrPropertyAdjectives::prepare_schemas` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 61-66).
    ///
    /// Simplified: no-op. The full implementation would call
    /// `RTProperties::write_either_or_schemas` to generate I6 schemas.
    pub fn prepare_schemas(_am_idx: usize, _task: i32) {
        // Not implemented — run-time compilation deferred.
    }

    /// Produce index text for an either-or property adjective.
    ///
    /// Corresponds to `EitherOrPropertyAdjectives::index` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 68-73).
    ///
    /// Simplified: no-op. The full implementation would call
    /// `RTInferences::index_either_or` to generate index text.
    pub fn index(_am_idx: usize) -> Option<&'static str> {
        None // Not implemented — index generation deferred.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inferences::Certainty;

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::start
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_family_with_correct_name_and_priority() {
        let (families, meanings, adjectives) = EitherOrPropertyAdjectives::start();

        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name, "either_or_property");
        assert_eq!(families[0].definition_claim_priority, 1);
        assert!(meanings.is_empty());
        assert!(adjectives.is_empty());
    }

    #[test]
    fn test_start_creates_family_with_assert_method() {
        let (families, _, _) = EitherOrPropertyAdjectives::start();

        assert!(families[0].methods.assert.is_some());
    }

    #[test]
    fn test_start_creates_family_with_prepare_schemas_method() {
        let (families, _, _) = EitherOrPropertyAdjectives::start();

        assert!(families[0].methods.prepare_schemas.is_some());
    }

    #[test]
    fn test_start_creates_family_with_index_method() {
        let (families, _, _) = EitherOrPropertyAdjectives::start();

        assert!(families[0].methods.index.is_some());
    }

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::is
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_returns_true_for_either_or_property_meaning() {
        let (_families, mut meanings, _) = EitherOrPropertyAdjectives::start();

        let am_idx = AdjectiveMeanings::new(0, None, None, &mut meanings);

        assert!(EitherOrPropertyAdjectives::is(am_idx, &meanings));
    }

    #[test]
    fn test_is_returns_false_for_different_family() {
        let (mut families, mut meanings, _) = EitherOrPropertyAdjectives::start();

        // Create a second family.
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let other_fam_idx = AdjectiveMeanings::new_family("other", 0, methods, &mut families);

        let am_idx = AdjectiveMeanings::new(other_fam_idx, None, None, &mut meanings);

        assert!(!EitherOrPropertyAdjectives::is(am_idx, &meanings));
    }

    #[test]
    fn test_is_returns_false_for_invalid_index() {
        let meanings = Vec::new();

        assert!(!EitherOrPropertyAdjectives::is(0, &meanings));
    }

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::create_for_property
    // -----------------------------------------------------------------------

    fn make_either_or_property(name: &'static str) -> Property {
        Property {
            name,
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: Some(crate::knowledge::properties::EitherOrPropertyData {
                negation: None,
                as_adjective: None,
            }),
            value_data: None,
            compilation_data: None,
            possession_marker: false,
        }
    }

    #[test]
    fn test_create_for_property_declares_new_adjective() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(adjectives[adj_idx].name, "open");
    }

    #[test]
    fn test_create_for_property_stores_adjective_in_property_data() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(
            properties[0].either_or_data.as_ref().unwrap().as_adjective,
            Some(adj_idx)
        );
    }

    #[test]
    fn test_create_for_property_creates_meaning_with_correct_family() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        // There should be one meaning, belonging to the either-or property family.
        assert_eq!(meanings.len(), 1);
        assert_eq!(meanings[0].family, EITHER_OR_PROPERTY_FAMILY);
    }

    #[test]
    fn test_create_for_property_sets_family_specific_data_to_property_index() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(meanings[0].family_specific_data, Some(0));
    }

    #[test]
    fn test_create_for_property_adds_meaning_to_adjective() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(adjectives[adj_idx].meanings.len(), 1);
        assert_eq!(adjectives[adj_idx].meanings[0], 0);
        assert_eq!(meanings[0].owning_adjective, Some(adj_idx));
    }

    #[test]
    fn test_create_for_property_sets_domain_from_kind() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(meanings[0].domain.domain_kind, Some(42));
    }

    #[test]
    fn test_create_for_property_is_idempotent_for_same_kind() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        let adj_idx1 = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        let adj_idx2 = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        // Should return the same adjective index.
        assert_eq!(adj_idx1, adj_idx2);
        // Should not create a new meaning.
        assert_eq!(meanings.len(), 1);
    }

    #[test]
    fn test_create_for_property_creates_new_meaning_for_different_kind() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];

        EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        EitherOrPropertyAdjectives::create_for_property(
            0, "open", 99, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        // Should create a second meaning for the different kind.
        assert_eq!(meanings.len(), 2);
        assert_eq!(meanings[0].domain.domain_kind, Some(42));
        assert_eq!(meanings[1].domain.domain_kind, Some(99));
    }

    #[test]
    fn test_create_for_property_handles_non_either_or_property() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![Property {
            name: "carrying_capacity",
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(crate::knowledge::properties::ValuePropertyData {
                property_value_kind: None,
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }];

        // Should not panic; creates a placeholder adjective.
        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "carrying_capacity", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        assert_eq!(adjectives[adj_idx].name, "carrying_capacity");
    }

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::assert
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_with_positive_parity_draws_property_inference() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];
        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: None,
        }];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
        let inference_families = vec![PropertyInferences::start()];

        // Create the adjective meaning.
        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        // Get the meaning index.
        let am_idx = adjectives[adj_idx].meanings[0];

        // Assert with positive parity.
        let result = EitherOrPropertyAdjectives::assert(
            am_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &properties,
            &inference_families,
            &mut inferences,
            &mut data_registry,
            &mut [],
        );

        assert!(result);
        // An inference should have been added to the subject.
        assert_eq!(subjects[0].inf_list.len(), 1);
        // The inference should have the property name as data.
        assert_eq!(inferences[0].data, Some("open"));
    }

    #[test]
    fn test_assert_with_negative_parity_draws_negated_inference() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];
        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: None,
        }];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
        let inference_families = vec![PropertyInferences::start()];

        // Create the adjective meaning.
        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );

        // Get the meaning index.
        let am_idx = adjectives[adj_idx].meanings[0];

        let result = EitherOrPropertyAdjectives::assert(
            am_idx,
            0,
            false,
            &mut meanings,
            &mut subjects,
            &properties,
            &inference_families,
            &mut inferences,
            &mut data_registry,
            &mut [],
        );

        assert!(result);
        // An inference should have been added to the subject.
        assert_eq!(subjects[0].inf_list.len(), 1);
        // The inference should have the property name as data.
        assert_eq!(inferences[0].data, Some("open"));
        // The certainty should be negated (Unlikely instead of Likely).
        assert_eq!(inferences[0].certainty, Certainty::Unlikely);
    }

    #[test]
    fn test_assert_returns_true() {
        let (families, mut meanings, mut adjectives) = EitherOrPropertyAdjectives::start();
        let mut properties = vec![make_either_or_property("open")];
        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: None,
        }];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
        let inference_families = vec![PropertyInferences::start()];

        let adj_idx = EitherOrPropertyAdjectives::create_for_property(
            0, "open", 42, &mut adjectives, &mut meanings, &families, &mut properties,
        );
        let am_idx = adjectives[adj_idx].meanings[0];
        let result = EitherOrPropertyAdjectives::assert(
            am_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &properties,
            &inference_families,
            &mut inferences,
            &mut data_registry,
            &mut [],
        );

        assert!(result);
    }

    #[test]
    fn test_assert_returns_false_when_no_family_specific_data() {
        let (_families, mut meanings, _) = EitherOrPropertyAdjectives::start();
        let mut subjects = vec![InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: None,
        }];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();

        // Create a meaning without family-specific data.
        let am_idx = AdjectiveMeanings::new(0, None, None, &mut meanings);
        let result = EitherOrPropertyAdjectives::assert(
            am_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &[], // no properties
            &[],
            &mut inferences,
            &mut data_registry,
            &mut [],
        );

        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::prepare_schemas
    // -----------------------------------------------------------------------

    #[test]
    fn test_prepare_schemas_is_noop() {
        // prepare_schemas is a no-op in the simplified implementation.
        // It should not panic.
        EitherOrPropertyAdjectives::prepare_schemas(0, 0);
    }

    // -----------------------------------------------------------------------
    // EitherOrPropertyAdjectives::index
    // -----------------------------------------------------------------------

    #[test]
    fn test_index_returns_none() {
        let result = EitherOrPropertyAdjectives::index(0);
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // EitherOrProperties::as_adjective
    // -----------------------------------------------------------------------

    #[test]
    fn test_as_adjective_returns_correct_adjective_index() {
        let mut properties = vec![make_either_or_property("open")];

        // Manually set the adjective index.
        properties[0].either_or_data.as_mut().unwrap().as_adjective = Some(7);

        let result = EitherOrProperties::as_adjective(0, &properties);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn test_as_adjective_returns_none_for_property_without_adjective() {
        let properties = vec![make_either_or_property("open")];

        let result = EitherOrProperties::as_adjective(0, &properties);
        assert_eq!(result, None);
    }

    #[test]
    fn test_as_adjective_returns_none_for_valued_property() {
        let properties = vec![Property {
            name: "carrying_capacity",
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(crate::knowledge::properties::ValuePropertyData {
                property_value_kind: None,
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }];

        let result = EitherOrProperties::as_adjective(0, &properties);
        assert_eq!(result, None);
    }

    #[test]
    fn test_as_adjective_returns_none_for_out_of_bounds_index() {
        let properties = Vec::new();

        let result = EitherOrProperties::as_adjective(0, &properties);
        assert_eq!(result, None);
    }
}
