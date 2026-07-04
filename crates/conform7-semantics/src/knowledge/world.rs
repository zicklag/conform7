//! World module — model completion stages.
//!
//! This module implements the five-stage model world building process.
//! After the three passes through major nodes, the world is built in stages:
//!
//! - **Stage I**: Deduce object instance kinds
//! - **Stage II**: Complete the model (per-subject)
//! - **Stage III**: Plugin notification after completion
//! - **Stage IV**: Check the model (per-subject)
//! - **Stage V**: Augment model world with low-level properties
//!
//! # References
//!
//! - C reference: `inform7/core-module/Chapter 1/Pass 3 of 3.w`
//! - C reference: `inform7/knowledge-module/Chapter 4/Inference Subjects.w`

use std::sync::atomic::{AtomicI32, Ordering};

use crate::knowledge::inference_subjects::{
    InferenceSubject, InferenceSubjectFamily, InferenceSubjects,
};

/// World-building stage I: deduce object instance kinds.
pub const WORLD_STAGE_I: i32 = 0;
/// World-building stage II: complete the model.
pub const WORLD_STAGE_II: i32 = 1;
/// World-building stage III: plugin notification after completion.
pub const WORLD_STAGE_III: i32 = 2;
/// World-building stage IV: check the model.
pub const WORLD_STAGE_IV: i32 = 3;
/// World-building stage V: augment model world with low-level properties.
pub const WORLD_STAGE_V: i32 = 4;

/// The current world-building stage, or -1 if not yet started.
static CURRENT_STAGE: AtomicI32 = AtomicI32::new(-1);

/// The world module — orchestrates the five-stage model building process.
pub struct World;

#[allow(non_snake_case)]
impl World {
    /// Get the current model world building stage.
    ///
    /// Returns -1 if no stage has been entered yet.
    pub fn current_building_stage() -> i32 {
        CURRENT_STAGE.load(Ordering::SeqCst)
    }

    /// Stage I: deduce object instance kinds.
    ///
    /// Sets the current stage and notifies plugins.
    /// Corresponds to `World::deduce_object_instance_kinds` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 3 of 3.w`).
    pub fn deduce_object_instance_kinds() {
        CURRENT_STAGE.store(WORLD_STAGE_I, Ordering::SeqCst);
        Self::ask_plugins_at_stage(WORLD_STAGE_I);
    }

    /// Stages II and III: complete the model world.
    ///
    /// Stage II iterates all inference subjects and calls `complete_model`
    /// on each subject's family. Stage III notifies plugins.
    ///
    /// Corresponds to `World::stages_II_and_III` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 3 of 3.w`).
    pub fn stages_II_and_III(
        subjects: &mut [InferenceSubject],
        families: &[InferenceSubjectFamily],
    ) {
        CURRENT_STAGE.store(WORLD_STAGE_II, Ordering::SeqCst);
        for i in 0..subjects.len() {
            InferenceSubjects::complete_model(i, subjects, families);
        }
        Self::ask_plugins_at_stage(WORLD_STAGE_II);
        CURRENT_STAGE.store(WORLD_STAGE_III, Ordering::SeqCst);
        Self::ask_plugins_at_stage(WORLD_STAGE_III);
    }

    /// Stage IV: check the model world for consistency.
    ///
    /// Iterates all inference subjects and calls `check_model`
    /// on each subject's family.
    ///
    /// Corresponds to `World::stage_IV` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 3 of 3.w`).
    pub fn stage_IV(
        subjects: &mut [InferenceSubject],
        families: &[InferenceSubjectFamily],
    ) {
        CURRENT_STAGE.store(WORLD_STAGE_IV, Ordering::SeqCst);
        for i in 0..subjects.len() {
            InferenceSubjects::check_model(i, subjects, families);
        }
        Self::ask_plugins_at_stage(WORLD_STAGE_IV);
    }

    /// Stage V: augment model world with low-level properties.
    ///
    /// Corresponds to `World::stage_V` in the C reference
    /// (`inform7/core-module/Chapter 1/Pass 3 of 3.w`).
    pub fn stage_V() {
        CURRENT_STAGE.store(WORLD_STAGE_V, Ordering::SeqCst);
        Self::ask_plugins_at_stage(WORLD_STAGE_V);
    }

    /// Notify plugins at a given world stage (stub).
    ///
    /// The real plugin system is deferred.
    fn ask_plugins_at_stage(_stage: i32) {
        // Deferred: plugin system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubjectFamilyMethods;

    fn make_families() -> Vec<InferenceSubjectFamily> {
        vec![InferenceSubjectFamily::fundamentals()]
    }

    #[test]
    fn test_initial_stage_is_negative_one() {
        // Reset for test isolation
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        assert_eq!(World::current_building_stage(), -1);
    }

    #[test]
    fn test_deduce_object_instance_kinds_sets_stage_i() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        World::deduce_object_instance_kinds();
        assert_eq!(World::current_building_stage(), WORLD_STAGE_I);
    }

