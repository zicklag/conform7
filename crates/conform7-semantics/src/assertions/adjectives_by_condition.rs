//! Adjectives by Condition — adjectives defined by a one-line Inform 7 condition.
//!
//! Corresponds to `AdjectivesByCondition` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`).
//!
//! This module creates the `condition_amf` family and claims adjective
//! definitions whose body is a single I7 condition
//! (`Definition: a ... is ... if ...`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No Preform grammar parsing.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::support_for_I7_condition`.
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

/// Global family index for the condition family.
///
/// Mirrors the C static `condition_amf`. Set by `AdjectivesByCondition::start`.
static CONDITION_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Condition module.
///
/// Corresponds to `AdjectivesByCondition` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`).
pub struct AdjectivesByCondition;

impl AdjectivesByCondition {
    /// Priority of the condition family in the definition-claim order.
    ///
    /// Corresponds to the `7` passed to `AdjectiveMeanings::new_family` in the C reference.
    pub const CONDITION_FAMILY_PRIORITY: u8 = 7;

    /// Create the condition adjective family and install its methods.
    ///
    /// Corresponds to `AdjectivesByCondition::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Condition.w`, lines 12-21).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByCondition::claim_definition_family_method),
            prepare_schemas: Some(AdjectivesByCondition::prepare_schemas),
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "condition",
            Self::CONDITION_FAMILY_PRIORITY,
            methods,
            families,
        );
        CONDITION_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the condition family set by the most recent `start`.
    fn condition_family_idx() -> usize {
        CONDITION_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the condition family.
    ///
    /// Corresponds to testing `am->family == condition_amf` in the C reference.
    pub fn is_defined_by_condition(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        condition_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == condition_family_idx)
    }

    /// No-op placeholder for the run-time support function generator.
    ///
    /// Corresponds to `RTAdjectives::support_for_I7_condition` in the C reference
    /// (`inform7/runtime-module/Chapter 5/Adjectives.w`, lines 481ff).
    ///
    /// Simplified: does nothing. The full implementation would retrieve the
    /// `definition` stored in the meaning's `family_specific_data` and emit the
    /// condition as Inform 6 code inside the support function for the task.
    pub fn prepare_schemas(_am_idx: usize, _task: i32) {
        // Run-time compilation deferred.
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only condition-based adjectives (`sense != 0`). Creates the
    /// adjective meaning, declares the adjective, links them, stores the domain,
    /// and marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` and calling wording `CALLW` are ignored.
    /// - The condition text `CONW` and sense value are accepted as parameters
    ///   but not stored; they will be persisted when the `Definition` struct is
    ///   introduced.
    pub fn claim_definition(
        condition_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        if sense == 0 {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            condition_family_idx,
            None,
            Some(headword),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific and calling parameters and reads the condition
    /// family index from the static set by `start`.
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
        Self::claim_definition(
            Self::condition_family_idx(),
            headword,
            sense,
            domain_text,
            adjectives,
            meanings,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::adjectives::VIA_SUPPORT_FUNCTION_TASKMODE;

    // -----------------------------------------------------------------------
    // AdjectivesByCondition::start
    // -----------------------------------------------------------------------

    #[test]
    fn start_creates_condition_family_with_priority_7() {
        let mut families = Vec::new();
        let idx = AdjectivesByCondition::start(&mut families);
        assert_eq!(families[idx].name, "condition");
        assert_eq!(families[idx].definition_claim_priority, 7);
        assert!(families[idx].methods.claim_definition.is_some());
        assert!(families[idx].methods.prepare_schemas.is_some());
    }

    // -----------------------------------------------------------------------
    // AdjectivesByCondition::claim_definition
    // -----------------------------------------------------------------------

    #[test]
    fn claim_definition_declines_phrasal_sense() {
        let mut families = Vec::new();
        let condition_family = AdjectivesByCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByCondition::claim_definition(
            condition_family, "roomy", 0, Some("container"), &mut adjectives, &mut meanings,
        );
        assert!(result.is_none());
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_accepts_positive_condition_sense() {
        let mut families = Vec::new();
        let condition_family = AdjectivesByCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByCondition::claim_definition(
            condition_family, "roomy", 1, Some("container"), &mut adjectives, &mut meanings,
        ).unwrap();

        assert_eq!(meanings[am_idx].family, condition_family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(meanings[am_idx].indexing_text, Some("roomy"));
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
        assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

        let adj_idx = meanings[am_idx].owning_adjective.unwrap();
        assert_eq!(adjectives[adj_idx].name, "roomy");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    #[test]
    fn claim_definition_accepts_negative_condition_sense() {
        let mut families = Vec::new();
        let condition_family = AdjectivesByCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByCondition::claim_definition(
            condition_family, "roomy", -1, Some("container"), &mut adjectives, &mut meanings,
        ).unwrap();

        assert_eq!(meanings[am_idx].family, condition_family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(meanings[am_idx].indexing_text, Some("roomy"));
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
        assert_eq!(meanings[am_idx].task_modes[TEST_ATOM_TASK], VIA_SUPPORT_FUNCTION_TASKMODE);

        let adj_idx = meanings[am_idx].owning_adjective.unwrap();
        assert_eq!(adjectives[adj_idx].name, "roomy");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    // -----------------------------------------------------------------------
    // AdjectivesByCondition::is_defined_by_condition
    // -----------------------------------------------------------------------

    #[test]
    fn is_defined_by_condition_true_for_condition_meaning() {
        let mut families = Vec::new();
        let condition_family = AdjectivesByCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByCondition::claim_definition(
            condition_family, "roomy", 1, None, &mut adjectives, &mut meanings,
        ).unwrap();

        assert!(AdjectivesByCondition::is_defined_by_condition(am_idx, &meanings, condition_family));
    }

    #[test]
    fn is_defined_by_condition_false_for_other_meaning() {
        let mut families = Vec::new();
        let condition_family = AdjectivesByCondition::start(&mut families);
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

        assert!(!AdjectivesByCondition::is_defined_by_condition(am_idx, &meanings, condition_family));
    }

    // -----------------------------------------------------------------------
    // AdjectivesByCondition::prepare_schemas
    // -----------------------------------------------------------------------

    #[test]
    fn prepare_schemas_is_installed() {
        let mut families = Vec::new();
        let idx = AdjectivesByCondition::start(&mut families);
        assert!(families[idx].methods.prepare_schemas.is_some());
    }
}
