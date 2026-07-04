# Plan 47: To Phrase Family — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Wire `identify`, `assess`, `register`, `given_body`, `allows_inline`, `compile`, and `phrasebook_index` methods into the `TO_PHRASE_EFF` family created by `BUILTIN_IMP_DEFN_FAMILIES` (PLAN-41, Complete). This is the second concrete imperative definition family in the assertions-module startup sequence, corresponding to `ToPhraseFamily::create_family()` in the C reference.

The methods are installed as simplified stubs — they exist and dispatch correctly, but their full logic (preamble parsing, `to_family_data` allocation, logical-order linked-list insertion, excerpt-parser registration, I6 compilation requests) is deferred until the `imperative_defn`, `id_body`, and `parse_node` types are introduced in later plans.

## Background

### C reference architecture

The C reference (`inform7/assertions-module/Chapter 5/To Phrase Family.w`) defines seven methods on the `to_phrase_idf` family:

```c
imperative_defn_family *to_phrase_idf = NULL; /* "To award (some - number) points: ..." */
void ToPhraseFamily::create_family(void) {
    to_phrase_idf = ImperativeDefinitionFamilies::new(I"TO_PHRASE_EFF", TRUE);
    METHOD_ADD(to_phrase_idf, IDENTIFY_IMP_DEFN_MTID, ToPhraseFamily::claim);
    METHOD_ADD(to_phrase_idf, ASSESS_IMP_DEFN_MTID, ToPhraseFamily::assess);
    METHOD_ADD(to_phrase_idf, REGISTER_IMP_DEFN_MTID, ToPhraseFamily::register);
    METHOD_ADD(to_phrase_idf, GIVEN_BODY_IMP_DEFN_MTID, ToPhraseFamily::given_body);
    METHOD_ADD(to_phrase_idf, ALLOWS_INLINE_IMP_DEFN_MTID, ToPhraseFamily::allows_inline);
    METHOD_ADD(to_phrase_idf, COMPILE_IMP_DEFN_MTID, ToPhraseFamily::compile);
    METHOD_ADD(to_phrase_idf, PHRASEBOOK_INDEX_IMP_DEFN_MTID, ToPhraseFamily::include_in_Phrasebook_index);
}
```
(lines 17-27)

**`claim`** (lines 92-120): The `IDENTIFY_IMP_DEFN_MTID` method. Matches the preamble against `<to-phrase-preamble>` (lines 70-76), which recognises `To ...` preambles with optional `(this is the ... function)` constant-name clauses and documentation-symbol clauses. On a match it sets `id->family = to_phrase_idf`, allocates a `to_family_data` (lines 38-49) and stores it on `id->family_specific_data`, then parses out the constant name (`(this is ...)`), documentation symbol, prototype text, and (for forms 1 and 2) the `explicit_name_for_inverse` wording used for mathematical inverses like `exp` for `log`.

**`assess`** (lines 134-148): The `ASSESS_IMP_DEFN_MTID` method. Takes a closer look at the wording. If the preamble has a constant name, runs `<The preamble parses to a named To phrase>` (lines 160-178) to attach the `constant_phrase` and reject indefinite kinds (`PM_NamedGeneric`). It also detects `<now-phrase-preamble>` (`to now ...`, lines 126-127) — only one `now` variant is allowed — and `<begin-phrase-preamble>` (`to begin`, lines 129-130) for Basic mode's main phrase.

**`given_body`** (lines 193-219): The `GIVEN_BODY_IMP_DEFN_MTID` method. Called once the `id_body` exists. It calls `CompileImperativeDefn::prepare_for_requests(body)`, parses the prototype text via `ParsingIDTypeData::parse`, then inserts `id` into the `first_in_logical_order` linked list at the position determined by `ToPhraseFamily::cmp` (lines 245-278), which compares two phrase bodies' `IDTypeData` for specificity (`BEFORE_PH`, `SUBSCHEMA_PH`, `EQUAL_PH`, etc.).

**`register`** (lines 285-293): The `REGISTER_IMP_DEFN_MTID` method. Walks the `first_in_logical_order` list, calls `ParseInvocations::register_excerpt(id->body_of_defn)` to enter each phrase into the excerpt parser, and assigns each a `sequence_count` from 0 upward.

