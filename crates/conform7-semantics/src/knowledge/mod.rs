//! Knowledge module — the world model that stores and reconciles facts about the game world.
//!
//! This module defines the core data structures for representing facts about the game world:
//! inference subjects (anything a proposition can discuss), inferences (single facts with
//! certainty levels), inference families (method dispatch for different inference types),
//! and property permissions (the bridge between subjects and properties).
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`inference_subjects`] | `Chapter 4/Inference Subjects.w` | `InferenceSubject` and `InferenceSubjectFamily` |
//! | [`inferences`] | `Chapter 5/Inferences.w` | `Inference`, `InferenceFamily`, `Certainty` |
//! | [`property_inferences`] | `Chapter 5/Property Inferences.w` | `PropertyInferences` family and `PropertyInferenceData` |
//! | [`relation_subjects`] | `Chapter 4/Relation Subjects.w` | `RelationSubjects` — bridge between binary predicates and inference subjects |
//! # References
//!
//! - C reference: `inform7/knowledge-module/Chapter 4/Inference Subjects.w`
//! - C reference: `inform7/knowledge-module/Chapter 5/Inferences.w`
//! - C reference: `inform7/knowledge-module/Chapter 4/Property Permissions.w`

pub mod inference_subjects;
pub mod inferences;
pub mod property_permissions;
pub mod setup;
pub mod kind_subjects;
pub mod property_inferences;
pub mod relation_inferences;
pub mod provision_relation;
pub mod relation_subjects;
pub mod properties;
pub mod same_property_relation;
pub mod setting_property_relation;
pub mod adjectives;
pub mod either_or_property_adjectives;
pub mod measurements;
pub mod instances;
pub mod instance_subjects;
pub mod instance_adjectives;
pub mod measurement_adjectives;
pub mod value_properties;
pub mod ordering_instances;
pub use value_properties::ValueProperties;
