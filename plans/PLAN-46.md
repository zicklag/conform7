# Plan 46: Adjectival Definition Family — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Wire `identify`, `given_body`, `allows_empty`, and `compile` methods into the `adjectival-idf` family created by `BUILTIN_IMP_DEFN_FAMILIES` (PLAN-41, Complete). This is the first concrete imperative definition family in the assertions-module startup sequence, corresponding to `AdjectivalDefinitionFamily::create_family()` in the C reference.

The methods are installed as simplified stubs — they exist and dispatch correctly, but their full logic (parse-node manipulation, `imperative_defn` iteration, I6 compilation) is deferred until the `imperative_defn`, `id_body`, and `parse_node` types are introduced in later plans.

## Background

### C reference architecture

The C reference (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`) defines four methods on the `adjectival_idf` family:

```c
void AdjectivalDefinitionFamily::create_family(void) {
    adjectival_idf = ImperativeDefinitionFamilies::new(I"adjectival-idf", FALSE);
    METHOD_ADD(adjectival_idf, IDENTIFY_IMP_DEFN_MTID, AdjectivalDefinitionFamily::identify);
    METHOD_ADD(adjectival_idf, GIVEN_BODY_IMP_DEFN_MTID, AdjectivalDefinitionFamily::given_body);
    METHOD_ADD(adjectival_idf, ALLOWS_EMPTY_IMP_DEFN_MTID, AdjectivalDefinitionFamily::allows_empty);
    METHOD_ADD(adjectival_idf, COMPILE_IMP_DEFN_MTID, AdjectivalDefinitionFamily::compile);
}
```

**`identify`** (lines 48-59): Checks if the preamble text matches `<definition-preamble>` (the single word `definition`). If so, sets `id->family = adjectival_idf`, handles continuation-node rewriting (`IMPERATIVE_NT` -> `DEFN_CONT_NT`), and calls `AdjectivalDefinitionFamily::look_for_headers` which parses the adjective definition and registers it via `AdjectiveMeanings::claim_definition`.

**`allows_empty`** (lines 65-67): Returns `TRUE` — adjectival definitions may have empty bodies (the code may be under a continuation node).

**`given_body`** (lines 76-86): Sets the body's type data to `DECIDES_CONDITION_MOR`, calls `AdjectivesByPhrase::define_adjective_by_phrase` to parse the definition body, and enables the `it` pronoun via `Frames::enable_it`.

**`compile`** (lines 91-101): Loops over all `imperative_defn` objects belonging to `adjectival_idf`, calls `RTAdjectives::make_adjective_phrase_package` and `CompileImperativeDefn::not_from_phrase` for each, then calls `RTAdjectives::compile()`.

### Current Rust state

- `crates/conform7-semantics/src/assertions/imperative_definition_families.rs` defines:
  - `ImpDefFamilyMethods` with method slots: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index`
  - `ImpDefFamily` with `name`, `methods`, `compile_last`
  - `BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>>` with four families: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`
  - All method slots default to `None`
  - Dispatch helpers: `ImperativeDefinitionFamilies::identify`, `allows_empty`, `given_body`, `compile`, etc.
- `adjectival_idf()` accessor returns `&BUILTIN_IMP_DEFN_FAMILIES[1]`
- 1401 tests pass, clippy clean

### Current method signatures (simplified)

```rust
pub identify: Option<fn(&ImpDefFamily)>,
pub given_body: Option<fn(&ImpDefFamily)>,
pub allows_empty: Option<fn(&ImpDefFamily) -> bool>,
pub compile: Option<fn(&ImpDefFamily, &mut i32, i32)>,
```

These are simplified from the C — they take `&ImpDefFamily` only, without `imperative_defn *id`, `id_body *body`, or `parse_node *at` parameters. The full signatures will be introduced when those types exist.

## Decision

### 1. Is the Adjectival Definition Family the correct next step?

**Yes.** In the C assertions-module startup sequence, `AdjectivalDefinitionFamily::create_family()` is called from `ImperativeDefinitionFamilies::create()` immediately after creating `unknown-idf`. The Rust `BUILTIN_IMP_DEFN_FAMILIES` already creates `adjectival-idf` with default methods; PLAN-46 wires the four methods into it.

### 2. Is it independently testable?

**Yes.** The foundation consists of:
- Creating the `adjectival_definition_family` module with four stub method functions
- Wiring them into the `adjectival-idf` family during `ImperativeDefinitionFamilies::create()`
- Verifying that dispatch helpers call the installed methods and return correct defaults

### 3. What is the smallest independently testable subset?

1. `AdjectivalDefinitionFamily::wire_methods(family: &mut ImpDefFamily)` installs the four methods.
2. `identify` can be dispatched without panic (no-op stub).
3. `allows_empty` returns `true` (the only method with real behavior at this stage).
4. `given_body` can be dispatched without panic (no-op stub).
5. `compile` can be dispatched without modifying `total` (no-op stub).
6. The `adjectival_idf()` accessor still returns the correct family.
7. All existing tests continue to pass.

### 4. What simplifications are appropriate?

- **No `imperative_defn` or `id_body` types.** Method signatures remain `fn(&ImpDefFamily)` etc. The full `identify` logic (preamble matching, continuation-node rewriting, `look_for_headers`) is deferred.
- **No `AdjectivesByPhrase::define_adjective_by_phrase` call in `given_body`.** The body setup (type data, pronoun enabling) is deferred.
- **No `RTAdjectives` or `CompileImperativeDefn` calls in `compile`.** The definition loop and I6 compilation are deferred.
- **No `parse_node` handling.** Source-location manipulation is deferred.
- **No Preform grammar.** The `<definition-preamble>` grammar match is deferred.
- **No problem messages.** Error reporting is deferred.

## Tasks

### Task 1: Create the `AdjectivalDefinitionFamily` module

Create `crates/conform7-semantics/src/assertions/adjectival_definition_family.rs`.

Module-level doc comment:

```rust
//! Adjectival Definition Family — imperative definitions of "Definition: X is Y: ..." adjectives.
//!
//! Corresponds to `AdjectivalDefinitionFamily` in the C reference
//! (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`).
//!
//! This family is used for adjective definitions, whether or not they run on
//! into substantial amounts of code. It wires four methods into the
//! `adjectival-idf` imperative definition family:
//!
//! - `identify` — decide whether a definition preamble belongs to this family
//! - `given_body` — set up the body of a definition
//! - `allows_empty` — whether the body is allowed to be empty
//! - `compile` — compile the definition body
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types — method signatures take `&ImpDefFamily` only.
//! - No `AdjectivesByPhrase::define_adjective_by_phrase` call in `given_body`.
//! - No `RTAdjectives` or `CompileImperativeDefn` calls in `compile`.
//! - No `parse_node` handling or Preform grammar.
//! - No problem messages.
```

Imports:

```rust
use crate::assertions::imperative_definition_families::ImpDefFamily;
```

The module struct:

```rust
/// The Adjectival Definition Family module.
///
/// Corresponds to `AdjectivalDefinitionFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`).
pub struct AdjectivalDefinitionFamily;
```

Implementation:

```rust
impl AdjectivalDefinitionFamily {
    /// Wire the four adjectival-family methods into the given family.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::create_family` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 11-17).
    ///
    /// Simplified: all methods are stubs. The full logic (preamble matching,
    /// body setup, definition iteration, I6 compilation) is deferred until
    /// `imperative_defn`, `id_body`, and `parse_node` types are introduced.
    pub fn wire_methods(family: &mut ImpDefFamily) {
        family.methods.identify = Some(Self::identify);
        family.methods.given_body = Some(Self::given_body);
        family.methods.allows_empty = Some(Self::allows_empty);
        family.methods.compile = Some(Self::compile);
    }

    /// Decide whether a definition preamble belongs to the adjectival family.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::identify` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 48-59).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Check if the preamble text matches `<definition-preamble>` ("definition")
    /// 2. Set `id->family = adjectival_idf`
    /// 3. Handle continuation-node rewriting (`IMPERATIVE_NT` -> `DEFN_CONT_NT`)
    /// 4. Call `look_for_headers` to parse and register the adjective definition
    fn identify(_self: &ImpDefFamily) {
        // No-op: full preamble matching and node manipulation deferred.
    }

    /// Set up the body of an adjectival definition.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::given_body` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 76-86).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Set `body->type_data` to `DECIDES_CONDITION_MOR`
    /// 2. Call `AdjectivesByPhrase::define_adjective_by_phrase` to parse the body
    /// 3. Enable the `it` pronoun via `Frames::enable_it`
    fn given_body(_self: &ImpDefFamily) {
        // No-op: body setup and phrase parsing deferred.
    }

    /// Whether the body is allowed to be empty for adjectival definitions.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::allows_empty` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 65-67).
    ///
    /// Adjectival definitions always allow empty bodies because the code may
    /// be under a continuation node (`DEFN_CONT_NT`) rather than directly
    /// under the `Definition:` node.
    fn allows_empty(_self: &ImpDefFamily) -> bool {
        true
    }

    /// Compile the adjectival definition bodies.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::compile` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 91-101).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Loop over all `imperative_defn` objects belonging to `adjectival_idf`
    /// 2. Call `RTAdjectives::make_adjective_phrase_package` for each
    /// 3. Call `CompileImperativeDefn::not_from_phrase` for each
    /// 4. Call `RTAdjectives::compile()` at the end
    fn compile(_self: &ImpDefFamily, _total: &mut i32, _target: i32) {
        // No-op: definition iteration and I6 compilation deferred.
    }
}
```

### Task 2: Wire the module into `ImperativeDefinitionFamilies::create()`

Edit `crates/conform7-semantics/src/assertions/imperative_definition_families.rs`.

**2a. Add the module declaration at the top of the file (after the doc comment, before `use` statements):**

```rust
pub mod adjectival_definition_family;
```

**2b. Add the import at the top of the file:**

```rust
use adjectival_definition_family::AdjectivalDefinitionFamily;
```

**2c. Modify the `BUILTIN_IMP_DEFN_FAMILIES` LazyLock initialization to wire the adjectival family methods:**

Replace the current LazyLock body with:

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
    families
});
```

**2d. Update the doc comment on `BUILTIN_IMP_DEFN_FAMILIES` to mention the wiring:**

```rust
/// The built-in imperative definition families registry.
///
/// Created in the C-mandated order:
/// 1. `unknown-idf` — placeholder for unclassified definitions
/// 2. `adjectival-idf` — adjectival definitions (`Definition: ...`)
/// 3. `TO_PHRASE_EFF` — To phrase definitions (`To ...`)
/// 4. `rule-idf` — rule definitions (`Every turn: ...`, `Instead of ...`)
///
/// The order matters: `rule-idf` must come last because
/// `ImperativeDefinitionFamilies::identify` iterates the list in creation
/// order and the rule family claims anything not already claimed.
///
/// Methods for the `adjectival-idf` family are wired during initialization
/// by `AdjectivalDefinitionFamily::wire_methods` (PLAN-46).
```

**2e. Update the module-level doc comment to reference the new module:**

In the doc comment at the top of the file, add a module-map row:

```rust
//! | [`adjectival_definition_family`] | `Chapter 5/Adjectival Definition Family.w` | Adjectival definition family |
```

And add the C reference:

```rust
//! - C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
```

### Task 3: Wire the module into the assertions module

Edit `crates/conform7-semantics/src/assertions/mod.rs`.

**3a. Add the module declaration:**

```rust
pub mod adjectival_definition_family;
```

**3b. Add a module-map row:**

```
| [`adjectival_definition_family`] | `Chapter 5/Adjectival Definition Family.w` | Adjectival definition family |
```

**3c. Add the C reference to the References list:**

```
- C reference: `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
```

**3d. Update the startup-sequence comment to mention PLAN-46:**

```
//! `KindPredicatesRevisited::start()` (PLAN-40), then
//! `ImperativeDefinitionFamilies::start()` (this module) — which wires
//! `AdjectivalDefinitionFamily` methods (PLAN-46), then
//! `AdjectivesByPhrase::start()` (PLAN-42), then `AdjectivesByCondition::start()` (PLAN-43),
//! then `AdjectivesByInterFunction::start()` (PLAN-44),
//! then `AdjectivesByInterCondition::start()` (PLAN-45), etc.
```

### Task 4: Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `crates/conform7-semantics/src/assertions/adjectival_definition_family.rs`.

Required imports in tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        ImperativeDefinitionFamilies, ImpDefFamily,
    };
}
```

**Test 1: `wire_methods_installs_all_four_methods`**

```rust
#[test]
fn wire_methods_installs_all_four_methods() {
    let mut family = ImpDefFamily::new("adjectival-idf", false);
    assert!(family.methods.identify.is_none());
    assert!(family.methods.given_body.is_none());
    assert!(family.methods.allows_empty.is_none());
    assert!(family.methods.compile.is_none());

    AdjectivalDefinitionFamily::wire_methods(&mut family);

    assert!(family.methods.identify.is_some());
    assert!(family.methods.given_body.is_some());
    assert!(family.methods.allows_empty.is_some());
    assert!(family.methods.compile.is_some());
}
```

**Test 2: `allows_empty_returns_true`**

```rust
#[test]
fn allows_empty_returns_true() {
    let mut family = ImpDefFamily::new("adjectival-idf", false);
    AdjectivalDefinitionFamily::wire_methods(&mut family);

    let result = ImperativeDefinitionFamilies::allows_empty(&family);
    assert!(result, "adjectival definitions should allow empty bodies");
}
```

**Test 3: `identify_does_not_panic`**

```rust
#[test]
fn identify_does_not_panic() {
    let mut family = ImpDefFamily::new("adjectival-idf", false);
    AdjectivalDefinitionFamily::wire_methods(&mut family);

    // Should not panic — no-op stub
    ImperativeDefinitionFamilies::identify(&family);
}
```

**Test 4: `given_body_does_not_panic`**

```rust
#[test]
fn given_body_does_not_panic() {
    let mut family = ImpDefFamily::new("adjectival-idf", false);
    AdjectivalDefinitionFamily::wire_methods(&mut family);

    // Should not panic — no-op stub
    ImperativeDefinitionFamilies::given_body(&family);
}
```

**Test 5: `compile_does_not_modify_total`**

```rust
#[test]
fn compile_does_not_modify_total() {
    let mut family = ImpDefFamily::new("adjectival-idf", false);
    AdjectivalDefinitionFamily::wire_methods(&mut family);

    let mut total = 42;
    ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
    assert_eq!(total, 42, "compile stub should not modify total");
}
```

**Test 6: `builtin_family_has_methods_wired`**

```rust
#[test]
fn builtin_family_has_methods_wired() {
    // Force initialization
    ImperativeDefinitionFamilies::create();

    let family = crate::assertions::imperative_definition_families::adjectival_idf();
    assert_eq!(family.name, "adjectival-idf");
    assert!(family.methods.identify.is_some(), "identify should be wired");
    assert!(family.methods.given_body.is_some(), "given_body should be wired");
    assert!(family.methods.allows_empty.is_some(), "allows_empty should be wired");
    assert!(family.methods.compile.is_some(), "compile should be wired");
}
```

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test -- assertions::adjectival_definition_family` — new tests pass
- [ ] `cargo test` — all 1401 existing tests still pass
- [ ] `cargo clippy --all-targets` — clean

## Success Criteria

- [ ] `assertions::adjectival_definition_family` module exists and compiles.
- [ ] `AdjectivalDefinitionFamily::wire_methods` installs `identify`, `given_body`, `allows_empty`, and `compile` on a family.
- [ ] `allows_empty` returns `true` for the adjectival family.
- [ ] `identify`, `given_body`, and `compile` are no-op stubs that do not panic.
- [ ] `compile` does not modify the `total` parameter.
- [ ] The `adjectival-idf` built-in family has all four methods wired after `ImperativeDefinitionFamilies::create()`.
- [ ] `cargo clippy --all-targets` is clean.
- [ ] All existing tests (1401) still pass.

## Out of Scope

- **Full `identify` logic.** Preamble matching (`<definition-preamble>`), continuation-node rewriting (`IMPERATIVE_NT` -> `DEFN_CONT_NT`), and `look_for_headers` parsing are deferred.
- **Full `given_body` logic.** Body type data setup (`DECIDES_CONDITION_MOR`), `AdjectivesByPhrase::define_adjective_by_phrase` call, and `Frames::enable_it` pronoun setup are deferred.
- **Full `compile` logic.** Definition iteration, `RTAdjectives::make_adjective_phrase_package`, `CompileImperativeDefn::not_from_phrase`, and `RTAdjectives::compile()` are deferred.
- **`imperative_defn` and `id_body` types.** These will be introduced in a later plan; method signatures remain simplified.
- **`parse_node` handling.** Source-location manipulation is deferred.
- **Preform grammar.** The `<definition-preamble>` grammar match is deferred.
- **Problem messages.** No `StandardProblems` calls for malformed definitions.
- **Other imperative definition families.** `ToPhraseFamily` and `RuleFamily` method wiring is deferred.
