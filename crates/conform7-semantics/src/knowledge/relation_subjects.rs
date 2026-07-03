/// Relation Subjects — the bridge between binary predicates and inference subjects.
///
/// Every binary predicate has an associated inference subject, making it
/// possible to draw inferences about relations. This module provides the
/// family and functions to create and access these subjects.
///
/// Corresponds to `RelationSubjects` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`).
///
/// Simplified: uses BP indices instead of `binary_predicate*` pointers.
/// The `check_model` and `complete_model` methods are no-ops (they depend
/// on `ExplicitRelations` and `RTRelations`, which are not yet implemented).
use crate::calculus::binary_predicates::BinaryPredicate;
use crate::knowledge::inference_subjects::{
    InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
};

/// Index of the relations family in the inference subject family registry.
///
/// The relations family is typically pushed after the fundamentals family
/// (index 0), so it occupies index 1.
pub const RELATIONS_FAMILY: usize = 1;

/// Create the relations inference subject family.
///
/// Corresponds to `RelationSubjects::family` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 10-20).
///
/// The family has four methods:
/// - certainty: always returns CERTAIN_CE
/// - check_model: simplified no-op (full version checks 1-to-1 relations)
/// - complete_model: simplified no-op (full version sets up equivalence relations)
/// - get_name: returns empty (nameless)
pub fn family() -> InferenceSubjectFamily {
    InferenceSubjectFamily {
        name: "relations",
        methods: InferenceSubjectFamilyMethods {
            get_name_text: |_| None, // nameless
            get_default_certainty: |_| 3, // CERTAIN_CE
            new_permission_granted: |_, _| {
                // Stub: in the C reference, this allocates run-time storage
                // for the permission. That requires run-time compilation which
                // is out of scope for now.
            },
            make_adj_const_domain: |_, _, _| {
                // Stub: in the C reference, this registers an adjectival
                // constant for the relation. That requires instance adjectives
                // which are out of scope for now.
            },
            complete_model: |_| {
                // Deferred: depends on ExplicitRelations and RTRelations.
                // In the C reference, this sets up equivalence relations
                // and merges equivalence classes
                // (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 50-69).
            },
            check_model: |_| {
                // Deferred: depends on ExplicitRelations and RTRelations.
                // In the C reference, this checks 1-to-1 relations for violations
                // (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 79-87).
            },
        },
    }
}

/// Return the inference subject index for a binary predicate.
///
/// Corresponds to `RelationSubjects::from_bp` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 22-24).
///
/// Simplified: looks up the BP's `knowledge_about_bp` field, which stores
/// the inference subject index.
pub fn from_bp(
    bp_idx: usize,
    bp_registry: &[BinaryPredicate],
) -> Option<usize> {
    bp_registry.get(bp_idx).and_then(|bp| bp.knowledge_about_bp)
}

/// Create a new inference subject for a binary predicate.
///
/// Corresponds to `RelationSubjects::new` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 26-29).
///
/// Creates a new inference subject under the `relations` parent (a child of
/// model_world), using the relations family, with the BP index as family-specific
/// data.
///
/// Also sets the BP's `knowledge_about_bp` field to the new subject's index.
///
/// `bp_to_subject` is a side table mapping subject indices to BP indices,
/// the inverse of `knowledge_about_bp` on the BP registry.
///
/// Returns the index of the new subject.
pub fn new(
    bp_idx: usize,
    subjects: &mut Vec<InferenceSubject>,
    _families: &[InferenceSubjectFamily],
    bp_registry: &mut [BinaryPredicate],
    bp_to_subject: &mut Vec<Option<usize>>,
) -> usize {
    let parent = 0; // model_world

    let subject = InferenceSubject::new(
        RELATIONS_FAMILY,
        Some(parent),
        Some("relation"),
        None,
    );

    let subject_idx = subjects.len();
    subjects.push(subject);

    // Set the BP's knowledge_about_bp field.
    if let Some(bp) = bp_registry.get_mut(bp_idx) {
        bp.knowledge_about_bp = Some(subject_idx);
    }

    // Record the inverse mapping.
    if subject_idx >= bp_to_subject.len() {
        bp_to_subject.resize(subject_idx + 1, None);
    }
    bp_to_subject[subject_idx] = Some(bp_idx);

    subject_idx
}

