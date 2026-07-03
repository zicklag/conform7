/// The Kind Predicates Revisited system — typechecking, assertion, and schema
/// for the kind unary predicate family.
///
/// Corresponds to `KindPredicatesRevisited` in the C reference
/// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`).
///
/// Adds methods to the `kind_up_family` created by the calculus module's
/// kind predicates system (`services/calculus-module/Chapter 2/Kind Predicates.w`).
///
/// Simplified:
/// - No `Kinds::compatible` (typecheck always returns ALWAYS_MATCH)
/// - No `Instances::set_kind` or `Kinds::make_subkind` (assert returns `false`)
/// - No `Calculus::Schemas::modify` (schema returns `false`)
/// - No `StandardProblems::sentence_problem` (no problem messages)
/// - No `TypecheckPropositions::kind_of_term` (no term kind resolution)
use std::sync::LazyLock;

use crate::calculus::atoms::PcalcProp;
use crate::calculus::kind_predicates::KIND_UP_FAMILY;
use crate::calculus::unary_predicate_families::UpFamily;
use crate::calculus::unary_predicates::UnaryPredicate;

/// The Kind Predicates Revisited module.
///
/// Corresponds to `KindPredicatesRevisited` in the C reference
/// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`).
pub struct KindPredicatesRevisited;

impl KindPredicatesRevisited {
    /// Install typecheck, assert, and schema methods on a unary predicate family.
    ///
    /// Corresponds to the `METHOD_ADD` calls in the C reference
    /// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`, lines 8-16).
    ///
    /// # Arguments
    ///
    /// * `family` - The family to install methods on.
    pub fn wire(family: &mut UpFamily) {
        family.methods.typecheck = Some(KindPredicatesRevisited::typecheck);
        family.methods.assert = Some(KindPredicatesRevisited::assert);
        family.methods.schema = Some(KindPredicatesRevisited::get_schema);
    }

    /// Force initialization of the global `KIND_UP_FAMILY`.
    ///
    /// Corresponds to `KindPredicatesRevisited::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`, lines 8-16).
    ///
    /// In the C version, this adds methods to the already-created family.
    /// In Rust, the wiring is done inside the `LazyLock::new` closure that
    /// builds `KIND_UP_FAMILY`; this function forces that initialization
    /// and serves as the C-compatible entry point for the assertions-module
    /// startup sequence.
    pub fn start() {
        LazyLock::<UpFamily>::force(&KIND_UP_FAMILY);
    }

    /// Typecheck the terms of a kind predicate.
    ///
    /// Corresponds to `KindPredicatesRevisited::typecheck` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`, lines 23-53).
    ///
    /// Simplified: returns `1` (`ALWAYS_MATCH`), accepting any kinds.
    /// The full implementation handles:
    /// - Checking `Kinds::compatible` between the asserted kind and the term's kind
    /// - Issuing problem messages for incompatible kinds
    /// - Object kind normalization via `Kinds::Behaviour::is_object`
    pub fn typecheck(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        1 // ALWAYS_MATCH
    }

    /// Assert a kind predicate as a true fact about the model world.
    ///
    /// Corresponds to `KindPredicatesRevisited::assert` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`, lines 55-68).
    ///
    /// Simplified: returns `false`. The full implementation handles:
    /// - Setting the kind of an instance via `Instances::set_kind`
    /// - Making a subkind via `Kinds::make_subkind`
    /// - Issuing problem messages for negated kind assertions
    pub fn assert(
        _family: &UpFamily,
        _up: &UnaryPredicate,
        _now_negated: bool,
        _prop: &PcalcProp,
    ) -> bool {
        false
    }

    /// Compile run-time code for a kind predicate task.
    ///
    /// Corresponds to `KindPredicatesRevisited::get_schema` in the C reference
    /// (`inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w`, lines 70-83).
    ///
    /// Simplified: returns `false`. The full implementation handles:
    /// - `TEST_ATOM_TASK`: Compile to I6 `ofclass` or `true`
    /// - `NOW_ATOM_TRUE_TASK` / `NOW_ATOM_FALSE_TASK`: Issue problem messages
    pub fn get_schema(
        _family: &UpFamily,
        _task: u8,
        _up: &UnaryPredicate,
    ) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::unary_predicate_families::UpFamilyMethods;

    /// Create a test family with no methods installed.
    fn empty_family() -> &'static UpFamily {
        static FAMILY: std::sync::LazyLock<UpFamily> = std::sync::LazyLock::new(|| {
            UpFamily::new("test", UpFamilyMethods::default())
        });
        &FAMILY
    }

    /// Test that `wire` installs the three optional methods on a local family.
    #[test]
    fn test_wire_installs_methods() {
        let mut family = UpFamily::new("test", UpFamilyMethods::default());
        assert!(family.methods.typecheck.is_none());
        assert!(family.methods.assert.is_none());
        assert!(family.methods.schema.is_none());

        KindPredicatesRevisited::wire(&mut family);

        assert!(family.methods.typecheck.is_some());
        assert!(family.methods.assert.is_some());
        assert!(family.methods.schema.is_some());
    }

    /// Test that `typecheck` returns `1` (ALWAYS_MATCH).
    #[test]
    fn test_typecheck_returns_always_match() {
        let family = empty_family();
        let up = UnaryPredicate::new(family);

        let result = KindPredicatesRevisited::typecheck(
            family,
            &up,
            &[],
            &[],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH
    }

    /// Test that `assert` returns `false`.
    #[test]
    fn test_assert_returns_false() {
        let family = empty_family();
        let up = UnaryPredicate::new(family);
        let prop = PcalcProp::new(crate::calculus::atoms::AtomElement::Predicate);

        let result = KindPredicatesRevisited::assert(
            family,
            &up,
            false,
            &prop,
        );
        assert!(!result);
    }

    /// Test that `get_schema` returns `false`.
    #[test]
    fn test_get_schema_returns_false() {
        let family = empty_family();
        let up = UnaryPredicate::new(family);

        let result = KindPredicatesRevisited::get_schema(
            family,
            0,
            &up,
        );
        assert!(!result);
    }

    /// Test that `start` runs without panic and the global `kind_up_family`
    /// has the three methods installed.
    #[test]
    fn test_start_initializes_global_family() {
        KindPredicatesRevisited::start();

        let family = &*KIND_UP_FAMILY;
        assert_eq!(family.name, "kind");
        assert!(family.methods.typecheck.is_some());
        assert!(family.methods.assert.is_some());
        assert!(family.methods.schema.is_some());
    }
}
