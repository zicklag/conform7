use crate::calculus::unary_predicate_families::UpFamily;

/// A unary predicate — true or false when applied to a single term.
///
/// Corresponds to `unary_predicate` in the C reference
/// (`services/calculus-module/Chapter 2/Unary Predicates.w`, lines 11-18).
#[derive(Clone, Debug)]
pub struct UnaryPredicate {
    /// The family this predicate belongs to.
    pub family: &'static UpFamily,
    /// The kind asserted by this predicate (for kind=K predicates).
    /// Simplified: a string name for the kind.
    pub assert_kind: Option<&'static str>,
    /// Whether this is a composited predicate (composite determiner/noun like "somewhere").
    pub composited: bool,
    /// Whether this is an unarticled predicate (unarticled usage like "vehicle").
    pub unarticled: bool,
    /// Calling name (for calling predicates only).
    pub calling_name: Option<&'static str>,
}

impl UnaryPredicate {
    /// Create a new unary predicate belonging to the given family.
    ///
    /// Corresponds to `UnaryPredicates::new` in the C reference
    /// (`services/calculus-module/Chapter 2/Unary Predicates.w`, lines 21-30).
    pub fn new(family: &'static UpFamily) -> Self {
        UnaryPredicate {
            family,
            assert_kind: None,
            composited: false,
            unarticled: false,
            calling_name: None,
        }
    }

    /// Create a copy of this unary predicate.
    ///
    /// Corresponds to `UnaryPredicates::copy` in the C reference
    /// (`services/calculus-module/Chapter 2/Unary Predicates.w`, lines 32-46).
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::unary_predicate_families::{UpFamily, UpFamilyMethods};

    fn test_family() -> &'static UpFamily {
        static FAMILY: std::sync::LazyLock<UpFamily> = std::sync::LazyLock::new(|| {
            UpFamily::new(
                "test",
                UpFamilyMethods {
                    log: |_, up| format!("test({})", up.assert_kind.unwrap_or("?")),
                    infer_kind: |_, up| up.assert_kind,
                    testable: |_, _| true,
                    test: |_, _| true,
                    ..Default::default()
                },
            )
        });
        &FAMILY
    }

    #[test]
    fn test_new_creates_predicate_with_correct_family() {
        let family = test_family();
        let up = UnaryPredicate::new(family);
        assert_eq!(up.family.name, "test");
        assert!(up.assert_kind.is_none());
        assert!(!up.composited);
        assert!(!up.unarticled);
        assert!(up.calling_name.is_none());
    }

    #[test]
    fn test_copy_produces_independent_copy() {
        let family = test_family();
        let up = UnaryPredicate::new(family);
        let copy = up.copy();
        assert_eq!(up.family.name, copy.family.name);
        assert_eq!(up.assert_kind, copy.assert_kind);
        assert_eq!(up.composited, copy.composited);
    }

    #[test]
    fn test_family_methods_can_be_called() {
        let family = test_family();
        let mut up = UnaryPredicate::new(family);
        up.assert_kind = Some("number");
        let log_result = (family.methods.log)(family, &up);
        assert_eq!(log_result, "test(number)");
        let inferred = (family.methods.infer_kind)(family, &up);
        assert_eq!(inferred, Some("number"));
        assert!((family.methods.testable)(family, &up));
        assert!((family.methods.test)(family, &up));
    }
}
