# Plan 48: Rule Family — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Wire `identify`, `assess`, `assessment_complete`, `allows_rule_only_phrases`, `given_body`, `to_rcd`, and `compile` methods into the `rule-idf` family created by `BUILTIN_IMP_DEFN_FAMILIES` (PLAN-41, Complete). This is the last of the four built-in imperative definition families in the assertions-module startup sequence, corresponding to `RuleFamily::create_family()` in the C reference.

The methods are installed as simplified stubs — they exist and dispatch correctly, but their full logic (preamble matching, `rule_family_data` allocation, rulebook placement, `rule` structure creation, runtime context data, compilation) is deferred until the `imperative_defn`, `id_body`, `rule`, `rulebook`, and `parse_node` types are introduced in later plans.

## Background

### C reference architecture

The C reference (`inform7/assertions-module/Chapter 5/Rule Family.w`) defines seven methods on the `rule_idf` family:

```c
imperative_defn_family *rule_idf = NULL; /* "Before taking a container, ..." */
void RuleFamily::create_family(void) {
    rule_idf = ImperativeDefinitionFamilies::new(I"rule-idf", FALSE);
    METHOD_ADD(rule_idf, IDENTIFY_IMP_DEFN_MTID, RuleFamily::identify);
    METHOD_ADD(rule_idf, ASSESS_IMP_DEFN_MTID, RuleFamily::assess);
    METHOD_ADD(rule_idf, ASSESSMENT_COMPLETE_IMP_DEFN_MTID, RuleFamily::assessment_complete);
    METHOD_ADD(rule_idf, ALLOWS_RULE_ONLY_PHRASES_IMP_DEFN_MTID, RuleFamily::allows_rule_only);
    METHOD_ADD(rule_idf, GIVEN_BODY_IMP_DEFN_MTID, RuleFamily::given_body);
    METHOD_ADD(rule_idf, TO_RCD_IMP_DEFN_MTID, RuleFamily::to_rcd);
    METHOD_ADD(rule_idf, COMPILE_IMP_DEFN_MTID, RuleFamily::compile);
}
```
(lines 19-29)

**`identify`** (lines 134-161): The `IDENTIFY_IMP_DEFN_MTID` method. Matches the preamble against `<rule-preamble>` (lines 98-107), which has a catch-all `...` production at the end — this is why `rule-idf` must be the last family to claim. On a match it sets `id->family = rule_idf`, allocates a `rule_family_data` (lines 47-64) and stores it on `id->family_specific_data`, then for forms 1 and 2 extracts the constant name and calls `Rules::obtain` to create a named `rule` structure. For forms 2 and 3 it extracts the `usage_preamble` wording. It also calls `PluginCalls::new_rule_defn_notify` to allow plugins to attach data.

**`assess`** (lines 223-245): The `ASSESS_IMP_DEFN_MTID` method. Parses the usage preamble through `<rule-preamble-fine>` (lines 167-174) and `<rulebook-stem-embellished>` (lines 176-181) to extract the rulebook, when/while text, during-spec scene, and placement. It calls `Rulebooks::match()` to get a `rulebook_match` structure, then disassembles the stem into pruned stem, bud, applicability, and prewhile-applicability. It also issues problem messages for bad preambles (`PM_BadRulePreamble`, `PM_BadRulePreambleWhen`, `PM_RuleWithDefiniteArticle`).

**`assessment_complete`** (lines 421-434): The `ASSESSMENT_COMPLETE_IMP_DEFN_MTID` method. Walks all `imperative_defn` objects belonging to `rule_idf`, calls `Rules::request_automatic_placement` for each, then calls `RuleBookings::make_automatic_placements()`. Finally traverses the syntax tree with `RuleFamily::visit_to_parse_placements` to parse manual placement sentences.

**`allows_rule_only`** (lines 846-848): The `ALLOWS_RULE_ONLY_PHRASES_IMP_DEFN_MTID` method. Returns `TRUE` — rules allow rule-only phrases (phrases that end rules/rulebooks) in their bodies.

