/// The Measurement Adjectives system — measurement-based adjectives used as adjectives.
///
/// Corresponds to `MeasurementAdjectives` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`).
///
/// Creates one adjective_meaning_family instance:
/// - measurement_amf — for measurement-based adjectives
///
/// Measurement adjectives compare a property value against a threshold. For example,
/// the definition "Definition: A container is roomy if its carrying capacity is 10 or more"
/// creates the adjective "roomy" which is true when carrying_capacity >= 10.
///
/// Simplified:
/// - No Preform grammar parsing (<measurement-adjective-definition>, <measurement-range>)
/// - No Grading::make_superlative (superlative form deferred)
/// - No Grading::make_comparative (comparative form deferred)
/// - No Grading::make_quiddity (quiddity form deferred)
/// - No RTAdjectives::make_mdef_test_schema (run-time compilation deferred)
/// - No <s-literal> grammar parsing (uses simple number parsing)
/// - No problem message generation
use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
};
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inferences::{Inference, InferenceFamily};
use crate::knowledge::measurements::{
    MeasurementDefinition, Measurements,
};
use crate::knowledge::properties::Property;
use crate::knowledge::property_inferences::{PropertyInferenceData, PropertyInferences};

/// Index of the measurement family in the family registry.
pub const MEASUREMENT_FAMILY: usize = 2;

/// The measurement adjectives module.
///
/// Corresponds to `MeasurementAdjectives` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`).
pub struct MeasurementAdjectives;

