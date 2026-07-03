//! Adjectives by Inter Function — adjectives defined by a named Inter/I6 routine.
//!
//! Corresponds to `AdjectivesByInterFunction` in the C reference
//! (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`).
//!
//! This module creates the `inter_routine_amf` family and claims adjective
//! definitions whose body names an Inter routine/function
//! (`Definition: a ... is ... if i6/inter routine/function "R" says so`).
//!
//! Simplified:
//! - No `Definition` struct or `AdjectivalDefinitionFamily` integration.
//! - No real Preform/Salsa grammar parsing; a small string helper recognizes
//!   the two legal templates and extracts the routine name, setting flag, and
//!   remaining text.
//! - No `parse_node` handling (source location is ignored).
//! - No `RTAdjectives::set_schemas_for_raw_Inter_function` schema generation;
//!   task-mode marking is performed so that TEST (and NOW, for setting
//!   routines) are flagged via-support-function.
//! - The family method wrapper uses a static family index to fit the existing
//!   measurement-shaped `ClaimDefinitionFn` signature.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::knowledge::adjectives::{
    Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
    AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
    NOW_ATOM_FALSE_TASK, NOW_ATOM_TRUE_TASK, TEST_ATOM_TASK,
};
use crate::knowledge::measurements::MeasurementDefinition;
use crate::knowledge::properties::Property;

/// Global family index for the Inter routine family.
///
/// Mirrors the C static `inter_routine_amf`. Set by `AdjectivesByInterFunction::start`.
static INTER_ROUTINE_FAMILY: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The Adjectives by Inter Function module.
///
/// Corresponds to `AdjectivesByInterFunction` in the C reference
/// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`).
pub struct AdjectivesByInterFunction;

/// Result of matching `<inform6-routine-adjective-definition>`.
///
/// `setting == false` corresponds to `says so` (a test routine).
/// `setting == true` corresponds to `makes it so` (a now routine).
#[derive(Clone, Copy, Debug, PartialEq)]
struct InterRoutineDefinition {
    routine_name: &'static str,
    extra_text: &'static str,
    setting: bool,
}

impl AdjectivesByInterFunction {
    /// Priority of the Inter routine family in the definition-claim order.
    ///
    /// Corresponds to the `5` passed to `AdjectiveMeanings::new_family` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, line 20).
    pub const INTER_ROUTINE_FAMILY_PRIORITY: u8 = 5;

    /// Create the Inter routine adjective family and install its `claim_definition` method.
    ///
    /// Corresponds to `AdjectivesByInterFunction::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, lines 19-23).
    ///
    /// Returns the index of the newly created family in `families`.
    pub fn start(families: &mut Vec<AdjectiveMeaningFamily>) -> usize {
        let methods = AdjectiveMeaningFamilyMethods {
            assert: None,
            claim_definition: Some(AdjectivesByInterFunction::claim_definition_family_method),
            prepare_schemas: None,
            index: None,
        };
        let idx = AdjectiveMeanings::new_family(
            "inter_routine",
            Self::INTER_ROUTINE_FAMILY_PRIORITY,
            methods,
            families,
        );
        INTER_ROUTINE_FAMILY.store(idx, Ordering::SeqCst);
        idx
    }

    /// Return the index of the Inter routine family set by the most recent `start`.
    fn inter_routine_family_idx() -> usize {
        INTER_ROUTINE_FAMILY.load(Ordering::SeqCst)
    }

    /// Check whether a meaning belongs to the Inter routine family.
    ///
    /// Corresponds to `AdjectivesByInterFunction::is_by_Inter_function` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`, lines 25-28).
    pub fn is_by_inter_function(
        am_idx: usize,
        meanings: &[AdjectiveMeaning],
        inter_routine_family_idx: usize,
    ) -> bool {
        meanings
            .get(am_idx)
            .is_some_and(|am| am.family == inter_routine_family_idx)
    }

