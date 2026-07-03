/// Instance Subjects — the bridge between instances and the inference subject system.
///
/// Every instance gets its own inference subject, making it possible to
/// draw inferences about instances. The `InstanceSubjects` family provides
/// methods for getting the instance's name, default certainty, and handling
/// adjectival constant domains.
///
/// | Struct | C Reference | Purpose |
/// |--------|-------------|---------|
/// | [`InstanceSubjects`] | `Chapter 4/Instance Subjects.w` | Instance subject management |
///
/// # References
///
/// - C reference: `inform7/knowledge-module/Chapter 4/Instance Subjects.w`
use crate::kind_constructors::KindConstructor;
use crate::knowledge::inference_subjects::{
    InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
};

/// Name of the instances family in the inference subject family registry.
pub const INSTANCES_FAMILY_NAME: &str = "instances";

/// The instance subjects module.
///
/// Corresponds to `InstanceSubjects` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
pub struct InstanceSubjects;

impl InstanceSubjects {
    /// Return the instances family of inference subjects.
    ///
    /// Corresponds to `InstanceSubjects::family` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 97-114).
    ///
    /// The family provides methods for:
    /// - Getting the default certainty (always CERTAIN_CE)
    /// - Getting the name text (returns the instance name)
    /// - Making an adjectival constant domain (calls `InstanceAdjectives::make_adjectival`)
    /// - Handling new permission grants (simplified: no-op)
    ///
    /// Simplified:
    /// - No `EMIT_ALL` method (RT compilation deferred)
    /// - No `EMIT_ELEMENT` method (RT compilation deferred)
    pub fn family() -> InferenceSubjectFamily {
        InferenceSubjectFamily::new(
            INSTANCES_FAMILY_NAME,
            InferenceSubjectFamilyMethods {
                get_name_text: |subj| subj.represents,
                get_default_certainty: |_| 3, // CERTAIN_CE
                new_permission_granted: |_, _| {
                    // Simplified: no-op. The full implementation would
                    // call `InstanceAdjectives::make_adjectival` when a
                    // property permission is granted for a kind whose name
                    // coincides with a property.
                },
                make_adj_const_domain: |_, _, _| {
                    // Simplified: no-op. The full implementation would
                    // call `InstanceAdjectives::make_adjectival` to
                    // register the instance as an adjective.
                },
                complete_model: |_| {
                    // Simplified: no-op. The full implementation would
                    // call `RTInstances::compile_all`.
                },
                check_model: |_| {
                    // Simplified: no-op. The full implementation would
                    // call `RTInstances::emit_element_of_condition`.
                },
            },
        )
    }

