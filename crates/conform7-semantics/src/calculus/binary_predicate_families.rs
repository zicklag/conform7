use crate::calculus::binary_predicates::BinaryPredicate;

/// Return value from typecheck when the family does not handle the given kinds.
///
/// Corresponds to `DECLINE_TO_MATCH` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`).
pub const DECLINE_TO_MATCH: i8 = -1;

/// Methods that can be implemented for a binary predicate family.
///
/// Corresponds to the method IDs in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 29-161).
///
/// All methods are optional — the default implementations return
/// DECLINE_TO_MATCH for typecheck, FALSE for assert, etc.
#[derive(Clone, Debug, Default)]
pub struct BpFamilyMethods {
    /// Stock up on relations (stage 1: built-in essentials; stage 2: one per value property).
    /// Corresponds to STOCK_BPF_MTID.
    pub stock: Option<fn(&BpFamily, u8)>,
    /// Typecheck the terms of a relation.
    /// Corresponds to TYPECHECK_BPF_MTID.
    #[allow(clippy::type_complexity)]
    pub typecheck:
        Option<fn(&BpFamily, &BinaryPredicate, &[Option<usize>], &[Option<usize>]) -> i8>,
    #[allow(clippy::type_complexity)]
    /// Assert a relation as a true fact about the model world.
    /// Corresponds to ASSERT_BPF_MTID.
    pub assert: Option<
        fn(
            &BpFamily,
            &BinaryPredicate,
            usize,
            Option<&'static str>,
            usize,
            Option<&'static str>,
        ) -> bool,
    >,
    /// Compile run-time code for a task (test, make-true, make-false).
    /// Corresponds to SCHEMA_BPF_MTID.
    pub schema: Option<fn(&BpFamily, u8, &BinaryPredicate) -> bool>,
    /// Describe the relation in problem messages.
    /// Corresponds to DESCRIBE_FOR_PROBLEMS_BPF_MTID.
    pub describe_for_problems: Option<fn(&BpFamily, &BinaryPredicate) -> String>,
    /// Describe the relation in the Phrasebook index.
    /// Corresponds to DESCRIBE_FOR_INDEX_BPF_MTID.
    pub describe_for_index: Option<fn(&BpFamily, &BinaryPredicate) -> String>,
}


/// A family of related binary predicates.
///
/// Corresponds to `bp_family` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 18-21).
///
/// Each family provides method dispatch for typechecking, asserting, and
/// compiling binary predicates. Inform currently has a little over 10
/// different families.
#[derive(Clone, Debug)]
pub struct BpFamily {
    /// Name of this family (for debugging).
    pub name: &'static str,
    /// Method implementations for this family.
    pub methods: BpFamilyMethods,
}

impl BpFamily {
    /// Create a new binary predicate family with default (no-op) methods.
    ///
    /// Corresponds to `BinaryPredicateFamilies::new` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 23-27).
    pub fn new(name: &'static str) -> Self {
        BpFamily {
            name,
            methods: BpFamilyMethods::default(),
        }
    }

    /// Create a new binary predicate family with the given methods.
    pub fn new_with_methods(name: &'static str, methods: BpFamilyMethods) -> Self {
        BpFamily { name, methods }
    }
}

/// Management functions for binary predicate families.
///
/// Corresponds to `BinaryPredicateFamilies` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 23-161).
pub struct BinaryPredicateFamilies;

impl BinaryPredicateFamilies {
    /// Create a new family with the given name and default methods.
    ///
    /// Corresponds to `BinaryPredicateFamilies::new` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 23-27).
    pub fn new_family(name: &'static str) -> BpFamily {
        BpFamily::new(name)
    }

    /// Call stock(1) on all families — built-in essentials.
    ///
    /// Corresponds to `BinaryPredicateFamilies::first_stock` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 100-110).
    pub fn first_stock(families: &mut [BpFamily]) {
        for family in families.iter_mut() {
            if let Some(stock) = family.methods.stock {
                stock(family, 1);
            }
        }
    }