/// Extract the binary predicate index from an inference subject index.
///
/// Corresponds to `RelationSubjects::to_bp` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 31-35).
///
/// Simplified: uses a side table (`bp_to_subject`) that maps subject indices
/// to BP indices, instead of the C reference's direct pointer retrieval.
pub fn to_bp(
    infs_idx: usize,
    bp_to_subject: &[Option<usize>],
) -> Option<usize> {
    bp_to_subject.get(infs_idx).copied().flatten()
}

/// Return the default certainty for relation subjects.
///
/// Corresponds to `RelationSubjects::certainty` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 38-40).
///
/// Relations are always certain (CERTAIN_CE = 3).
pub fn certainty(_infs: &InferenceSubject) -> i8 {
    3 // CERTAIN_CE
}

/// Return the name of a relation subject.
///
/// Corresponds to `RelationSubjects::get_name` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 42-45).
///
/// Relations are nameless in the C reference (returns EMPTY_WORDING).
pub fn get_name(_infs: &InferenceSubject) -> Option<&'static str> {
    None // nameless
}

/// Check the model for a relation subject.
///
/// Corresponds to `RelationSubjects::check_model` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 79-87).
///
/// Simplified: no-op. The full implementation checks 1-to-1 relations
/// for violations, which depends on `ExplicitRelations` and `RTRelations`.
pub fn check_model(_infs: &InferenceSubject) {
    // Deferred: depends on ExplicitRelations and RTRelations.
}

