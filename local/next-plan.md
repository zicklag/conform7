# Handoff: PLAN-45 — Adjectives by Inter Condition Foundation

## Summary

`plans/PLAN-45.md` has been updated with a detailed, implementation-ready plan for the Adjectives by Inter Condition foundation. The work creates a new assertions-module family, `adjectives_by_inter_condition`, that mirrors `AdjectivesByInterCondition` in the C reference (`inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`).

## What to build

1. **New module:** `crates/conform7-semantics/src/assertions/adjectives_by_inter_condition.rs`
   - Static `INTER_CONDITION_FAMILY` index.
   - `AdjectivesByInterCondition::start(&mut Vec<AdjectiveMeaningFamily>) -> usize` at priority **4**.
   - `is_by_inter_condition(...)` predicate.
   - `claim_definition(...)` that:
     - parses the condition text with a small stand-in for `<inform6-condition-adjective-definition>`,
     - accepts only `sense == 1`,
     - declines if calling text is non-empty,
     - creates an `AdjectiveMeaning` + `Adjective`, links them, sets the domain,
     - marks only `TEST_ATOM_TASK` as via-support-function (Inter-condition adjectives are test-only; no NOW tasks).
   - A `ClaimDefinitionFn`-shaped wrapper for the existing family-methods slot.
   - Unit tests covering declines, accepts, and `is_by_inter_condition`.

2. **Wire into assertions module:** `crates/conform7-semantics/src/assertions/mod.rs`
   - Add `pub mod adjectives_by_inter_condition;`.
   - Add module-map row and C reference.
   - Update startup-sequence comment to show `AdjectivesByInterCondition::start()` after `AdjectivesByInterFunction::start()`.

## Key design decisions

- **Priority 4** (lower than inter-routine/condition/phrase), matching C.
- **Only `claim_definition` is installed**; the C family has no `GENERATE_IN_SUPPORT_FUNCTION` method. Schema side effects happen inline.
- **No `prepare_schemas` placeholder** — keep `prepare_schemas: None` to mirror C.
- **No `Definition` struct yet** — `family_specific_data` stays `None`.
- **No real Preform/Salsa parser** — a minimal Rust string parser recognizes the single `i6/inter condition "..." says so` template. Full grammar integration is out of scope.
- **Task-mode marking reproduces `RTAdjectives::set_schemas_for_raw_Inter_condition`** without generating I6 schemas. Because Inter conditions are test-only, only `TEST_ATOM_TASK` is marked; `NOW_ATOM_TRUE_TASK` and `NOW_ATOM_FALSE_TASK` remain `NO_TASKMODE`.
- **Indexing text** is set to the quoted condition text (capture 1 in the C reference). The raw word number used for schema generation is deferred.

## Success criteria at a glance

- Module compiles, 7 new unit tests pass.
- `cargo test` keeps all 1394 existing tests passing.
- `cargo clippy --all-targets` remains clean.

## Next steps

1. Implement `adjectives_by_inter_condition.rs` exactly per the code snippets in `plans/PLAN-45.md`.
2. Update `assertions/mod.rs` as shown in the plan.
3. Run `cargo test -- assertions::adjectives_by_inter_condition` and then `cargo test` / `cargo clippy --all-targets`.

## References

- `plans/PLAN-45.md` (updated)
- `plans/PLAN-44.md` (inter-function pattern reference)
- `gitignore/inform/inform7/assertions-module/Chapter 8/Adjectives by Inter Condition.w`
- `gitignore/inform/inform7/runtime-module/Chapter 5/Adjectives.w` (lines 431-434)
- `crates/conform7-semantics/src/assertions/adjectives_by_inter_function.rs`
- `crates/conform7-semantics/src/knowledge/adjectives.rs`
