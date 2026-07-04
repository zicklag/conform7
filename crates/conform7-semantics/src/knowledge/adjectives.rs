/// The Adjective Meaning System — core data structures for adjectives and their meanings.
///
/// Adjectives are words that can be applied to subjects to describe them (e.g., "empty",
/// "open", "red"). Each adjective may have multiple meanings, each belonging to a family
/// and having a domain (what it can apply to).
///
/// | Struct | C Reference | Purpose |
/// |--------|-------------|---------|
/// | [`Adjective`] | `services/linguistics-module/Chapter 2/Adjectives.w` | Core adjective struct |
/// | [`AdjectiveMeaning`] | `inform7/assertions-module/Chapter 8/Adjective Meanings.w` | One meaning an adjective can have |
/// | [`AdjectiveMeaningFamily`] | `inform7/assertions-module/Chapter 8/Adjective Meanings.w` | A family of related adjective meanings |
/// | [`AdjectiveDomainData`] | `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w` | What a meaning can apply to |
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 2/Adjectives.w`
/// - C reference: `inform7/assertions-module/Chapter 8/Adjective Meanings.w`
/// - C reference: `inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`
/// - C reference: `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inferences::{Inference, InferenceFamily};
use crate::knowledge::property_inferences::PropertyInferenceData;
use crate::knowledge::properties::Property;
use crate::knowledge::measurements::MeasurementDefinition;

/// Task indices for adjective meaning atom tasks.
///
/// Corresponds to the `TEST_ATOM_TASK` etc. constants in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 160-163).
pub const TEST_ATOM_TASK: usize = 0;
pub const NOW_ATOM_TRUE_TASK: usize = 1;
pub const NOW_ATOM_FALSE_TASK: usize = 2;

/// Task mode values for adjective meaning atom tasks.
///
/// Corresponds to the `NO_TASKMODE` etc. constants in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 165-167).
pub const NO_TASKMODE: i8 = 0;
pub const DIRECT_TASKMODE: i8 = 1;
pub const VIA_SUPPORT_FUNCTION_TASKMODE: i8 = 2;

/// An adjective — a word that can be applied to subjects to describe them.
///
/// Corresponds to `adjective` in the C reference
/// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 18-30).
///
/// Adjectives can have multiple meanings. For example, "empty" can mean
/// "a container with nothing in it" or "a rulebook with no rules".
/// Each meaning is represented by an `AdjectiveMeaning` struct.
///
/// Simplified: uses string names instead of `lexical_cluster`, and a
/// `Vec<usize>` for meanings instead of linked lists.
#[derive(Clone, Debug)]
pub struct Adjective {
    /// Name of the adjective (simplified: a string instead of `lexical_cluster`).
    pub name: &'static str,
    /// Meanings of this adjective, in definition order.
    /// Corresponds to `in_defn_order` in the C reference.
    pub meanings: Vec<usize>,
    /// Meanings sorted into precedence order.
    /// Corresponds to `in_precedence_order` in the C reference.
    pub sorted_meanings: Vec<usize>,
    /// Compilation data (simplified: a string tag).
    /// Full `adjective_compilation_data` is deferred.
    pub compilation_data: Option<&'static str>,
}

/// One individual meaning which an adjective can have.
///
/// Corresponds to `adjective_meaning` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 10-28).
///
/// For example, "odd" in the sense of numbers is a single meaning.
/// Each meaning belongs to a family and has a domain (what it can apply to).
#[derive(Clone, Debug)]
pub struct AdjectiveMeaning {
    /// The adjective this meaning belongs to (index into the adjective registry).
    pub owning_adjective: Option<usize>,
    /// The domain of this meaning — what kinds/instances it can apply to.
    pub domain: AdjectiveDomainData,
    /// The family this meaning belongs to (index into the family registry).
    pub family: usize,
    /// Family-specific data (index into a family-specific registry).
    pub family_specific_data: Option<usize>,
    /// If this meaning is a negation of another, the index of the original.
    pub negated_from: Option<usize>,
    /// Text to use in the Phrasebook index (simplified: a string).
    pub indexing_text: Option<&'static str>,
    /// Have schemas been prepared yet?
    pub schemas_prepared: bool,
    /// Task mode flags (simplified: no I6 schemas).
    /// Full `adjective_task_data` with I6 schemas is deferred.
    pub task_modes: [i8; 4], // 0=TEST, 1=NOW_TRUE, 2=NOW_FALSE, 3=unused
}

