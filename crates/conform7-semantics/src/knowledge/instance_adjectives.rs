/// Instance Adjectives — the enumerative adjective meaning family.
///
/// The second concrete adjective meaning family. Creates the `enumerative_amf`
/// family with an assert method, enabling instances of enumerated kinds
/// (e.g., "red", "blue", "green" for the colour kind) to be used as
/// adjectives in the model world.
///
/// | Struct | C Reference | Purpose |
/// |--------|-------------|---------|
/// | [`InstanceAdjectives`] | `Chapter 2/Instances as Adjectives.w` | Instance adjective management |
///
/// # References
///
/// - C reference: `inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`
use crate::kind_constructors::KindConstructor;
use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
};
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inferences::{Inference, InferenceFamily};
use crate::knowledge::instances::{Instance, Instances};
use crate::knowledge::properties::{Properties, Property};
use crate::knowledge::property_inferences::{PropertyInferenceData, PropertyInferences};
use crate::knowledge::measurements::MeasurementDefinition;

/// Index of the enumerative family in the family registry.
pub const ENUMERATIVE_FAMILY: usize = 0;

/// The instance adjectives module.
///
/// Corresponds to `InstanceAdjectives` in the C reference
/// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`).
pub struct InstanceAdjectives;

impl InstanceAdjectives {
    /// Create the enumerative adjective meaning family with its methods.
    ///
    /// Corresponds to `InstanceAdjectives::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 135-138).
    ///
    /// Returns (families, meanings, adjectives) where:
    /// - families[0] = enumerative_amf
    /// - meanings is empty (make_adjectival fills it)
    /// - adjectives is empty (make_adjectival fills it)
    pub fn start() -> (Vec<AdjectiveMeaningFamily>, Vec<AdjectiveMeaning>, Vec<Adjective>) {
        let enumerative_family = AdjectiveMeaningFamily {
            name: "enumerative",
            definition_claim_priority: 2,
            methods: AdjectiveMeaningFamilyMethods {
                assert: Some(InstanceAdjectives::assert),
                claim_definition: None,
                prepare_schemas: None,
                index: None,
            },
        };

        (vec![enumerative_family], Vec::new(), Vec::new())
    }