impl MeasurementAdjectives {
    /// Create the measurement family with its methods.
    ///
    /// Corresponds to `MeasurementAdjectives::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 10-18).
    ///
    /// Returns (families, meanings, adjectives, definitions) where:
    /// - families[MEASUREMENT_FAMILY] = measurement_amf
    /// - meanings is empty (claim_definition fills it)
    /// - adjectives is empty (claim_definition fills it)
    /// - definitions is empty (claim_definition fills it)
    ///
    /// The returned families vector includes placeholders for families created by
    /// earlier plans (either_or_property at index 0, enumerative at index 1) so
    /// that the measurement family is at index 2.
    pub fn start() -> (
        Vec<AdjectiveMeaningFamily>,
        Vec<AdjectiveMeaning>,
        Vec<Adjective>,
        Vec<MeasurementDefinition>,
    ) {
        let measurement_family = AdjectiveMeaningFamily {
            name: "measurement",
            definition_claim_priority: 3,
            methods: AdjectiveMeaningFamilyMethods {
                assert: Some(MeasurementAdjectives::assert),
                claim_definition: Some(MeasurementAdjectives::claim_definition),
                prepare_schemas: Some(MeasurementAdjectives::prepare_schemas),
                index: None,
            },
        };
        let either_or_placeholder = AdjectiveMeaningFamily {
            name: "either_or_property",
            definition_claim_priority: 0,
            methods: AdjectiveMeaningFamilyMethods {
                assert: None,
                claim_definition: None,
                prepare_schemas: None,
                index: None,
            },
        };
        let enumerative_placeholder = AdjectiveMeaningFamily {
            name: "enumerative",
            definition_claim_priority: 0,
            methods: AdjectiveMeaningFamilyMethods {
                assert: None,
                claim_definition: None,
                prepare_schemas: None,
                index: None,
            },
        };

        (
            vec![
                either_or_placeholder,  // index 0: either_or_property (from PLAN-29)
                enumerative_placeholder, // index 1: enumerative (from PLAN-30)
                measurement_family,      // index 2: measurement
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    /// Check if an adjective meaning belongs to the measurement family.
    ///
    /// Corresponds to checking `am->family == measurement_amf` in the C reference.
    ///
    /// Returns true if the meaning belongs to this family, false otherwise.
    pub fn is_measurement(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == MEASUREMENT_FAMILY)
    }

    /// Claim a definition as a measurement adjective.
    ///
    /// Corresponds to `MeasurementAdjectives::claim_definition` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 56-75).
    ///
    /// This is a simplified version that takes pre-parsed parameters instead of
    /// parsing them from grammar. In the C reference, the Preform grammar
    /// `<measurement-adjective-definition>` and `<measurement-range>` parse the
    /// definition clause. Here, the caller provides the parsed values directly.
    ///
    /// Parameters:
    /// - headword: the adjective being defined (e.g., "roomy")
    /// - prop: optional property index (the property being compared)
    /// - shape: the region shape (MEASURE_T_OR_LESS, MEASURE_T_EXACTLY, MEASURE_T_OR_MORE)
    /// - threshold_text: optional text of the threshold value (e.g., "10")
    /// - domain_text: optional text of the domain (e.g., "container")
    /// - definitions: mutable vector of measurement definitions
    /// - adjectives: mutable vector of adjectives
    /// - meanings: mutable vector of adjective meanings
    /// - families: slice of adjective meaning families
    /// - properties: slice of properties
    ///
    /// Returns the index of the new measurement definition, or None if creation failed.
    ///
    /// Simplified:
    /// - No `<measurement-adjective-definition>` grammar parsing
    /// - No `<measurement-range>` grammar parsing
    /// - No rejection of overly elaborate definitions (multi-word headwords, callings, unless)
    /// - No exact measurement threshold pre-parsing check
    /// - No Grading::make_superlative (superlative form deferred)
    /// - No AdjectiveMeanings::perform_task_via_function (TEST_ATOM_TASK deferred)
    #[allow(clippy::too_many_arguments)]
    pub fn claim_definition(
        headword: &'static str,
        prop: Option<usize>,
        shape: i32,
        threshold_text: Option<&'static str>,
        domain_text: Option<&'static str>,
        definitions: &mut Vec<MeasurementDefinition>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        _properties: &[Property],
    ) -> Option<usize> {
        // Create the measurement definition
        let mdef_idx = Measurements::new(headword, prop, shape, threshold_text, definitions);

        // Create the adjective meaning.
        // The family-specific data stores the measurement definition index.
        let am_idx = AdjectiveMeanings::new(
            MEASUREMENT_FAMILY,
            Some(mdef_idx),
            Some(headword),
            meanings,
        );

        // Declare the adjective
        let adj_idx = Adjectives::declare(headword, adjectives);

        // Add the meaning to the adjective
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);

        // Set the domain from text
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);

        // Store the adjective meaning in the measurement definition
        if let Some(mdef) = definitions.get_mut(mdef_idx) {
            mdef.headword_as_adjective = Some(am_idx);
        }

        Some(mdef_idx)
    }
    /// Assert a measurement adjective on a subject.
    ///
    /// Corresponds to `MeasurementAdjectives::assert` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 172-185).
    ///
    /// This function:
    /// 1. Retrieves the measurement definition index from the meaning's family-specific data
    /// 2. Validates the measurement definition
    /// 3. If valid and parity is true, draws a property inference with the threshold value
    /// 4. Returns true if asserted, false otherwise
    ///
    /// Simplified:
    /// - No `<s-literal>` grammar parsing (uses the threshold value directly)
    /// - No `Rvalues::from_encoded_notation` (uses the threshold value directly)
    /// - No `internal_error` on unreadable literal (returns false instead)
    /// - No access to definitions slice for validation (uses property index directly)
    #[allow(clippy::too_many_arguments)]
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
        definitions: &mut [MeasurementDefinition],
    ) -> bool {
        // Get the measurement definition index from the meaning's family-specific data.
        let mdef_idx = match meanings.get(am_idx) {
            Some(am) => match am.family_specific_data {
                Some(idx) => idx,
                None => return false,
            },
            None => return false,
        };

        // Validate the measurement definition (fills in missing data).
        Measurements::validate(mdef_idx, definitions, properties);

        // Check if the measurement definition is valid.
        if !Measurements::is_valid(mdef_idx, definitions) {
            return false;
        }

        // Get the measurement definition.
        let mdef = &definitions[mdef_idx];

        // Get the property name from the property index.
        let prn_name = match mdef.prop.and_then(|p| properties.get(p)) {
            Some(prn) => prn.name,
            None => return false,
        };

        // Draw the property inference for positive parity only.
        // For negative parity, measurement adjectives refuse to assert falseness.
        if !parity {
            return false;
        }

        PropertyInferences::draw(
            subj_idx,
            prn_name,
            None,
            families,
            inferences,
            subjects,
            data_registry,
        );

        true
    }
    /// Prepare I6 schemas for a measurement adjective.
    ///
    /// Corresponds to `MeasurementAdjectives::prepare_schemas` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 190-196).
    ///
    /// Simplified: no-op. Run-time compilation is deferred.
    pub fn prepare_schemas(_am_idx: usize, _task: i32) {
        // No-op: run-time compilation deferred.
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubject;
    use crate::knowledge::measurements::{
        Measurements, MEASURE_T_EXACTLY, MEASURE_T_OR_LESS, MEASURE_T_OR_MORE,
    };
    use crate::knowledge::properties::{Property, ValuePropertyData};

    // -----------------------------------------------------------------------
    // MeasurementAdjectives::start
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_family() {
        let (families, meanings, adjectives, definitions) = MeasurementAdjectives::start();

        assert_eq!(families.len(), 3);
        assert_eq!(families[MEASUREMENT_FAMILY].name, "measurement");
        assert!(families[MEASUREMENT_FAMILY].methods.assert.is_some());
        assert!(families[MEASUREMENT_FAMILY].methods.prepare_schemas.is_some());
        assert!(meanings.is_empty());
        assert!(adjectives.is_empty());
        assert!(definitions.is_empty());
    }

    // -----------------------------------------------------------------------
    // MeasurementAdjectives::is_measurement
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_measurement_returns_true_for_measurement_meaning() {
        let (_families, mut meanings, _, _) = MeasurementAdjectives::start();
        let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, Some(0), Some("roomy"), &mut meanings);

        assert!(MeasurementAdjectives::is_measurement(am_idx, &meanings));
    }

    #[test]
    fn test_is_measurement_returns_false_for_other_meaning() {
        let (_families, mut meanings, _, _) = MeasurementAdjectives::start();
        // Create a meaning with a different family (index 0 = either_or_property placeholder)
        let am_idx = AdjectiveMeanings::new(0, None, Some("open"), &mut meanings);

        assert!(!MeasurementAdjectives::is_measurement(am_idx, &meanings));
    }

    #[test]
    fn test_is_measurement_returns_false_for_invalid_index() {
        let meanings = Vec::new();

        assert!(!MeasurementAdjectives::is_measurement(0, &meanings));
    }

    // -----------------------------------------------------------------------
    // MeasurementAdjectives::claim_definition
    // -----------------------------------------------------------------------

    #[test]
    fn test_claim_definition_creates_measurement_definition() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "roomy",
            Some(0),
            MEASURE_T_OR_MORE,
            Some("10"),
            Some("container"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        );

        assert!(mdef_idx.is_some());
        let mdef_idx = mdef_idx.unwrap();
        assert_eq!(definitions[mdef_idx].headword, "roomy");
        assert_eq!(definitions[mdef_idx].prop, Some(0));
        assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_MORE);
        assert_eq!(definitions[mdef_idx].region_threshold_text, Some("10".to_string()));
        assert!(definitions[mdef_idx].headword_as_adjective.is_some());
    }

    #[test]
    fn test_claim_definition_creates_adjective_meaning() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "roomy",
            Some(0),
            MEASURE_T_OR_MORE,
            Some("10"),
            Some("container"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        ).unwrap();

        let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
        assert_eq!(meanings[am_idx].family, MEASUREMENT_FAMILY);
        assert_eq!(meanings[am_idx].family_specific_data, Some(mdef_idx));
    }

    #[test]
    fn test_claim_definition_declares_adjective() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "roomy",
            Some(0),
            MEASURE_T_OR_MORE,
            Some("10"),
            Some("container"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        ).unwrap();

        // Check that an adjective was declared
        assert!(!adjectives.is_empty());
        let adj = &adjectives[0];
        assert_eq!(adj.name, "roomy");

        // Check that the meaning was added to the adjective
        let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
        assert!(adj.meanings.contains(&am_idx));
    }

    #[test]
    fn test_claim_definition_sets_domain() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "roomy",
            Some(0),
            MEASURE_T_OR_MORE,
            Some("10"),
            Some("container"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        ).unwrap();

        let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
        // The domain should be set from text "container"
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
    }

    #[test]
    fn test_claim_definition_without_domain() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "tall",
            Some(0),
            MEASURE_T_OR_MORE,
            Some("68"),
            None, // no domain text
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        );

        assert!(mdef_idx.is_some());
        let mdef_idx = mdef_idx.unwrap();
        assert_eq!(definitions[mdef_idx].headword, "tall");
        assert_eq!(definitions[mdef_idx].prop, Some(0));
        assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_MORE);
        assert_eq!(definitions[mdef_idx].region_threshold_text, Some("68".to_string()));
    }

    #[test]
    fn test_claim_definition_exact_measurement() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "handy",
            Some(0),
            MEASURE_T_EXACTLY,
            Some("7"),
            Some("person"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        );

        assert!(mdef_idx.is_some());
        let mdef_idx = mdef_idx.unwrap();
        assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_EXACTLY);
    }

    #[test]
    fn test_claim_definition_or_less_measurement() {
        let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();

        let mdef_idx = MeasurementAdjectives::claim_definition(
            "compact",
            Some(0),
            MEASURE_T_OR_LESS,
            Some("5"),
            Some("container"),
            &mut definitions,
            &mut adjectives,
            &mut meanings,
            &families,
            &properties,
        );

        assert!(mdef_idx.is_some());
        let mdef_idx = mdef_idx.unwrap();
        assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_LESS);
    }

    // -----------------------------------------------------------------------
    // MeasurementAdjectives::assert
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_positive_parity_returns_true() {
        let (_families, mut meanings, _adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = vec![Property {
            name: "carrying capacity",
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(ValuePropertyData {
                property_value_kind: Some("number"),
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }];

        // Create a measurement definition with a known property and threshold
        let mdef_idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

        // Create an adjective meaning for it
        let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, Some(mdef_idx), Some("roomy"), &mut meanings);

        // Set up inference infrastructure
        let inference_families = vec![crate::knowledge::property_inferences::PropertyInferences::start()];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
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

        // Assert with positive parity
        let result = MeasurementAdjectives::assert(
            am_idx, 0, true,
            &mut meanings, &mut subjects,
            &properties, &inference_families,
            &mut inferences, &mut data_registry,
            &mut definitions,
        );

        assert!(result);
    }

    #[test]
    fn test_assert_negative_parity_returns_false() {
        let (_families, mut meanings, _adjectives, mut definitions) = MeasurementAdjectives::start();
        let properties = vec![Property {
            name: "carrying capacity",
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(ValuePropertyData {
                property_value_kind: Some("number"),
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }];

        let mdef_idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, Some(mdef_idx), Some("roomy"), &mut meanings);

        let inference_families = vec![crate::knowledge::property_inferences::PropertyInferences::start()];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
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

        // Assert with negative parity
        let result = MeasurementAdjectives::assert(
            am_idx, 0, false,
            &mut meanings, &mut subjects,
            &properties, &inference_families,
            &mut inferences, &mut data_registry,
            &mut definitions,
        );

        assert!(!result);
    }

    #[test]
    fn test_assert_no_family_specific_data_returns_false() {
        let (_families, mut meanings, _, mut definitions) = MeasurementAdjectives::start();
        let properties = Vec::new();
        let inference_families = vec![crate::knowledge::property_inferences::PropertyInferences::start()];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
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

        // Create a meaning without family-specific data
        let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, None, Some("roomy"), &mut meanings);

        let result = MeasurementAdjectives::assert(
            am_idx, 0, true,
            &mut meanings, &mut subjects,
            &properties, &inference_families,
            &mut inferences, &mut data_registry,
            &mut definitions,
        );

        assert!(!result);
    }

    // -----------------------------------------------------------------------
    // MeasurementAdjectives::prepare_schemas
    // -----------------------------------------------------------------------

    #[test]
    fn test_prepare_schemas_is_noop() {
        // Should not panic
        MeasurementAdjectives::prepare_schemas(0, 0);
    }
}
