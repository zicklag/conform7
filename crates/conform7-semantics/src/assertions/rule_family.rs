//! Rule Family — imperative definitions of rules.
//!
//! Corresponds to `RuleFamily` in the C reference
//! (`inform7/assertions-module/Chapter 5/Rule Family.w`).
//!
//! This family handles definitions of rules which give explicit Inform 7
//! source text to show what they do. For example:
//!
//! ```text
//! Every turn:
//!     say "The grandfather clock ticks reprovingly."
//! ```
//!
//! The preamble is recognised by the `<rule-preamble>` grammar, which ends
//! with a catch-all `...` production — this is why `rule-idf` must be the
//! last family to claim. It wires seven methods into the `rule-idf`
//! imperative definition family:
//!
//! - `identify` — decide whether a preamble belongs to this family
//! - `assess` — parse the usage preamble in more detail
//! - `assessment_complete` — file rules into their rulebooks
//! - `allows_rule_only_phrases` — whether rule-only phrases may be used
//! - `given_body` — create or obtain a `rule` structure for the body
//! - `to_rcd` — provide runtime context data for the body
//! - `compile` — clear compilation flags and check type safety
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types — method signatures take `&ImpDefFamily` only.
//! - No `rule_family_data` structure or `new_data` allocation in `identify`.
//! - No `<rule-preamble>`, `<rule-preamble-fine>`, or `<rulebook-stem-embellished>` grammars.
//! - No `rule` or `rulebook` structures — `given_body` and `assessment_complete` are no-ops.
//! - No `ActionPatterns`, `Scenes`, or `Rulebooks::match` calls in `assess`/`to_rcd`.
//! - No `Rules::obtain`, `Rules::by_name`, or `Rules::check_constraints_are_typesafe` calls.
//! - No `PluginCalls::new_rule_defn_notify` call in `identify`.
//! - No `parse_node` handling or Preform grammar.
//! - No problem messages.

use crate::assertions::imperative_definition_families::ImpDefFamily;

/// The Rule Family module.
///
/// Corresponds to `RuleFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/Rule Family.w`).
pub struct RuleFamily;

impl RuleFamily {
    /// Wire the seven rule-family methods into the given family.
    ///
    /// Corresponds to `RuleFamily::create_family` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 19-29).
    ///
    /// Simplified: all methods are stubs. The full logic (preamble matching,
    /// `rule_family_data` allocation, rulebook placement, `rule` structure
    /// creation, runtime context data, compilation) is deferred until
    /// `imperative_defn`, `id_body`, `rule`, `rulebook`, and `parse_node`
    /// types are introduced.
    pub fn wire_methods(family: &mut ImpDefFamily) {
        family.methods.identify = Some(Self::identify);
        family.methods.assess = Some(Self::assess);
        family.methods.assessment_complete = Some(Self::assessment_complete);
        family.methods.allows_rule_only_phrases = Some(Self::allows_rule_only_phrases);
        family.methods.given_body = Some(Self::given_body);
        family.methods.to_rcd = Some(Self::to_rcd);
        family.methods.compile = Some(Self::compile);
    }

    /// Decide whether a definition preamble belongs to the rule family.
    ///
    /// Corresponds to `RuleFamily::identify` (the `IDENTIFY_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 134-161).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Match the preamble text against `<rule-preamble>` (lines 98-107)
    /// 2. Set `id->family = rule_idf`
    /// 3. Allocate a `rule_family_data` (lines 47-64) and store it on
    ///    `id->family_specific_data`
    /// 4. For forms 1 and 2, extract the constant name and call
    ///    `Rules::obtain` (lines 143-156)
    /// 5. For forms 2 and 3, extract the `usage_preamble` wording (line 157)
    /// 6. Call `PluginCalls::new_rule_defn_notify` (line 159)
    fn identify(_self: &ImpDefFamily) {
        // No-op: preamble matching, rule_family_data allocation, and
        // Rules::obtain calls deferred.
    }

    /// Parse the usage preamble in more detail.
    ///
    /// Corresponds to `RuleFamily::assess` (the `ASSESS_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 223-245).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Parse `<rule-preamble-fine>` (lines 167-174) to extract the
    ///    rulebook, when/while text, and during-spec scene
    /// 2. Call `Rulebooks::match()` to get a `rulebook_match` (line 230)
    /// 3. Disassemble the stem into pruned stem, bud, applicability, and
    ///    prewhile-applicability (lines 236-307)
    /// 4. Issue problem messages for bad preambles (lines 192-215, 250-257)
    fn assess(_self: &ImpDefFamily) {
        // No-op: grammar parsing, Rulebooks::match, and problem reporting deferred.
    }

    /// File rules into their rulebooks after assessment is complete.
    ///
    /// Corresponds to `RuleFamily::assessment_complete` (the
    /// `ASSESSMENT_COMPLETE_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 421-434).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Walk all `imperative_defn` objects belonging to `rule_idf`
    /// 2. Call `Rules::request_automatic_placement` for each (line 426)
    /// 3. Call `RuleBookings::make_automatic_placements()` (line 430)
    /// 4. Traverse the syntax tree to parse manual placement sentences
    ///    (lines 433, 436-443)
    fn assessment_complete(_self: &ImpDefFamily) {
        // No-op: rulebook placement and syntax tree traversal deferred.
    }