    /// Public, testable implementation of the family claim.
    ///
    /// Claims only Inter-routine definitions that match the expected template,
    /// have `sense == 1`, and have no calling wording. Creates the adjective
    /// meaning, declares the adjective, links them, stores the domain, and
    /// marks the appropriate tasks as via-support-function.
    ///
    /// Simplified from the C:
    /// - No `definition` struct is created (`family_specific_data` is `None`).
    /// - The source-node `q` is ignored.
    /// - The routine name `RW` is extracted but not stored in the meaning
    ///   (the `Definition` struct that will hold it is deferred).
    /// - No Inform 6 schemas are generated; only the task-mode side effects of
    ///   `RTAdjectives::set_schemas_for_raw_Inter_function` are reproduced.
    #[allow(clippy::too_many_arguments)]
    pub fn claim_definition(
        inter_routine_family_idx: usize,
        headword: &'static str,
        sense: i32,
        domain_text: Option<&'static str>,
        condition_text: Option<&'static str>,
        calling_text: Option<&'static str>,
        adjectives: &mut Vec<Adjective>,
        meanings: &mut Vec<AdjectiveMeaning>,
    ) -> Option<usize> {
        let parsed = Self::parse_inter_routine_definition(condition_text?)?;

        if sense != 1 {
            return None;
        }
        if calling_text.is_some_and(|s| !s.is_empty()) {
            return None;
        }

        let am_idx = AdjectiveMeanings::new(
            inter_routine_family_idx,
            None,
            Some(parsed.extra_text),
            meanings,
        );
        let adj_idx = Adjectives::declare(headword, adjectives);
        AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);
        AdjectiveMeaningDomains::set_from_text(am_idx, domain_text, meanings);

        Self::mark_inter_routine_tasks(am_idx, parsed.setting, meanings);

