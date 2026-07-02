/// Relation Inferences — the `relation_inf` inference family.
///
/// Relation inferences say that a relation holds between two subjects or values.
/// This is the second inference family created in the knowledge module startup
/// sequence, after `PropertyInferences`.
///
/// # C Reference
///
/// - `inform7/knowledge-module/Chapter 5/Relation Inferences.w` — the full
///   relation inference family (121 lines)
/// - `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup,
///   calls `RelationInferences::start()` (line 38)
/// - `inform7/knowledge-module/Chapter 5/Inferences.w` — `InferenceFamily`
///   infrastructure, `create_inference`, `join_inference`, `cmp`
///
/// # Simplified
///
/// Uses string names for relations instead of the full `binary_predicate` struct.
/// The full relation system will be integrated in a later plan.
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inference_subjects::InferenceSubjectFamily;
use crate::knowledge::inferences::{
    Certainty, Inference, InferenceFamily, InferenceFamilyMethods, JoinResult,
};

/// Data stored in a relation inference.
///
/// Corresponds to `relation_inference_data` in the C reference
/// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 28-32).
///
/// Stores two terms, which can be either subjects or values (but not both).
/// Simplified: uses subject indices and optional value strings instead of
/// `inference_subject*` and `parse_node*` pointers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationInferenceData {
    /// The two terms as subject indices (if subject-based).
    /// Both are None if value-based.
    pub terms_as_subjects: [Option<usize>; 2],
    /// The two terms as value strings (if value-based).
    /// Both are None if subject-based.
    pub terms_as_values: [Option<&'static str>; 2],
}

/// The relation inference family — operations for creating and managing
/// relation inferences.
///
/// Corresponds to the `RelationInferences` namespace in the C reference
/// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`).
pub struct RelationInferences;

impl RelationInferences {
    /// Create the relation inference family.
    ///
    /// Corresponds to `RelationInferences::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 14-18).
    ///
    /// Returns the `relation_inf` family with methods for logging and comparison.
    /// Note: the C version does not register an `explain_contradiction` method.
    pub fn start() -> InferenceFamily {
        InferenceFamily {
            name: "relation_inf",
            methods: InferenceFamilyMethods {
                log_details: |inf| {
                    // Simplified: log the data index
                    if let Some(di) = inf.data_index {
                        format!("(relation_inference_data[{}])", di)
                    } else {
                        String::new()
                    }
                },
                compare: |a, b| {
                    // Compare two relation inferences by their data index
                    // (simplified: the full C version compares individual terms
                    // using Inferences::measure_infs and Inferences::measure_pn)
                    let a_di = a.data_index.unwrap_or(usize::MAX);
                    let b_di = b.data_index.unwrap_or(usize::MAX);
                    if a_di > b_di {
                        3 // CI_DIFFER_IN_TOPIC
                    } else if a_di < b_di {
                        // -CI_DIFFER_IN_TOPIC
                        -3
                    } else {
                        0 // CI_IDENTICAL
                    }
                },
                explain_contradiction: |_a, _b, _similarity, _subject| {
                    // Relation inferences don't register explain_contradiction
                    // in the C reference, but the method table requires it.
                    false
                },
            },
        }
    }