    /// Whether rule-only phrases may be used in the body.
    ///
    /// Corresponds to `RuleFamily::allows_rule_only` (the
    /// `ALLOWS_RULE_ONLY_PHRASES_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 846-848).
    ///
    /// Rules always allow rule-only phrases (phrases that end rules/rulebooks).
    fn allows_rule_only_phrases(_self: &ImpDefFamily) -> bool {
        true
    }

    /// Create or obtain a `rule` structure for the body.
    ///
    /// Corresponds to `RuleFamily::given_body` (the `GIVEN_BODY_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 350-359).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Create or obtain a `rule` structure via `Rules::obtain` (lines 361-371)
    /// 2. Set `rfd->defines = R` (line 354)
    /// 3. Enable run-time debugging on the body (line 355)
    /// 4. Set the method-outcome-return type to `DECIDES_NOTHING_AND_RETURNS_MOR`
    ///    (lines 356-357)
    /// 5. If not in a rulebook, permit all outcomes (line 358)
    /// 6. Check for duplicate rule names (lines 375-385)
    /// 7. Merge applicability and when/while text for indexing (lines 387-397)
    fn given_body(_self: &ImpDefFamily) {
        // No-op: rule structure creation and body setup deferred.
    }

    /// Provide runtime context data for the body.
    ///
    /// Corresponds to `RuleFamily::to_rcd` (the `TO_RCD_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 452-463).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Parse the applicability text into the runtime context data's
    ///    action pattern (lines 468-488)
    /// 2. Extract the when/while text into `rcd->activity_context` (lines 459-460)
    /// 3. Set the during-spec via `Scenes::set_rcd_spec` (line 461)
    /// 4. Issue problem messages for bad action patterns (lines 498-801)
    fn to_rcd(_self: &ImpDefFamily) {
        // No-op: action pattern parsing and problem reporting deferred.
    }

    /// Clear compilation flags and check type safety for all rules.
    ///
    /// Corresponds to `RuleFamily::compile` (the `COMPILE_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/Rule Family.w`, lines 808-817).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Loop over all `rule` structures (lines 811-816)
    /// 2. Clear the `at_least_one_compiled_form_needed` flag on each body
    ///    (lines 812-814)
    /// 3. Call `Rules::check_constraints_are_typesafe` for each (line 815)
    fn compile(_self: &ImpDefFamily, _total: &mut i32, _target: i32) {
        // No-op: rule iteration and type-safety checks deferred.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        ImperativeDefinitionFamilies, BUILTIN_IMP_DEFN_FAMILIES,
    };

    /// Test that `wire_methods` installs all seven methods on a family.
    #[test]
    fn wire_methods_installs_all_seven_methods() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        assert!(family.methods.identify.is_some(), "identify should be installed");
        assert!(family.methods.assess.is_some(), "assess should be installed");
        assert!(family.methods.assessment_complete.is_some(), "assessment_complete should be installed");
        assert!(family.methods.allows_rule_only_phrases.is_some(), "allows_rule_only_phrases should be installed");
        assert!(family.methods.given_body.is_some(), "given_body should be installed");
        assert!(family.methods.to_rcd.is_some(), "to_rcd should be installed");
        assert!(family.methods.compile.is_some(), "compile should be installed");
    }

    /// Test that `allows_rule_only_phrases` returns `true`.
    #[test]
    fn allows_rule_only_phrases_returns_true() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        let result = ImperativeDefinitionFamilies::allows_rule_only_phrases(&family);
        assert!(result, "allows_rule_only_phrases should return true for rules");
    }

    /// Test that `identify` does not panic when dispatched.
    #[test]
    fn identify_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::identify(&family);
    }

    /// Test that `assess` does not panic when dispatched.
    #[test]
    fn assess_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::assess(&family);
    }

    /// Test that `assessment_complete` does not panic when dispatched.
    #[test]
    fn assessment_complete_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::assessment_complete(&family);
    }

    /// Test that `given_body` does not panic when dispatched.
    #[test]
    fn given_body_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::given_body(&family);
    }

    /// Test that `to_rcd` does not panic when dispatched.
    #[test]
    fn to_rcd_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::to_rcd(&family);
    }

    /// Test that `compile` does not modify `total` (no-op stub).
    #[test]
    fn compile_does_not_modify_total() {
        let mut family = ImpDefFamily::new("test", false);
        RuleFamily::wire_methods(&mut family);

        let mut total = 42;
        ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
        assert_eq!(total, 42, "compile should not modify total (no-op stub)");
    }

    /// Test that the builtin `rule-idf` family has all seven methods wired.
    #[test]
    fn builtin_family_has_methods_wired() {
        // Force initialization of the builtin families
        ImperativeDefinitionFamilies::create();
        let family = &BUILTIN_IMP_DEFN_FAMILIES[3];

        assert_eq!(family.name, "rule-idf");
        assert!(family.methods.identify.is_some(), "rule-idf should have identify installed");
        assert!(family.methods.assess.is_some(), "rule-idf should have assess installed");
        assert!(family.methods.assessment_complete.is_some(), "rule-idf should have assessment_complete installed");
        assert!(family.methods.allows_rule_only_phrases.is_some(), "rule-idf should have allows_rule_only_phrases installed");
        assert!(family.methods.given_body.is_some(), "rule-idf should have given_body installed");
        assert!(family.methods.to_rcd.is_some(), "rule-idf should have to_rcd installed");
        assert!(family.methods.compile.is_some(), "rule-idf should have compile installed");
    }
}