**`given_body`** (lines 350-359): The `GIVEN_BODY_IMP_DEFN_MTID` method. Creates or obtains a `rule` structure via `Rules::obtain`, sets `rfd->defines = R`, enables run-time debugging on the body, sets the method-outcome-return type to `DECIDES_NOTHING_AND_RETURNS_MOR`, and if the rule is not in a rulebook, permits all outcomes.

**`to_rcd`** (lines 452-463): The `TO_RCD_IMP_DEFN_MTID` method. Parses the applicability text into the runtime context data's action pattern, extracts the when/while text into `rcd->activity_context`, and sets the during-spec via `Scenes::set_rcd_spec`.

**`compile`** (lines 808-817): The `COMPILE_IMP_DEFN_MTID` method. Loops over all `rule` structures, clears the `at_least_one_compiled_form_needed` flag on each rule's body, and calls `Rules::check_constraints_are_typesafe` for each.

### Current Rust state

- `crates/conform7-semantics/src/assertions/imperative_definition_families.rs` defines:
  - `ImpDefFamilyMethods` with method slots: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index` — all `Option<fn>`, default `None`.
  - `ImpDefFamily` with `name`, `methods`, `compile_last`.
  - `BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>>` with four families: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`.
  - Dispatch helpers on `ImperativeDefinitionFamilies`: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index`.
  - `rule_idf()` accessor returns `&BUILTIN_IMP_DEFN_FAMILIES[3]`.
- `crates/conform7-semantics/src/assertions/adjectival_definition_family.rs` (PLAN-46, Complete) wires `identify`, `given_body`, `allows_empty` (returns `true`), `compile` (no-op) into `adjectival-idf` (families[1]).
- `crates/conform7-semantics/src/assertions/to_phrase_family.rs` (PLAN-47, Complete) wires `identify`, `assess`, `register`, `given_body`, `allows_inline` (returns `true`), `compile` (no-op), `phrasebook_index` (returns `true`) into `TO_PHRASE_EFF` (families[2]).
- All 1416 tests pass; clippy clean (modulo pre-existing warnings in unrelated files).

### Current method signatures (simplified)

```rust
pub identify: Option<fn(&ImpDefFamily)>,
pub assess: Option<fn(&ImpDefFamily)>,
pub given_body: Option<fn(&ImpDefFamily)>,
pub to_rcd: Option<fn(&ImpDefFamily)>,
pub assessment_complete: Option<fn(&ImpDefFamily)>,
pub allows_rule_only_phrases: Option<fn(&ImpDefFamily) -> bool>,
pub compile: Option<fn(&ImpDefFamily, &mut i32, i32)>,
```

These are simplified from the C — they take `&ImpDefFamily` only, without `imperative_defn *id`, `id_body *body`, `rule *R`, `rulebook *rb`, `id_runtime_context_data *rcd`, or `parse_node *at` parameters. The full signatures will be introduced when those types exist.

## Decision

### 1. Is the Rule Family the correct next step?

**Yes.** In the C `ImperativeDefinitionFamilies::create()` startup sequence, families are created in order: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`. PLAN-46 wired the `adjectival-idf` family, PLAN-47 wired the `TO_PHRASE_EFF` family, and the next family in creation order is `rule-idf`, wired by `RuleFamily::create_family()`.

The `rule-idf` family must be created last because its `<rule-preamble>` grammar ends with a catch-all `...` production (line 107) and must claim anything not already claimed by the other families. This ordering is already established in the `BUILTIN_IMP_DEFN_FAMILIES` initialization (families[3]).

### 2. Why Rule Family over other candidates?

