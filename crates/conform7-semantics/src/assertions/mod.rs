//! Assertions module — the assertion-processing engine for Inform 7.
//!
//! This module corresponds to the `assertions-module` in the C reference
//! (`inform7/assertions-module/Chapter 1/Assertions Module.w`). It is the
//! central dispatch for processing assertion sentences — the sentences that
//! describe the model world in Inform 7 source text.
//!
//! The assertions module is initialized as part of the startup sequence:
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module) — which wires
//! `AdjectivalDefinitionFamily` methods (PLAN-46), `ToPhraseFamily` methods
//! (PLAN-47), and `RuleFamily` methods (PLAN-48), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
//! | [`adjectival_definition_family`] | `Chapter 5/Adjectival Definition Family.w` | Adjectival definition family |
//! | [`to_phrase_family`] | `Chapter 5/To Phrase Family.w` | To phrase definition family |
//! | [`rule_family`] | `Chapter 5/Rule Family.w` | Rule definition family |
//! | [`adjectives_by_phrase`] | `Chapter 8/Adjectives by Phrase.w` | Phrase-defined adjectives |
//! | [`adjectives_by_condition`] | `Chapter 8/Adjectives by Condition.w` | Condition-defined adjectives |
//! | [`adjectives_by_inter_function`] | `Chapter 8/Adjectives by Inter Function.w` | Inter-routine-defined adjectives |
//! | [`adjectives_by_inter_condition`] | `Chapter 8/Adjectives by Inter Condition.w` | Inter-condition-defined adjectives |
//! | [`major_nodes`] | `Chapter 2/Major Nodes.w` | Three-pass traversal through the syntax tree |
//! | [`classifying`] | `Chapter 2/Classifying Sentences.w` | Sentence diagramming via Preform matching |
//! | [`anaphora`] | `Chapter 2/Anaphora.w` | Anaphoric reference tracking |
//! | [`imperative_subtrees`] | `Chapter 2/Imperative Subtrees.w` | Imperative sentence processing |
//! | [`plugin_calls`] | `Chapter 2/Plugin Calls.w` | Plugin notification for new assertions |
//! | [`tables`] | `Chapter 2/Tables.w` | Table creation from TABLE_NT nodes |
//! | [`equations`] | `Chapter 2/Equations.w` | Equation creation from EQUATION_NT nodes |
//! | [`property_sentences`] | `Chapter 2/Property Sentences.w` | Property creation detection in sentences |
//! | [`refiner`] | `Chapter 4/Refine Parse Tree.w` | Parse tree refinement for assertion processing |
//! | [`creator`] | `Chapter 4/The Creator.w` | Object/kind creation from assertions |
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 1/Assertions Module.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
//! - C reference: `inform7/assertions-module/Chapter 5/To Phrase Family.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Condition.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Function.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`
//! - C reference: `inform7/assertions-module/Chapter 4/Refine Parse Tree.w`
//! - C reference: `inform7/assertions-module/Chapter 4/The Creator.w`

pub mod imperative_definition_families;

pub mod major_nodes;
pub mod classifying;
pub mod anaphora;
pub mod imperative_subtrees;
pub mod plugin_calls;
pub mod tables;
pub mod tables_cseq;
pub mod equations;
pub mod property_sentences;
pub mod adjectives_by_condition;
pub mod adjectives_by_phrase;
pub mod adjectives_by_inter_function;
pub mod adjectives_by_inter_condition;
pub mod adjectival_definition_family;

pub mod to_phrase_family;
pub mod rule_family;
pub mod refiner;
pub mod creator;
#[allow(clippy::module_inception)]
pub mod assertions;
pub mod property_knowledge;
pub mod relational;
pub mod new_property_assertions;
pub mod implications;
pub mod assemblies;
pub mod special_meanings;
pub mod intervention_requests;
pub mod bibliographic_data;