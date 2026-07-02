/// The root of the inference subject hierarchy.
///
/// Corresponds to `model_world` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 35-55).
pub const MODEL_WORLD: usize = 0;

/// Global constants subject (a child of model_world).
///
/// Corresponds to `global_constants` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 35-55).
pub const GLOBAL_CONSTANTS: usize = 1;

/// Global variables subject (a child of model_world).
///
/// Corresponds to `global_variables` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 35-55).
pub const GLOBAL_VARIABLES: usize = 2;

use super::inference_subjects::{InferenceSubject, InferenceSubjectFamily};

/// Create the fundamental subjects and set up the knowledge module.
///
/// This must be called before any other knowledge module operations.
///
/// Corresponds to `InferenceSubjects::start` and `KnowledgeModule::start`
/// in the C reference.
///
/// Returns (subjects, families) where:
/// - subjects[0] = model_world (root)
/// - subjects[1] = global_constants (child of model_world)
/// - subjects[2] = global_variables (child of model_world)
pub fn setup_knowledge_module() -> (Vec<InferenceSubject>, Vec<InferenceSubjectFamily>) {
    let fundamentals = InferenceSubjectFamily::fundamentals();
    let families = vec![fundamentals];

    let model_world = InferenceSubject::new_fundamental(None, "model world");
    let global_constants = InferenceSubject::new_fundamental(Some(0), "global constants");
    let global_variables = InferenceSubject::new_fundamental(Some(0), "global variables");

    let subjects = vec![model_world, global_constants, global_variables];
    (subjects, families)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_creates_three_subjects() {
        let (subjects, families) = setup_knowledge_module();
        assert_eq!(subjects.len(), 3);
        assert_eq!(families.len(), 1);
    }

    #[test]
    fn model_world_is_root() {
        let (subjects, _) = setup_knowledge_module();
        assert_eq!(subjects[MODEL_WORLD].broader_than, None);
        assert_eq!(subjects[MODEL_WORLD].log_name, Some("model world"));
        assert_eq!(subjects[MODEL_WORLD].infs_family, 0);
    }

    #[test]
    fn global_constants_is_child_of_model_world() {
        let (subjects, _) = setup_knowledge_module();
        assert_eq!(subjects[GLOBAL_CONSTANTS].broader_than, Some(MODEL_WORLD));
        assert_eq!(subjects[GLOBAL_CONSTANTS].log_name, Some("global constants"));
        assert_eq!(subjects[GLOBAL_CONSTANTS].infs_family, 0);
    }

    #[test]
    fn global_variables_is_child_of_model_world() {
        let (subjects, _) = setup_knowledge_module();
        assert_eq!(subjects[GLOBAL_VARIABLES].broader_than, Some(MODEL_WORLD));
        assert_eq!(subjects[GLOBAL_VARIABLES].log_name, Some("global variables"));
        assert_eq!(subjects[GLOBAL_VARIABLES].infs_family, 0);
    }

    #[test]
    fn fundamental_subjects_use_fundamentals_family() {
        let (subjects, _) = setup_knowledge_module();
        for subject in &subjects {
            assert_eq!(subject.infs_family, 0);
        }
    }

    #[test]
    fn is_within_works_for_fundamental_hierarchy() {
        let (subjects, _) = setup_knowledge_module();

        // model_world is not within anything (it's the root)
        assert!(!subjects[MODEL_WORLD].is_within(&subjects[GLOBAL_CONSTANTS], &subjects));
        assert!(!subjects[MODEL_WORLD].is_within(&subjects[GLOBAL_VARIABLES], &subjects));

        // global_constants is within model_world
        assert!(subjects[GLOBAL_CONSTANTS].is_within(&subjects[MODEL_WORLD], &subjects));

        // global_variables is within model_world
        assert!(subjects[GLOBAL_VARIABLES].is_within(&subjects[MODEL_WORLD], &subjects));

        // global_constants is not within global_variables
        assert!(!subjects[GLOBAL_CONSTANTS].is_within(&subjects[GLOBAL_VARIABLES], &subjects));
    }

    #[test]
    fn is_strictly_within_works() {
        let (subjects, _) = setup_knowledge_module();

        // A subject is not strictly within itself
        assert!(!subjects[MODEL_WORLD].is_strictly_within(&subjects[MODEL_WORLD], &subjects));
        assert!(!subjects[GLOBAL_CONSTANTS].is_strictly_within(&subjects[GLOBAL_CONSTANTS], &subjects));

        // global_constants is strictly within model_world
        assert!(subjects[GLOBAL_CONSTANTS].is_strictly_within(&subjects[MODEL_WORLD], &subjects));
    }

    #[test]
    fn narrowest_broader_subject_works() {
        let (subjects, _) = setup_knowledge_module();

        assert_eq!(subjects[MODEL_WORLD].narrowest_broader_subject(), None);
        assert_eq!(
            subjects[GLOBAL_CONSTANTS].narrowest_broader_subject(),
            Some(MODEL_WORLD)
        );
        assert_eq!(
            subjects[GLOBAL_VARIABLES].narrowest_broader_subject(),
            Some(MODEL_WORLD)
        );
    }

    #[test]
    fn constants_are_not_within_each_other() {
        let (subjects, _) = setup_knowledge_module();

        // global_constants is not within global_variables
        assert!(!subjects[GLOBAL_CONSTANTS].is_within(&subjects[GLOBAL_VARIABLES], &subjects));

        // global_variables is not within global_constants
        assert!(!subjects[GLOBAL_VARIABLES].is_within(&subjects[GLOBAL_CONSTANTS], &subjects));
    }
}