- **Startup order**: `rule-idf` is the last family in the creation sequence (family #4). The other assertions-module startup items (AdjectivalPredicates, CreationPredicates, QuasinumericRelations, Universal, ExplicitRelations, EqualityDetails) are separate subsystems that depend on types not yet introduced.
- **Completes the IDF family set**: With this plan, all four built-in imperative definition families will have their method slots wired. This completes the `ImperativeDefinitionFamilies::create()` startup step.
- **Pattern consistency**: The Rule Family follows the exact same pattern as PLAN-46 and PLAN-47 — create a module, wire methods as stubs, add tests. The implementer can apply the same mechanical steps.
- **Testable behavior**: Like the prior families' `allows_empty`/`allows_inline`/`phrasebook_index`, the Rule Family has one method with a real, simple return value: `allows_rule_only_phrases` returns `true` (C lines 846-848).

### 3. Is it independently testable?

**Yes.** The foundation consists of:
- Creating the `rule_family` module with seven stub method functions
- Wiring them into the `rule-idf` family during `ImperativeDefinitionFamilies::create()`
- Verifying that dispatch helpers call the installed methods and return correct defaults

### 4. What is the smallest independently testable subset?

1. `RuleFamily::wire_methods(family: &mut ImpDefFamily)` installs the seven methods.
2. `allows_rule_only_phrases` returns `true` (real behavior at this stage).
3. `identify`, `assess`, `assessment_complete`, `given_body`, `to_rcd` can be dispatched without panic (no-op stubs).
4. `compile` can be dispatched without modifying `total` (no-op stub).
5. The `rule_idf()` accessor still returns the correct family with all seven methods wired.
6. All existing tests continue to pass.

### 5. What simplifications are appropriate?

- **No `imperative_defn` or `id_body` types.** Method signatures remain `fn(&ImpDefFamily)` etc. The full `identify` logic (preamble matching, `rule_family_data` allocation, `Rules::obtain` calls) is deferred.
- **No `rule_family_data` structure.** The `rule_family_data` typedef (C lines 47-64) and `RuleFamily::new_data` (C lines 66-82) are deferred — they require `wording`, `rule`, `rulebook`, `parse_node`, and plugin types.
- **No `<rule-preamble>` grammar.** The Preform nonterminal (C lines 98-107) and its problem-message subroutines (lines 109-129) are deferred.
- **No `<rule-preamble-fine>` or `<rulebook-stem-embellished>` grammars.** The assessment grammar (C lines 167-191) and `Rulebooks::match` integration are deferred.
- **No `rule` or `rulebook` structures.** The `given_body` logic (C lines 350-398), `Rules::obtain`, `Rules::by_name`, and `Rules::set_imperative_definition` are deferred.
- **No `assessment_complete` logic.** The `imperative_defn` iteration, `Rules::request_automatic_placement`, `RuleBookings::make_automatic_placements`, and `SyntaxTree::traverse` (C lines 421-434) are deferred.
- **No `to_rcd` logic.** The `ActionPatterns::parse_action_based`, `Scenes::set_rcd_spec`, and problem-message subroutines (C lines 452-463, 468-801) are deferred.
- **No `compile` logic.** The `rule` iteration, `Rules::check_constraints_are_typesafe` (C lines 808-817) are deferred.
- **No `parse_node` handling.** Source-location and wording manipulation is deferred.
- **No problem messages.** No `StandardProblems` calls for malformed rule preambles.
- **No plugin support.** The `plugin_rfd[MAX_PLUGINS]` array (C line 62) and `PluginCalls::new_rule_defn_notify` (C line 159) are deferred.

## Tasks

### Task 1: Create the `RuleFamily` module

Create `crates/conform7-semantics/src/assertions/rule_family.rs`.

Module-level doc comment:

```rust
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
```

Imports:

```rust
use crate::assertions::imperative_definition_families::ImpDefFamily;
```

The module struct:

```rust
/// The Rule Family module.
///
/// Corresponds to `RuleFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/Rule Family.w`).
pub struct RuleFamily;
```

Implementation:

```rust
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
```

### Task 2: Wire the module into `ImperativeDefinitionFamilies::create()`

Edit `crates/conform7-semantics/src/assertions/imperative_definition_families.rs`.

**2a. Add the import at the top of the file (alongside the existing `ToPhraseFamily` import, line 32):**

```rust
use crate::assertions::rule_family::RuleFamily;
```

**2b. Modify the `BUILTIN_IMP_DEFN_FAMILIES` LazyLock initialization to wire the Rule family methods.**

Replace the current LazyLock body (lines 240-254) with:

```rust
pub static BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>> = LazyLock::new(|| {
    let mut families = vec![
        ImpDefFamily::new("unknown-idf", false),
        ImpDefFamily::new("adjectival-idf", false),
        ImpDefFamily::new("TO_PHRASE_EFF", true),
        ImpDefFamily::new("rule-idf", false),
    ];
    // Wire methods for the adjectival definition family.
    // Corresponds to AdjectivalDefinitionFamily::create_family() in the C reference.
    AdjectivalDefinitionFamily::wire_methods(&mut families[1]);
    // Wire methods for the To phrase family.
    // Corresponds to ToPhraseFamily::create_family() in the C reference.
    ToPhraseFamily::wire_methods(&mut families[2]);
    // Wire methods for the rule family.
    // Corresponds to RuleFamily::create_family() in the C reference.
    RuleFamily::wire_methods(&mut families[3]);
    families
});
```

**2c. Update the doc comment on `BUILTIN_IMP_DEFN_FAMILIES` to mention the Rule family wiring.**

Replace the trailing doc-comment lines (lines 238-239) with:

```rust
/// Methods for the `TO_PHRASE_EFF` family are wired during initialization
/// by `ToPhraseFamily::wire_methods` (PLAN-47).
///
/// Methods for the `rule-idf` family are wired during initialization
/// by `RuleFamily::wire_methods` (PLAN-48).
```

**2d. Update the module-level doc comment to reference the new module.**

In the `## Module Map` table (around line 20), add a row:

```rust
//! | [`rule_family`] | `Chapter 5/Rule Family.w` | Rule definition family |
```

And in the `## References` list (around line 25), the C reference `inform7/assertions-module/Chapter 5/Rule Family.w` is already present (added in an earlier plan), so no new reference line is needed.

### Task 3: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`.

**3a. Add the module declaration (after `pub mod to_phrase_family;`, line 47):**

```rust
pub mod rule_family;
```

**3b. Add a module-map row in the `# Module Map` table (after the To phrase row, line 23):**

```
| [`rule_family`] | `Chapter 5/Rule Family.w` | Rule definition family |
```

**3c. The C reference `inform7/assertions-module/Chapter 5/Rule Family.w` is already present** in the `# References` list (line 38), so no new reference line is needed.

**3d. Update the startup-sequence comment (lines 9-15) to mention PLAN-48:**

```
//! The assertions module is initialized as part of the startup sequence:
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module) — which wires
//! `AdjectivalDefinitionFamily` methods (PLAN-46), `ToPhraseFamily` methods
//! (PLAN-47), and `RuleFamily` methods (PLAN-48), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
```

### Task 4: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/rule_family.rs`.

Required imports in tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        ImperativeDefinitionFamilies, BUILTIN_IMP_DEFN_FAMILIES,
    };
}
```

**Test 1: `wire_methods_installs_all_seven_methods`**

```rust
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
```

**Test 2: `allows_rule_only_phrases_returns_true`**

```rust
/// Test that `allows_rule_only_phrases` returns `true`.
#[test]
fn allows_rule_only_phrases_returns_true() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    let result = ImperativeDefinitionFamilies::allows_rule_only_phrases(&family);
    assert!(result, "allows_rule_only_phrases should return true for rules");
}
```

**Test 3: `identify_does_not_panic`**

```rust
/// Test that `identify` does not panic when dispatched.
#[test]
fn identify_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::identify(&family);
}
```

**Test 4: `assess_does_not_panic`**

```rust
/// Test that `assess` does not panic when dispatched.
#[test]
fn assess_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::assess(&family);
}
```

**Test 5: `assessment_complete_does_not_panic`**

```rust
/// Test that `assessment_complete` does not panic when dispatched.
#[test]
fn assessment_complete_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::assessment_complete(&family);
}
```

**Test 6: `given_body_does_not_panic`**

```rust
/// Test that `given_body` does not panic when dispatched.
#[test]
fn given_body_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::given_body(&family);
}
```

**Test 7: `to_rcd_does_not_panic`**

```rust
/// Test that `to_rcd` does not panic when dispatched.
#[test]
fn to_rcd_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::to_rcd(&family);
}
```

**Test 8: `compile_does_not_modify_total`**

```rust
/// Test that `compile` does not modify `total` (no-op stub).
#[test]
fn compile_does_not_modify_total() {
    let mut family = ImpDefFamily::new("test", false);
    RuleFamily::wire_methods(&mut family);

    let mut total = 42;
    ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
    assert_eq!(total, 42, "compile should not modify total (no-op stub)");
}
```

**Test 9: `builtin_family_has_methods_wired`**

```rust
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
```

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test -- assertions::rule_family` — new tests pass
- [ ] `cargo test` — all 1416 existing tests still pass
- [ ] `cargo clippy --all-targets` — no new warnings (pre-existing warnings in unrelated files are acceptable)