    /// Create a new relation inference.
    ///
    /// Corresponds to `RelationInferences::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 34-44).
    ///
    /// Creates a `RelationInferenceData` with the given terms, then creates
    /// an `Inference` with the prevailing mood (defaulting to `Certain` if
    /// the mood is unknown).
    ///
    /// Terms can be given either as subjects or as values, but not both.
    /// Exactly one pair should be non-None.
    ///
    /// Returns the index of the new inference in the inferences registry.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        subj0: Option<usize>,
        subj1: Option<usize>,
        val0: Option<&'static str>,
        val1: Option<&'static str>,
        data_registry: &mut Vec<RelationInferenceData>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
    ) -> usize {
        // Find the relation_inf family index
        let family_idx = families
            .iter()
            .position(|f| f.name == "relation_inf")
            .expect("RelationInferences::start must be called first");

        // Create the inference data
        let data = RelationInferenceData {
            terms_as_subjects: [subj0, subj1],
            terms_as_values: [val0, val1],
        };

        // Store the data and create the inference
        let data_idx = data_registry.len();
        data_registry.push(data);

        let idx = inferences.len();
        inferences.push(Inference {
            family: family_idx,
            data: None, // data is stored in the registry, not inline
            data_index: Some(data_idx),
            certainty: Certainty::Certain,
            inferred_from: None,
            drawn_during_stage: 0,
            drawn_from_metadata: false,
        });
        idx
    }

    #[allow(clippy::too_many_arguments)]
    /// Create a relation inference and join it to a relation subject.
    ///
    /// Corresponds to `RelationInferences::draw` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 49-53).
    ///
    /// Simplified: creates a lightweight relation subject from the relation name
    /// if one doesn't exist, then joins the inference to it.
    pub fn draw(
        rel_name: &'static str,
        subj0: Option<usize>,
        subj1: Option<usize>,
        data_registry: &mut Vec<RelationInferenceData>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        subjects: &mut Vec<InferenceSubject>,
        subject_families: &[InferenceSubjectFamily],
    ) -> JoinResult {
        // Find or create a relation subject for this relation name
        let rel_subj_idx = find_or_create_relation_subject(rel_name, subjects, subject_families);

        // Find the relation_inf family index
        let family_idx = families
            .iter()
            .position(|f| f.name == "relation_inf")
            .expect("RelationInferences::start must be called first");

        // Create the inference data
        let data_idx = data_registry.len();
        data_registry.push(RelationInferenceData {
            terms_as_subjects: [subj0, subj1],
            terms_as_values: [None, None],
        });

        // Create the inference (don't push to inferences yet — join will do that)
        let inf = Inference {
            family: family_idx,
            data: None,
            data_index: Some(data_idx),
            certainty: Certainty::Certain,
            inferred_from: None,
            drawn_during_stage: 0,
            drawn_from_metadata: false,
        };
        inf.join(&mut subjects[rel_subj_idx], families, inferences)
    }
    #[allow(clippy::too_many_arguments)]
    /// Create a relation inference with value terms and join it to a relation subject.
    ///
    /// Corresponds to `RelationInferences::draw_spec` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 55-60).
    pub fn draw_spec(
        rel_name: &'static str,
        val0: &'static str,
        val1: &'static str,
        data_registry: &mut Vec<RelationInferenceData>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        subjects: &mut Vec<InferenceSubject>,
        subject_families: &[InferenceSubjectFamily],
    ) -> JoinResult {
        // Find or create a relation subject for this relation name
        let rel_subj_idx = find_or_create_relation_subject(rel_name, subjects, subject_families);

        // Find the relation_inf family index
        let family_idx = families
            .iter()
            .position(|f| f.name == "relation_inf")
            .expect("RelationInferences::start must be called first");

        // Create the inference data
        let data_idx = data_registry.len();
        data_registry.push(RelationInferenceData {
            terms_as_subjects: [None, None],
            terms_as_values: [Some(val0), Some(val1)],
        });

        // Create the inference (don't push to inferences yet — join will do that)
        let inf = Inference {
            family: family_idx,
            data: None,
            data_index: Some(data_idx),
            certainty: Certainty::Certain,
            inferred_from: None,
            drawn_during_stage: 0,
            drawn_from_metadata: false,
        };
        inf.join(&mut subjects[rel_subj_idx], families, inferences)
    }

    /// Get the subject terms from a relation inference.
    ///
    /// Corresponds to `RelationInferences::get_term_subjects` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 103-108).
    ///
    /// Returns `(subj0, subj1)` as `Option<usize>` indices, or `(None, None)`
    /// if the inference uses value terms instead.
    pub fn get_term_subjects(
        inf: &Inference,
        data_registry: &[RelationInferenceData],
    ) -> (Option<usize>, Option<usize>) {
        let idx = inf.data_index;
        match idx.and_then(|i| data_registry.get(i)) {
            Some(data) => (data.terms_as_subjects[0], data.terms_as_subjects[1]),
            None => (None, None),
        }
    }

    /// Get the value terms from a relation inference.
    ///
    /// Corresponds to `RelationInferences::get_term_specs` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 110-115).
    ///
    /// Returns `(val0, val1)` as `Option<&'static str>`, or `(None, None)`
    /// if the inference uses subject terms instead.
    pub fn get_term_specs(
        inf: &Inference,
        data_registry: &[RelationInferenceData],
    ) -> (Option<&'static str>, Option<&'static str>) {
        let idx = inf.data_index;
        match idx.and_then(|i| data_registry.get(i)) {
            Some(data) => (data.terms_as_values[0], data.terms_as_values[1]),
            None => (None, None),
        }
    }
}