/// The domain of an adjective meaning — what it can validly apply to.
///
/// Corresponds to `adjective_domain_data` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 28-34).
///
/// For example, the meaning of "odd" for numbers has the set of all numbers
/// as its domain, whereas a meaning from "Mrs Elspeth Spong can be odd"
/// has only a single instance as domain.
#[derive(Clone, Debug)]
pub struct AdjectiveDomainData {
    /// Text given by author about the domain (simplified: a string).
    pub domain_text: Option<&'static str>,
    /// What domain the definition applies to (inference subject index).
    pub domain_infs: Option<usize>,
    /// What kind of values (kind index).
    pub domain_kind: Option<usize>,
    /// Are we currently working this out? (for circularity detection)
    pub currently_determining: bool,
}

/// A family of related adjective meanings.
///
/// Corresponds to `adjective_meaning_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 215-219).
///
/// Each family represents a distinct purpose for creating adjectives:
/// - Enumerative (instance-based) adjectives
/// - Either-or property adjectives
/// - Measurement adjectives
/// - Condition-based adjectives
/// - etc.
#[derive(Clone, Debug)]
pub struct AdjectiveMeaningFamily {
    /// Name of the family (for debugging).
    pub name: &'static str,
    /// Priority for claiming definitions (0 to 9: lower is better).
    pub definition_claim_priority: u8,
    /// Methods for this family.
    pub methods: AdjectiveMeaningFamilyMethods,
}
/// Type alias for the assert method on adjective meaning families.
///
/// Takes (meaning_idx, subject_idx, parity, meanings_slice, subjects_slice,
/// properties, families, inferences, data_registry, definitions) and returns true if the assertion was handled.
type AssertFn = fn(
    usize,
    usize,
    bool,
    &mut [AdjectiveMeaning],
    &mut [InferenceSubject],
    &[Property],
    &[InferenceFamily],
    &mut Vec<Inference>,
    &mut Vec<PropertyInferenceData>,
    &mut [MeasurementDefinition],
) -> bool;
/// Type alias for the claim_definition method on adjective meaning families.
type ClaimDefinitionFn = fn(
    &'static str,
    Option<usize>,
    i32,
    Option<&'static str>,
    Option<&'static str>,
    &mut Vec<MeasurementDefinition>,
    &mut Vec<Adjective>,
    &mut Vec<AdjectiveMeaning>,
    &[AdjectiveMeaningFamily],
    &[Property],
) -> Option<usize>;

/// Methods that an adjective meaning family can provide.
///
/// All methods are optional. Corresponds to the method set in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 228-301).
#[derive(Clone, Debug)]
pub struct AdjectiveMeaningFamilyMethods {
    /// Assert the meaning on an inference subject.
    /// Returns true if the assertion was handled.
    pub assert: Option<AssertFn>,
    /// Claim a definition from source text (simplified: no-op).
    pub claim_definition: Option<ClaimDefinitionFn>,
    /// Prepare I6 schemas (simplified: no-op).
    pub prepare_schemas: Option<fn(usize, i32)>,
    /// Produce index text (simplified: no-op).
    pub index: Option<fn(usize) -> Option<&'static str>>,
}

/// The adjective meanings module.
///
/// Corresponds to `AdjectiveMeanings` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`).
pub struct AdjectiveMeanings;

/// The adjective ambiguity module.
///
/// Corresponds to `AdjectiveAmbiguity` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`).
pub struct AdjectiveAmbiguity;

/// The adjective meaning domains module.
///
/// Corresponds to `AdjectiveMeaningDomains` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`).
pub struct AdjectiveMeaningDomains;

// ============================================================================
// Adjectives
// ============================================================================

impl Adjectives {
    /// Find or create an adjective by name.
    ///
    /// Corresponds to `Adjectives::declare` in the C reference
    /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 57-78).
    ///
    /// Simplified:
    /// - No lexical cluster (uses string comparison)
    /// - No linguistic stock
    /// - No lexicon registration
    /// - No compilation data initialisation
    ///
    /// Returns the index of the adjective in the registry.
    pub fn declare(
        name: &'static str,
        registry: &mut Vec<Adjective>,
    ) -> usize {
        // Check if the adjective already exists.
        for (i, adj) in registry.iter().enumerate() {
            if adj.name == name {
                return i;
            }
        }

        // Create a new adjective.
        let adj = Adjective {
            name,
            meanings: Vec::new(),
            sorted_meanings: Vec::new(),
            compilation_data: None,
        };
        let idx = registry.len();
        registry.push(adj);
        idx
    }

    /// Find an existing adjective by name.
    ///
    /// Corresponds to `Adjectives::parse` in the C reference
    /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 122-125).
    ///
    /// Returns the adjective index, or None if not found.
    pub fn find(name: &str, registry: &[Adjective]) -> Option<usize> {
        registry.iter().position(|adj| adj.name == name)
    }

    /// Get the nominative singular form of an adjective.
    ///
    /// Corresponds to `Adjectives::get_nominative_singular` in the C reference
    /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 99-101).
    ///
    /// Simplified: returns the adjective's name directly (no inflection).
    pub fn get_nominative_singular(adj: &Adjective) -> &str {
        adj.name
    }
}

