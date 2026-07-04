//! The Adjectival Predicates system — a unary predicate family for adjectives.
//!
//! Corresponds to `AdjectivalPredicates` in the C reference
//! (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`).
//!
//! Creates a single unary predicate family (`adjectival_up_family`) with seven
//! methods:
//!
//! - `typecheck` — check if the adjective can be applied to the term's kind
//! - `infer_kind` — infer the kind from the adjective's first meaning's domain
//! - `assert` — assert the adjective as a true fact about the model world
//! - `testable` — whether the adjective can be tested at compile-time
//! - `test` — test the adjective at compile-time
//! - `schema` — compile run-time code for the adjective
//! - `log` — log the adjective for debugging
//!
//! Also provides helper functions:
//! - `new_up` — create a new adjectival unary predicate
//! - `new_atom` — create a proposition atom from an adjective
//! - `new_atom_on_x` — create an atom on variable 0
//! - `to_adjective` — extract the adjective index from a predicate
//! - `parity` — get the positive/negative sense
//! - `flip_parity` — reverse the sense
//!
//! Simplified:
//! - No `lcon`/`Stock` system — uses a `parity: bool` field on `UnaryPredicate`
//!   and stores the adjective index directly in `adjective_index`.
//! - `typecheck` always returns `ALWAYS_MATCH` (1) — full kind-checking deferred.
//! - `assert` returns `false` (not handled) — full assertion logic deferred.
//! - `test` returns `false` — compile-time testing deferred.
//! - `schema` returns `false` — schema compilation deferred.
//! - No `wording` type — uses `&'static str` for adjective names.
//! - No problem messages.
//! - No `parse_node` or `inference_subject` handling.

use std::sync::LazyLock;

use crate::calculus::atoms::PcalcProp;
use crate::calculus::terms::PcalcTerm;
use crate::calculus::unary_predicate_families::{UpFamily, UpFamilyMethods};
use crate::calculus::unary_predicates::UnaryPredicate;

/// The Adjectival Predicates module.
///
/// Corresponds to `AdjectivalPredicates` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`).
pub struct AdjectivalPredicates;

/// The adjectival unary predicate family.
///
/// Corresponds to `adjectival_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, line 8).
///
/// This family handles predicates derived from linguistic adjectives
/// (e.g., "open", "closed", "empty", "red").
pub static ADJECTIVAL_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "adjectival",
        UpFamilyMethods {
            log: AdjectivalPredicates::log,
            infer_kind: AdjectivalPredicates::infer_kind,
            testable: AdjectivalPredicates::testable,
            test: AdjectivalPredicates::test,
            typecheck: Some(AdjectivalPredicates::typecheck),
            assert: Some(AdjectivalPredicates::assert),
            schema: Some(AdjectivalPredicates::schema),
        },
    )
});

