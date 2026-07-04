//! Calculus module — predicate calculus engine for Inform 7.
//!
//! This module defines the core data structures for representing logical
//! propositions in Inform 7: terms (variables, constants, functions), atoms
//! (the building blocks of propositions), propositions (linked lists of atoms
//! with conjunction, negation, and quantification), and unary predicates with
//! their families.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`terms`] | `Chapter 4/Terms.w` | `PcalcTerm` and `PcalcFunc` structs |
//! | [`atoms`] | `Chapter 4/Atomic Propositions.w` | `AtomElement`, `PcalcProp`, `PredicateRef`, `QuantifierRef` |
//! | [`propositions`] | `Chapter 4/Propositions.w` | Proposition operations (conjunction, negation, quantification, validity) |
//! | [`unary_predicates`] | `Chapter 2/Unary Predicates.w` | `UnaryPredicate` struct |
//! | [`unary_predicate_families`] | `Chapter 2/Unary Predicate Families.w` | `UpFamily` struct with method dispatch |
//! | [`kind_predicates_revisited`] | `inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w` | Typecheck, assert, and schema for the kind predicate family |
//! | [`adjectival_predicates`] | `inform7/assertions-module/Chapter 8/The Adjectival Predicates.w` | Adjectival unary predicate family |
//! | [`bp_term_details`] | `Chapter 3/Binary Predicate Term Details.w` | `BpTermDetails` struct and `BPTerms` functions |
//! | [`binary_predicates`] | `Chapter 3/Binary Predicates.w` | `BinaryPredicate` struct, creation, reversal, accessors |
//! | [`binary_predicate_families`] | `Chapter 3/Binary Predicate Families.w` | `BpFamily` struct with method dispatch |
//! | [`equality_details`] | `inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w` | Typecheck, assert, and schema for equality and empty families |
//! | [`explicit_relations`] | `Chapter 8/Explicit Relations.w` | Explicit and by-function binary predicate families |
//! # References
//!
//! - C reference: `services/calculus-module/Chapter 4/Terms.w`
//! - C reference: `services/calculus-module/Chapter 4/Atomic Propositions.w`
//! - C reference: `services/calculus-module/Chapter 4/Propositions.w`
//! - C reference: `services/calculus-module/Chapter 2/Unary Predicates.w`
//! - C reference: `services/calculus-module/Chapter 2/Unary Predicate Families.w`
//! - C reference: `services/calculus-module/Chapter 2/Kind Predicates.w`
//! - C reference: `services/calculus-module/Chapter 3/Binary Predicate Term Details.w`
//! - C reference: `services/calculus-module/Chapter 3/Binary Predicates.w`
//! - C reference: `services/calculus-module/Chapter 3/Binary Predicate Families.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`
//! - C reference: `services/calculus-module/Chapter 3/The Equality Relation.w`
//! - C reference: `inform7/assertions-module/Chapter 8/The Equality Relation Revisited.w`
//! - C reference: `inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`
//! - C reference: `inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`

pub mod atoms;
pub mod adjectival_predicates;
pub mod creation_predicates;
pub mod kind_predicates;
pub mod kind_predicates_revisited;
pub mod propositions;
pub mod terms;
pub mod unary_predicate_families;
pub mod unary_predicates;
pub mod binary_predicate_families;
pub mod binary_predicates;
pub mod bp_term_details;
pub mod equality_relation;
pub mod equality_details;
pub mod quasinumeric_relations;
pub mod universal_relation;
pub mod explicit_relations;