/// Creation and accessor functions for adjectives.
///
/// Corresponds to `Adjectives` in the C reference
/// (`services/linguistics-module/Chapter 2/Adjectives.w`).
pub struct Adjectives;

// ============================================================================
// AdjectiveMeanings
// ============================================================================

impl AdjectiveMeanings {
    /// Create a new adjective meaning family.
    ///
    /// Corresponds to `AdjectiveMeanings::new_family` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 221-226).
    ///
    /// The priority determines the order in which families are offered the
    /// chance to claim definitions (0 to 9: lower is better).
    #[allow(clippy::new_ret_no_self)]
    pub fn new_family(
        name: &'static str,
        priority: u8,
        methods: AdjectiveMeaningFamilyMethods,
        families: &mut Vec<AdjectiveMeaningFamily>,
    ) -> usize {
        let family = AdjectiveMeaningFamily {
            name,
            definition_claim_priority: priority,
            methods,
        };
        let idx = families.len();
        families.push(family);
        idx
    }

    /// Create a new adjective meaning.
    ///
    /// Corresponds to `AdjectiveMeanings::new` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 33-47).
    ///
    /// Simplified:
    /// - No current_sentence tracking
    /// - No task data initialisation (task_modes default to 0 = NO_TASKMODE)
    ///
    /// Returns the index of the new meaning in the registry.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        family_idx: usize,
        details: Option<usize>,
        name: Option<&'static str>,
        registry: &mut Vec<AdjectiveMeaning>,
    ) -> usize {
        let am = AdjectiveMeaning {
            owning_adjective: None,
            domain: AdjectiveDomainData {
                domain_text: None,
                domain_infs: None,
                domain_kind: None,
                currently_determining: false,
            },
            family: family_idx,
            family_specific_data: details,
            negated_from: None,
            indexing_text: name,
            schemas_prepared: false,
            task_modes: [0, 0, 0, 0], // NO_TASKMODE for all tasks
        };
        let idx = registry.len();
        registry.push(am);
        idx
    }

    /// Create a negated copy of an existing adjective meaning.
    ///
    /// Corresponds to `AdjectiveMeanings::negate` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 53-74).
    ///
    /// Simplified:
    /// - No task data negation (task_modes are copied directly)
    /// - No schema modification
    pub fn negate(
        other_idx: usize,
        registry: &mut Vec<AdjectiveMeaning>,
    ) -> usize {
        let other = &registry[other_idx];
        let am = AdjectiveMeaning {
            owning_adjective: None,
            domain: other.domain.clone(),
            family: other.family,
            family_specific_data: other.family_specific_data,
            negated_from: Some(other_idx),
            indexing_text: other.indexing_text,
            schemas_prepared: false,
            task_modes: other.task_modes, // simplified: copy directly
        };
        let idx = registry.len();
        registry.push(am);
        idx
    }

    /// Assert an adjective meaning on an inference subject.
    ///
    /// Corresponds to `AdjectiveMeanings::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 293-301).
    ///
    /// Dispatches to the family's assert method if available.
    /// If the meaning is a negation, it follows the negation chain and
    /// flips the parity.
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
        families: &[AdjectiveMeaningFamily],
        inference_families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        data_registry: &mut Vec<PropertyInferenceData>,
        definitions: &mut [MeasurementDefinition],
    ) -> bool {
        let am = &meanings[am_idx];


        // Follow negation chain.
        let (actual_am_idx, actual_parity) = if let Some(negated_from) = am.negated_from {
            (negated_from, !parity)
        } else {
            (am_idx, parity)
        };

        let actual_am = &meanings[actual_am_idx];
        let family = &families[actual_am.family];

        if let Some(assert_fn) = family.methods.assert {
            assert_fn(actual_am_idx, subj_idx, actual_parity, meanings, subjects, properties, inference_families, inferences, data_registry, definitions)
        } else {
            false // no assert method — decline
        }
    }

    /// Mark an atom task as needing to be performed via a support function.
    ///
    /// Corresponds to `AdjectiveMeanings::perform_task_via_function` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 165-167).
    pub fn perform_task_via_function(am_idx: usize, task: usize, meanings: &mut [AdjectiveMeaning]) {
        if let Some(am) = meanings.get_mut(am_idx) {
            if task < am.task_modes.len() {
                am.task_modes[task] = VIA_SUPPORT_FUNCTION_TASKMODE;
            }
        }
    }
}