**`allows_inline`** (lines 388-390): The `ALLOWS_INLINE_IMP_DEFN_MTID` method. Returns `TRUE` — To phrases may be given as `(- ... -)` inline I6 material.

**`compile`** (lines 299-304): The `COMPILE_IMP_DEFN_MTID` method. Does not compile directly; instead it makes "requests". It marks To phrases with definite kinds for future compilation via `PhraseRequests::simple_request` (lines 306-313), and throws problems for phrases with return kinds too vaguely defined (`PM_ReturnKindVague`, `PM_ReturnKindUndetermined`, lines 315-349) and for inline phrases named as constants (`PM_NamedInline`, lines 351-365).

**`include_in_Phrasebook_index`** (lines 392-395): The `PHRASEBOOK_INDEX_IMP_DEFN_MTID` method. Returns `TRUE` — To phrases appear in the Phrasebook index.

### Current Rust state

- `crates/conform7-semantics/src/assertions/imperative_definition_families.rs` defines:
  - `ImpDefFamilyMethods` with method slots: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index` — all `Option<fn>`, default `None`.
  - `ImpDefFamily` with `name`, `methods`, `compile_last`.
  - `BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>>` with four families: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`.
  - Dispatch helpers on `ImperativeDefinitionFamilies`: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index`.
  - `to_phrase_idf()` accessor returns `&BUILTIN_IMP_DEFN_FAMILIES[2]`.
- `crates/conform7-semantics/src/assertions/adjectival_definition_family.rs` (PLAN-46, Complete) wires `identify`, `given_body`, `allows_empty` (returns `true`), `compile` (no-op) into `adjectival-idf` (families[1]).
- All 1407 tests pass; clippy clean (modulo pre-existing warnings in unrelated files).

### Current method signatures (simplified)

```rust
pub identify: Option<fn(&ImpDefFamily)>,
pub assess: Option<fn(&ImpDefFamily)>,
pub given_body: Option<fn(&ImpDefFamily)>,
pub register: Option<fn(&ImpDefFamily)>,
pub allows_inline: Option<fn(&ImpDefFamily) -> bool>,
pub compile: Option<fn(&ImpDefFamily, &mut i32, i32)>,
pub phrasebook_index: Option<fn(&ImpDefFamily) -> bool>,
```

These are simplified from the C — they take `&ImpDefFamily` only, without `imperative_defn *id`, `id_body *body`, or `parse_node *at` parameters. The full signatures will be introduced when those types exist.

## Decision

### 1. Is the To Phrase Family the correct next step?

**Yes.** In the C `ImperativeDefinitionFamilies::create()` startup sequence, families are created in order: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`. PLAN-46 wired the `adjectival-idf` family; the next family in creation order is `TO_PHRASE_EFF`, wired by `ToPhraseFamily::create_family()`. The `rule-idf` family is created last because its `<rule-preamble>` grammar ends with a catch-all `...` production (line 107) and must claim anything not already claimed — so its methods should be wired after To phrases.

### 2. Why To Phrase Family over Rule Family?