/// Complete the model for a relation subject.
///
/// Corresponds to `RelationSubjects::complete_model` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 50-69).
///
/// Simplified: no-op. The full implementation sets up equivalence relations
/// and merges equivalence classes, which depends on `ExplicitRelations`
/// and `RTRelations`.
pub fn complete_model(_infs: &InferenceSubject) {
    // Deferred: depends on ExplicitRelations and RTRelations.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::setup::setup_knowledge_module;
    use crate::calculus::bp_term_details::BpTermDetails;
    fn make_test_bp(name: &'static str) -> BinaryPredicate {
        BinaryPredicate {
            relation_family: 0,
            family_specific: Some(name.to_string()),
            relation_name: Some(name.to_string()),
            debugging_log_name: Some(name.to_string()),
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
        }
    }

    #[test]
    fn relations_family_has_correct_name() {
        let f = family();
        assert_eq!(f.name, "relations");
    }

    #[test]
    fn relations_family_has_certain_certainty() {
        let f = family();
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        assert_eq!((f.methods.get_default_certainty)(&subject), 3);
    }

    #[test]
    fn relations_family_get_name_returns_none() {
        let f = family();
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        assert_eq!((f.methods.get_name_text)(&subject), None);
    }

    #[test]
    fn relations_family_check_model_is_noop() {
        let f = family();
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        // Should not panic
        (f.methods.check_model)(&subject);
    }

    #[test]
    fn relations_family_complete_model_is_noop() {
        let f = family();
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        // Should not panic
        (f.methods.complete_model)(&subject);
    }

    #[test]
    fn new_creates_subject_for_binary_predicate() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        assert_eq!(idx, 3); // After 3 fundamental subjects
        assert_eq!(subjects[idx].infs_family, RELATIONS_FAMILY);
        assert_eq!(subjects[idx].broader_than, Some(0)); // model_world
        assert_eq!(subjects[idx].represents, Some("relation"));
    }

    #[test]
    fn new_sets_knowledge_about_bp() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        assert_eq!(bp_registry[0].knowledge_about_bp, Some(idx));
    }

    #[test]
    fn from_bp_returns_correct_subject_index() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        let result = from_bp(0, &bp_registry);
        assert_eq!(result, Some(idx));
    }

    #[test]
    fn from_bp_returns_none_for_invalid_bp_index() {
        let bp_registry: Vec<BinaryPredicate> = vec![];
        let result = from_bp(0, &bp_registry);
        assert_eq!(result, None);
    }

    #[test]
    fn from_bp_returns_none_for_bp_without_subject() {
        let bp = make_test_bp("contains");
        let bp_registry = vec![bp];
        let result = from_bp(0, &bp_registry);
        assert_eq!(result, None);
    }

    #[test]
    fn to_bp_returns_correct_bp_index() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        let result = to_bp(idx, &bp_to_subject);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn to_bp_returns_none_for_invalid_subject_index() {
        let bp_to_subject: Vec<Option<usize>> = vec![];
        let result = to_bp(0, &bp_to_subject);
        assert_eq!(result, None);
    }

    #[test]
    fn to_bp_returns_none_for_subject_without_bp() {
        let bp_to_subject: Vec<Option<usize>> = vec![None];
        let result = to_bp(0, &bp_to_subject);
        assert_eq!(result, None);
    }

    #[test]
    fn from_bp_to_bp_round_trip() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        // from_bp: BP index -> subject index
        let subject_idx = from_bp(0, &bp_registry).unwrap();
        assert_eq!(subject_idx, idx);

        // to_bp: subject index -> BP index
        let bp_idx = to_bp(subject_idx, &bp_to_subject).unwrap();
        assert_eq!(bp_idx, 0);
    }

    #[test]
    fn multiple_relation_subjects_can_be_created() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp1 = make_test_bp("contains");
        let bp2 = make_test_bp("supports");
        let bp3 = make_test_bp("wears");
        let mut bp_registry = vec![bp1, bp2, bp3];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx1 = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);
        let idx2 = new(1, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);
        let idx3 = new(2, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        assert_eq!(idx1, 3); // After 3 fundamental subjects
        assert_eq!(idx2, 4);
        assert_eq!(idx3, 5);

        // Verify round-trip for all three
        assert_eq!(from_bp(0, &bp_registry), Some(3));
        assert_eq!(from_bp(1, &bp_registry), Some(4));
        assert_eq!(from_bp(2, &bp_registry), Some(5));

        assert_eq!(to_bp(3, &bp_to_subject), Some(0));
        assert_eq!(to_bp(4, &bp_to_subject), Some(1));
        assert_eq!(to_bp(5, &bp_to_subject), Some(2));
    }

    #[test]
    fn relation_subject_uses_relations_family_methods() {
        let (mut subjects, mut families) = setup_knowledge_module();
        families.push(family());

        let bp = make_test_bp("contains");
        let mut bp_registry = vec![bp];
        let mut bp_to_subject: Vec<Option<usize>> = vec![];

        let idx = new(0, &mut subjects, &families, &mut bp_registry, &mut bp_to_subject);

        // Test family method dispatch
        let subject = &subjects[idx];
        let name = subject.get_name_text(&families);
        assert_eq!(name, None); // nameless

        let certainty = subject.get_default_certainty(&families);
        assert_eq!(certainty, 3); // CERTAIN_CE
    }

    #[test]
    fn certainty_function_returns_certain() {
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        assert_eq!(certainty(&subject), 3);
    }

    #[test]
    fn get_name_function_returns_none() {
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        assert_eq!(get_name(&subject), None);
    }

    #[test]
    fn check_model_function_is_noop() {
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        // Should not panic
        check_model(&subject);
    }

    #[test]
    fn complete_model_function_is_noop() {
        let subject = InferenceSubject::new(0, None, None, Some("test"));
        // Should not panic
        complete_model(&subject);
    }
}
