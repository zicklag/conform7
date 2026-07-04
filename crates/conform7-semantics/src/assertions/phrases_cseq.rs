//! PHRASES_CSEQ — Phrases and Rules bench dispatch.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 1/Assertions Module.w` section
//! "Phrases and rules" (lines 171-178). It dispatches five benches:
//!
//! 1. `LiteralPatterns::define_named_phrases` — stub (deferred)
//! 2. `ImperativeDefinitions::assess_all` — the 4-step assessment
//! 3. `Equations::traverse_to_stock` — stub (deferred)
//! 4. `Tables::traverse_to_stock` — stub (deferred)
//! 5. `RTRulebooks::RulebookOutcomePrintingRule` — stub (deferred)
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 1/Assertions Module.w` (lines 171-178)
//! - Plan: PLAN-60

use crate::assertions::imperative_definition_families::ImpDefFamily;
use crate::assertions::imperative_definitions::{ImperativeDefn, ImperativeDefinitions};

/// Run all five PHRASES_CSEQ benches.
///
/// # Parameters
///
/// - `defns` — mutable slice of imperative definitions to assess
/// - `families` — slice of imperative definition families for dispatch
pub fn run_phrases_cseq(defns: &mut [ImperativeDefn], families: &[ImpDefFamily]) {
    // 1. LiteralPatterns::define_named_phrases — stub (deferred)
    // No-op: deferred to a later plan.

    // 2. ImperativeDefinitions::assess_all
    ImperativeDefinitions::assess_all(defns, families);

    // 3. Equations::traverse_to_stock — stub (deferred)
    // No-op: deferred to a later plan.

    // 4. Tables::traverse_to_stock — stub (deferred)
    // No-op: deferred to a later plan.

    // 5. RTRulebooks::RulebookOutcomePrintingRule — stub (deferred)
    // No-op: deferred to a later plan.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::BUILTIN_IMP_DEFN_FAMILIES;

    /// Test that `run_phrases_cseq` runs all 5 benches without panicking
    /// when given empty inputs.
    #[test]
    fn run_phrases_cseq_empty_inputs() {
        let mut defns: Vec<ImperativeDefn> = vec![];
        let families: &[ImpDefFamily] = &[];

        run_phrases_cseq(&mut defns, families);
        // Should not panic
    }

    /// Test that `run_phrases_cseq` runs all 5 benches with built-in
    /// families and definitions.
    #[test]
    fn run_phrases_cseq_with_builtin_families() {
        let families: &[ImpDefFamily] = &BUILTIN_IMP_DEFN_FAMILIES;
        let mut defns = vec![
            ImperativeDefn::new(0), // unknown-idf
            ImperativeDefn::new(1), // adjectival-idf
            ImperativeDefn::new(2), // TO_PHRASE_EFF
            ImperativeDefn::new(3), // rule-idf
        ];

        run_phrases_cseq(&mut defns, families);

        // All definitions should have bodies after assess_all
        for (i, defn) in defns.iter().enumerate() {
            assert!(
                defn.body.is_some(),
                "definition {} (family {}) should have a body after run_phrases_cseq",
                i,
                defn.family
            );
        }
    }
}