- **Startup order**: `TO_PHRASE_EFF` is created before `rule-idf` (family #3 vs #4).
- **Simplicity**: To Phrase Family is 567 lines vs Rule Family's 848 lines. The Rule Family has a substantially more elaborate preamble grammar (`<rule-preamble>`, `<rule-preamble-fine>`, `<rule-preamble-finer>`, `<rulebook-stem-embellished>`, `<rulebook-bud>`, `<unrecognised-rule-stem-diagnosis>`) and a larger `rule_family_data` structure with plugin-attached storage.
- **Foundational**: To phrases are Inform 7's function-definition mechanism; rules are built atop phrases. Wiring the To family first establishes the simpler, more general family before the rule-specific complications.
- **Testable behavior**: Like the adjectival family's `allows_empty`, the To family has two methods with real, simple return values that are testable as more than no-op stubs: `allows_inline` returns `true` (C lines 388-390) and `phrasebook_index` returns `true` (C lines 392-395).

### 3. Is it independently testable?

**Yes.** The foundation consists of:
- Creating the `to_phrase_family` module with seven stub method functions
- Wiring them into the `TO_PHRASE_EFF` family during `ImperativeDefinitionFamilies::create()`
- Verifying that dispatch helpers call the installed methods and return correct defaults

### 4. What is the smallest independently testable subset?

1. `ToPhraseFamily::wire_methods(family: &mut ImpDefFamily)` installs the seven methods.
2. `allows_inline` returns `true` (real behavior at this stage).
3. `phrasebook_index` returns `true` (real behavior at this stage).
4. `identify`, `assess`, `register`, `given_body` can be dispatched without panic (no-op stubs).
5. `compile` can be dispatched without modifying `total` (no-op stub).
6. The `to_phrase_idf()` accessor still returns the correct family with all seven methods wired.
7. All existing tests continue to pass.

### 5. What simplifications are appropriate?

- **No `imperative_defn` or `id_body` types.** Method signatures remain `fn(&ImpDefFamily)` etc. The full `claim` logic (preamble matching, `to_family_data` allocation, constant-name parsing, inverse-name extraction) is deferred.
- **No `to_family_data` structure.** The `to_family_data` typedef (C lines 38-49) and `ToPhraseFamily::new_data` (C lines 51-63) are deferred — they require `wording`, `constant_phrase`, and `imperative_defn` types.
- **No logical-order linked list.** The `first_in_logical_order` global (C line 191), `given_body`'s insertion logic (C lines 200-218), `get_next`/`set_next` (C lines 221-231), and `cmp` (C lines 245-278) are deferred — they require `id_body` and `IDTypeData`.
- **No excerpt-parser registration.** The `ParseInvocations::register_excerpt` call in `register` (C line 289) is deferred.
- **No `PhraseRequests` or problem messages.** The `compile` body (C lines 306-365) is deferred — it requires `IDTypeData`, `Kinds::Behaviour`, and `StandardProblems`.
- **No Preform grammar.** The `<to-phrase-preamble>`, `<now-phrase-preamble>`, and `<begin-phrase-preamble>` grammars are deferred.
- **No `parse_node` handling.** Source-location and wording manipulation is deferred.

## Tasks

### Task 1: Create the `ToPhraseFamily` module

Create `crates/conform7-semantics/src/assertions/to_phrase_family.rs`.

Module-level doc comment:

```rust
//! To Phrase Family — imperative definitions of "To ..." phrases.
//!
//! Corresponds to `ToPhraseFamily` in the C reference
//! (`inform7/assertions-module/Chapter 5/To Phrase Family.w`).
//!
//! This family handles definitions of "To..." phrases: Inform's equivalent of
//! function definitions. For example, `To chime (N - a number) times: ...`.
//! The preamble is recognised by its opening word "To". It wires seven
//! methods into the `TO_PHRASE_EFF` imperative definition family:
//!
//! - `identify` — decide whether a preamble belongs to this family (C: `claim`)
//! - `assess` — take a closer look at the wording after identification
//! - `register` — enter phrases into the excerpt parser in logical order
//! - `given_body` — insert the body into the logical-order linked list
//! - `allows_inline` — whether the body may be `(- ... -)` inline I6
//! - `compile` — make compilation requests for the phrase bodies
//! - `phrasebook_index` — whether definitions appear in the Phrasebook index
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types — method signatures take `&ImpDefFamily` only.
//! - No `to_family_data` structure or `new_data` allocation in `identify`.
//! - No logical-order linked list, `cmp`, or `get_next`/`set_next` in `given_body`.
//! - No `ParseInvocations::register_excerpt` call in `register`.
//! - No `PhraseRequests` calls or problem messages in `compile`.
//! - No `parse_node` handling or Preform grammar.
```

Imports:

```rust
use crate::assertions::imperative_definition_families::ImpDefFamily;
```

The module struct:

```rust
/// The To Phrase Family module.
///
/// Corresponds to `ToPhraseFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`).
pub struct ToPhraseFamily;
```

Implementation:

```rust
impl ToPhraseFamily {
    /// Wire the seven To-phrase-family methods into the given family.
    ///
    /// Corresponds to `ToPhraseFamily::create_family` in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 18-27).
    ///
    /// Simplified: all methods are stubs. The full logic (preamble matching,
    /// `to_family_data` allocation, logical-order insertion, excerpt-parser
    /// registration, compilation requests) is deferred until `imperative_defn`,
    /// `id_body`, and `parse_node` types are introduced.
    pub fn wire_methods(family: &mut ImpDefFamily) {
        family.methods.identify = Some(Self::identify);
        family.methods.assess = Some(Self::assess);
        family.methods.register = Some(Self::register);
        family.methods.given_body = Some(Self::given_body);
        family.methods.allows_inline = Some(Self::allows_inline);
        family.methods.compile = Some(Self::compile);
        family.methods.phrasebook_index = Some(Self::phrasebook_index);
    }

    /// Decide whether a definition preamble belongs to the To phrase family.
    ///
    /// Corresponds to `ToPhraseFamily::claim` (the `IDENTIFY_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 92-120).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Match the preamble text against `<to-phrase-preamble>` (lines 70-76)
    /// 2. Set `id->family = to_phrase_idf`
    /// 3. Allocate a `to_family_data` (lines 38-49) and store it on
    ///    `id->family_specific_data`
    /// 4. Parse out the constant name (`(this is ...)`), documentation symbol,
    ///    prototype text, and (for forms 1 and 2) the `explicit_name_for_inverse`
    fn identify(_self: &ImpDefFamily) {
        // No-op: preamble matching and to_family_data allocation deferred.
    }

    /// Take a closer look at the wording after identification.
    ///
    /// Corresponds to `ToPhraseFamily::assess` (the `ASSESS_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 134-148).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. If the preamble has a constant name, attach the `constant_phrase`
    ///    and reject indefinite kinds (`PM_NamedGeneric`, lines 160-178)
    /// 2. Detect `<now-phrase-preamble>` (`to now ...`) and reject redefinitions
    ///    (`PM_RedefinedNow`, lines 140-146)
    /// 3. Detect `<begin-phrase-preamble>` (`to begin`) and set `tfd->to_begin`
    fn assess(_self: &ImpDefFamily) {
        // No-op: wording assessment and problem reporting deferred.
    }

    /// Enter To phrases into the excerpt parser in logical order.
    ///
    /// Corresponds to `ToPhraseFamily::register` (the `REGISTER_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 285-293).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Walk the `first_in_logical_order` linked list
    /// 2. Call `ParseInvocations::register_excerpt(id->body_of_defn)` for each
    /// 3. Assign each a `sequence_count` from 0 upward
    fn register(_self: &ImpDefFamily) {
        // No-op: excerpt-parser registration deferred.
    }

    /// Insert the body into the logical-order linked list.
    ///
    /// Corresponds to `ToPhraseFamily::given_body` (the
    /// `GIVEN_BODY_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 193-219).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Call `CompileImperativeDefn::prepare_for_requests(body)`
    /// 2. Parse the prototype text via `ParsingIDTypeData::parse`
    /// 3. Insert `id` into the `first_in_logical_order` list at the position
    ///    determined by `ToPhraseFamily::cmp` (lines 245-278)
    fn given_body(_self: &ImpDefFamily) {
        // No-op: body preparation and logical-order insertion deferred.
    }

    /// Whether the body may be given as `(- ... -)` inline I6 material.
    ///
    /// Corresponds to `ToPhraseFamily::allows_inline` (the
    /// `ALLOWS_INLINE_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 388-390).
    ///
    /// To phrases always allow inline I6 bodies.
    fn allows_inline(_self: &ImpDefFamily) -> bool {
        true
    }

    /// Make compilation requests for the To phrase bodies.
    ///
    /// Corresponds to `ToPhraseFamily::compile` (the `COMPILE_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 299-304).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Mark To phrases with definite kinds for future compilation via
    ///    `PhraseRequests::simple_request` (lines 306-313)
    /// 2. Throw `PM_ReturnKindVague` / `PM_ReturnKindUndetermined` problems for
    ///    phrases with return kinds too vaguely defined (lines 315-349)
    /// 3. Throw `PM_NamedInline` problems for inline phrases named as
    ///    constants (lines 351-365)
    fn compile(_self: &ImpDefFamily, _total: &mut i32, _target: i32) {
        // No-op: compilation requests and problem reporting deferred.
    }

    /// Whether definitions in this family appear in the Phrasebook index.
    ///
    /// Corresponds to `ToPhraseFamily::include_in_Phrasebook_index` (the
    /// `PHRASEBOOK_INDEX_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 392-395).
    ///
    /// To phrases always appear in the Phrasebook index.
    fn phrasebook_index(_self: &ImpDefFamily) -> bool {
        true
    }
}
```

### Task 2: Wire the module into `ImperativeDefinitionFamilies::create()`

Edit `crates/conform7-semantics/src/assertions/imperative_definition_families.rs`.

**2a. Add the import at the top of the file (alongside the existing `AdjectivalDefinitionFamily` import, line 30):**

```rust
use crate::assertions::to_phrase_family::ToPhraseFamily;
```

**2b. Modify the `BUILTIN_IMP_DEFN_FAMILIES` LazyLock initialization to wire the To phrase family methods:**

Replace the current LazyLock body (lines 235-246) with:

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
    families
});
```

