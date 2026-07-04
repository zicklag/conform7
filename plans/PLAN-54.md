# Plan 54: The Refiner — Refine Parse Tree for Assertion Processing

**Status**: In progress
**Target**: 2 days

## Goal

Implement the core Refiner module — the tree-annotation step that refines `UNPARSED_NOUN_NT` nodes into typed noun phrase nodes (`PROPER_NOUN_NT`, `COMMON_NOUN_NT`, `KIND_NT`, etc.) with proper annotations (subject, evaluation, creation proposition). This is the critical prerequisite for `MajorNodes::pass_1`.

## Background

### C pipeline

After `MajorNodes::pre_pass` (PLAN-53), the next step is `MajorNodes::pass_1` which calls `Refiner::refine_coupling(px, py, FALSE)` on each coupling in the syntax tree. The Refiner is the tree-annotation step that makes assertion processing possible.

### What the Refiner does

The Refiner takes a parse tree with `UNPARSED_NOUN_NT` nodes and refines them into typed nodes:

| Input Node Type | Output Node Type | Action |
|----------------|-----------------|--------|
| `UNPARSED_NOUN_NT` | `PROPER_NOUN_NT` / `COMMON_NOUN_NT` / `KIND_NT` / etc. | Noun phrase resolution via Preform grammar, instance/kind lookup |
| `WITH_NT` | Same | Recursive refine + with-surgery (merge possessive) |
| `AND_NT` | Same | Recursive refine + and-surgery (split conjunction) |
| `X_OF_Y_NT` | Same | Recursive refine (property of owner) |
| `RELATIONSHIP_NT` | Same | Recursive refine |
| `CALLED_NT` | `PROPER_NOUN_NT` | Convert called-name to proper noun |
| `KIND_NT` | Same | Look up kind subject |
| `PRONOUN_NT` | Same | Anaphora lookup (stub) |

### Current Rust state

- `MajorNodes::pre_pass` — complete, diagrams sentences via Preform
- `MajorNodes::pass_1` — stub
- `MajorNodes::pass_2` — stub
- Knowledge module: inference subjects, instances, kinds, properties, adjectives — all exist
- Calculus module: binary predicates, unary predicates, atoms, propositions, terms — all exist
- Syntax crate: ParseNode, NodeType, Preform, Diagrams, NounPhrases — all exist

## Decision

### 1. Is PLAN-54 the correct next step?

**Yes.** The Refiner is the critical prerequisite for pass_1. Without it, the assertion matrix has nothing to dispatch on — the noun phrases are still `UNPARSED_NOUN_NT` nodes.

### 2. Is it independently testable?

**Yes.** The Refiner takes a parse tree with known structure and annotates it. It can be tested with synthetic trees that exercise specific node types.

### 3. What is the smallest independently testable subset?

1. `Refiner` struct with `refine_coupling(px, py, now_negated)` entry point
2. `Refiner::refine(node)` — the core dispatch by node type
3. `Refiner::un_with(node)` — tree surgery to remove "with" wrappers
4. `Refiner::with_surgery(node)` — merge possessive "with" into the parent
5. `Refiner::and_surgery(node)` — split "and" conjunctions
6. Noun phrase resolution: instance lookup, kind lookup, value property lookup
7. `Creator::consult_the_creator(px, py)` — stub (creates objects/kinds)

### 4. What simplifications are appropriate?

- **Stub all plugin-dependent paths** — `PluginCalls::act_on_special_NPs`, `PluginCalls::refine_implicit_noun`, `PluginCalls::unusual_property_value`
- **Stub map direction handling** — `MapRelations::get_mapping_relationship`
- **Stub action pattern handling** — `ActionsNodes::convert_to_ACTION_node`
- **Stub pronoun handling** — `here_pronoun` and `implied_pronoun`
- **Stub `VerbPhrases::corrective_surgery`**
- **Stub description handling** — `Descriptions::to_instance`, `Descriptions::makes_kind_explicit`
- **Stub `NonlocalVariables`** — global variable parsing
- **Stub `Quantifiers`** — `Quantifiers::can_be_used_in_assertions`
- **Implement noun phrase resolution for simple cases only** — existing instances, existing kinds, value property names

## Tasks

### Task 1: Create `Refiner` module

Create `crates/conform7-semantics/src/assertions/refiner.rs` with:
- `Refiner::refine_coupling(px, py, now_negated)` — entry point
- `Refiner::refine(node)` — core dispatch
- `Refiner::un_with(node)` — tree surgery
- `Refiner::with_surgery(node)` — merge possessive
- `Refiner::and_surgery(node)` — split conjunction
- Noun phrase resolution helpers

### Task 2: Create `Creator` module (stub)

Create `crates/conform7-semantics/src/assertions/creator.rs` with:
- `Creator::consult_the_creator(px, py)` — stub

### Task 3: Wire into `assertions/mod.rs`

### Task 4: Add unit tests

### Task 5: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-54: The Refiner — Refine Parse Tree for Assertion Processing"`

## Success Criteria

- [ ] `Refiner::refine_coupling` refines both sides of a coupling
- [ ] `Refiner::refine` dispatches by node type (WITH_NT, AND_NT, X_OF_Y_NT, CALLED_NT, KIND_NT, PRONOUN_NT, UNPARSED_NOUN_NT)
- [ ] `Refiner::un_with` removes "with" wrappers
- [ ] `Refiner::with_surgery` merges possessive "with" into parent
- [ ] `Refiner::and_surgery` splits "and" conjunctions
- [ ] Noun phrase resolution works for existing instances and kinds
- [ ] `Creator::consult_the_creator` is a no-op stub
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- **Full noun phrase resolution** — descriptions, quantifiers, action patterns deferred
- **`MajorNodes::pass_1`** — deferred to PLAN-55
- **`MajorNodes::pass_2`** — deferred
- **`Tables::traverse_to_stock`** — deferred
- **`Task::verify`** — deferred
- **Real `Anaphora` implementation** — deferred
- **Real `PluginCalls`** — deferred
- **Map direction handling** — deferred
- **Action pattern parsing** — deferred