    /// Call stock(2) on all families — one relation per value property.
    ///
    /// Corresponds to `BinaryPredicateFamilies::second_stock` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 112-122).
    pub fn second_stock(families: &mut [BpFamily]) {
        for family in families.iter_mut() {
            if let Some(stock) = family.methods.stock {
                stock(family, 2);
            }
        }
    }

    /// Dispatch to the family's typecheck method.
    ///
    /// Returns `DECLINE_TO_MATCH` if the family has no typecheck method.
    ///
    /// Corresponds to `BinaryPredicateFamilies::typecheck` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 124-134).
    pub fn typecheck(
        bp: &BinaryPredicate,
        kinds_of_terms: &[Option<usize>],
        kinds_required: &[Option<usize>],
        families: &[BpFamily],
    ) -> i8 {
        if bp.relation_family < families.len() {
            if let Some(typecheck) = families[bp.relation_family].methods.typecheck {
                return typecheck(&families[bp.relation_family], bp, kinds_of_terms, kinds_required);
            }
        }
        DECLINE_TO_MATCH
    }

    /// Dispatch to the family's assert method.
    ///
    /// Returns `false` if the family has no assert method.
    ///
    /// Corresponds to `BinaryPredicateFamilies::assert` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 136-146).
    pub fn assert(
        bp: &BinaryPredicate,
        subj0: usize,
        spec0: Option<&'static str>,
        subj1: usize,
        spec1: Option<&'static str>,
        families: &[BpFamily],
    ) -> bool {
        if bp.relation_family < families.len() {
            if let Some(assert) = families[bp.relation_family].methods.assert {
                return assert(
                    &families[bp.relation_family],
                    bp,
                    subj0,
                    spec0,
                    subj1,
                    spec1,
                );
            }
        }
        false
    }

    /// Dispatch to the family's schema method, falling back to the BP's task_functions.
    ///
    /// Corresponds to `BinaryPredicateFamilies::get_schema` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 148-158).
    pub fn get_schema(task: u8, bp: &BinaryPredicate, families: &[BpFamily]) -> bool {
        if bp.relation_family < families.len() {
            if let Some(schema) = families[bp.relation_family].methods.schema {
                return schema(&families[bp.relation_family], task, bp);
            }
        }
        // Fall back to checking if the BP has a task function for this task
        let task_idx = task as usize;
        if task_idx > 0 && task_idx < bp.task_functions.len() {
            bp.task_functions[task_idx].is_some()
        } else {
            false
        }
    }

    /// Dispatch to the family's describe_for_problems method.
    ///
    /// Returns a default description if the family has no method.
    ///
    /// Corresponds to `BinaryPredicateFamilies::describe_for_problems` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 160-162).
    pub fn describe_for_problems(bp: &BinaryPredicate, families: &[BpFamily]) -> String {
        if bp.relation_family < families.len() {
            if let Some(describe) = families[bp.relation_family].methods.describe_for_problems {
                return describe(&families[bp.relation_family], bp);
            }
        }
        bp.relation_name.unwrap_or("(unnamed relation)").to_string()
    }

