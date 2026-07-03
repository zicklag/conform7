# Handoff: PLAN-41 — Imperative Definition Families Foundation

## Status

PLAN-41 was reviewed and developed into a detailed, independently testable plan.

- **Plan number and name:** Plan 41: Imperative Definition Families — Foundation
- **Why this is the next step:** `ImperativeDefinitionFamilies::create()` is the immediate successor to `KindPredicatesRevisited::start()` in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 33).
- **Key revision:** The original draft only listed out-of-scope items. The revised plan adds the concrete data structure, registry, method dispatch infrastructure, integration points, and testable success criteria.

## Key files to create / modify

### Create

- `crates/conform7-semantics/src/assertions/mod.rs`
  - Module-level docs, module map, references list.
  - Re-export `pub mod imperative_definition_families;`.
- `crates/conform7-semantics/src/assertions/imperative_definition_families.rs`
  - `ImpDefFamilyMethods` struct with optional method slots.
  - `ImpDefFamily` struct.
  - `ImperativeDefinitionFamilies` manager with `new_family`, `create`, `start`, dispatch helpers, and built-in accessors.
  - Unit tests.

### Modify

- `crates/conform7-semantics/src/lib.rs`
  - Add `pub mod assertions;` after `pub mod knowledge;`.
  - Update crate-level module map comment.

## Key types and functions to implement

### `ImpDefFamilyMethods`

Optional method slots, all defaulting to `None`, mirroring the C method IDs in `Imperative Definition Families.w`:

- `identify: Option<fn(&ImpDefFamily) -> ()>`
- `assess: Option<fn(&ImpDefFamily) -> ()>`
- `given_body: Option<fn(&ImpDefFamily) -> ()>`
- `register: Option<fn(&ImpDefFamily) -> ()>`
- `to_rcd: Option<fn(&ImpDefFamily) -> ()>`
- `assessment_complete: Option<fn(&ImpDefFamily) -> ()>`
- `allows_rule_only_phrases: Option<fn(&ImpDefFamily) -> bool>`
- `allows_empty: Option<fn(&ImpDefFamily) -> bool>`
- `allows_inline: Option<fn(&ImpDefFamily) -> bool>`
- `compile: Option<fn(&ImpDefFamily, &mut i32, i32) -> ()>`
- `phrasebook_index: Option<fn(&ImpDefFamily) -> bool>`

### `ImpDefFamily`

```rust
pub struct ImpDefFamily {
    pub name: &'static str,
    pub methods: ImpDefFamilyMethods,
    pub compile_last: bool,
}
```

- `ImpDefFamily::new(name: &'static str, compile_last: bool) -> Self`

### `ImperativeDefinitionFamilies`

- `ImperativeDefinitionFamilies::new_family(name, compile_last) -> ImpDefFamily` — thin wrapper over `ImpDefFamily::new`.
- `ImperativeDefinitionFamilies::create()` — forces the built-in registry.
- `ImperativeDefinitionFamilies::start()` — assertions-module hook, equivalent to `create()`.
- Built-in family accessors: `unknown_idf()`, `adjectival_idf()`, `to_phrase_idf()`, `rule_idf()`.
- Dispatch helpers: `identify`, `assess`, `given_body`, `register`, `to_rcd`, `assessment_complete`, `allows_rule_only_phrases`, `allows_empty`, `allows_inline`, `compile`, `phrasebook_index`.

## Test strategy

- **Family creation tests** verify `new` and `new_family` produce the correct name and `compile_last` flag.
- **Default method tests** verify every slot in a fresh `ImpDefFamilyMethods` is `None`.
- **Registry tests** call `create()` and assert the registry contains four families in the exact C order with the correct flags:
  1. `unknown-idf` — `false`
  2. `adjectival-idf` — `false`
  3. `TO_PHRASE_EFF` — `true`
  4. `rule-idf` — `false`
- **Accessor tests** verify `unknown_idf()`, `adjectival_idf()`, `to_phrase_idf()`, and `rule_idf()` return the matching family.
- **Dispatch helper tests** install a method (e.g., set `allows_empty` to a closure that mutates a captured flag), call the helper, and verify the closure ran. Also verify helpers return `false` / no-op when no method is installed.
- **Startup hook test** verifies `start()` runs without panic and forces the registry.
- After implementation run:
  - `cargo build`
  - `cargo test -- assertions::imperative_definition_families`
  - `cargo clippy --all-targets`

## Simplifications from the C reference

The full C implementation relies on several systems that do not exist yet. The foundation deliberately simplifies:

- **No concrete families:** `AdjectivalDefinitionFamily::create_family`, `ToPhraseFamily::create_family`, and `RuleFamily::create_family` are deferred. The registry contains placeholder families with default (`None`) methods.
- **No `imperative_defn` / `id_body` types:** Method slots take `&ImpDefFamily` only. They will be expanded to include the definition and body references when those types are added.
- **No runtime context data:** `id_runtime_context_data` is not implemented; `to_rcd` is a no-op stub.
- **No Preform grammar:** Preamble identification and parsing are deferred.
- **No problem messages:** `StandardProblems` calls are omitted.
- **No Inter compilation:** `compile` is a no-op stub; no actual phrase/rule compilation happens.
- **No Salsa integration:** The registry uses a simple `LazyLock<Vec<ImpDefFamily>>`, not a Salsa query.
- **Ordered registry:** Instead of Inform's `CLASS_DEFINITION` linked list, the four built-in families are stored in a `Vec` in the exact creation order required by `identify` (rule family last).
