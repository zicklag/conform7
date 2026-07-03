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
//! `AdjectivesByPhrase::start()`, etc.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 1/Assertions Module.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
//! - C reference: `inform7/assertions-module/Chapter 5/To Phrase Family.w`
//! - C reference: `inform7/assertions-module/Chapter 5/Rule Family.w`

pub mod imperative_definition_families;
