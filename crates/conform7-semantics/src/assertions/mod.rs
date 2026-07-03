//! Assertions module — the assertion-processing engine for Inform 7.
//!
//! This module corresponds to the `assertions-module` in the C reference
//! (`inform7/assertions-module/Chapter 1/Assertions Module.w`). It is the
//! central dispatch for processing assertion sentences — the sentences that
//! describe the model world in Inform 7 source text.
//!
//! The assertions module is initialized as part of the startup sequence:
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
//! | [`adjectives_by_phrase`] | `Chapter 8/Adjectives by Phrase.w` | Phrase-defined adjectives |
//! | [`adjectives_by_condition`] | `Chapter 8/Adjectives by Condition.w` | Condition-defined adjectives |
//! | [`adjectives_by_inter_function`] | `Chapter 8/Adjectives by Inter Function.w` | Inter-routine-defined adjectives |
//! | [`adjectives_by_inter_condition`] | `Chapter 8/Adjectives by Inter Condition.w` | Inter-condition-defined adjectives |
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
//! - C reference: `inform7/assertions-module/Chapter 5/Rule Family.w`

pub mod imperative_definition_families;
pub mod adjectives_by_condition;
pub mod adjectives_by_phrase;
pub mod adjectives_by_inter_function;
pub mod adjectives_by_inter_condition;