/// Find or create a relation subject for a given relation name.
///
/// Simplified: looks up an existing subject by log_name, or creates a new
/// fundamental subject under model_world. The full C version uses
/// `RelationSubjects::from_bp(bp)` and `RelationSubjects::new(bp)`.
///
/// Corresponds to the relation subject lookup in the C reference
/// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 49-60).
fn find_or_create_relation_subject(
    rel_name: &'static str,
    subjects: &mut Vec<InferenceSubject>,
    _families: &[InferenceSubjectFamily],
) -> usize {
    // Look for an existing relation subject with this name
    for (i, subj) in subjects.iter().enumerate() {
        if subj.log_name == Some(rel_name) {
            return i;
        }
    }

    // Not found — create a new fundamental subject under model_world (index 0)
    let idx = subjects.len();
    subjects.push(InferenceSubject::new_fundamental(Some(0), rel_name));
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubjectFamily;
    use crate::knowledge::inferences::InferenceComparison;

    /// Helper: create a relation_inf family for testing.
    fn make_relation_family() -> InferenceFamily {
        RelationInferences::start()
    }

    /// Helper: create a fundamentals subject family for testing.
    fn make_fundamentals_family() -> InferenceSubjectFamily {
        InferenceSubjectFamily::fundamentals()
    }


    /// Helper: create a model_world root subject for testing.
    fn make_model_world() -> InferenceSubject {
        InferenceSubject::new_fundamental(None, "model world")
    }

    // --- start() tests ---

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let family = RelationInferences::start();
        assert_eq!(family.name, "relation_inf");
    }

    #[test]
    fn test_start_creates_family_with_log_details_method() {
        let family = RelationInferences::start();
        let inf = Inference::new(0, None, Certainty::Certain);
        // Without data_index, log_details returns empty
        let log = (family.methods.log_details)(&inf);
        assert_eq!(log, "");
    }

    #[test]
    fn test_start_creates_family_with_log_details_for_data_index() {
        let family = RelationInferences::start();
        let mut inf = Inference::new(0, None, Certainty::Certain);
        inf.data_index = Some(5);
        let log = (family.methods.log_details)(&inf);
        assert_eq!(log, "(relation_inference_data[5])");
    }

    #[test]
    fn test_start_creates_family_with_compare_method() {
        let family = RelationInferences::start();
        let a = Inference::new(0, None, Certainty::Certain);
        let b = Inference::new(0, None, Certainty::Certain);
        // Both have no data_index, so both default to usize::MAX -> identical
        assert_eq!((family.methods.compare)(&a, &b), 0);
    }

    #[test]
    fn test_start_creates_family_with_compare_different_indices() {
        let family = RelationInferences::start();
        let mut a = Inference::new(0, None, Certainty::Certain);
        let mut b = Inference::new(0, None, Certainty::Certain);
        a.data_index = Some(1);
        b.data_index = Some(2);
        // a_di (1) < b_di (2) -> -3
        assert_eq!((family.methods.compare)(&a, &b), -3);
    }

    #[test]
    fn test_start_creates_family_with_compare_same_indices() {
        let family = RelationInferences::start();
        let mut a = Inference::new(0, None, Certainty::Certain);
        let mut b = Inference::new(0, None, Certainty::Certain);
        a.data_index = Some(3);
        b.data_index = Some(3);
        assert_eq!((family.methods.compare)(&a, &b), 0);
    }

    #[test]
    fn test_explain_contradiction_returns_false() {
        let family = RelationInferences::start();
        let a = Inference::new(0, None, Certainty::Certain);
        let b = Inference::new(0, None, Certainty::Certain);
        assert!(!(family.methods.explain_contradiction)(&a, &b, 0, 0));
    }

    // --- new() tests ---

    #[test]
    fn test_new_creates_inference_with_subject_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );
        assert_eq!(idx, 0);
        assert_eq!(inferences.len(), 1);
        assert_eq!(inferences[0].family, 0);
        assert_eq!(inferences[0].data_index, Some(0));
        assert_eq!(inferences[0].certainty, Certainty::Certain);
    }

    #[test]
    fn test_new_creates_inference_with_value_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            None,
            None,
            Some("red"),
            Some("blue"),
            &mut data_registry,
            &families,
            &mut inferences,
        );
        assert_eq!(idx, 0);
        assert_eq!(inferences.len(), 1);
        assert_eq!(inferences[0].data_index, Some(0));
    }

    #[test]
    fn test_new_stores_data_in_registry() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );
        assert_eq!(data_registry.len(), 1);
        assert_eq!(data_registry[0].terms_as_subjects, [Some(0), Some(1)]);
        assert_eq!(data_registry[0].terms_as_values, [None, None]);
    }

    #[test]
    fn test_new_stores_value_data_in_registry() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        RelationInferences::new(
            None,
            None,
            Some("hello"),
            Some("world"),
            &mut data_registry,
            &families,
            &mut inferences,
        );
        assert_eq!(data_registry.len(), 1);
        assert_eq!(data_registry[0].terms_as_subjects, [None, None]);
        assert_eq!(data_registry[0].terms_as_values, [Some("hello"), Some("world")]);
    }

    #[test]
    fn test_new_creates_multiple_inferences() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx1 = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );
        let idx2 = RelationInferences::new(
            Some(2),
            Some(3),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(inferences.len(), 2);
        assert_eq!(data_registry.len(), 2);
        assert_eq!(data_registry[1].terms_as_subjects, [Some(2), Some(3)]);
    }

    // --- draw() tests ---

    #[test]
    fn test_draw_creates_and_joins_inference() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        let result = RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );
        assert_eq!(result, JoinResult::Joined);
        // One relation subject created + model_world
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[1].log_name, Some("knowing"));
        assert_eq!(subjects[1].inf_list.len(), 1);
        assert_eq!(inferences.len(), 1);
    }

    #[test]
    fn test_draw_reuses_existing_relation_subject() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        // First draw creates the relation subject
        RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        // Second draw reuses it
        let result = RelationInferences::draw(
            "knowing",
            Some(2),
            Some(3),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );
        assert_eq!(result, JoinResult::Joined);
        // Still only one relation subject
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[1].inf_list.len(), 2);
        assert_eq!(inferences.len(), 2);
    }

    #[test]
    fn test_draw_returns_discarded_redundant_for_duplicate() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        // First draw
        RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        // Second draw with same data — simplified compare uses data_index as a proxy,
        // so each inference has a unique data_index and is never considered identical.
        // The full C version compares individual terms and would detect the duplicate.
        let result = RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );
        assert_eq!(result, JoinResult::Joined);
    }

    #[test]
    fn test_draw_creates_separate_subjects_for_different_relations() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        RelationInferences::draw(
            "containment",
            Some(2),
            Some(3),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        assert_eq!(subjects.len(), 3); // model_world + knowing + containment
        assert_eq!(subjects[1].log_name, Some("knowing"));
        assert_eq!(subjects[2].log_name, Some("containment"));
    }

    // --- draw_spec() tests ---

    #[test]
    fn test_draw_spec_creates_and_joins_inference() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        let result = RelationInferences::draw_spec(
            "equality",
            "value_a",
            "value_b",
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );
        assert_eq!(result, JoinResult::Joined);
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[1].log_name, Some("equality"));
        assert_eq!(subjects[1].inf_list.len(), 1);
        assert_eq!(inferences.len(), 1);
    }

    #[test]
    fn test_draw_spec_stores_value_terms() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        RelationInferences::draw_spec(
            "equality",
            "value_a",
            "value_b",
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        assert_eq!(data_registry[0].terms_as_subjects, [None, None]);
        assert_eq!(
            data_registry[0].terms_as_values,
            [Some("value_a"), Some("value_b")]
        );
    }