    /// Dispatch to the family's describe_for_index method.
    ///
    /// Returns a default description if the family has no method.
    ///
    /// Corresponds to `BinaryPredicateFamilies::describe_for_index` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`).
    pub fn describe_for_index(bp: &BinaryPredicate, families: &[BpFamily]) -> String {
        if bp.relation_family < families.len() {
            if let Some(describe) = families[bp.relation_family].methods.describe_for_index {
                return describe(&families[bp.relation_family], bp);
            }
        }
        bp.relation_name.unwrap_or("(unnamed relation)").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::bp_term_details::BPTerms;
    use crate::calculus::binary_predicates::BinaryPredicates;


    fn make_typecheck_family() -> BpFamily {
        BpFamily::new_with_methods(
            "typecheck_family",
            BpFamilyMethods {
                typecheck: Some(|_, _, kinds_of_terms, kinds_required| {
                    if kinds_of_terms[0] == kinds_required[0] && kinds_of_terms[1] == kinds_required[1]
                    {
                        0
                    } else {
                        DECLINE_TO_MATCH
                    }
                }),
                ..Default::default()
            },
        )
    }

    fn make_assert_family() -> BpFamily {
        BpFamily::new_with_methods(
            "assert_family",
            BpFamilyMethods {
                assert: Some(|_, _, subj0, _, subj1, _| subj0 == 42 && subj1 == 99),
                ..Default::default()
            },
        )
    }

    fn make_describe_family() -> BpFamily {
        BpFamily::new_with_methods(
            "describe_family",
            BpFamilyMethods {
                describe_for_problems: Some(|_, bp| {
                    format!("problem: {}", bp.relation_name.unwrap_or("?"))
                }),
                describe_for_index: Some(|_, bp| {
                    format!("index: {}", bp.relation_name.unwrap_or("?"))
                }),
                ..Default::default()
            },
        )
    }

    fn make_schema_family() -> BpFamily {
        BpFamily::new_with_methods(
            "schema_family",
            BpFamilyMethods {
                schema: Some(|_, task, _| task == 1),
                ..Default::default()
            },
        )
    }

    fn make_test_bp(family_idx: usize) -> BinaryPredicate {
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(
            family_idx,
            left,
            right,
            "test_relation",
            Some("TEST_TASK"),
            Some("MAKE_TRUE_TASK"),
            Some("test_relation_name"),
            &mut registry,
        );
        registry.into_iter().next().unwrap()
    }

    #[test]
    fn test_bp_family_new_creates_family_with_correct_name() {
        let family = BpFamily::new("equality");
        assert_eq!(family.name, "equality");
    }

    #[test]
    fn test_bp_family_new_creates_family_with_default_methods() {
        let family = BpFamily::new("test");
        assert!(family.methods.stock.is_none());
        assert!(family.methods.typecheck.is_none());
        assert!(family.methods.assert.is_none());
        assert!(family.methods.schema.is_none());
        assert!(family.methods.describe_for_problems.is_none());
        assert!(family.methods.describe_for_index.is_none());
    }

    #[test]
    fn test_new_family_creates_family_with_correct_name() {
        let family = BinaryPredicateFamilies::new_family("my_family");
        assert_eq!(family.name, "my_family");
    }

    #[test]
    fn test_first_stock_calls_stock_1_on_all_families() {
        // Use a static to track calls
        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        static STAGE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

        let mut families = vec![
            BpFamily::new("empty"),
            BpFamily::new_with_methods(
                "stocked",
                BpFamilyMethods {
                    stock: Some(|_, stage| {
                        CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
                        STAGE.store(stage, std::sync::atomic::Ordering::SeqCst);
                    }),
                    ..Default::default()
                },
            ),
        ];

        BinaryPredicateFamilies::first_stock(&mut families);

        assert!(CALLED.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(STAGE.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_second_stock_calls_stock_2_on_all_families() {
        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        static STAGE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

        let mut families = vec![
            BpFamily::new("empty"),
            BpFamily::new_with_methods(
                "stocked",
                BpFamilyMethods {
                    stock: Some(|_, stage| {
                        CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
                        STAGE.store(stage, std::sync::atomic::Ordering::SeqCst);
                    }),
                    ..Default::default()
                },
            ),
        ];

        BinaryPredicateFamilies::second_stock(&mut families);

        assert!(CALLED.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(STAGE.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn test_typecheck_dispatches_to_family_method() {
        let families = vec![make_typecheck_family()];
        let bp = make_test_bp(0);

        // Matching kinds
        let result = BinaryPredicateFamilies::typecheck(
            &bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, 0);

        // Non-matching kinds
        let result = BinaryPredicateFamilies::typecheck(
            &bp,
            &[Some(0), Some(1)],
            &[Some(2), Some(3)],
            &families,
        );
        assert_eq!(result, DECLINE_TO_MATCH);
    }

    #[test]
    fn test_typecheck_returns_decline_to_match_for_family_without_typecheck() {
        let families = vec![BpFamily::new("no_typecheck")];
        let bp = make_test_bp(0);

        let result = BinaryPredicateFamilies::typecheck(
            &bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, DECLINE_TO_MATCH);
    }

    #[test]
    fn test_typecheck_returns_decline_to_match_for_out_of_range_family() {
        let families = vec![BpFamily::new("only_one")];
        // BP references family index 5 which doesn't exist
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(5, left, right, "test", None, None, None, &mut registry);
        let bp = registry.into_iter().next().unwrap();

        let result = BinaryPredicateFamilies::typecheck(
            &bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, DECLINE_TO_MATCH);
    }

    #[test]
    fn test_assert_dispatches_to_family_method() {
        let families = vec![make_assert_family()];
        let bp = make_test_bp(0);

        // Matching subjects
        assert!(BinaryPredicateFamilies::assert(
            &bp, 42, None, 99, None, &families
        ));

        // Non-matching subjects
        assert!(!BinaryPredicateFamilies::assert(
            &bp, 1, None, 2, None, &families
        ));
    }

    #[test]
    fn test_assert_returns_false_for_family_without_assert() {
        let families = vec![BpFamily::new("no_assert")];
        let bp = make_test_bp(0);

        assert!(!BinaryPredicateFamilies::assert(
            &bp, 42, None, 99, None, &families
        ));
    }

    #[test]
    fn test_describe_for_problems_dispatches_to_family_method() {
        let families = vec![make_describe_family()];
        let bp = make_test_bp(0);

        let desc = BinaryPredicateFamilies::describe_for_problems(&bp, &families);
        assert_eq!(desc, "problem: test_relation_name");
    }

    #[test]
    fn test_describe_for_problems_returns_default_for_family_without_method() {
        let families = vec![BpFamily::new("no_describe")];
        let bp = make_test_bp(0);

        let desc = BinaryPredicateFamilies::describe_for_problems(&bp, &families);
        assert_eq!(desc, "test_relation_name");
    }

    #[test]
    fn test_describe_for_index_dispatches_to_family_method() {
        let families = vec![make_describe_family()];
        let bp = make_test_bp(0);

        let desc = BinaryPredicateFamilies::describe_for_index(&bp, &families);
        assert_eq!(desc, "index: test_relation_name");
    }

    #[test]
    fn test_describe_for_index_returns_default_for_family_without_method() {
        let families = vec![BpFamily::new("no_describe")];
        let bp = make_test_bp(0);

        let desc = BinaryPredicateFamilies::describe_for_index(&bp, &families);
        assert_eq!(desc, "test_relation_name");
    }

    #[test]
    fn test_get_schema_dispatches_to_family_method() {
        let families = vec![make_schema_family()];
        let bp = make_test_bp(0);

        // Family schema returns true for task 1
        assert!(BinaryPredicateFamilies::get_schema(1, &bp, &families));
        // Family schema returns false for task 2
        assert!(!BinaryPredicateFamilies::get_schema(2, &bp, &families));
    }

    #[test]
    fn test_get_schema_falls_back_to_task_functions() {
        let families = vec![BpFamily::new("no_schema")];
        let bp = make_test_bp(0);

        // BP has TEST_TASK at index 1
        assert!(BinaryPredicateFamilies::get_schema(1, &bp, &families));
        // BP has MAKE_TRUE_TASK at index 2
        assert!(BinaryPredicateFamilies::get_schema(2, &bp, &families));
        // Index 0 is unused
        assert!(!BinaryPredicateFamilies::get_schema(0, &bp, &families));
    }

    #[test]
    fn test_get_schema_returns_false_when_no_task_function() {
        let families = vec![BpFamily::new("no_schema")];
        let left = BPTerms::new_kind(Some(0));
        let right = BPTerms::new_kind(Some(1));
        let mut registry = Vec::new();
        BinaryPredicates::make_single(
            0, left, right, "test", None, None, None, &mut registry,
        );
        let bp = registry.into_iter().next().unwrap();

        assert!(!BinaryPredicateFamilies::get_schema(1, &bp, &families));
    }
}