**2c. Update the doc comment on `BUILTIN_IMP_DEFN_FAMILIES` to mention the To phrase wiring:**

Replace the trailing doc-comment lines (lines 233-234) with:

```rust
/// Methods for the `adjectival-idf` family are wired during initialization
/// by `AdjectivalDefinitionFamily::wire_methods` (PLAN-46).
///
/// Methods for the `TO_PHRASE_EFF` family are wired during initialization
/// by `ToPhraseFamily::wire_methods` (PLAN-47).
```

**2d. Update the module-level doc comment to reference the new module:**

In the `## Module Map` table (around line 20), add a row:

```rust
//! | [`to_phrase_family`] | `Chapter 5/To Phrase Family.w` | To phrase definition family |
```

And in the `## References` list (around line 24), the C reference
`inform7/assertions-module/Chapter 5/To Phrase Family.w` is already present
(added in an earlier plan), so no new reference line is needed.

### Task 3: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`.

**3a. Add the module declaration (after `pub mod adjectival_definition_family;`, line 42):**

```rust
pub mod to_phrase_family;
```

**3b. Add a module-map row in the `# Module Map` table (after the adjectival row, line 20):**

```
| [`to_phrase_family`] | `Chapter 5/To Phrase Family.w` | To phrase definition family |
```

