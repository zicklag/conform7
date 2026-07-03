# Plan 41: Imperative Definition Families — Foundation

**Status**: Complete
**Target**: 1 day

## Goal

Implement the foundation of the Imperative Definition Families system — the `imperative_defn_family` data structure, the `ImperativeDefinitionFamilies` module with `create()` and `new()`, and the method dispatch infrastructure. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 33), immediately after `KindPredicatesRevisited::start()` (PLAN-40, Complete).

This requires:
1. Creating a new top-level `assertions` module in `conform7-semantics`.
2. Defining `ImpDefFamilyMethods` with optional method slots mirroring the C method IDs.
3. Defining `ImpDefFamily` with name, methods, and `compile_last` flag.
4. Implementing `ImperativeDefinitionFamilies::new_family()` and `ImperativeDefinitionFamilies::create()`.
5. Wiring the built-in family registry into a `LazyLock` and providing a `start()` hook for the assertions-module startup sequence.

## Background

### C reference architecture

The Imperative Definition Families system is the dispatch layer for different categories of imperative definition in Inform 7: adjectives (`Definition: ...`), To phrases (`To ...`), and rules (`Every turn: ...`, `Instead of ...`). It lives in the assertions module, Chapter 5.

Key C sources:

- `inform7/assertions-module/Chapter 5/Imperative Definition Families.w` — `ImperativeDefinitionFamilies::create`, `new`, and method dispatch
- `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w` — concrete adjectival family
- `inform7/assertions-module/Chapter 5/To Phrase Family.w` — concrete To phrase family
- `inform7/assertions-module/Chapter 5/Rule Family.w` — concrete rule family

At startup the assertions module calls:

```c
void AssertionsModule::start(void) {
    ...
    KindPredicatesRevisited::start();
    ImperativeDefinitionFamilies::create();  // <-- this plan
    AdjectivesByPhrase::start();
    ...
}
```

The family registry is created in fixed order:

```c
imperative_defn_family *unknown_idf = NULL; /* used only temporarily */

void ImperativeDefinitionFamilies::create(void) {
    unknown_idf = ImperativeDefinitionFamilies::new(I"unknown-idf", FALSE);
    AdjectivalDefinitionFamily::create_family();
    ToPhraseFamily::create_family();
    RuleFamily::create_family();
}

typedef struct imperative_defn_family {
    struct text_stream *family_name;
    struct method_set *methods;
    int compile_last;
    CLASS_DEFINITION
} imperative_defn_family;

imperative_defn_family *ImperativeDefinitionFamilies::new(text_stream *name, int last) {
    imperative_defn_family *family = CREATE(imperative_defn_family);
    family->family_name = Str::duplicate(name);
    family->methods = Methods::new_set();
    family->compile_last = last;
    return family;
}
```

The order matters: `RuleFamily` must come last because `ImperativeDefinitionFamilies::identify` iterates the global `CLASS_DEFINITION` list in creation order and the rule family claims anything not already claimed (its grammar ends with a catch-all `...` production).

The family method set provides slots for these method IDs (all void unless noted):

| Method ID | Purpose | Return |
|-----------|---------|--------|
| `IDENTIFY_IMP_DEFN_MTID` | Decide from a preamble whether a definition belongs to the family | void |
| `ASSESS_IMP_DEFN_MTID` | Parse the preamble in more detail | void |
| `GIVEN_BODY_IMP_DEFN_MTID` | Called just after `id->body_of_defn` is created | void |
| `REGISTER_IMP_DEFN_MTID` | Called after all assessments/bodies are done | void |
| `TO_RCD_IMP_DEFN_MTID` | Provide runtime context data for the body | void |
| `ASSESSMENT_COMPLETE_IMP_DEFN_MTID` | Called after the whole assessment pass | void |
| `ALLOWS_RULE_ONLY_PHRASES_IMP_DEFN_MTID` | Whether rule-only phrases may appear in the body | int |
| `ALLOWS_EMPTY_IMP_DEFN_MTID` | Whether the body may be empty | int |
| `ALLOWS_INLINE_IMP_DEFN_MTID` | Whether the body may be `(- ... -)` inline I6 | int |
| `COMPILE_IMP_DEFN_MTID` | Main compilation round for this family | void |
| `PHRASEBOOK_INDEX_IMP_DEFN_MTID` | Whether definitions in this family appear in the Phrasebook index | int |

### Current Rust state