## Success Criteria

- [ ] `assertions::rule_family` module exists and compiles.
- [ ] `RuleFamily::wire_methods` installs `identify`, `assess`, `assessment_complete`, `allows_rule_only_phrases`, `given_body`, `to_rcd`, and `compile` on a family.
- [ ] `allows_rule_only_phrases` returns `true` for the rule family.
- [ ] `identify`, `assess`, `assessment_complete`, `given_body`, and `to_rcd` are no-op stubs that do not panic.
- [ ] `compile` does not modify the `total` parameter.
- [ ] The `rule-idf` built-in family has all seven methods wired after `ImperativeDefinitionFamilies::create()`.
- [ ] `cargo clippy --all-targets` introduces no new warnings.
- [ ] All existing tests (1416) still pass.

## Out of Scope

- **Full `identify` logic.** `<rule-preamble>` grammar matching (lines 98-107), `rule_family_data` allocation (lines 47-64, 66-82), constant-name extraction (lines 143-156), `Rules::obtain` calls, and `PluginCalls::new_rule_defn_notify` (line 159) are deferred.
- **Full `assess` logic.** `<rule-preamble-fine>` grammar parsing (lines 167-174), `<rulebook-stem-embellished>` parsing (lines 176-181), `Rulebooks::match()` integration (line 230), stem disassembly (lines 236-307), and problem messages (`PM_BadRulePreamble`, `PM_BadRulePreambleWhen`, `PM_RuleWithDefiniteArticle`, lines 192-257) are deferred.
- **Full `assessment_complete` logic.** `imperative_defn` iteration, `Rules::request_automatic_placement` (line 426), `RuleBookings::make_automatic_placements` (line 430), and `SyntaxTree::traverse` (lines 433, 436-443) are deferred.
- **Full `given_body` logic.** `Rules::obtain`/`Rules::by_name` (lines 361-371), `Rules::set_imperative_definition` (line 372), duplicate-name checking (lines 375-385), and indexing-text merging (lines 387-397) are deferred.
- **Full `to_rcd` logic.** `ActionPatterns::parse_action_based` (line 471), `ActionPatterns::parse_parametric` (line 476), `Scenes::set_rcd_spec` (line 461), and all problem-message subroutines (lines 498-801) are deferred.
- **Full `compile` logic.** `rule` iteration (lines 811-816), `Rules::check_constraints_are_typesafe` (line 815) are deferred.
- **`rule_family_data` and `rule` types.** These require `wording`, `imperative_defn`, `id_body`, `rulebook`, and `parse_node` types, deferred to a later plan.
- **`imperative_defn` and `id_body` types.** These will be introduced in a later plan; method signatures remain simplified.
- **`parse_node` handling.** Source-location and wording manipulation is deferred.
- **Preform grammar.** The `<rule-preamble>`, `<rule-preamble-fine>`, `<rule-preamble-finer>`, `<rulebook-stem-embellished>`, `<rulebook-bud>`, and `<unrecognised-rule-stem-diagnosis>` grammars are deferred.
- **Problem messages.** No `StandardProblems` calls for malformed rule preambles.
- **Plugin support.** The `plugin_rfd[MAX_PLUGINS]` array and `PluginCalls::new_rule_defn_notify` are deferred.
- **Other assertions-module startup items.** `AdjectivalPredicates::start()`, `CreationPredicates::start()`, `QuasinumericRelations::start()`, `Relations::Universal::start()`, `ExplicitRelations::start()`, and `EqualityDetails::start()` are deferred to later plans.