**3c. The C reference `inform7/assertions-module/Chapter 5/To Phrase Family.w` is already present** in the `# References` list (line 31), so no new reference line is needed.

**3d. Update the startup-sequence comment (lines 9-13) to mention PLAN-47:**

```
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module) — which wires
//! `AdjectivalDefinitionFamily` methods (PLAN-46) and `ToPhraseFamily` methods
//! (PLAN-47), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
```

### Task 4: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/to_phrase_family.rs`.

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
    ToPhraseFamily::wire_methods(&mut family);

    assert!(family.methods.identify.is_some(), "identify should be installed");
    assert!(family.methods.assess.is_some(), "assess should be installed");
    assert!(family.methods.register.is_some(), "register should be installed");
    assert!(family.methods.given_body.is_some(), "given_body should be installed");
    assert!(family.methods.allows_inline.is_some(), "allows_inline should be installed");
    assert!(family.methods.compile.is_some(), "compile should be installed");
    assert!(family.methods.phrasebook_index.is_some(), "phrasebook_index should be installed");
}
```

**Test 2: `allows_inline_returns_true`**

```rust
/// Test that `allows_inline` returns `true`.
#[test]
fn allows_inline_returns_true() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    let result = ImperativeDefinitionFamilies::allows_inline(&family);
    assert!(result, "allows_inline should return true for To phrases");
}
```

**Test 3: `phrasebook_index_returns_true`**

```rust
/// Test that `phrasebook_index` returns `true`.
#[test]
fn phrasebook_index_returns_true() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    let result = ImperativeDefinitionFamilies::phrasebook_index(&family);
    assert!(result, "phrasebook_index should return true for To phrases");
}
```

**Test 4: `identify_does_not_panic`**

```rust
/// Test that `identify` does not panic when dispatched.
#[test]
fn identify_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::identify(&family);
}
```

**Test 5: `assess_does_not_panic`**

```rust
/// Test that `assess` does not panic when dispatched.
#[test]
fn assess_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::assess(&family);
}
```

**Test 6: `register_does_not_panic`**

```rust
/// Test that `register` does not panic when dispatched.
#[test]
fn register_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::register(&family);
}
```

**Test 7: `given_body_does_not_panic`**

```rust
/// Test that `given_body` does not panic when dispatched.
#[test]
fn given_body_does_not_panic() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    // Should not panic
    ImperativeDefinitionFamilies::given_body(&family);
}
```

**Test 8: `compile_does_not_modify_total`**

```rust
/// Test that `compile` does not modify `total` (no-op stub).
#[test]
fn compile_does_not_modify_total() {
    let mut family = ImpDefFamily::new("test", false);
    ToPhraseFamily::wire_methods(&mut family);

    let mut total = 42;
    ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
    assert_eq!(total, 42, "compile should not modify total (no-op stub)");
}
```

**Test 9: `builtin_family_has_methods_wired`**

```rust
/// Test that the builtin `TO_PHRASE_EFF` family has all seven methods wired.
#[test]
fn builtin_family_has_methods_wired() {
    // Force initialization of the builtin families
    ImperativeDefinitionFamilies::create();
    let family = &BUILTIN_IMP_DEFN_FAMILIES[2];

    assert_eq!(family.name, "TO_PHRASE_EFF");
    assert!(family.methods.identify.is_some(), "TO_PHRASE_EFF should have identify installed");
    assert!(family.methods.assess.is_some(), "TO_PHRASE_EFF should have assess installed");
    assert!(family.methods.register.is_some(), "TO_PHRASE_EFF should have register installed");
    assert!(family.methods.given_body.is_some(), "TO_PHRASE_EFF should have given_body installed");
    assert!(family.methods.allows_inline.is_some(), "TO_PHRASE_EFF should have allows_inline installed");
    assert!(family.methods.compile.is_some(), "TO_PHRASE_EFF should have compile installed");
    assert!(family.methods.phrasebook_index.is_some(), "TO_PHRASE_EFF should have phrasebook_index installed");
}
```

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test -- assertions::to_phrase_family` — new tests pass
- [ ] `cargo test` — all 1407 existing tests still pass
- [ ] `cargo clippy --all-targets` — no new warnings (pre-existing warnings in unrelated files are acceptable)