    /// Create a new inference subject for an instance.
    ///
    /// Corresponds to `InstanceSubjects::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 116-119).
    ///
    /// Creates an inference subject with:
    /// - The kind's subject as the broader (more general) subject
    /// - The instances family
    /// - The instance name stored in `represents`
    ///
    /// Returns the index of the new inference subject.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        inst_name: &'static str,
        kind_idx: usize,
        subjects: &mut Vec<InferenceSubject>,
        families: &[InferenceSubjectFamily],
        constructors: &[KindConstructor],
    ) -> usize {
        // Find the instances family index
        let family_idx = families
            .iter()
            .position(|f| f.name == INSTANCES_FAMILY_NAME)
            .expect("InstanceSubjects::family must be registered first");

        // Find the kind's inference subject to use as the broader subject
        let kind_subj_idx = constructors
            .get(kind_idx)
            .and_then(|c| c.base_as_infs)
            .expect("Kind must have an inference subject");

        // Store the instance name in represents so that to_instance can
        // look it up by name. This follows the pattern used by KindSubjects
        // which stores the constructor name in represents.
        let subject = InferenceSubject::new(
            family_idx,
            Some(kind_subj_idx),
            Some(inst_name),
            Some(inst_name),
        );

        let idx = subjects.len();
        subjects.push(subject);
        idx
    }

    /// Extract the instance index from an inference subject.
    ///
    /// Corresponds to `InstanceSubjects::to_instance` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 121-125).
    ///
    /// Looks up the instance by name from the subject's `represents` field.
    /// Returns `None` if the subject is not an instance subject or if no
    /// matching instance is found.
    pub fn to_instance(
        subj_idx: usize,
        subjects: &[InferenceSubject],
        instances: &[crate::knowledge::instances::Instance],
    ) -> Option<usize> {
        let subj = subjects.get(subj_idx)?;
        let inst_name = subj.represents?;
        instances.iter().position(|i| i.name == inst_name)
    }

    /// Get the default certainty for an instance subject.
    ///
    /// Corresponds to `InstanceSubjects::certainty` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
    ///
    /// Instance subjects always have CERTAIN_CE (3) as their default certainty.
    pub fn certainty() -> i8 {
        3 // CERTAIN_CE
    }

    /// Get the name of an instance from its inference subject.
    ///
    /// Corresponds to `InstanceSubjects::get_name` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
    ///
    /// Returns the instance name stored in the subject's `represents` field.
    pub fn get_name(subj: &InferenceSubject) -> Option<&'static str> {
        subj.represents
    }

    /// Make an instance the domain of an adjectival constant.
    ///
    /// Corresponds to `InstanceSubjects::make_adj_const_domain` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 55-65).
    ///
    /// Simplified: no-op. The full implementation would call
    /// `InstanceAdjectives::make_adjectival` to register the instance as
    /// an adjective for the property whose name coincides with the kind.
    pub fn make_adj_const_domain(_subj: &InferenceSubject, _domain_infs: usize, _to_what: usize) {
        // Simplified: no-op.
        // The full implementation would:
        // 1. Extract the instance from the subject
        // 2. Call InstanceAdjectives::make_adjectival(owner, NULL, NULL, NULL)
    }

    /// Handle a new property permission grant for an instance subject.
    ///
    /// Corresponds to `InstanceSubjects::new_permission_granted` in the C reference
    /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
    ///
    /// Simplified: no-op. The full implementation would handle the case
    /// where a property permission is granted for a kind whose name
    /// coincides with a property (e.g., "colour" kind and "colour" property).
    pub fn new_permission_granted(_subj: &InferenceSubject, _pp: usize) {
        // Simplified: no-op.
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubject;
    use crate::knowledge::instances::Instances;
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
    fn test_family_has_correct_name() {
        let family = InstanceSubjects::family();
        assert_eq!(family.name, INSTANCES_FAMILY_NAME);
    }

    #[test]
    fn test_family_has_get_name_text_method() {
        let family = InstanceSubjects::family();
        // The get_name_text method should return the represents field
        let subj = InferenceSubject::new(0, None, Some("test_instance"), Some("test_instance"));
        let name = (family.methods.get_name_text)(&subj);
        assert_eq!(name, Some("test_instance"));
    }

    #[test]
    fn test_family_has_certainty_method() {
        let family = InstanceSubjects::family();
        let subj = InferenceSubject::new(0, None, None, None);
        let certainty = (family.methods.get_default_certainty)(&subj);
        assert_eq!(certainty, 3); // CERTAIN_CE
    }

    #[test]
    fn test_new_creates_subject_with_correct_family() {
        let families = vec![
            InferenceSubjectFamily::fundamentals(),
            kind_subjects::family(),
            InstanceSubjects::family(),
        ];
        let mut subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let mut constructors = Vec::new();
        let mut instances = Vec::new();

        let colour_idx = make_kind_constructor("colour", &mut constructors, &mut subjects, &families);

        // Create an instance first
        let inst_idx = Instances::new("red", colour_idx, 1, &mut instances, &mut subjects, &families, &constructors);

        // The subject should have been created by Instances::new
        let subj_idx = instances[inst_idx].as_subject.unwrap();
        let subj = &subjects[subj_idx];

        // The family should be the instances family (index 2, after fundamentals and kinds)
        assert_eq!(subj.infs_family, 2);
    }

    #[test]
    fn test_new_creates_subject_with_broader_kind_subject() {
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

        let subj_idx = instances[inst_idx].as_subject.unwrap();
        let subj = &subjects[subj_idx];

        // The broader subject should be the colour kind's subject
        assert!(subj.broader_than.is_some());
        let broader = &subjects[subj.broader_than.unwrap()];
        assert_eq!(broader.represents, Some("colour"));
    }

    #[test]
    fn test_to_instance_returns_correct_instance() {
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

        let subj_idx = instances[inst_idx].as_subject.unwrap();
        let result = InstanceSubjects::to_instance(subj_idx, &subjects, &instances);

        assert_eq!(result, Some(inst_idx));
    }

    #[test]
    fn test_to_instance_returns_none_for_non_instance_subject() {
        let subjects = vec![InferenceSubject::new_fundamental(None, "model_world")];
        let instances = Vec::new();

        let result = InstanceSubjects::to_instance(0, &subjects, &instances);
        assert_eq!(result, None);
    }

    #[test]
    fn test_to_instance_returns_none_for_invalid_index() {
        let subjects = Vec::new();
        let instances = Vec::new();

        let result = InstanceSubjects::to_instance(0, &subjects, &instances);
        assert_eq!(result, None);
    }

    #[test]
    fn test_certainty_returns_three() {
        assert_eq!(InstanceSubjects::certainty(), 3);
    }

    #[test]
    fn test_get_name_returns_instance_name() {
        let subj = InferenceSubject::new(0, None, Some("red"), Some("red"));
        assert_eq!(InstanceSubjects::get_name(&subj), Some("red"));
    }

    #[test]
    fn test_get_name_returns_none_when_no_represents() {
        let subj = InferenceSubject::new(0, None, None, None);
        assert_eq!(InstanceSubjects::get_name(&subj), None);
    }

    #[test]
    fn test_make_adj_const_domain_is_noop() {
        let subj = InferenceSubject::new(0, None, Some("red"), Some("red"));
        // Should not panic
        InstanceSubjects::make_adj_const_domain(&subj, 0, 0);
    }

    #[test]
    fn test_new_permission_granted_is_noop() {
        let subj = InferenceSubject::new(0, None, Some("red"), Some("red"));
        // Should not panic
        InstanceSubjects::new_permission_granted(&subj, 0);
    }
}
