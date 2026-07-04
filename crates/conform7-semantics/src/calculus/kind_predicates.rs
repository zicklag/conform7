use std::sync::LazyLock;

use crate::calculus::atoms::{AtomElement, PcalcProp, PredicateRef};
use crate::calculus::terms::PcalcTerm;
use crate::calculus::unary_predicate_families::{UpFamily, UpFamilyMethods};
use crate::calculus::unary_predicates::UnaryPredicate;
use crate::calculus::kind_predicates_revisited::KindPredicatesRevisited;

/// The family for kind=K unary predicates.
///
/// Corresponds to `kind_up_family` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`, line 9).
pub static KIND_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    let mut family = UpFamily::new(
        "kind",
        UpFamilyMethods {
            log: kind_log,
            infer_kind: kind_infer_kind,
            testable: kind_testable,
            test: kind_test,
            ..Default::default()
        },
    );
    KindPredicatesRevisited::wire(&mut family);
    family
});

/// Log a kind predicate.
///
/// Corresponds to `KindPredicates::log_kind` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 119-124).
fn kind_log(_family: &UpFamily, up: &UnaryPredicate) -> String {
    format!("kind={}", up.assert_kind.unwrap_or("?"))
}

/// Infer the kind from a kind predicate.
///
/// Corresponds to `KindPredicates::infer_kind` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 99-101).
fn kind_infer_kind(_family: &UpFamily, up: &UnaryPredicate) -> Option<&'static str> {
    up.assert_kind
}

/// Kind predicates are always testable at compile-time.
///
/// Corresponds to `KindPredicates::testable` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 109-111).
fn kind_testable(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
    true
}

/// Kind predicates always test true at compile-time.
///
/// Corresponds to `KindPredicates::test` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 113-116).
fn kind_test(_family: &UpFamily, _up: &UnaryPredicate) -> bool {
    true
}

/// Operations on kind=K unary predicates.
///
/// Corresponds to `KindPredicates` in the C reference
/// (`services/calculus-module/Chapter 2/Kind Predicates.w`).
pub struct KindPredicates;

impl KindPredicates {
    /// Create a `kind=K` unary predicate atom.
    ///
    /// The `kind_name` should be the full predicate name like "kind=number".
    ///
    /// Corresponds to `KindPredicates::new_atom` in the C reference
    /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 23-27).
    pub fn new_atom(kind_name: &'static str, term: PcalcTerm) -> PcalcProp {
        PcalcProp::unary_predicate_new(kind_name, term)
    }

    /// Check if an atom is a kind predicate.
    ///
    /// A kind predicate is a unary predicate atom whose predicate name
    /// starts with "kind=".
    ///
    /// Corresponds to `KindPredicates::is_kind_atom` in the C reference
    /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 29-35).
    pub fn is_kind_atom(prop: &PcalcProp) -> bool {
        if prop.element != AtomElement::Predicate {
            return false;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => name.starts_with("kind="),
            _ => false,
        }
    }

    /// Extract the kind name from a kind predicate atom.
    ///
    /// Corresponds to `KindPredicates::get_kind` in the C reference
    /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 37-43).
    pub fn get_kind(prop: &PcalcProp) -> Option<&str> {
        if !KindPredicates::is_kind_atom(prop) {
            return None;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => {
                // Strip the "kind=" prefix
                name.strip_prefix("kind=")
            }
            _ => None,
        }
    }

    /// Create a composited kind predicate atom.
    ///
    /// A composited kind predicate has the composited flag set, for
    /// composite determiner/noun usage like "somewhere".
    ///
    /// Corresponds to `KindPredicates::new_composited_atom` in the C reference
    /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 50-55).
    pub fn new_composited_atom(kind_name: &'static str, term: PcalcTerm) -> PcalcProp {
        let mut atom = KindPredicates::new_atom(kind_name, term);
        // Mark the atom as composited using the quantification_parameter field.
        // This field is only used for quantifiers; for predicates it is always 0,
        // so setting it to 1 serves as a composited flag without changing the struct.
        atom.quantification_parameter = 1;
        atom
    }

    /// Check if a kind predicate atom is composited.
    ///
    /// Corresponds to `KindPredicates::is_composited_atom` in the C reference
    /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, lines 57-65).
    pub fn is_composited_atom(prop: &PcalcProp) -> bool {
        // A composited atom is a kind predicate atom with the composited flag set.
        // The flag is stored in quantification_parameter (1 = composited).
        prop.quantification_parameter == 1 && KindPredicates::is_kind_atom(prop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::atoms::AtomElement;

    #[test]
    fn test_new_atom_creates_predicate_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = KindPredicates::new_atom("kind=number", term);
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
        assert!(atom.terms[0].is_some());
    }

    #[test]
    fn test_is_kind_atom_returns_true() {
        let term = PcalcTerm::new_variable(0);
        let atom = KindPredicates::new_atom("kind=number", term);
        assert!(KindPredicates::is_kind_atom(&atom));
    }

    #[test]
    fn test_is_kind_atom_returns_false_for_non_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("test", term);
        assert!(!KindPredicates::is_kind_atom(&atom));
    }

    #[test]
    fn test_get_kind_returns_correct_name() {
        let term = PcalcTerm::new_variable(0);
        let atom = KindPredicates::new_atom("kind=number", term);
        assert_eq!(KindPredicates::get_kind(&atom), Some("number"));
    }

    #[test]
    fn test_get_kind_returns_none_for_non_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("test", term);
        assert_eq!(KindPredicates::get_kind(&atom), None);
    }

    #[test]
    fn test_new_composited_atom_creates_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = KindPredicates::new_composited_atom("kind=number", term);
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
    }

    #[test]
    fn test_is_composited_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = KindPredicates::new_composited_atom("kind=number", term);
        assert!(KindPredicates::is_composited_atom(&atom));
    }

    #[test]
    fn test_kind_family_methods() {
        let mut up = UnaryPredicate::new(&KIND_UP_FAMILY);
        up.assert_kind = Some("number");

        let log_result = (KIND_UP_FAMILY.methods.log)(&KIND_UP_FAMILY, &up);
        assert_eq!(log_result, "kind=number");

        let inferred = (KIND_UP_FAMILY.methods.infer_kind)(&KIND_UP_FAMILY, &up);
        assert_eq!(inferred, Some("number"));

        assert!((KIND_UP_FAMILY.methods.testable)(&KIND_UP_FAMILY, &up));
        assert!((KIND_UP_FAMILY.methods.test)(&KIND_UP_FAMILY, &up));
    }
}