        Some(am_idx)
    }

    /// Mark the atom tasks that are implemented by the raw Inter function.
    ///
    /// Mirrors the `perform_task_via_function` calls made by
    /// `RTAdjectives::set_schemas_for_raw_Inter_function`
    /// (`inform7/runtime-module/Chapter 5/Adjectives.w`, lines 441-460).
    fn mark_inter_routine_tasks(am_idx: usize, setting: bool, meanings: &mut [AdjectiveMeaning]) {
        AdjectiveMeanings::perform_task_via_function(am_idx, TEST_ATOM_TASK, meanings);
        if setting {
            AdjectiveMeanings::perform_task_via_function(am_idx, NOW_ATOM_TRUE_TASK, meanings);
            AdjectiveMeanings::perform_task_via_function(am_idx, NOW_ATOM_FALSE_TASK, meanings);
        }
    }

    /// A minimal parser for the two `<inform6-routine-adjective-definition>` templates.
    ///
    /// This is a foundation stand-in for the full Preform grammar engine. It
    /// accepts strings of the form:
    ///
    /// - `i6/inter routine/function "Name" says so (...)` -> `setting = false`
    /// - `i6/inter routine/function "Name" makes it so (...)` -> `setting = true`
    ///
    /// The `routine_name` returned is the text inside the quotes (without the
    /// quotes). The `extra_text` returned is the original `condition_text`.
    fn parse_inter_routine_definition(
        text: &'static str,
    ) -> Option<InterRoutineDefinition> {
        let trimmed = text.trim_start();

        // Strip the required prefix; accept the exact Inform 7 keyword phrase.
        let after_prefix = trimmed
            .strip_prefix("i6/inter routine/function")
            .or_else(|| trimmed.strip_prefix("inter routine/function"))?
            .trim_start();

        // The routine name must be a quoted string.
        if !after_prefix.starts_with('"') {
            return None;
        }
        let inner = &after_prefix[1..];
        let end = inner.find('"')?;
        let routine_name = &inner[..end];
        let tail = &inner[end + 1..];

        // Decide which of the two templates matched.
        let tail_trimmed = tail.trim_start();
        let setting = if tail_trimmed.starts_with("makes it so") {
            true
        } else if tail_trimmed.starts_with("says so") {
            false
        } else {
            return None;
        };

        Some(InterRoutineDefinition {
            routine_name,
            extra_text: text,
            setting,
        })
    }

    /// Wrapper matching the existing `ClaimDefinitionFn` type alias.
    ///
    /// This is a temporary fit to the measurement-shaped signature. It ignores
    /// measurement-specific parameters and the calling wording (assumed empty),
    /// and reads the Inter routine family index from the static set by `start`.
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
            Self::inter_routine_family_idx(),
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
    use crate::knowledge::adjectives::{NO_TASKMODE, VIA_SUPPORT_FUNCTION_TASKMODE};

    // -----------------------------------------------------------------------
    // AdjectivesByInterFunction::start
    // -----------------------------------------------------------------------

    #[test]
    fn start_creates_inter_routine_family_with_priority_5() {
        let mut families = Vec::new();
        let idx = AdjectivesByInterFunction::start(&mut families);
        assert_eq!(families[idx].name, "inter_routine");
        assert_eq!(families[idx].definition_claim_priority, 5);
        assert!(families[idx].methods.claim_definition.is_some());
        assert!(families[idx].methods.prepare_schemas.is_none());
    }

    // -----------------------------------------------------------------------
    // AdjectivesByInterFunction::claim_definition
    // -----------------------------------------------------------------------

    #[test]
    fn claim_definition_declines_unrecognised_condition_text() {
        let mut families = Vec::new();
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByInterFunction::claim_definition(
            family,
            "shiny",
            1,
            Some("thing"),
            Some("it is shiny"),
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
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        for sense in [0, -1, 2] {
            let result = AdjectivesByInterFunction::claim_definition(
                family,
                "shiny",
                sense,
                Some("thing"),
                Some("i6/inter routine/function \"IsShiny\" says so"),
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
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let result = AdjectivesByInterFunction::claim_definition(
            family,
            "shiny",
            1,
            Some("thing"),
            Some("i6/inter routine/function \"IsShiny\" says so"),
            Some("call it here"),
            &mut adjectives,
            &mut meanings,
        );
        assert!(result.is_none());
        assert!(adjectives.is_empty());
        assert!(meanings.is_empty());
    }

    #[test]
    fn claim_definition_creates_test_routine_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByInterFunction::claim_definition(
            family,
            "see-through",
            1,
            Some("container"),
            Some("i6/inter routine/function \"MapScope\" says so"),
            None,
            &mut adjectives,
            &mut meanings,
        )
        .unwrap();

        assert_eq!(meanings[am_idx].family, family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(
            meanings[am_idx].indexing_text,
            Some("i6/inter routine/function \"MapScope\" says so")
        );
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
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
        assert_eq!(adjectives[adj_idx].name, "see-through");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    #[test]
    fn claim_definition_creates_setting_routine_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByInterFunction::claim_definition(
            family,
            "opaque",
            1,
            Some("container"),
            Some("i6/inter routine/function \"MakeOpaque\" makes it so"),
            None,
            &mut adjectives,
            &mut meanings,
        )
        .unwrap();

        assert_eq!(meanings[am_idx].family, family);
        assert!(meanings[am_idx].family_specific_data.is_none());
        assert_eq!(meanings[am_idx].domain.domain_text, Some("container"));
        assert_eq!(
            meanings[am_idx].task_modes[TEST_ATOM_TASK],
            VIA_SUPPORT_FUNCTION_TASKMODE
        );
        assert_eq!(
            meanings[am_idx].task_modes[NOW_ATOM_TRUE_TASK],
            VIA_SUPPORT_FUNCTION_TASKMODE
        );
        assert_eq!(
            meanings[am_idx].task_modes[NOW_ATOM_FALSE_TASK],
            VIA_SUPPORT_FUNCTION_TASKMODE
        );

        let adj_idx = meanings[am_idx].owning_adjective.unwrap();
        assert_eq!(adjectives[adj_idx].name, "opaque");
        assert!(adjectives[adj_idx].meanings.contains(&am_idx));
    }

    // -----------------------------------------------------------------------
    // AdjectivesByInterFunction::is_by_inter_function
    // -----------------------------------------------------------------------

    #[test]
    fn is_by_inter_function_true_for_inter_routine_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterFunction::start(&mut families);
        let mut adjectives = Vec::new();
        let mut meanings = Vec::new();

        let am_idx = AdjectivesByInterFunction::claim_definition(
            family,
            "see-through",
            1,
            None,
            Some("i6/inter routine/function \"MapScope\" says so"),
            None,
            &mut adjectives,
            &mut meanings,
        )
        .unwrap();

        assert!(AdjectivesByInterFunction::is_by_inter_function(
            am_idx, &meanings, family
        ));
    }

    #[test]
    fn is_by_inter_function_false_for_other_meaning() {
        let mut families = Vec::new();
        let family = AdjectivesByInterFunction::start(&mut families);
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

        assert!(!AdjectivesByInterFunction::is_by_inter_function(
            am_idx, &meanings, family
        ));
    }
}