// ============================================================================
// AdjectiveAmbiguity
// ============================================================================

impl AdjectiveAmbiguity {
    /// Add a meaning to an adjective.
    ///
    /// Corresponds to `AdjectiveAmbiguity::add_meaning_to_adjective` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 45-50).
    ///
    /// Adds the meaning to the adjective's definition-order list and sets
    /// the meaning's owning_adjective field.
    pub fn add_meaning_to_adjective(
        am_idx: usize,
        adj_idx: usize,
        adjectives: &mut [Adjective],
        meanings: &mut [AdjectiveMeaning],
    ) {
        adjectives[adj_idx].meanings.push(am_idx);
        meanings[am_idx].owning_adjective = Some(adj_idx);
    }

    /// Check if an adjective can be applied to a given kind.
    ///
    /// Corresponds to `AdjectiveAmbiguity::can_be_applied_to` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 95-113).
    ///
    /// Simplified: checks if any meaning of the adjective has a domain kind
    /// that is compatible with the given kind. Uses kind compatibility
    /// instead of the C reference's object/value distinction.
    ///
    /// Returns true if the adjective can be applied, false otherwise.
    pub fn can_be_applied_to(
        adj_idx: usize,
        kind_idx: Option<usize>,
        adjectives: &[Adjective],
        meanings: &[AdjectiveMeaning],
    ) -> bool {
        let adj = &adjectives[adj_idx];
        for &am_idx in &adj.meanings {
            let am = &meanings[am_idx];
            if let Some(am_kind) = am.domain.domain_kind {
                if let Some(target_kind) = kind_idx {
                    if am_kind == target_kind {
                        return true;
                    }
                } else {
                    return true; // null kind matches anything (simplified)
                }
            } else {
                return true; // undetermined domain matches anything (simplified)
            }
        }
        false
    }

    /// Get the first meaning of an adjective.
    ///
    /// Corresponds to `AdjectiveAmbiguity::first_meaning` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 144-148).
    ///
    /// Returns the index of the first meaning, or None if the adjective has no meanings.
    pub fn first_meaning(adj_idx: usize, adjectives: &[Adjective]) -> Option<usize> {
        adjectives
            .get(adj_idx)
            .and_then(|adj| adj.meanings.first().copied())
    }
}

// ============================================================================
// AdjectiveMeaningDomains
// ============================================================================

impl AdjectiveMeaningDomains {
    /// Create domain data from a kind.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::new_from_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 70-76).
    ///
    /// Simplified: no KindSubjects::from_kind (uses kind index directly).
    pub fn new_from_kind(kind_idx: usize) -> AdjectiveDomainData {
        AdjectiveDomainData {
            domain_text: None,
            domain_infs: None, // simplified: no inference subject lookup
            domain_kind: Some(kind_idx),
            currently_determining: false,
        }
    }

    /// Create domain data from text (simplified: no resolution).
    ///
    /// Corresponds to `AdjectiveMeaningDomains::new_from_text` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 36-40).
    ///
    /// Simplified: stores the text but does not resolve it to a kind or subject.
    /// Full domain determination is deferred.
    pub fn new_from_text(text: Option<&'static str>) -> AdjectiveDomainData {
        AdjectiveDomainData {
            domain_text: text,
            domain_infs: None,
            domain_kind: None,
            currently_determining: false,
        }
    }

    /// Create domain data from an instance (simplified: uses instance index as kind).
    ///
    /// Corresponds to `AdjectiveMeaningDomains::new_from_instance` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 78-83).
    ///
    /// Simplified: stores the instance index as the domain kind. In the full
    /// implementation, this would create an inference subject for the instance.
    pub fn new_from_instance(instance_idx: usize) -> AdjectiveDomainData {
        AdjectiveDomainData {
            domain_text: None,
            domain_infs: None, // simplified: no inference subject lookup
            domain_kind: Some(instance_idx),
            currently_determining: false,
        }
    }

    /// Set the domain of an adjective meaning from a kind.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::set_from_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 67-69).
    pub fn set_from_kind(
        am_idx: usize,
        kind_idx: usize,
        meanings: &mut [AdjectiveMeaning],
    ) {
        meanings[am_idx].domain = AdjectiveMeaningDomains::new_from_kind(kind_idx);
    }

