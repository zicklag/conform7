use crate::calculus::unary_predicates::UnaryPredicate;

/// Methods that can be implemented for a unary predicate family.
///
/// Corresponds to the method dispatch table in the C reference
/// (`services/calculus-module/Chapter 2/Unary Predicate Families.w`).
#[derive(Clone, Debug)]
pub struct UpFamilyMethods {
    /// Log a unary predicate to the debug log.
    pub log: fn(&UpFamily, &UnaryPredicate) -> String,
    /// Infer the kind from a unary predicate.
    pub infer_kind: fn(&UpFamily, &UnaryPredicate) -> Option<&'static str>,
    /// Whether predicates in this family are testable at compile-time.
    pub testable: fn(&UpFamily, &UnaryPredicate) -> bool,
    /// Test a predicate at compile-time (only called if testable returns true).
    pub test: fn(&UpFamily, &UnaryPredicate) -> bool,
}

/// A family of related unary predicates.
///
/// Corresponds to `up_family` in the C reference
/// (`services/calculus-module/Chapter 2/Unary Predicate Families.w`).
#[derive(Clone, Debug)]
pub struct UpFamily {
    /// Name of this family (for debugging).
    pub name: &'static str,
    /// Method implementations for this family.
    pub methods: UpFamilyMethods,
}

impl UpFamily {
    /// Create a new unary predicate family.
    ///
    /// Corresponds to the family creation in the C reference
    /// (`services/calculus-module/Chapter 2/Unary Predicate Families.w`).
    pub fn new(name: &'static str, methods: UpFamilyMethods) -> Self {
        UpFamily { name, methods }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::unary_predicates::UnaryPredicate;

    fn make_family() -> &'static UpFamily {
        static FAMILY: std::sync::LazyLock<UpFamily> = std::sync::LazyLock::new(|| {
            UpFamily::new(
                "custom",
                UpFamilyMethods {
                    log: |_, up| format!("custom_log({})", up.assert_kind.unwrap_or("?")),
                    infer_kind: |_, up| up.assert_kind,
                    testable: |_, up| up.assert_kind.is_some(),
                    test: |_, up| up.assert_kind == Some("number"),
                },
            )
        });
        &FAMILY
    }

    #[test]
    fn test_up_family_creation() {
        let family = make_family();
        assert_eq!(family.name, "custom");
    }

    #[test]
    fn test_custom_family_methods() {
        let family = make_family();
        let mut up = UnaryPredicate::new(family);
        up.assert_kind = Some("number");

        assert_eq!(
            (family.methods.log)(family, &up),
            "custom_log(number)"
        );
        assert_eq!((family.methods.infer_kind)(family, &up), Some("number"));
        assert!((family.methods.testable)(family, &up));
        assert!((family.methods.test)(family, &up));
    }

    #[test]
    fn test_custom_family_not_testable_without_kind() {
        let family = make_family();
        let up = UnaryPredicate::new(family);
        assert!(!(family.methods.testable)(family, &up));
    }

    #[test]
    fn test_custom_family_test_fails_for_wrong_kind() {
        let family = make_family();
        let mut up = UnaryPredicate::new(family);
        up.assert_kind = Some("text");
        assert!((family.methods.testable)(family, &up));
        assert!(!(family.methods.test)(family, &up));
    }
}