    #[test]
    fn test_stages_ii_and_iii_sets_stages() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        let mut subjects = Vec::new();
        let families = make_families();

        World::stages_II_and_III(&mut subjects, &families);
        // After stages II and III, the stage should be WORLD_STAGE_III
        assert_eq!(World::current_building_stage(), WORLD_STAGE_III);
    }

    #[test]
    fn test_stage_iv_sets_stage() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        let mut subjects = Vec::new();
        let families = make_families();

        World::stage_IV(&mut subjects, &families);
        assert_eq!(World::current_building_stage(), WORLD_STAGE_IV);
    }

    #[test]
    fn test_stage_v_sets_stage() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        World::stage_V();
        assert_eq!(World::current_building_stage(), WORLD_STAGE_V);
    }

    #[test]
    fn test_stages_ii_and_iii_calls_complete_model() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        CALLED.store(false, std::sync::atomic::Ordering::SeqCst);
        let families = vec![InferenceSubjectFamily::new(
            "test",
            InferenceSubjectFamilyMethods {
                get_name_text: |_| None,
                get_default_certainty: |_| 0,
                new_permission_granted: |_, _| {},
                make_adj_const_domain: |_, _, _| {},
                complete_model: |_| {
                    CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
                },
                check_model: |_| {},
            },
        )];
        let mut subjects = vec![InferenceSubject::new(0, None, None, Some("test_subject"))];

        World::stages_II_and_III(&mut subjects, &families);
        assert!(CALLED.load(std::sync::atomic::Ordering::SeqCst), "complete_model should have been called");
    }

    #[test]
    fn test_stage_iv_calls_check_model() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        CALLED.store(false, std::sync::atomic::Ordering::SeqCst);
        let families = vec![InferenceSubjectFamily::new(
            "test",
            InferenceSubjectFamilyMethods {
                get_name_text: |_| None,
                get_default_certainty: |_| 0,
                new_permission_granted: |_, _| {},
                make_adj_const_domain: |_, _, _| {},
                complete_model: |_| {},
                check_model: |_| {
                    CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
                },
            },
        )];
        let mut subjects = vec![InferenceSubject::new(0, None, None, Some("test_subject"))];

        World::stage_IV(&mut subjects, &families);
        assert!(CALLED.load(std::sync::atomic::Ordering::SeqCst), "check_model should have been called");
    }

    #[test]
    fn test_stages_ii_and_iii_with_multiple_subjects() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        static CALL_COUNT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
        CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let families = vec![InferenceSubjectFamily::new(
            "test",
            InferenceSubjectFamilyMethods {
                get_name_text: |_| None,
                get_default_certainty: |_| 0,
                new_permission_granted: |_, _| {},
                make_adj_const_domain: |_, _, _| {},
                complete_model: |_| {
                    CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                },
                check_model: |_| {},
            },
        )];
        let mut subjects = vec![
            InferenceSubject::new(0, None, None, Some("a")),
            InferenceSubject::new(0, None, None, Some("b")),
            InferenceSubject::new(0, None, None, Some("c")),
        ];

        World::stages_II_and_III(&mut subjects, &families);
        assert_eq!(CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst), 3, "complete_model should be called for each subject");
    }

    #[test]
    fn test_stage_iv_with_multiple_subjects() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        static CALL_COUNT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
        CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let families = vec![InferenceSubjectFamily::new(
            "test",
            InferenceSubjectFamilyMethods {
                get_name_text: |_| None,
                get_default_certainty: |_| 0,
                new_permission_granted: |_, _| {},
                make_adj_const_domain: |_, _, _| {},
                complete_model: |_| {},
                check_model: |_| {
                    CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                },
            },
        )];
        let mut subjects = vec![
            InferenceSubject::new(0, None, None, Some("a")),
            InferenceSubject::new(0, None, None, Some("b")),
        ];

        World::stage_IV(&mut subjects, &families);
        assert_eq!(CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst), 2, "check_model should be called for each subject");
    }

    #[test]
    fn test_stages_ii_and_iii_with_empty_subjects() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        let mut subjects = Vec::new();
        let families = make_families();
        // Should not panic
        World::stages_II_and_III(&mut subjects, &families);
        assert_eq!(World::current_building_stage(), WORLD_STAGE_III);
    }

    #[test]
    fn test_stage_iv_with_empty_subjects() {
        CURRENT_STAGE.store(-1, Ordering::SeqCst);
        let mut subjects = Vec::new();
        let families = make_families();
        // Should not panic
        World::stage_IV(&mut subjects, &families);
        assert_eq!(World::current_building_stage(), WORLD_STAGE_IV);
    }
}