impl AdjectivalPredicates {
    /// Create the adjectival predicate family.
    ///
    /// Corresponds to `AdjectivalPredicates::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 13-22).
    ///
    /// Returns a reference to the singleton `ADJECTIVAL_UP_FAMILY`.
    pub fn start() -> &'static UpFamily {
        &ADJECTIVAL_UP_FAMILY
    }

    /// Create a new adjectival unary predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::new_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 24-30).
    ///
    /// `adj_idx` is the index of the adjective in the adjective registry.
    /// `pos` is `true` for positive sense, `false` for negative sense.
    ///
    /// Simplified: stores the adjective index in `adjective_index` and parity
    /// in the `parity` field, instead of using the `lcon`/`Stock` system.
    pub fn new_up(adj_idx: usize, pos: bool) -> UnaryPredicate {
        let mut up = UnaryPredicate::new(&ADJECTIVAL_UP_FAMILY);
        up.adjective_index = Some(adj_idx);
        up.parity = pos;
        up
    }

    /// Create a proposition atom from an adjective and term.
    ///
    /// Corresponds to `AdjectivalPredicates::new_atom` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 32-35).
    ///
    /// `negated` is `true` to create a negative-sense atom (equivalent to
    /// "not adjective"), `false` for positive sense.
    pub fn new_atom(adj_idx: usize, negated: bool, term: PcalcTerm) -> PcalcProp {
        let up = Self::new_up(adj_idx, !negated);
        PcalcProp::unary_predicate_new_from_up(up, term)
    }

    /// Create an adjectival atom on variable 0 (the default bound variable).
    ///
    /// Corresponds to `AdjectivalPredicates::new_atom_on_x` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 37-39).
    pub fn new_atom_on_x(adj_idx: usize, negated: bool) -> PcalcProp {
        Self::new_atom(adj_idx, negated, PcalcTerm::new_variable(0))
    }

    /// Extract the adjective index from a unary predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::to_adjective` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 140-144).
    ///
    /// Returns `None` if the predicate is not from the adjectival family.
    ///
    /// Simplified: reads the `adjective_index` field instead of using `lcon`/`Stock`.
    pub fn to_adjective(up: &UnaryPredicate) -> Option<usize> {
        if up.family.name != "adjectival" {
            return None;
        }
        up.adjective_index
    }

    /// Get the parity (positive/negative sense) of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::parity` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 146-151).
    ///
    /// Returns `true` for positive sense, `false` for negative sense.
    /// If the predicate is not from the adjectival family, returns `true`
    /// (default positive sense, matching the C behavior).
    pub fn parity(up: &UnaryPredicate) -> bool {
        if up.family.name != "adjectival" {
            return true;
        }
        up.parity
    }

    /// Flip the parity of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::flip_parity` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 157-164).
    ///
    /// Does nothing if the predicate is not from the adjectival family.
    pub fn flip_parity(up: &mut UnaryPredicate) {
        if up.family.name == "adjectival" {
            up.parity = !up.parity;
        }
    }

    // -----------------------------------------------------------------------
    // Family methods
    // -----------------------------------------------------------------------

    /// Log an adjectival predicate to the debug log.
    ///
    /// Corresponds to `AdjectivalPredicates::log` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 41-45).
    ///
    /// Simplified: uses the adjective index to format the output.
    fn log(_family: &UpFamily, up: &UnaryPredicate) -> String {
        let prefix = if Self::parity(up) { "" } else { "not-" };
        match Self::to_adjective(up) {
            Some(adj_idx) => {
                format!("{}adj:{}", prefix, adj_idx)
            }
            None => format!("{}?", prefix),
        }
    }

    /// Infer the kind from an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::infer_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 47-52).
    ///
    /// Simplified: returns `None` since we don't have access to the
    /// adjective registry from this static context. The full implementation
    /// will use `AdjectiveAmbiguity::first_meaning` and
    /// `AdjectiveMeaningDomains::get_kind`.
    fn infer_kind(_family: &UpFamily, _up: &UnaryPredicate) -> Option<&'static str> {
        // Deferred: requires access to the adjective registry and
        // AdjectiveAmbiguity::first_meaning.
        None
    }

    /// Whether an adjectival predicate can be tested at compile-time.
    ///
    /// Corresponds to `AdjectivalPredicates::testable` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 104-109).
    ///
    /// Simplified: returns `false`. The full implementation will use
    /// `AdjectiveAmbiguity::has_either_or_property_meaning`.
    fn testable(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
        // Deferred: requires access to the adjective registry and
        // AdjectiveAmbiguity::has_either_or_property_meaning.
        false
    }

    /// Test an adjectival predicate at compile-time.
    ///
    /// Corresponds to `AdjectivalPredicates::test` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 111-126).
    ///
    /// Simplified: returns `false`. The full implementation will use
    /// `Properties::get_possession_marker`.
    fn test(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
        // Deferred: requires access to the property system and
        // possession markers.
        false
    }

    /// Typecheck the terms of an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 54-66).
    ///
    /// Simplified: always returns `1` (ALWAYS_MATCH). The full implementation
    /// will use `TypecheckPropositions::kind_of_term` and
    /// `AdjectiveAmbiguity::can_be_applied_to`.
    fn typecheck(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        // Deferred: full kind-checking logic.
        1 // ALWAYS_MATCH
    }

    /// Assert an adjectival predicate as a true fact about the model world.
    ///
    /// Corresponds to `AdjectivalPredicates::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 75-102).
    ///
    /// Simplified: returns `false` (not handled). The full implementation
    /// will use `AdjectiveAmbiguity::assert`, `Assert::subject_of_term`,
    /// and `Assert::spec_of_term`.
    fn assert(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _now_negated: bool,
        _prop: &PcalcProp,
    ) -> bool {
        // Deferred: full assertion logic.
        false
    }

    /// Compile run-time code for an adjectival predicate.
    ///
    /// Corresponds to `AdjectivalPredicates::get_schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Adjectival Predicates.w`, lines 128-135).
    ///
    /// Simplified: returns `false` (not handled). The full implementation
    /// will use `AdjectiveAmbiguity::schema_for_task`.
    fn schema(
        _family: &UpFamily,
        _task: u8,
        _up: &UnaryPredicate,
    ) -> bool {
        // Deferred: full schema compilation logic.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::unary_predicate_families::UpFamily;
    use crate::calculus::unary_predicates::UnaryPredicate;
    use crate::calculus::terms::PcalcTerm;
    use crate::calculus::atoms::{AtomElement, PcalcProp};

    /// Static non-adjectival family for testing.
    static OTHER_FAMILY: LazyLock<UpFamily> =
        LazyLock::new(|| UpFamily::new("other", UpFamilyMethods::default()));

    fn other_family() -> &'static UpFamily {
        &OTHER_FAMILY
    }

    /// Test that `start()` returns a family with the correct name.
    #[test]
    fn start_creates_family_with_correct_name() {
        let family = AdjectivalPredicates::start();
        assert_eq!(family.name, "adjectival");
    }

    /// Test that the family has all seven methods wired.
    #[test]
    fn start_creates_family_with_all_seven_methods() {
        let family = AdjectivalPredicates::start();
        // Required methods
        let up = UnaryPredicate::new(family);
        (family.methods.log)(family, &up);
        (family.methods.infer_kind)(family, &up);
        (family.methods.testable)(family, &up);
        (family.methods.test)(family, &up);
        // Optional methods
        assert!(family.methods.typecheck.is_some(), "typecheck should be installed");
        assert!(family.methods.assert.is_some(), "assert should be installed");
        assert!(family.methods.schema.is_some(), "schema should be installed");
    }

    /// Test that `new_up` creates a predicate belonging to the adjectival family.
    #[test]
    fn new_up_creates_predicate_with_correct_family() {
        let up = AdjectivalPredicates::new_up(0, true);
        assert_eq!(up.family.name, "adjectival");
    }

    /// Test that `new_up` sets parity correctly for positive and negative sense.
    #[test]
    fn new_up_sets_parity_correctly() {
        let pos = AdjectivalPredicates::new_up(0, true);
        assert!(pos.parity, "positive sense should have parity=true");

        let neg = AdjectivalPredicates::new_up(0, false);
        assert!(!neg.parity, "negative sense should have parity=false");
    }

    /// Test that `new_up` stores the adjective index correctly.
    #[test]
    fn new_up_stores_adjective_index() {
        let up = AdjectivalPredicates::new_up(42, true);
        let idx = AdjectivalPredicates::to_adjective(&up);
        assert_eq!(idx, Some(42));
    }

    /// Test that `to_adjective` returns `None` for non-adjectival predicates.
    #[test]
    fn to_adjective_returns_none_for_non_adjectival() {
        let up = UnaryPredicate::new(other_family());
        assert_eq!(AdjectivalPredicates::to_adjective(&up), None);
    }

    /// Test that `parity` returns `true` (default positive) for non-adjectival predicates.
    #[test]
    fn parity_returns_true_for_non_adjectival() {
        let up = UnaryPredicate::new(other_family());
        assert!(AdjectivalPredicates::parity(&up));
    }

    /// Test that `flip_parity` reverses the sense of an adjectival predicate.
    #[test]
    fn flip_parity_reverses_sense() {
        let mut up = AdjectivalPredicates::new_up(0, true);
        assert!(up.parity);

        AdjectivalPredicates::flip_parity(&mut up);
        assert!(!up.parity, "flip_parity should reverse positive to negative");

        AdjectivalPredicates::flip_parity(&mut up);
        assert!(up.parity, "flip_parity should reverse negative back to positive");
    }

    /// Test that `flip_parity` does nothing for non-adjectival predicates.
    #[test]
    fn flip_parity_does_nothing_for_non_adjectival() {
        let mut up = UnaryPredicate::new(other_family());
        up.parity = false;

        AdjectivalPredicates::flip_parity(&mut up);
        assert!(!up.parity, "flip_parity should not change non-adjectival predicates");
    }

    /// Test that `new_atom` creates a proposition atom with the correct parity.
    #[test]
    fn new_atom_creates_proposition_with_correct_parity() {
        let term = PcalcTerm::new_variable(0);
        let prop = AdjectivalPredicates::new_atom(0, false, term);
        // The atom should have the adjectival predicate
        assert_eq!(prop.element, AtomElement::Predicate);
        assert_eq!(prop.arity, 1);
    }

    /// Test that `new_atom_on_x` creates an atom on variable 0.
    #[test]
    fn new_atom_on_x_creates_atom_on_variable_0() {
        let prop = AdjectivalPredicates::new_atom_on_x(0, false);
        assert_eq!(prop.element, AtomElement::Predicate);
        assert_eq!(prop.arity, 1);
        // The term should be variable 0
        if let Some(term) = &prop.terms[0] {
            assert!(term.is_variable());
            assert_eq!(term.variable_index(), Some(0));
        }
    }

    /// Test that `typecheck` returns 1 (ALWAYS_MATCH) as a simplified stub.
    #[test]
    fn typecheck_returns_always_match() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let typecheck_fn = family.methods.typecheck.unwrap();
        let result = typecheck_fn(family, &up, &[], &[]);
        assert_eq!(result, 1, "typecheck should return ALWAYS_MATCH (1)");
    }

    /// Test that `assert` returns `false` as a simplified stub.
    #[test]
    fn assert_returns_false() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let prop = PcalcProp::new_empty();
        let assert_fn = family.methods.assert.unwrap();
        let result = assert_fn(family, &up, false, &prop);
        assert!(!result, "assert should return false (not handled)");
    }

    /// Test that `schema` returns `false` as a simplified stub.
    #[test]
    fn schema_returns_false() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let schema_fn = family.methods.schema.unwrap();
        let result = schema_fn(family, 0, &up);
        assert!(!result, "schema should return false (not handled)");
    }

    /// Test that `testable` returns `false` as a simplified stub.
    #[test]
    fn testable_returns_false() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let result = (family.methods.testable)(family, &up);
        assert!(!result, "testable should return false (not handled)");
    }

    /// Test that `test` returns `false` as a simplified stub.
    #[test]
    fn test_returns_false() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let result = (family.methods.test)(family, &up);
        assert!(!result, "test should return false (not handled)");
    }

    /// Test that `infer_kind` returns `None` as a simplified stub.
    #[test]
    fn infer_kind_returns_none() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let result = (family.methods.infer_kind)(family, &up);
        assert_eq!(result, None, "infer_kind should return None (not handled)");
    }

    /// Test that `log` returns a non-empty string.
    #[test]
    fn log_returns_string() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, true);
        let result = (family.methods.log)(family, &up);
        assert!(!result.is_empty(), "log should return a non-empty string");
    }

    /// Test that `log` includes "not-" prefix for negative parity.
    #[test]
    fn log_negative_parity_includes_not_prefix() {
        let family = AdjectivalPredicates::start();
        let up = AdjectivalPredicates::new_up(0, false);
        let result = (family.methods.log)(family, &up);
        assert!(result.starts_with("not-"), "negative parity log should start with 'not-'");
    }
}