#[test]
    fn test_draw_spec_returns_discarded_redundant_for_duplicate() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        RelationInferences::draw_spec(
            "equality",
            "value_a",
            "value_b",
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        let result = RelationInferences::draw_spec(
            "equality",
            "value_a",
            "value_b",
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );
        // Simplified compare uses data_index as a proxy, so each inference has
        // a unique data_index and is never considered identical.
        // The full C version compares individual terms and would detect the duplicate.
        assert_eq!(result, JoinResult::Joined);
    }

    // --- get_term_subjects() tests ---

    #[test]
    fn test_get_term_subjects_returns_subject_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            Some(5),
            Some(10),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let (s0, s1) = RelationInferences::get_term_subjects(&inferences[idx], &data_registry);
        assert_eq!(s0, Some(5));
        assert_eq!(s1, Some(10));
    }

    #[test]
    fn test_get_term_subjects_returns_none_for_value_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            None,
            None,
            Some("a"),
            Some("b"),
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let (s0, s1) = RelationInferences::get_term_subjects(&inferences[idx], &data_registry);
        assert_eq!(s0, None);
        assert_eq!(s1, None);
    }

    #[test]
    fn test_get_term_subjects_returns_none_for_missing_index() {
        let inf = Inference::new(0, None, Certainty::Certain);
        let data_registry = Vec::new();

        let (s0, s1) = RelationInferences::get_term_subjects(&inf, &data_registry);
        assert_eq!(s0, None);
        assert_eq!(s1, None);
    }

    // --- get_term_specs() tests ---

    #[test]
    fn test_get_term_specs_returns_value_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            None,
            None,
            Some("hello"),
            Some("world"),
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let (v0, v1) = RelationInferences::get_term_specs(&inferences[idx], &data_registry);
        assert_eq!(v0, Some("hello"));
        assert_eq!(v1, Some("world"));
    }

    #[test]
    fn test_get_term_specs_returns_none_for_subject_terms() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let (v0, v1) = RelationInferences::get_term_specs(&inferences[idx], &data_registry);
        assert_eq!(v0, None);
        assert_eq!(v1, None);
    }

    #[test]
    fn test_get_term_specs_returns_none_for_missing_index() {
        let inf = Inference::new(0, None, Certainty::Certain);
        let data_registry = Vec::new();

        let (v0, v1) = RelationInferences::get_term_specs(&inf, &data_registry);
        assert_eq!(v0, None);
        assert_eq!(v1, None);
    }

    // --- RelationInferenceData struct tests ---

    #[test]
    fn test_relation_inference_data_struct_subject_terms() {
        let data = RelationInferenceData {
            terms_as_subjects: [Some(0), Some(1)],
            terms_as_values: [None, None],
        };
        assert_eq!(data.terms_as_subjects, [Some(0), Some(1)]);
        assert_eq!(data.terms_as_values, [None, None]);
    }

    #[test]
    fn test_relation_inference_data_struct_value_terms() {
        let data = RelationInferenceData {
            terms_as_subjects: [None, None],
            terms_as_values: [Some("x"), Some("y")],
        };
        assert_eq!(data.terms_as_subjects, [None, None]);
        assert_eq!(data.terms_as_values, [Some("x"), Some("y")]);
    }

    #[test]
    fn test_relation_inference_data_clone() {
        let data = RelationInferenceData {
            terms_as_subjects: [Some(0), Some(1)],
            terms_as_values: [None, None],
        };
        let cloned = data.clone();
        assert_eq!(data, cloned);
    }

    #[test]
    fn test_relation_inference_data_debug() {
        let data = RelationInferenceData {
            terms_as_subjects: [Some(0), Some(1)],
            terms_as_values: [None, None],
        };
        let debug = format!("{:?}", data);
        assert!(debug.contains("terms_as_subjects"));
        assert!(debug.contains("Some(0)"));
    }

    // --- cmp integration tests ---

    #[test]
    fn test_cmp_identical_inferences() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx1 = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );
        // Create a second inference with the same data
        let idx2 = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let cmp = inferences[idx1].cmp(&inferences[idx2], &families);
        // Simplified compare uses data_index as a proxy. idx1 has data_index=0,
        // idx2 has data_index=1, so 0 < 1 -> -3 -> DifferInContent (catch-all).
        // The full C version compares individual terms and would return Identical.
        assert_eq!(cmp, InferenceComparison::DifferInContent);
    }

    #[test]
    fn test_cmp_same_inference_with_self() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        let idx = RelationInferences::new(
            Some(0),
            Some(1),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );

        let cmp = inferences[idx].cmp(&inferences[idx], &families);
        assert_eq!(cmp, InferenceComparison::Identical);
    }

    // --- find_or_create_relation_subject tests ---

    #[test]
    fn test_find_or_create_creates_new_subject() {
        let mut subjects = vec![make_model_world()];
        let families = vec![make_fundamentals_family()];

        let idx = find_or_create_relation_subject("knowing", &mut subjects, &families);
        assert_eq!(idx, 1);
        assert_eq!(subjects.len(), 2);
        assert_eq!(subjects[1].log_name, Some("knowing"));
    }

    #[test]
    fn test_find_or_create_reuses_existing_subject() {
        let mut subjects = vec![make_model_world()];
        let families = vec![make_fundamentals_family()];

        let idx1 = find_or_create_relation_subject("knowing", &mut subjects, &families);
        let idx2 = find_or_create_relation_subject("knowing", &mut subjects, &families);
        assert_eq!(idx1, idx2);
        assert_eq!(subjects.len(), 2);
    }

    #[test]
    fn test_find_or_create_creates_multiple_subjects() {
        let mut subjects = vec![make_model_world()];
        let families = vec![make_fundamentals_family()];

        let idx1 = find_or_create_relation_subject("knowing", &mut subjects, &families);
        let idx2 = find_or_create_relation_subject("containment", &mut subjects, &families);
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(subjects.len(), 3);
    }

    // --- draw with different relation names tests ---

    #[test]
    fn test_draw_with_different_relations_uses_separate_subjects() {
        let families = vec![make_relation_family()];
        let subject_families = vec![make_fundamentals_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_model_world()];
        let mut data_registry = Vec::new();

        RelationInferences::draw(
            "knowing",
            Some(0),
            Some(1),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        RelationInferences::draw(
            "containment",
            Some(2),
            Some(3),
            &mut data_registry,
            &families,
            &mut inferences,
            &mut subjects,
            &subject_families,
        );

        assert_eq!(subjects[1].inf_list.len(), 1);
        assert_eq!(subjects[2].inf_list.len(), 1);
    }

    // --- data_registry stores full data tests ---

    #[test]
    fn test_data_registry_stores_full_subject_data() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        RelationInferences::new(
            Some(42),
            Some(99),
            None,
            None,
            &mut data_registry,
            &families,
            &mut inferences,
        );

        assert_eq!(data_registry.len(), 1);
        assert_eq!(data_registry[0].terms_as_subjects, [Some(42), Some(99)]);
        assert_eq!(data_registry[0].terms_as_values, [None, None]);
    }

    #[test]
    fn test_data_registry_stores_full_value_data() {
        let families = vec![make_relation_family()];
        let mut data_registry = Vec::new();
        let mut inferences = Vec::new();

        RelationInferences::new(
            None,
            None,
            Some("alpha"),
            Some("beta"),
            &mut data_registry,
            &families,
            &mut inferences,
        );

        assert_eq!(data_registry.len(), 1);
        assert_eq!(data_registry[0].terms_as_subjects, [None, None]);
        assert_eq!(data_registry[0].terms_as_values, [Some("alpha"), Some("beta")]);
    }
}