## Success Criteria

- [ ] `assertions::to_phrase_family` module exists and compiles.
- [ ] `ToPhraseFamily::wire_methods` installs `identify`, `assess`, `register`, `given_body`, `allows_inline`, `compile`, and `phrasebook_index` on a family.
- [ ] `allows_inline` returns `true` for the To phrase family.
- [ ] `phrasebook_index` returns `true` for the To phrase family.
- [ ] `identify`, `assess`, `register`, `given_body`, and `compile` are no-op stubs that do not panic.
- [ ] `compile` does not modify the `total` parameter.
- [ ] The `TO_PHRASE_EFF` built-in family has all seven methods wired after `ImperativeDefinitionFamilies::create()`.
- [ ] `cargo clippy --all-targets` introduces no new warnings.
- [ ] All existing tests (1407) still pass.

## Out of Scope

- **Full `identify` (claim) logic.** `<to-phrase-preamble>` grammar matching (lines 70-76), `to_family_data` allocation (lines 38-49, 51-63), constant-name parsing (`(this is ...)`, lines 99-114), documentation-symbol extraction, and `explicit_name_for_inverse` handling are deferred.
- **Full `assess` logic.** `PM_NamedGeneric` rejection (lines 160-178), `PM_RedefinedNow` for `to now ...` (lines 140-146), and `to_begin` detection (lines 147-148) are deferred.
- **Full `given_body` logic.** `CompileImperativeDefn::prepare_for_requests`, `ParsingIDTypeData::parse`, the `first_in_logical_order` linked list (lines 191-218), `get_next`/`set_next` (lines 221-231), and `cmp` (lines 245-278) are deferred.
- **Full `register` logic.** The `first_in_logical_order` walk and `ParseInvocations::register_excerpt` call (lines 285-293) are deferred.
- **Full `compile` logic.** `PhraseRequests::simple_request` (lines 306-313), `PM_ReturnKindVague`/`PM_ReturnKindUndetermined`/`PM_NamedInline` problem reporting (lines 315-365) are deferred.
- **`to_family_data` and `constant_phrase` types.** These require `wording`, `imperative_defn`, `id_body`, and `noun` types, deferred to a later plan.
- **`imperative_defn` and `id_body` types.** These will be introduced in a later plan; method signatures remain simplified.
- **`parse_node` handling.** Source-location and wording manipulation is deferred.
- **Preform grammar.** The `<to-phrase-preamble>`, `<now-phrase-preamble>`, and `<begin-phrase-preamble>` grammars are deferred.
- **Problem messages.** No `StandardProblems` calls for malformed To phrases.
- **Other imperative definition families.** `RuleFamily` method wiring is deferred to a later plan.