    /// Check if an adjective meaning belongs to the enumerative family.
    ///
    /// Corresponds to `InstanceAdjectives::is_enumerative` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 140-143).
    ///
    /// Returns true if the meaning belongs to this family, false otherwise.
    pub fn is_enumerative(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == ENUMERATIVE_FAMILY)
    }

    /// Register an instance as an adjective for a given property.
    ///
    /// Corresponds to `InstanceAdjectives::make_adjectival` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 149-156).
    ///
    /// This function:
    /// 1. Finds the kind domain (from singleton or set)
    /// 2. Gets the instance name
    /// 3. Declares a new adjective via `Adjectives::declare`
    /// 4. Finds the property with the same name as the instance's kind
    /// 5. Creates a new adjective meaning with the enumerative family and
    ///    the property name as family-specific data
    /// 6. Adds the meaning to the adjective via `AdjectiveAmbiguity::add_meaning_to_adjective`
    /// 7. Stores the adjective index in `instance.as_adjective`
    /// 8. Sets the domain from the kind or instance via
    ///    `AdjectiveMeaningDomains::set_from_kind` or `set_from_instance`
    ///
    /// Simplified:
    /// - No I6 schema writing (run-time compilation deferred)
    /// - No internal_error for missing domain (returns silently)
    ///
    /// Returns the adjective index.
    #[allow(clippy::too_many_arguments)]
    pub fn make_adjectival(
        inst_idx: usize,
        prn_idx: Option<usize>,
        set_kind_idx: Option<usize>,
        singleton_idx: Option<usize>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        properties: &[Property],
        instances: &mut [Instance],
        subjects: &[InferenceSubject],
        constructors: &[KindConstructor],
    ) -> usize {
        // Find the kind domain within which the adjective applies.
        // Corresponds to @<Find the kind domain within which the adjective applies@>
        // in the C reference.
        let domain_kind_idx = if let Some(singleton) = singleton_idx {
            Instances::to_kind(singleton, instances, subjects, constructors)
                .map(|k| k.construct_id)
                .or(set_kind_idx)
        } else {
            set_kind_idx
        };

        if domain_kind_idx.is_none() {
            // No domain — can't create the adjective meaning.
            // In the C reference, this is an internal_error.
            // Simplified: return a placeholder adjective index.
            return Adjectives::declare(Instances::get_name(inst_idx, instances).unwrap_or("?"), adjectives);
        }

        // Get the instance name.
        let inst_name = match Instances::get_name(inst_idx, instances) {
            Some(name) => name,
            None => return Adjectives::declare("?", adjectives),
        };

        // Find the property with the same name as the instance's kind.
        // Use prn_idx if provided, otherwise derive from the kind.
        let prn_idx_val = if let Some(prn) = prn_idx {
            Some(prn)
        } else {
            let kind = Instances::to_kind(inst_idx, instances, subjects, constructors);
            kind.as_ref().and_then(|k| {
                let con_name = constructors.get(k.construct_id).map(|c| c.name).unwrap_or(k.construct.name);
                Properties::property_with_same_name_as(con_name, properties)
            })
        };

        // Declare a new adjective.
        // Corresponds to `Adjectives::declare(NW, NULL)` in the C reference.
        let adj_idx = Adjectives::declare(inst_name, adjectives);

        // Create a new adjective meaning with the enumerative family.
        // The family-specific data stores the property index (used by assert).
        // The indexing text stores the instance name (used as the value).
        // Corresponds to `AdjectiveMeanings::new(enumerative_amf, STORE_POINTER_instance(I), NW)`
        // in the C reference.
        let am = AdjectiveMeanings::new(
            ENUMERATIVE_FAMILY,
            prn_idx_val,    // family_specific_data = property index
            Some(inst_name), // indexing_text = instance name
            meanings,
        );

        // Add the meaning to the adjective.
        // Corresponds to `AdjectiveAmbiguity::add_meaning_to_adjective(am, adj)`.
        AdjectiveAmbiguity::add_meaning_to_adjective(am, adj_idx, adjectives, meanings);

        // Store the adjective index in the instance.
        // Corresponds to `I->as_adjective = ...` in the C reference.
        if let Some(inst) = instances.get_mut(inst_idx) {
            inst.as_adjective = Some(adj_idx);
        }

        // Set the domain from the kind or instance.
        // Corresponds to `AdjectiveMeaningDomains::set_from_kind` or
        // `AdjectiveMeaningDomains::set_from_instance` in the C reference.
        if let Some(singleton) = singleton_idx {
            AdjectiveMeaningDomains::set_from_instance(am, singleton, meanings);
        } else if let Some(kind) = domain_kind_idx {
            AdjectiveMeaningDomains::set_from_kind(am, kind, meanings);
        }

        adj_idx
    }

    /// Assert an instance adjective on an inference subject.
    ///
    /// Corresponds to `InstanceAdjectives::assert` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 177-185).
    ///
    /// For positive parity (e.g., "the ball is red"), calls
    /// `PropertyInferences::draw` with the property and instance value.
    ///
    /// For negative parity (e.g., "the ball is not red"), returns FALSE
    /// (refuses to assert falseness since it's unclear what to infer).
    ///
    /// Simplified:
    /// - Property name is retrieved from the meaning's family_specific_data
    /// - Instance name is retrieved from the meaning's indexing_text
    ///
    /// Returns true if the assertion was handled, false otherwise.
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
        _definitions: &mut [MeasurementDefinition],
    ) -> bool {
        // For negative parity, return FALSE (refuse to assert falseness).
        // In the C reference: "if (parity == FALSE) return FALSE;"
        if !parity {
            return false;
        }

        let am = &meanings[am_idx];

        // Retrieve the property index from the meaning's family-specific data.
        let prn_idx = match am.family_specific_data {
            Some(idx) => idx,
            None => return false, // No property data — decline.
        };

        // Look up the property name from the index.
        let prn_name = match properties.get(prn_idx) {
            Some(prn) => prn.name,
            None => return false, // Property not found — decline.
        };

        // Retrieve the instance name from the meaning's indexing text.
        let inst_name = am.indexing_text;

        // Draw the property inference.
        // Corresponds to `PropertyInferences::draw(infs_to_assert_on, P, Rvalues::from_instance(I))`
        // in the C reference.
        PropertyInferences::draw(
            subj_idx,
            prn_name,
            inst_name,
            families,
            inferences,
            subjects,
            data_registry,
        );

        true
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind_constructors::KindConstructor;
    use crate::knowledge::adjectives::AdjectiveMeaningFamilyMethods;
    use crate::knowledge::inference_subjects::{InferenceSubject, InferenceSubjectFamily};
    use crate::knowledge::instance_subjects::InstanceSubjects;
    use crate::knowledge::kind_subjects;
    use crate::knowledge::properties::ValuePropertyData;

    /// Helper: create a minimal kind constructor with an inference subject.
    fn make_kind_constructor(
        name: &'static str,
        constructors: &mut Vec<KindConstructor>,
        subjects: &mut Vec<InferenceSubject>,
        families: &[InferenceSubjectFamily],
    ) -> usize {
        let idx = constructors.len();
        let con = KindConstructor::new(name, crate::kind_constructors::ConstructorGroup::Base, 0);
        constructors.push(con);
        kind_subjects::new(&mut constructors[idx], subjects, families);
        idx
    }

    /// Helper: create a valued property (not either-or).
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

    // -----------------------------------------------------------------------
    // InstanceAdjectives::start
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_family_with_correct_name_and_priority() {
        let (families, meanings, adjectives) = InstanceAdjectives::start();

        assert_eq!(families.len(), 1);
        assert_eq!(families[0].name, "enumerative");
        assert_eq!(families[0].definition_claim_priority, 2);
        assert!(meanings.is_empty());
        assert!(adjectives.is_empty());
    }

    #[test]
    fn test_start_creates_family_with_assert_method() {
        let (families, _, _) = InstanceAdjectives::start();

        assert!(families[0].methods.assert.is_some());
    }

    // -----------------------------------------------------------------------
    // InstanceAdjectives::is_enumerative
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_enumerative_returns_true_for_enumerative_meaning() {
        let (_families, mut meanings, _) = InstanceAdjectives::start();

        let am_idx = AdjectiveMeanings::new(0, None, None, &mut meanings);

        assert!(InstanceAdjectives::is_enumerative(am_idx, &meanings));
    }

    #[test]
    fn test_is_enumerative_returns_false_for_different_family() {
        let (mut families, mut meanings, _) = InstanceAdjectives::start();

        // Create a second family.
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let other_fam_idx = AdjectiveMeanings::new_family("other", 0, methods, &mut families);

        let am_idx = AdjectiveMeanings::new(other_fam_idx, None, None, &mut meanings);

        assert!(!InstanceAdjectives::is_enumerative(am_idx, &meanings));
    }

    #[test]
    fn test_is_enumerative_returns_false_for_invalid_index() {
        let meanings = Vec::new();

        assert!(!InstanceAdjectives::is_enumerative(0, &meanings));
    }

    // -----------------------------------------------------------------------
    // InstanceAdjectives::make_adjectival
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_adjectival_declares_new_adjective() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        let adj_idx = InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        assert_eq!(adjectives[adj_idx].name, "red");
    }

    #[test]
    fn test_make_adjectival_creates_meaning_with_correct_family() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        // There should be one meaning, belonging to the enumerative family.
        assert_eq!(meanings.len(), 1);
        assert_eq!(meanings[0].family, ENUMERATIVE_FAMILY);
    }

    #[test]
    fn test_make_adjectival_sets_family_specific_data_to_property_index() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        // The family_specific_data should be the property index (0)
        // since the property "colour" is at index 0.
        assert_eq!(meanings[0].family_specific_data, Some(0));
    }

    #[test]
    fn test_make_adjectival_sets_indexing_text_to_instance_name() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        // The indexing_text should be the instance name ("red").
        assert_eq!(meanings[0].indexing_text, Some("red"));
    }

    #[test]
    fn test_make_adjectival_adds_meaning_to_adjective() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        let adj_idx = InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        assert_eq!(adjectives[adj_idx].meanings.len(), 1);
        assert_eq!(adjectives[adj_idx].meanings[0], 0);
        assert_eq!(meanings[0].owning_adjective, Some(adj_idx));
    }

    #[test]
    fn test_make_adjectival_sets_domain_from_kind() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();
        let properties = vec![make_valued_property("colour")];

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        assert_eq!(meanings[0].domain.domain_kind, Some(colour_idx));
    }

    // -----------------------------------------------------------------------
    // InstanceAdjectives::assert
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_with_positive_parity_draws_property_inference() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
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
        let properties = vec![make_valued_property("colour")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        // Create the adjective meaning.
        let adj_idx = InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        // Get the meaning index.
        let am_idx = adjectives[adj_idx].meanings[0];

        // Assert with positive parity.
        let result = InstanceAdjectives::assert(
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
        assert_eq!(inferences[0].data, Some("colour"));
    }

    #[test]
    fn test_assert_with_negative_parity_returns_false() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
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
        let properties = vec![make_valued_property("colour")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        let adj_idx = InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        let am_idx = adjectives[adj_idx].meanings[0];

        let result = InstanceAdjectives::assert(
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

        assert!(!result);
        // No inference should have been added.
        assert_eq!(subjects[0].inf_list.len(), 0);
    }

    #[test]
    fn test_assert_returns_false_when_no_family_specific_data() {
        let (_families, mut meanings, _) = InstanceAdjectives::start();
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
        let result = InstanceAdjectives::assert(
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

    #[test]
    fn test_assert_returns_true() {
        let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
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
        let properties = vec![make_valued_property("colour")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let infs_families = vec![
            crate::knowledge::inference_subjects::InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &infs_families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &infs_families, &constructors);

        let adj_idx = InstanceAdjectives::make_adjectival(inst_idx, None, Some(colour_idx), None, &mut adjectives, &mut meanings, &families, &properties, &mut instances, &subjects, &constructors);

        let am_idx = adjectives[adj_idx].meanings[0];
        let result = InstanceAdjectives::assert(
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
}
