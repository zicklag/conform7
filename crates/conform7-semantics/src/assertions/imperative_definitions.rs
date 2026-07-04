//! Imperative Definitions — the definition objects and their assessment.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! section on `imperative_defn` and `id_body` types.
//!
//! It defines:
//! - [`ImperativeDefn`] — a single imperative definition, referencing a family
//! - [`IdBody`] — the body of an imperative definition (minimal placeholder)
//! - [`ImperativeDefinitions`] — management functions, including [`assess_all`]
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
//! - Plan: PLAN-60
//!
//! [`assess_all`]: ImperativeDefinitions::assess_all

use crate::assertions::imperative_definition_families::{
    ImpDefFamily, ImperativeDefinitionFamilies,
};

/// A body of an imperative definition (minimal placeholder).
///
/// Corresponds to `id_body` in the C reference
/// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
///
/// In the full implementation, this will hold the parsed body content,
/// type data, and compilation state for an imperative definition.
#[derive(Clone, Debug, Default)]
pub struct IdBody;

/// An imperative definition.
///
/// Corresponds to `imperative_defn` in the C reference
/// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
///
/// Each definition belongs to a family (identified by index into the
/// families array) and may have a body.
#[derive(Clone, Debug)]
pub struct ImperativeDefn {
    /// Index into the families array identifying which family this
    /// definition belongs to.
    pub family: usize,
    /// The body of this definition, created during Step 1 of assessment.
    pub body: Option<IdBody>,
}

impl ImperativeDefn {
    /// Create a new imperative definition for the given family index.
    ///
    /// The body is initially `None`; it is created during `assess_all` Step 1.
    pub fn new(family: usize) -> Self {
        ImperativeDefn { family, body: None }
    }
}

/// Management functions for imperative definitions.
///
/// Corresponds to the imperative-definition management functions in the C
/// reference (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
pub struct ImperativeDefinitions;

impl ImperativeDefinitions {
    /// 4-step assessment of all imperative definitions.
    ///
    /// This is the main assessment pipeline for imperative definitions,
    /// corresponding to the `ImperativeDefinitions::assess_all` function in
    /// the C reference.
    ///
    /// # Steps
    ///
    /// 1. **Assess**: For each definition, call the family's `assess` method,
    ///    create an `IdBody`, and call the family's `given_body` method.
    /// 2. **Register**: For each family, call the family's `register` method.
    /// 3. **To phrcd**: For each definition, call the family's `to_rcd` method
    ///    to provide runtime context data.
    /// 4. **Complete**: For each family, call the family's `assessment_complete`
    ///    method.
    ///
    /// # Parameters
    ///
    /// - `defns` — mutable slice of imperative definitions to assess
    /// - `families` — slice of imperative definition families for dispatch
    pub fn assess_all(defns: &mut [ImperativeDefn], families: &[ImpDefFamily]) {
        // Step 1: Assess each definition
        for defn in defns.iter_mut() {
            ImperativeDefinitionFamilies::assess(&families[defn.family]);
            defn.body = Some(IdBody);
            ImperativeDefinitionFamilies::given_body(&families[defn.family]);
        }

        // Step 2: Register each family
        for family in families {
            ImperativeDefinitionFamilies::register(family);
        }

        // Step 3: Provide runtime context data for each definition
        for defn in defns.iter() {
            ImperativeDefinitionFamilies::to_rcd(&families[defn.family]);
        }

        // Step 4: Complete assessment for each family
        for family in families {
            ImperativeDefinitionFamilies::assessment_complete(family);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        BUILTIN_IMP_DEFN_FAMILIES,
    };

    /// Test that `assess_all` runs all 4 steps without panicking when given
    /// empty inputs.
    #[test]
    fn assess_all_empty_inputs() {
        let mut defns: Vec<ImperativeDefn> = vec![];
        let families: &[ImpDefFamily] = &[];

        ImperativeDefinitions::assess_all(&mut defns, families);
        // Should not panic
    }

    /// Test that `assess_all` runs all 4 steps with a single definition
    /// and the built-in families.
    #[test]
    fn assess_all_with_builtin_families() {
        let families: &[ImpDefFamily] = &BUILTIN_IMP_DEFN_FAMILIES;
        let mut defns = vec![
            ImperativeDefn::new(0), // unknown-idf
        ];

        ImperativeDefinitions::assess_all(&mut defns, families);

        // Step 1 should have created a body
        assert!(defns[0].body.is_some());
    }

    /// Test that `assess_all` runs all 4 steps with multiple definitions
    /// across different families.
    #[test]
    fn assess_all_multiple_defns_multiple_families() {
        let families: &[ImpDefFamily] = &BUILTIN_IMP_DEFN_FAMILIES;
        let mut defns = vec![
            ImperativeDefn::new(0), // unknown-idf
            ImperativeDefn::new(1), // adjectival-idf
            ImperativeDefn::new(2), // TO_PHRASE_EFF
            ImperativeDefn::new(3), // rule-idf
        ];

        ImperativeDefinitions::assess_all(&mut defns, families);

        // All definitions should have bodies after Step 1
        for (i, defn) in defns.iter().enumerate() {
            assert!(
                defn.body.is_some(),
                "definition {} (family {}) should have a body after assess_all",
                i,
                defn.family
            );
        }
    }

    /// Test that `ImperativeDefn::new` creates a definition with no body.
    #[test]
    fn imperative_defn_new_has_no_body() {
        let defn = ImperativeDefn::new(2);
        assert_eq!(defn.family, 2);
        assert!(defn.body.is_none());
    }

    /// Test that `IdBody` can be constructed.
    #[test]
    fn id_body_construct() {
        let body = IdBody;
        // Just verify it exists — no fields yet
        let _ = body;
    }
}
