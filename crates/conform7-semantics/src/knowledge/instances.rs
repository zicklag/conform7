/// The Instance system — named constant values of enumerated kinds.
///
/// Instances are named constants giving a finite range of possible values of
/// a kind. For example, "red", "blue" and "green" are instances of the kind
/// "colour". Objects are instances too: "Peter" and "blue ball" are instances
/// of the kind "object".
///
/// | Struct | C Reference | Purpose |
/// |--------|-------------|---------|
/// | [`Instance`] | `Chapter 2/Instances.w` | Core instance struct |
/// | [`Instances`] | `Chapter 2/Instances.w` | Instance management functions |
///
/// # References
///
/// - C reference: `inform7/knowledge-module/Chapter 2/Instances.w`
use crate::kind_constructors::KindConstructor;
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::kind_subjects;
use crate::kinds::Kind;

/// An instance — a named constant value of an enumerated kind or an object.
///
/// Corresponds to `instance` in the C reference
/// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 34-46).
///
/// Instances are named constants giving a finite range of possible values of
/// a kind. For example, "red", "blue" and "green" are instances of the kind
/// "colour". Objects are instances too: "Peter" and "blue ball" are instances
/// of the kind "object".
///
/// Simplified:
/// - No `noun *` (simplified to a string name)
/// - No `parse_node *` (creation tracking deferred)
/// - No `instance_compilation_data` (run-time compilation deferred)
/// - No `PROTECTED_MODEL_PROCEDURE` guard
/// - No `PluginCalls` notifications
/// - No `Assertions::Assemblies` generalisations
#[derive(Clone, Debug)]
pub struct Instance {
    /// The name of the instance (simplified: a string instead of `noun *`).
    pub name: &'static str,
    /// The adjective index, if this instance is used adjectivally (e.g., "red").
    /// Set by `InstanceAdjectives::make_adjectival`.
    /// Corresponds to `as_adjective` in the C reference.
    pub as_adjective: Option<usize>,
    /// The inference subject index, from which the kind can be deduced.
    /// Set by `InstanceSubjects::new`.
    /// Corresponds to `as_subject` in the C reference.
    pub as_subject: Option<usize>,
    /// The enumeration index within each non-object kind (counted from 1).
    /// Corresponds to `enumeration_index` in the C reference.
    pub enumeration_index: i32,
    /// Whether this instance's name coincides with a kind name.
    /// Set by `Instances::make_kind_coincident`.
    /// Corresponds to `kind_coincident` in the C reference.
    pub kind_coincident: bool,
}

/// Creation and accessor functions for instances.
///
/// Corresponds to `Instances` in the C reference
/// (`inform7/knowledge-module/Chapter 2/Instances.w`).
pub struct Instances;