    /// Set the domain of an adjective meaning from an instance.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::set_from_instance` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 78-83).
    pub fn set_from_instance(
        am_idx: usize,
        instance_idx: usize,
        meanings: &mut [AdjectiveMeaning],
    ) {
        meanings[am_idx].domain = AdjectiveMeaningDomains::new_from_instance(instance_idx);
    }

    /// Set the domain of an adjective meaning from text.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::set_from_text` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 36-40).
    ///
    /// Simplified: stores the text but does not resolve it.
    pub fn set_from_text(
        am_idx: usize,
        text: Option<&'static str>,
        meanings: &mut [AdjectiveMeaning],
    ) {
        meanings[am_idx].domain = AdjectiveMeaningDomains::new_from_text(text);
    }

    /// Get the kind of a meaning's domain.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::get_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 238-242).
    ///
    /// Returns the kind index, or None if the domain is undetermined.
    pub fn get_kind(am_idx: usize, meanings: &[AdjectiveMeaning]) -> Option<usize> {
        meanings.get(am_idx).and_then(|am| am.domain.domain_kind)
    }

    /// Get the inference subject of a meaning's domain.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::get_subject` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 244-248).
    ///
    /// Returns the inference subject index, or None if the domain has no subject.
    pub fn get_subject(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
    ) -> Option<usize> {
        meanings.get(am_idx).and_then(|am| am.domain.domain_infs)
    }

    /// Weak domain matching — check if a kind is close enough for run-time checking.
    ///
    /// Corresponds to `AdjectiveMeaningDomains::weak_match` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 265-269).
    ///
    /// Simplified: exact kind comparison (no weak_iname weakening).
    /// In the full implementation, this would use RTKindIDs::weak_iname to
    /// treat all object kinds as just "object".
    pub fn weak_match(
        kind_idx: usize,
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
    ) -> bool {
        if let Some(am_kind) = AdjectiveMeaningDomains::get_kind(am_idx, meanings) {
            kind_idx == am_kind
        } else {
            false // undetermined domain never matches
        }
    }

    /// Determine the domain of an adjective meaning (simplified: no-op).
    ///
    /// Corresponds to `AdjectiveMeaningDomains::determine` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 85-236).
    ///
    /// Simplified: does nothing. The full implementation would resolve text-based
    /// domains to kinds or inference subjects, with circularity detection.
    pub fn determine(_am_idx: usize, _meanings: &mut [AdjectiveMeaning]) {
        // No-op: full domain determination is deferred.
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Adjectives::declare
    // -----------------------------------------------------------------------

    #[test]
    fn test_declare_creates_new_adjective() {
        let mut registry = Vec::new();
        let idx = Adjectives::declare("empty", &mut registry);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry[idx].name, "empty");
        assert!(registry[idx].meanings.is_empty());
        assert!(registry[idx].sorted_meanings.is_empty());
        assert!(registry[idx].compilation_data.is_none());
    }