- `crates/conform7-semantics/src/calculus/unary_predicate_families.rs` — `UpFamily` and `UpFamilyMethods` pattern with optional `Option<fn(...)>` method slots and a manual/default `Default` impl.
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily`, `BpFamilyMethods`, and the `BinaryPredicateFamilies` manager. Provides `new_family()` and dispatch helpers (`typecheck`, `assert`, etc.).
- `crates/conform7-semantics/src/calculus/kind_predicates_revisited.rs` — assertions-module hook wiring methods into a family (PLAN-40, Complete).
- `crates/conform7-semantics/src/calculus/mod.rs` — module map and references list pattern.
- `crates/conform7-semantics/src/lib.rs` — exposes `calculus` and `knowledge`; no `assertions` module yet.
- There is no `imperative_defn`, `id_body`, or `id_runtime_context_data` type yet.

### What's needed

1. **Create the `assertions` module.** Add `crates/conform7-semantics/src/assertions/mod.rs` with a module map and references, and expose it from `lib.rs`.

2. **Create `imperative_definition_families.rs`.** Define:
   - `ImpDefFamilyMethods` with optional method slots for all C method IDs. Because `imperative_defn` and `id_body` do not yet exist, the signatures are simplified to take `&ImpDefFamily` only (and `&mut i32, i32` for `compile`). All slots default to `None`.
   - `ImpDefFamily` with `name`, `methods`, and `compile_last: bool`.
   - `ImpDefFamily::new(name, compile_last)`.
   - `ImperativeDefinitionFamilies::new_family(name, compile_last)` — thin wrapper around `ImpDefFamily::new`.
   - `ImperativeDefinitionFamilies::create()` — initializes the built-in family registry in the C-mandated order.

3. **Wire the built-in registry.** Use a `LazyLock<Vec<ImpDefFamily>>` for the four built-in families. Provide `ImperativeDefinitionFamilies::start()` to force initialization, matching the assertions-module startup hook.

4. **Add dispatch helpers and accessors.** Provide helper functions that call a family's installed method or return a sensible default, plus named accessors for the four built-in families.

5. **Integrate and test.** Add module declarations, references, and unit tests.

### Method signatures

The `ImpDefFamilyMethods` slots should be:

```rust
#[derive(Clone, Debug, Default)]
pub struct ImpDefFamilyMethods {
    /// Decide whether a definition preamble belongs to this family.
    pub identify: Option<fn(&ImpDefFamily) -> ()>,
    /// Parse the preamble in more detail.
    pub assess: Option<fn(&ImpDefFamily) -> ()>,
    /// Called after the definition body has been created.
    pub given_body: Option<fn(&ImpDefFamily) -> ()>,
    /// Called after all assessments/bodies are registered.
    pub register: Option<fn(&ImpDefFamily) -> ()>,
    /// Provide runtime context data for the body.
    pub to_rcd: Option<fn(&ImpDefFamily) -> ()>,
    /// Called when assessment is complete for all definitions.
    pub assessment_complete: Option<fn(&ImpDefFamily) -> ()>,
    /// Whether phrases that end rules/rulebooks may be used in the body.
    pub allows_rule_only_phrases: Option<fn(&ImpDefFamily) -> bool>,
    /// Whether the body is allowed to be empty.
    pub allows_empty: Option<fn(&ImpDefFamily) -> bool>,
    /// Whether the body can be given as (- ... -) inline I6 material.
    pub allows_inline: Option<fn(&ImpDefFamily) -> bool>,
    /// Main compilation round for resources needed by the family.
    pub compile: Option<fn(&ImpDefFamily, &mut i32, i32) -> ()>,
    /// Whether definitions in this family should appear in the Phrasebook index.
    pub phrasebook_index: Option<fn(&ImpDefFamily) -> bool>,
}
```

The signatures are simplified because `imperative_defn` and `id_body` are out of scope. They will be expanded to include `&ImperativeDefn`, `&mut IdBody`, `&mut RuntimeContextData`, etc., when those types are introduced.

## Tasks

### 1. Create the `assertions` module

Create `crates/conform7-semantics/src/assertions/mod.rs`:

- [ ] Module-level doc comment (`//!`) describing the assertions module, its place in the startup sequence, and the C reference `inform7/assertions-module/Chapter 1/Assertions Module.w`.
- [ ] Module map table:

```markdown
| Module | C Reference | Purpose |
|--------|-------------|---------|
| [`imperative_definition_families`] | `Chapter 5/Imperative Definition Families.w` | Family dispatch for imperative definitions |
```

- [ ] References list with the four C reference `.w` files.
- [ ] `pub mod imperative_definition_families;`

In `crates/conform7-semantics/src/lib.rs`:

- [ ] Add `pub mod assertions;` after `pub mod knowledge;`.
- [ ] Update the crate-level module map comment to mention `assertions`.

### 2. Create `imperative_definition_families.rs`

Create `crates/conform7-semantics/src/assertions/imperative_definition_families.rs`:

- [ ] Module-level doc comment referencing `Imperative Definition Families.w` and the simplifications (no `imperative_defn` yet, method slots simplified).
- [ ] Import `std::sync::LazyLock`.
- [ ] `#[derive(Clone, Debug, Default)] pub struct ImpDefFamilyMethods { ... }` with all optional slots from the signatures above.
- [ ] `#[derive(Clone, Debug)] pub struct ImpDefFamily { pub name: &'static str, pub methods: ImpDefFamilyMethods, pub compile_last: bool }`.
- [ ] `impl ImpDefFamily { pub fn new(name: &'static str, compile_last: bool) -> Self { ... } }`
- [ ] `pub struct ImperativeDefinitionFamilies;`
- [ ] `impl ImperativeDefinitionFamilies { pub fn new_family(name: &'static str, compile_last: bool) -> ImpDefFamily { ... } }`
- [ ] Static built-in registry:

```rust
pub static BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>> = LazyLock::new(|| {
    vec![
        ImpDefFamily::new("unknown-idf", false),
        ImpDefFamily::new("adjectival-idf", false),
        ImpDefFamily::new("TO_PHRASE_EFF", true),
        ImpDefFamily::new("rule-idf", false),
    ]
});
```

- [ ] Accessor functions for the built-in families. Each forces the registry and returns a `'static` reference:

```rust
pub fn unknown_idf() -> &'static ImpDefFamily { ... }
pub fn adjectival_idf() -> &'static ImpDefFamily { ... }
pub fn to_phrase_idf() -> &'static ImpDefFamily { ... }
pub fn rule_idf() -> &'static ImpDefFamily { ... }
```

- [ ] `pub fn create()` that forces the registry:

```rust
pub fn create() {
    LazyLock::<Vec<ImpDefFamily>>::force(&BUILTIN_IMP_DEFN_FAMILIES);
}
```

- [ ] `pub fn start()` as the assertions-module startup hook. It should be equivalent to `create()`:

```rust
pub fn start() {
    Self::create();
}
```

### 3. Add method dispatch helpers

In `imperative_definition_families.rs`, add minimal dispatch helpers as inherent functions on `ImperativeDefinitionFamilies`. They call the installed method if present and otherwise return a safe default:

```rust
impl ImperativeDefinitionFamilies {
    pub fn identify(family: &ImpDefFamily) { family.methods.identify.map(|f| f(family)); }
    pub fn assess(family: &ImpDefFamily) { family.methods.assess.map(|f| f(family)); }
    pub fn given_body(family: &ImpDefFamily) { family.methods.given_body.map(|f| f(family)); }
    pub fn register(family: &ImpDefFamily) { family.methods.register.map(|f| f(family)); }
    pub fn to_rcd(family: &ImpDefFamily) { family.methods.to_rcd.map(|f| f(family)); }
    pub fn assessment_complete(family: &ImpDefFamily) { family.methods.assessment_complete.map(|f| f(family)); }
    pub fn allows_rule_only_phrases(family: &ImpDefFamily) -> bool { family.methods.allows_rule_only_phrases.map(|f| f(family)).unwrap_or(false) }
    pub fn allows_empty(family: &ImpDefFamily) -> bool { family.methods.allows_empty.map(|f| f(family)).unwrap_or(false) }
    pub fn allows_inline(family: &ImpDefFamily) -> bool { family.methods.allows_inline.map(|f| f(family)).unwrap_or(false) }
    pub fn compile(family: &ImpDefFamily, total: &mut i32, target: i32) { family.methods.compile.map(|f| f(family, total, target)); }
    pub fn phrasebook_index(family: &ImpDefFamily) -> bool { family.methods.phrasebook_index.map(|f| f(family)).unwrap_or(false) }
}
```