impl Instances {
    /// Create a new instance.
    ///
    /// Corresponds to `Instances::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 53-64).
    ///
    /// Creates the instance struct, sets the enumeration index, and creates
    /// an inference subject via `InstanceSubjects::new`.
    ///
    /// Simplified:
    /// - No `PROTECTED_MODEL_PROCEDURE` guard
    /// - No `PluginCalls::new_named_instance_notify`
    /// - No `Assertions::Assemblies::satisfies_generalisations`
    /// - No grammatical gender handling
    ///
    /// Returns the index of the new instance.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: &'static str,
        kind_idx: usize,
        enumeration_index: i32,
        instances: &mut Vec<Instance>,
        subjects: &mut Vec<InferenceSubject>,
        families: &[crate::knowledge::inference_subjects::InferenceSubjectFamily],
        constructors: &[KindConstructor],
    ) -> usize {
        let inst_idx = instances.len();
        instances.push(Instance {
            name,
            as_adjective: None,
            as_subject: None,
            enumeration_index,
            kind_coincident: false,
        });

        // Create the inference subject via InstanceSubjects::new
        let subj_idx = crate::knowledge::instance_subjects::InstanceSubjects::new(
            name,
            kind_idx,
            subjects,
            families,
            constructors,
        );
        instances[inst_idx].as_subject = Some(subj_idx);

        inst_idx
    }

    /// Return the name of an instance.
    ///
    /// Corresponds to `Instances::get_name` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 68-71).
    pub fn get_name(inst_idx: usize, instances: &[Instance]) -> Option<&'static str> {
        instances.get(inst_idx).map(|i| i.name)
    }

    /// Return the inference subject index for an instance.
    ///
    /// Corresponds to `Instances::as_subject` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 73-76).
    pub fn as_subject(inst_idx: usize, instances: &[Instance]) -> Option<usize> {
        instances.get(inst_idx).and_then(|i| i.as_subject)
    }

    /// Return the adjective index for an instance, if it has been made adjectival.
    ///
    /// Corresponds to `Instances::as_adjective` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 78-81).
    pub fn as_adjective(inst_idx: usize, instances: &[Instance]) -> Option<usize> {
        instances.get(inst_idx).and_then(|i| i.as_adjective)
    }

    /// Deduce the kind of an instance from its subject hierarchy.
    ///
    /// Corresponds to `Instances::to_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 83-87).
    ///
    /// Follows the subject hierarchy upward from the instance's subject to
    /// find the kind's subject, then converts it to a `Kind`.
    pub fn to_kind(
        inst_idx: usize,
        instances: &[Instance],
        subjects: &[InferenceSubject],
        constructors: &[KindConstructor],
    ) -> Option<Kind> {
        let inst = instances.get(inst_idx)?;
        let subj_idx = inst.as_subject?;
        let subj = subjects.get(subj_idx)?;
        let inherits_from = subj.narrowest_broader_subject()?;
        let broader_subj = subjects.get(inherits_from)?;
        kind_subjects::to_kind(broader_subj, subjects, constructors)
    }

    /// Return the enumeration index (numerical value) of an instance.
    ///
    /// Corresponds to `Instances::get_numerical_value` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`).
    pub fn get_numerical_value(inst_idx: usize, instances: &[Instance]) -> Option<i32> {
        instances.get(inst_idx).map(|i| i.enumeration_index)
    }

    /// Mark an instance as coincident with a kind.
    ///
    /// Corresponds to `Instances::make_kind_coincident` in the C reference
    /// (`inform7/knowledge-module/Chapter 2/Instances.w`).
    ///
    /// Finds the instance whose name matches the given kind name and marks it
    /// as coincident with that kind.
    ///
    /// Simplified:
    /// - No `PROTECTED_MODEL_PROCEDURE` guard
    /// - No `PluginCalls` notifications
    pub fn make_kind_coincident(
        kind_name: &'static str,
        _prn_idx: usize,
        instances: &mut [Instance],
    ) {
        if let Some(inst) = instances.iter_mut().find(|i| i.name == kind_name) {
            inst.kind_coincident = true;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::{
        InferenceSubject, InferenceSubjectFamily,
    };
    use crate::knowledge::instance_subjects::InstanceSubjects;
    use crate::knowledge::kind_subjects;

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

    #[test]
    fn test_new_creates_instance_with_correct_name() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(instances[inst_idx].name, "red");
    }

    #[test]
    fn test_new_creates_instance_with_correct_enumeration_index() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(instances[inst_idx].enumeration_index, 1);
    }

    #[test]
    fn test_new_creates_inference_subject() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert!(instances[inst_idx].as_subject.is_some());
    }

    #[test]
    fn test_get_name_returns_correct_name() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(Instances::get_name(inst_idx, &instances), Some("red"));
    }

    #[test]
    fn test_get_name_returns_none_for_invalid_index() {
        let instances = Vec::new();
        assert_eq!(Instances::get_name(0, &instances), None);
    }

    #[test]
    fn test_as_subject_returns_subject_index() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        let subj_idx = Instances::as_subject(inst_idx, &instances);
        assert!(subj_idx.is_some());
        // The subject should be in the subjects vec
        assert!(subj_idx.unwrap() < subjects.len());
    }

    #[test]
    fn test_as_subject_returns_none_for_invalid_index() {
        let instances = Vec::new();
        assert_eq!(Instances::as_subject(0, &instances), None);
    }

    #[test]
    fn test_as_adjective_returns_none_initially() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(Instances::as_adjective(inst_idx, &instances), None);
    }

    #[test]
    fn test_as_adjective_returns_none_for_invalid_index() {
        let instances = Vec::new();
        assert_eq!(Instances::as_adjective(0, &instances), None);
    }

    #[test]
    fn test_get_numerical_value_returns_enumeration_index() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(Instances::get_numerical_value(inst_idx, &instances), Some(1));
    }

    #[test]
    fn test_get_numerical_value_returns_none_for_invalid_index() {
        let instances = Vec::new();
        assert_eq!(Instances::get_numerical_value(0, &instances), None);
    }

    #[test]
    fn test_to_kind_returns_correct_kind() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        let kind = Instances::to_kind(inst_idx, &instances, &subjects, &constructors);
        assert!(kind.is_some());
        // Check the constructor index (construct.name is simplified to CON_VALUE)
        let kind = kind.unwrap();
        assert_eq!(kind.construct_id, colour_idx);
        assert_eq!(constructors[kind.construct_id].name, "colour");
    }

    #[test]
    fn test_to_kind_returns_none_for_invalid_index() {
        let instances = Vec::new();
        let subjects = Vec::new();
        let constructors = Vec::new();
        assert_eq!(Instances::to_kind(0, &instances, &subjects, &constructors), None);
    }

    #[test]
    fn test_multiple_instances_have_unique_indices() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        let red_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);
        let blue_idx = Instances::new("blue", colour_idx, 2, &mut instances, &mut subjects, &families, &constructors);
        let green_idx = Instances::new("green", colour_idx, 3, &mut instances, &mut subjects, &families, &constructors);

        assert_eq!(red_idx, 0);
        assert_eq!(blue_idx, 1);
        assert_eq!(green_idx, 2);
        assert_eq!(instances.len(), 3);
    }
}
