//! Adjectives by Inter Condition — adjectives defined by a raw Inter condition.
//!
//! Corresponds to `AdjectivesByInterCondition` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`).
//!
//! This module creates the `inter_condition_amf` family and claims adjective
//! definitions whose body is a raw Inter condition
//! (`Definition: a ... is ... if i6/inter condition "C" says so`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No real Preform/Salsa grammar parsing; a small string helper recognizes
//!   the single legal template and extracts the condition text.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_raw_Inter_condition` schema generation;
//!   only `TEST_ATOM_TASK` is marked via-support-function.
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

/// Global family index for the Inter condition family.
///
/// Mirrors the C static `inter_condition_amf`. Set by `AdjectivesByInterCondition::start`.
static INTER_CONDITION_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Inter Condition module.
///
/// Corresponds to `AdjectivesByInterCondition` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`).
pub struct AdjectivesByInterCondition;

/// Result of matching `<inform6-condition-adjective-definition>`.
///
/// Inter-condition definitions are always test-only (`says so`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct InterConditionDefinition {
    condition_text: &'static str,
    quoted_text: &'static str,
}

impl AdjectivesByInterCondition {
    /// Priority of the Inter condition family in the definition-claim order.
    ///
    /// Corresponds to the `4` passed to `AdjectiveMeanings::new_family` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`, line 22).
    pub const INTER_CONDITION_FAMILY_PRIORITY: u8 = 4;