    #[test]
    fn test_declare_returns_existing_adjective() {
        let mut registry = Vec::new();
        let idx1 = Adjectives::declare("empty", &mut registry);
        let idx2 = Adjectives::declare("empty", &mut registry);
        assert_eq!(idx1, idx2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_declare_assigns_increasing_indices() {
        let mut registry = Vec::new();
        let idx1 = Adjectives::declare("empty", &mut registry);
        let idx2 = Adjectives::declare("open", &mut registry);
        let idx3 = Adjectives::declare("red", &mut registry);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(registry.len(), 3);
    }

    // -----------------------------------------------------------------------
    // Adjectives::find
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_finds_existing_adjective() {
        let mut registry = Vec::new();
        Adjectives::declare("empty", &mut registry);
        let result = Adjectives::find("empty", &registry);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_find_returns_none_for_non_existent() {
        let registry: Vec<Adjective> = Vec::new();
        let result = Adjectives::find("nonexistent", &registry);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_returns_none_when_not_declared() {
        let mut registry = Vec::new();
        Adjectives::declare("empty", &mut registry);
        let result = Adjectives::find("open", &registry);
        assert_eq!(result, None);
    }

    // -----------------------------------------------------------------------
    // Adjectives::get_nominative_singular
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_nominative_singular_returns_name() {
        let mut registry = Vec::new();
        let idx = Adjectives::declare("empty", &mut registry);
        assert_eq!(
            Adjectives::get_nominative_singular(&registry[idx]),
            "empty"
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeanings::new_family
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_family_creates_family_with_correct_priority() {
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family("test", 5, methods, &mut families);
        assert_eq!(families[idx].name, "test");
        assert_eq!(families[idx].definition_claim_priority, 5);
    }

    #[test]
    fn test_new_family_creates_family_with_correct_methods() {
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: Some(|_, _, _, _, _, _, _, _, _, _| true),
            claim_definition: Some(|_, _, _, _, _, _, _, _, _, _| Some(0)),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family("test", 3, methods, &mut families);
        assert!(families[idx].methods.assert.is_some());
        assert!(families[idx].methods.claim_definition.is_some());
        assert!(families[idx].methods.prepare_schemas.is_none());
        assert!(families[idx].methods.index.is_none());
    }

    #[test]
    fn test_new_family_assigns_increasing_indices() {
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let idx1 = AdjectiveMeanings::new_family("a", 1, methods.clone(), &mut families);
        let idx2 = AdjectiveMeanings::new_family("b", 2, methods.clone(), &mut families);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(families.len(), 2);
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeanings::new
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_meaning_with_correct_family() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        assert_eq!(meanings[am_idx].family, fam_idx);
        assert!(meanings[am_idx].owning_adjective.is_none());
        assert!(!meanings[am_idx].schemas_prepared);
    }

    #[test]
    fn test_new_creates_meaning_with_correct_family_specific_data() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(
            fam_idx,
            Some(42),
            None,
            &mut meanings,
        );
        assert_eq!(
            meanings[am_idx].family_specific_data,
            Some(42)
        );
    }

    #[test]
    fn test_new_creates_meaning_with_correct_indexing_text() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(
            fam_idx,
            None,
            Some("index text"),
            &mut meanings,
        );
        assert_eq!(meanings[am_idx].indexing_text, Some("index text"));
    }

    #[test]
    fn test_new_creates_meaning_with_default_task_modes() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        assert_eq!(meanings[am_idx].task_modes, [0, 0, 0, 0]);
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeanings::negate
    // -----------------------------------------------------------------------

    #[test]
    fn test_negate_creates_negated_meaning() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let orig_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let neg_idx = AdjectiveMeanings::negate(orig_idx, &mut meanings);

        assert_eq!(meanings.len(), 2);
        assert_eq!(meanings[neg_idx].negated_from, Some(orig_idx));
        assert_eq!(meanings[neg_idx].family, fam_idx);
    }

    #[test]
    fn test_negate_sets_negated_from_correctly() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let orig_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let neg_idx = AdjectiveMeanings::negate(orig_idx, &mut meanings);

        assert_eq!(meanings[neg_idx].negated_from, Some(orig_idx));
        assert!(meanings[orig_idx].negated_from.is_none());
    }

    #[test]
    fn test_negate_copies_domain_from_original() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let orig_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        // Set a domain kind on the original.
        AdjectiveMeaningDomains::set_from_kind(orig_idx, 42, &mut meanings);

        let neg_idx = AdjectiveMeanings::negate(orig_idx, &mut meanings);
        assert_eq!(
            AdjectiveMeaningDomains::get_kind(neg_idx, &meanings),
            Some(42)
        );
    }

    #[test]
    fn test_negate_copies_family_specific_data() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let orig_idx = AdjectiveMeanings::new(
            fam_idx,
            Some(42),
            None,
            &mut meanings,
        );
        let neg_idx = AdjectiveMeanings::negate(orig_idx, &mut meanings);

        assert_eq!(
            meanings[neg_idx].family_specific_data,
            Some(42)
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeanings::assert
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_dispatches_to_family_assert_method() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let mut subjects = Vec::new();
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();

        // Create a family with an assert method that records its arguments.
        let assert_fn: AssertFn =
            |am_idx, _subj_idx, parity, _meanings, _subjects, _properties, _families, _inferences, _data_registry, _definitions| {
                am_idx == 0 && parity
            };

        let methods = AdjectiveMeaningFamilyMethods {
            assert: Some(assert_fn),
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        let result = AdjectiveMeanings::assert(
            am_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &[], // no properties
            &families,
            &[],
            &mut inferences,
            &mut data_registry,
            &mut [], // no definitions
        );
        assert!(result);
    }

    #[test]
    fn test_assert_returns_false_when_no_assert_method() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let mut subjects = Vec::new();
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let result = AdjectiveMeanings::assert(
            am_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &[], // no properties
            &families,
            &[],
            &mut inferences,
            &mut data_registry,
            &mut [], // no definitions
        );
        assert!(!result);
    }

    #[test]
    fn test_assert_follows_negation_chain_and_flips_parity() {
        let mut families = Vec::new();
        let mut meanings = Vec::new();
        let mut subjects = Vec::new();
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();
        let assert_fn: AssertFn =
            |_am_idx, _subj_idx, parity, _meanings, _subjects, _properties, _families, _inferences, _data_registry, _definitions| parity;

        let methods = AdjectiveMeaningFamilyMethods {
            assert: Some(assert_fn),
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let orig_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let neg_idx = AdjectiveMeanings::negate(orig_idx, &mut meanings);

        // Assert the negated meaning with parity=true.
        // The negation chain should flip parity to false, so assert returns false.
        let result = AdjectiveMeanings::assert(
            neg_idx,
            0,
            true,
            &mut meanings,
            &mut subjects,
            &[], // no properties
            &families,
            &[],
            &mut inferences,
            &mut data_registry,
            &mut [], // no definitions
        );
        assert!(!result);

        // Assert the negated meaning with parity=false.
        let result2 = AdjectiveMeanings::assert(
            neg_idx,
            0,
            false,
            &mut meanings,
            &mut subjects,
            &[], // no properties
            &families,
            &[],
            &mut inferences,
            &mut data_registry,
            &mut [], // no definitions
        );
        assert!(result2);

    }

    // -----------------------------------------------------------------------
    // AdjectiveAmbiguity::add_meaning_to_adjective
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_meaning_to_adjective_adds_to_list() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("empty", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert_eq!(adjectives[adj_idx].meanings, vec![am_idx]);
    }

    #[test]
    fn test_add_meaning_to_adjective_sets_owning_adjective() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("empty", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert_eq!(meanings[am_idx].owning_adjective, Some(adj_idx));
    }

    #[test]
    fn test_add_meaning_to_adjective_appends_multiple_meanings() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("empty", &mut adjectives);
        let am1_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let am2_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am1_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );
        AdjectiveAmbiguity::add_meaning_to_adjective(
            am2_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert_eq!(adjectives[adj_idx].meanings, vec![am1_idx, am2_idx]);
    }

    // -----------------------------------------------------------------------
    // AdjectiveAmbiguity::can_be_applied_to
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_be_applied_to_returns_true_for_matching_kind() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("odd", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert!(
            AdjectiveAmbiguity::can_be_applied_to(
                adj_idx,
                Some(42),
                &adjectives,
                &meanings,
            )
        );
    }

    #[test]
    fn test_can_be_applied_to_returns_false_for_non_matching_kind() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("odd", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert!(
            !AdjectiveAmbiguity::can_be_applied_to(
                adj_idx,
                Some(99),
                &adjectives,
                &meanings,
            )
        );
    }

    #[test]
    fn test_can_be_applied_to_returns_true_for_undetermined_domain() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("odd", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        // Undetermined domain (no domain_kind set) matches anything.
        assert!(
            AdjectiveAmbiguity::can_be_applied_to(
                adj_idx,
                Some(42),
                &adjectives,
                &meanings,
            )
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveAmbiguity::first_meaning
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_meaning_returns_first_meaning() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("empty", &mut adjectives);
        let am1_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);
        let am2_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am1_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );
        AdjectiveAmbiguity::add_meaning_to_adjective(
            am2_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        assert_eq!(
            AdjectiveAmbiguity::first_meaning(adj_idx, &adjectives),
            Some(am1_idx)
        );
    }

    #[test]
    fn test_first_meaning_returns_none_for_no_meanings() {
        let mut adjectives = Vec::new();
        let adj_idx = Adjectives::declare("empty", &mut adjectives);

        assert_eq!(
            AdjectiveAmbiguity::first_meaning(adj_idx, &adjectives),
            None
        );
    }

    #[test]
    fn test_first_meaning_returns_none_for_invalid_index() {
        let adjectives: Vec<Adjective> = Vec::new();
        assert_eq!(
            AdjectiveAmbiguity::first_meaning(0, &adjectives),
            None
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::new_from_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_from_kind_creates_domain_with_correct_kind() {
        let domain = AdjectiveMeaningDomains::new_from_kind(42);
        assert_eq!(domain.domain_kind, Some(42));
        assert!(domain.domain_text.is_none());
        assert!(domain.domain_infs.is_none());
        assert!(!domain.currently_determining);
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::new_from_text
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_from_text_creates_domain_with_text() {
        let domain = AdjectiveMeaningDomains::new_from_text(Some("numbers"));
        assert_eq!(domain.domain_text, Some("numbers"));
        assert!(domain.domain_kind.is_none());
        assert!(domain.domain_infs.is_none());
    }

    #[test]
    fn test_new_from_text_creates_domain_with_none_text() {
        let domain = AdjectiveMeaningDomains::new_from_text(None);
        assert!(domain.domain_text.is_none());
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::new_from_instance
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_from_instance_creates_domain_with_instance() {
        let domain = AdjectiveMeaningDomains::new_from_instance(7);
        assert_eq!(domain.domain_kind, Some(7));
        assert!(domain.domain_text.is_none());
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::set_from_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_from_kind_updates_meaning_domain() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        assert_eq!(meanings[am_idx].domain.domain_kind, Some(42));
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::set_from_instance
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_from_instance_updates_meaning_domain() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_instance(am_idx, 7, &mut meanings);
        assert_eq!(meanings[am_idx].domain.domain_kind, Some(7));
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::set_from_text
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_from_text_updates_meaning_domain() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_text(
            am_idx,
            Some("numbers"),
            &mut meanings,
        );
        assert_eq!(
            meanings[am_idx].domain.domain_text,
            Some("numbers")
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::get_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_kind_returns_domain_kind() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        assert_eq!(
            AdjectiveMeaningDomains::get_kind(am_idx, &meanings),
            Some(42)
        );
    }

    #[test]
    fn test_get_kind_returns_none_for_undetermined_domain() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        assert_eq!(
            AdjectiveMeaningDomains::get_kind(am_idx, &meanings),
            None
        );
    }

    #[test]
    fn test_get_kind_returns_none_for_invalid_index() {
        let meanings: Vec<AdjectiveMeaning> = Vec::new();
        assert_eq!(
            AdjectiveMeaningDomains::get_kind(0, &meanings),
            None
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::get_subject
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_subject_returns_none_when_no_subject() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        assert_eq!(
            AdjectiveMeaningDomains::get_subject(am_idx, &meanings),
            None
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::weak_match
    // -----------------------------------------------------------------------

    #[test]
    fn test_weak_match_returns_true_for_matching_kinds() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        assert!(
            AdjectiveMeaningDomains::weak_match(42, am_idx, &meanings)
        );
    }

    #[test]
    fn test_weak_match_returns_false_for_non_matching_kinds() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveMeaningDomains::set_from_kind(am_idx, 42, &mut meanings);
        assert!(
            !AdjectiveMeaningDomains::weak_match(99, am_idx, &meanings)
        );
    }

    #[test]
    fn test_weak_match_returns_false_for_undetermined_domain() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        assert!(
            !AdjectiveMeaningDomains::weak_match(42, am_idx, &meanings)
        );
    }

    // -----------------------------------------------------------------------
    // AdjectiveMeaningDomains::determine
    // -----------------------------------------------------------------------

    #[test]
    fn test_determine_is_no_op() {
        let mut meanings = Vec::new();
        let mut families = Vec::new();
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        // Set a text domain.
        AdjectiveMeaningDomains::set_from_text(
            am_idx,
            Some("numbers"),
            &mut meanings,
        );

        // Determine should not change anything (no-op).
        AdjectiveMeaningDomains::determine(am_idx, &mut meanings);
        assert_eq!(
            meanings[am_idx].domain.domain_text,
            Some("numbers")
        );
        assert!(meanings[am_idx].domain.domain_kind.is_none());
    }

    // -----------------------------------------------------------------------
    // Integration: declare + add meaning + first meaning
    // -----------------------------------------------------------------------

    #[test]
    fn test_declare_and_add_meaning_integration() {
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();
        let mut families = Vec::new();

        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: None,
            prepare_schemas: None,
            index: None,
        };
        let fam_idx = AdjectiveMeanings::new_family("test", 0, methods, &mut families);
        let adj_idx = Adjectives::declare("empty", &mut adjectives);
        let am_idx = AdjectiveMeanings::new(fam_idx, None, None, &mut meanings);

        AdjectiveAmbiguity::add_meaning_to_adjective(
            am_idx,
            adj_idx,
            &mut adjectives,
            &mut meanings,
        );

        // Verify the round-trip: find the adjective, get its first meaning.
        let found = Adjectives::find("empty", &adjectives).unwrap();
        assert_eq!(found, adj_idx);
        assert_eq!(
            AdjectiveAmbiguity::first_meaning(adj_idx, &adjectives),
            Some(am_idx)
        );
    }

    // -----------------------------------------------------------------------
    // Integration: declare + find + get_nominative_singular
    // -----------------------------------------------------------------------

    #[test]
    fn test_declare_find_and_get_nominative_singular() {
        let mut adjectives = Vec::new();
        let idx = Adjectives::declare("open", &mut adjectives);

        let found = Adjectives::find("open", &adjectives);
        assert_eq!(found, Some(idx));

        let name = Adjectives::get_nominative_singular(&adjectives[idx]);
        assert_eq!(name, "open");
    }
}