These helpers mirror the C dispatch functions (`ImperativeDefinitionFamilies::identify`, `assess`, etc.) and make the infrastructure independently testable.

### 4. Add unit tests

Add `#[cfg(test)] mod tests { ... }` to `imperative_definition_families.rs`:

- [ ] `new_family_creates_family_with_name_and_flag` — create a family, assert `name` and `compile_last`.
- [ ] `default_methods_are_none` — create a family and assert every `methods` slot is `None`.
- [ ] `create_initializes_builtin_families_in_order` — call `create()`, then verify `BUILTIN_IMP_DEFN_FAMILIES` has four entries with names `"unknown-idf"`, `"adjectival-idf"`, `"TO_PHRASE_EFF"`, `"rule-idf"` and flags `false, false, true, false`.
- [ ] `start_runs_without_panic` — call `start()` and verify the registry is initialized.
- [ ] `accessors_return_correct_families` — call each accessor and verify name and flag.
- [ ] `dispatch_helpers_call_installed_methods` — install a closure in a method slot (e.g., `allows_empty`), call the helper, and verify it was invoked.
- [ ] `dispatch_helpers_return_defaults_when_unset` — call helpers on a family with all methods `None` and verify `false` / no panic / no mutation.

### 5. Integrate module map and references

In `crates/conform7-semantics/src/assertions/mod.rs`:

- [ ] Add module-map row for `imperative_definition_families`.
- [ ] Add C reference lines:
  - `inform7/assertions-module/Chapter 1/Assertions Module.w`
  - `inform7/assertions-module/Chapter 5/Imperative Definition Families.w`
  - `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`
  - `inform7/assertions-module/Chapter 5/To Phrase Family.w`
  - `inform7/assertions-module/Chapter 5/Rule Family.w`

### 6. Verify

- [ ] Run `cargo build` to ensure compilation.
- [ ] Run `cargo test -- assertions::imperative_definition_families` to verify all unit tests pass.
- [ ] Run `cargo clippy --all-targets` to confirm the crate remains clean.

## Success Criteria

- [ ] `assertions` module exists and is exported from `conform7_semantics`.
- [ ] `ImpDefFamilyMethods` has optional method slots for all C method IDs in `Imperative Definition Families.w`.
- [ ] `ImpDefFamily` has public `name`, `methods`, and `compile_last` fields.
- [ ] `ImpDefFamily::new(name, compile_last)` creates a family with the given values.
- [ ] `ImperativeDefinitionFamilies::new_family(name, compile_last)` returns an equivalent family.
- [ ] `ImperativeDefinitionFamilies::create()` initializes the built-in registry without panic.
- [ ] `ImperativeDefinitionFamilies::start()` runs without panic and forces initialization.
- [ ] The built-in registry contains exactly four families in order: `unknown-idf`, `adjectival-idf`, `TO_PHRASE_EFF`, `rule-idf`.
- [ ] The built-in flags are `false, false, true, false` respectively.
- [ ] Accessor functions return the correct built-in family references.
- [ ] All method slots default to `None`.
- [ ] Dispatch helpers call installed methods when present and return safe defaults (`false`, no-op) when absent.
- [ ] The module compiles without errors.
- [ ] All unit tests pass.
- [ ] No new clippy warnings.

## Out of Scope

- **AdjectivalDefinitionFamily, ToPhraseFamily, RuleFamily**: The concrete family implementations are deferred. Only placeholder families with default methods are created; their method slots remain `None`.
- **Imperative definition processing**: `identify`, `assess`, `given_body`, `register`, `compile` processing of actual `imperative_defn` objects is deferred. The dispatch helpers in this plan operate on the family only.
- **`imperative_defn` and `id_body` types**: These do not yet exist. Method signatures are simplified to omit them.
- **Runtime context data**: `id_runtime_context_data` is not implemented.
- **Full method signatures**: Method slots take `&ImpDefFamily` only. They will be expanded to include `&ImperativeDefn`, `&mut IdBody`, `&mut RuntimeContextData`, etc., when those types exist.
- **Preform grammar / Salsa integration**: No parsing, no Salsa queries.
- **Problem messages and I6 compilation**: No `StandardProblems`, no Inter emission, no actual compilation of definition bodies.
- **Adjective startup hooks**: `AdjectivesByPhrase::start`, `AdjectivesByCondition::start`, `AdjectivesByInterFunction::start`, `AdjectivesByInterCondition::start` are deferred.
