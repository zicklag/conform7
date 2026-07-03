//! Adjectives by Phrase — adjectives defined by an explicit Inform 7 phrase.
//!
//! Corresponds to `AdjectivesByPhrase` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`).
//!
//! This module creates the `phrase_amf` family and claims adjective definitions
//! whose body is an explicit phrase (`Definition: a ... is ...: ...`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No Preform grammar parsing.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_I7_phrase`.
//! - The family method wrapper uses a static family index to fit the existing
//!   measurement-shaped `ClaimDefinitionFn` signature.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
    TEST_ATOM_TASK,
};
use crate::knowledge::measurements::MeasurementDefinition;
use crate::knowledge::properties::Property;

/// Global family index for the phrase family.
///
/// Mirrors the C static `phrase_amf`. Set by `AdjectivesByPhrase::start`.
static PHRASE_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Phrase module.
///
/// Corresponds to `AdjectivesByPhrase` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`).
pub struct AdjectivesByPhrase;

impl AdjectivesByPhrase {
    /// Priority of the phrase family in the definition-claim order.
    ///
    /// Corresponds to the `6` passed to `AdjectiveMeanings::new_family` in the C reference.
    pub const PHRASE_FAMILY_PRIORITY: u8 = 6;

    /// Create the phrase adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByPhrase::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`, lines 12-15).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByPhrase::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family("phrase", Self::PHRASE_FAMILY_PRIORITY, methods, families);
        PHRASE_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the phrase family set by the most recent `start`.
    fn phrase_family_idx() -> usize {
        PHRASE_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the phrase family.
    ///
    /// Corresponds to `AdjectivesByPhrase::is_defined_by_phrase` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Phrase.w`, lines 17-20).
    pub fn is_defined_by_phrase(am_idx: usize, meanings: &[AdjectiveMeaning], phrase_family_idx: usize) -> bool {
        meanings.get(am_idx).is_some_and(|am| am.family == phrase_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only phrasally-defined adjectives (`sense == 0`). Creates the
    /// adjective meaning, declares the adjective, links them, stores the domain,
    /// and marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` and calling wording `CALLW` are ignored.
    pub fn claim_definition(
        phrase_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        if sense != 0 {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(phrase_family_idx, None, Some(headword), meanings);
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific and calling parameters and reads the phrase family
    /// index from the static set by `start`.
    #[allow(clippy::too_many_arguments)]
    fn claim_definition_family_method(
        headword: &'static str,
        _prop: Option<usize>,
        sense: i32,
        domain_text: Option<&'static str>,
        _condition_text: Option<&'static str>,
        _definitions: &mut Vec<MeasurementDefinition>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        _properties: &[Property],
    ) -> Option<usize> {
        Self::claim_definition(Self::phrase_family_idx(), headword, sense, domain_text, adjectives, meanings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::adjectives::VIA_SUPPORT_FUNCTION_TASKMODE;

    // -----------------------------------------------------------------------
    // AdjectivesByPhrase::start
    // -----------------------------------------------------------------------

    #[test]
    fn start_creates_phrase_family_with_priority_6() {
        let mut families = Vec::new();
        let idx = AdjectivesByPhrase::start(&mut families);
        assert_eq!(families[idx].name, "phrase");
        assert_eq!(families[idx].definition_claim_priority, 6);
        assert!(families[idx].methods.claim_definition.is_some());
    }

    // -----------------------------------------------------------------------
    // AdjectivesByPhrase::claim_definition
    // -----------------------------------------------------------------------

    #[test]
    fn claim_definition_declines_non_phrasal_senses() {
        let mut families = Vec::new();
        let phrase_family = AdjectivesByPhrase::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByPhrase::claim_definition(
            phrase_family, "roomy", 1, Some("container"), &mut adjectives, &mut meanings,
        );
        assert!(result.is_none());

        let result = AdjectivesByPhrase::claim_definition(
            phrase_family, "roomy", -1, Some("container"), &mut adjectives, &mut meanings,
        );
        assert!(result.is_none());
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_creates_phrase_meaning_and_adjective() {
        let mut families = Vec::new();
        let phrase_family = AdjectivesByPhrase::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByPhrase::claim_definition(
            phrase_family, "possessed", 0, Some("container"), &mut adjectives, &mut meanings,
        ).unwrap();

        assert_eq!(meanings[am_idx].family, phrase_family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(meanings[am_idx].indexing_text, Some("possessed"));
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
        assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

        let adj_idx = meanings[am_idx].owning_adjective.unwrap();
        assert_eq!(adjectives[adj_idx].name, "possessed");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    // -----------------------------------------------------------------------
    // AdjectivesByPhrase::is_defined_by_phrase
    // -----------------------------------------------------------------------

    #[test]
    fn is_defined_by_phrase_true_for_phrase_meaning() {
        let mut families = Vec::new();
        let phrase_family = AdjectivesByPhrase::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByPhrase::claim_definition(
            phrase_family, "possessed", 0, None, &mut adjectives, &mut meanings,
        ).unwrap();

        assert!(AdjectivesByPhrase::is_defined_by_phrase(am_idx, &meanings, phrase_family));
    }

    #[test]
    fn is_defined_by_phrase_false_for_other_meaning() {
        let mut families = Vec::new();
        let phrase_family = AdjectivesByPhrase::start(&mut families);
        let other_family = AdjectiveMeanings::new_family(
            "other",
            0,
            AdjectiveMeaningFamilyMethods {
                assert: None,
                claim_definition: None,
                prepare_schemas: None,
                index: None,
            },
            &mut families,
        );
        let mut meanings = Vec::new();
        let am_idx = AdjectiveMeanings::new(other_family, None, None, &mut meanings);

        assert!(!AdjectivesByPhrase::is_defined_by_phrase(am_idx, &meanings, phrase_family));
    }
}