    /// Create the Inter condition adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByInterCondition::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`, lines 21-25).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByInterCondition::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "inter_condition",
            Self::INTER_CONDITION_FAMILY_PRIORITY,
            methods,
            families,
        );
        INTER_CONDITION_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the Inter condition family set by the most recent `start`.
    fn inter_condition_family_idx() -> usize {
        INTER_CONDITION_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the Inter condition family.
    pub fn is_by_inter_condition(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        inter_condition_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == inter_condition_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only Inter-condition definitions that match the expected template,
    /// have `sense == 1`, and have no calling wording. Creates the adjective
    /// meaning, declares the adjective, links them, stores the domain, and
    /// marks `TEST_ATOM_TASK` as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` is ignored.
    /// - The quoted condition text is extracted and stored as `indexing_text`;
    ///   The raw word number and full `Definition` context that the C reference stores are deferred.
    ///
    /// - No Inform 6 schemas are generated; only the task-mode side effect of
    ///   `RTAdjectives::set_schemas_for_raw_Inter_condition` is reproduced.
    #[allow(clippy::too_many_arguments)]
    pub fn claim_definition(
        inter_condition_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        calling_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        let parsed = Self::parse_inter_condition_definition(condition_text?)?;

        if sense != 1 {
            return None;
        }
        if calling_text.is_some_and(|s| !s.is_empty()) {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            inter_condition_family_idx,
            None,
            Some(parsed.quoted_text),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);

        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);

        Some(am_idx)
    }

    /// A minimal parser for the single `<inform6-condition-adjective-definition>` template.
    ///
    /// This is a foundation stand-in for the full Preform grammar engine. It
    /// accepts strings of the form:
    ///
    /// - `i6/inter condition "Condition" says so (...)` -> returns the quoted text
    ///
    /// The `condition_text` returned is the text inside the quotes (without the
    /// quotes). The `quoted_text` returned is the original quoted segment.
    fn parse_inter_condition_definition(
        text: &'static str,
    ) -> Option<InterConditionDefinition> {
        let trimmed = text.trim_start();

        // Strip the required prefix; accept the exact Inform 7 keyword phrase.
        let after_prefix = trimmed
            .strip_prefix("i6/inter condition")
            .or_else(|| trimmed.strip_prefix("inter condition"))?
            .trim_start();

        // The condition must be a quoted string.
        if !after_prefix.starts_with('"') {
            return None;
        }
        let inner = &after_prefix[1..];
        let end = inner.find('"')?;
        let condition_text = &inner[..end];
        let tail = &inner[end + 1..];

        // The template must end with "says so"; ignore any parenthesised tail.
        if !tail.trim_start().starts_with("says so") {
            return None;
        }

        Some(InterConditionDefinition {
            condition_text,
            quoted_text: condition_text,
        })
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific parameters and the calling wording (assumed empty),
    /// and reads the Inter condition family index from the static set by `start`.
    #[allow(clippy::too_many_arguments)]
    fn claim_definition_family_method(
        headword: &'static str,
        _prop: Option<usize>,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        _definitions: &mut Vec<MeasurementDefinition>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
        _families: &[AdjectiveMeaningFamily],
        _properties: &[Property],
    ) -> Option<usize> {
        Self::claim_definition(
            Self::inter_condition_family_idx(),
            headword,
            sense,
            domain_text,
            condition_text,
            None,
            adjectives,
            meanings,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::adjectives::{
        NO_TASKMODE, NOW_ATOM_FALSE_TASK, NOW_ATOM_TRUE_TASK, VIA_SUPPORT_FUNCTION_TASKMODE,
    };

    // -----------------------------------------------------------------------
    // AdjectivesByInterCondition::start
    // -----------------------------------------------------------------------

    #[test]
    fn start_creates_inter_condition_family_with_priority_4() {
        let mut families = Vec::new();
        let idx = AdjectivesByInterCondition::start(&mut families);
        assert_eq!(families[idx].name, "inter_condition");
        assert_eq!(families[idx].definition_claim_priority, 4);
        assert!(families[idx].methods.claim_definition.is_some());
        assert!(families[idx].methods.prepare_schemas.is_none());
    }

    // -----------------------------------------------------------------------
    // AdjectivesByInterCondition::claim_definition
    // -----------------------------------------------------------------------

    #[test]
    fn claim_definition_declines_unrecognised_condition_text() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByInterCondition::claim_definition(
            family,
            "empty",
            1,
            Some("text"),
            Some("it is empty"),
            None,
            &mut adjectives,
            &mut meanings,
        );
        assert!(result.is_none());
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_declines_non_one_sense() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        for sense in [0, -1, 2] {
            let result = AdjectivesByInterCondition::claim_definition(
                family,
                "empty",
                sense,
                Some("text"),
                Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
                None,
                &mut adjectives,
                &mut meanings,
            );
            assert!(result.is_none(), "sense {} should be declined", sense);
        }
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_declines_non_empty_calling_text() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByInterCondition::claim_definition(
            family,
            "empty",
            1,
            Some("text"),
            Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
            Some("call it here"),
            &mut adjectives,
            &mut meanings,
        );
        assert!(result.is_none());
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_creates_inter_condition_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByInterCondition::claim_definition(
            family,
            "empty",
            1,
            Some("text"),
            Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
            None,
            &mut adjectives,
            &mut meanings,
        )
        .unwrap();

        assert_eq!(meanings[am_idx].family, family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(
            meanings[am_idx].indexing_text,
            Some("TEXT_TY_Empty")
        );
        assert_eq!(meanings[am_idx].domain.domain_text, Some("text"));
        assert_eq!(
            meanings[am_idx].task_modes[TEST_ATOM_TASK],
            VIA_SUPPORT_FUNCTION_TASKMODE
        );
        assert_eq!(
            meanings[am_idx].task_modes[NOW_ATOM_TRUE_TASK],
            NO_TASKMODE
        );
        assert_eq!(
            meanings[am_idx].task_modes[NOW_ATOM_FALSE_TASK],
            NO_TASKMODE
        );

        let adj_idx = meanings[am_idx].owning_adjective.unwrap();
        assert_eq!(adjectives[adj_idx].name, "empty");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    // -----------------------------------------------------------------------
    // AdjectivesByInterCondition::is_by_inter_condition
    // -----------------------------------------------------------------------

    #[test]
    fn is_by_inter_condition_true_for_inter_condition_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByInterCondition::claim_definition(
            family,
            "empty",
            1,
            None,
            Some("i6/inter condition \"TEXT_TY_Empty\" says so"),
            None,
            &mut adjectives,
            &mut meanings,
        )
        .unwrap();

        assert!(AdjectivesByInterCondition::is_by_inter_condition(
            am_idx, &meanings, family
        ));
    }

    #[test]
    fn is_by_inter_condition_false_for_other_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterCondition::start(&mut families);
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

        assert!(!AdjectivesByInterCondition::is_by_inter_condition(
            am_idx, &meanings, family
        ));
    }
}
