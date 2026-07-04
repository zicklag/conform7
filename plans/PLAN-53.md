# Plan 53: MajorNodes::pre_pass — Pre-pass Through Major Nodes

**Status**: In progress
**Target**: 2 days

## Goal

Implement `MajorNodes::pre_pass` — the pre-pass through major nodes (SEMANTIC_ANALYSIS_CSEQ). This is the first of three passes through the syntax tree, responsible for diagramming sentences via Preform matching and verb finding.

## Background

### C pipeline

From `How To Compile.w` lines 134-146:

```c
@<Pass three times through the major nodes@> =
    Task::advance_stage_to(SEMANTIC_ANALYSIS_CSEQ, ...);
    BENCH(MajorNodes::pre_pass)          // ← THIS PLAN
    BENCH(Task::verify)
    Task::advance_stage_to(ASSERTIONS_PASS_1_CSEQ, ...);
    BENCH(MajorNodes::pass_1)            // deferred
    BENCH(Tables::traverse_to_stock)
    Task::advance_stage_to(ASSERTIONS_PASS_2_CSEQ, ...);
    BENCH(MajorNodes::pass_2)            // deferred
```

### What pre_pass does

Traverses the syntax tree and for each major node type:

| Node Type | Action |
|-----------|--------|
| `ROOT_NT` | No-op |
| `HEADING_NT` | `Anaphora::new_discussion()` |
| `BEGINHERE_NT` | `Anaphora::new_discussion()` + set extension flag |
| `ENDHERE_NT` | `Anaphora::new_discussion()` + clear extension flag |
| `IMPERATIVE_NT` | `ImperativeSubtrees::accept(p)` |
| `DEFN_CONT_NT` | No-op |
| `SENTENCE_NT` | **`Classifying::sentence(p)`** — diagram via Preform |
| `TABLE_NT` | `Tables::create_table(p)` |
| `EQUATION_NT` | `Equations::new_at(p, FALSE)` |
| `TRACE_NT` | Toggle trace |
| `INFORM6CODE_NT` | No-op (handled in pass 2) |
| `BIBLIOGRAPHIC_NT` | No-op (handled in pass 2) |

The critical function is `Classifying::sentence(p)` which:
1. Marks the sentence as classified
2. Calls `<sentence-without-occurrences>` Preform match
3. Grafts the resulting VERB_NT subtree onto the SENTENCE_NT
4. Calls `PropertySentences::look_for_property_creation(p)`
5. Calls `PluginCalls::new_assertion_notify(p)`

### Current Rust state

**Syntax crate:**
- `ParseNode` with annotations, children, node types, `traverse_depth_first`
- `NodeType` enum with SENTENCE_NT, HEADING_NT, IMPERATIVE_NT, TABLE_NT, EQUATION_NT, VERB_NT, etc.
- `Sentence` and `SentenceClassification` from the sentence breaker
- `parse_structural()` and `parse_heading()` — converts sentences to parse nodes
- `Preform` grammar engine with full matching
- `InternalRegistry` with all needed internal NTs
- `VerbPhrases::seek()` — the verb-finding algorithm
- `Verbs::make_built_in()` — creates "to be" and "to mean" verbs
- `Diagrams` — diagram constructor functions
- `NounPhrases` — noun phrase parsing

**Semantics crate:**
- `calculus` module: binary predicates, unary predicates, atoms, propositions, terms, relation names
- `knowledge` module: inference subjects, inferences, properties, instances, adjectives
- `assertions` module: imperative definition families, adjectival definition family, to phrase family, rule family, adjectives by phrase/condition/inter

## Decision

### 1. Is PLAN-53 the correct next step?

**Yes.** It's the first pass in the three-pass sequence, directly after BUILT_IN_STUFF_CSEQ. All prerequisites exist: VerbPhrases::seek, internal NTs, Verbs::make_built_in, Diagrams, NounPhrases.

### 2. Is it independently testable?

**Yes.** Create a syntax tree with SENTENCE_NT nodes, populate the verb registry, load Preform grammar, run pre_pass, verify each SENTENCE_NT has a VERB_NT child.

### 3. What is the smallest independently testable subset?

1. 8 new stub modules in `conform7-semantics/src/assertions/`
2. `MajorNodes::pre_pass()` — the traversal and dispatch
3. `Classifying::sentence()` — the Preform diagramming
4. Integration test: create syntax tree → run pre_pass → verify VERB_NT children

### 4. What simplifications are appropriate?

- All non-essential handlers are stubs (Anaphora, ImperativeSubtrees, Tables, Equations, PropertySentences, PluginCalls)
- TRACE_NT, BEGINHERE_NT, ENDHERE_NT are stubs
- pass_1() and pass_2() are stubs returning Ok

## Tasks

### Task 1: Create 8 new modules in `conform7-semantics/src/assertions/`

1. `major_nodes.rs` — `MajorNodes` with `pre_pass()`, `pass_1()` (stub), `pass_2()` (stub)
2. `classifying.rs` — `Classifying` with `sentence()` (diagrams via Preform)
3. `anaphora.rs` — `Anaphora` with `new_discussion()` (stub)
4. `imperative_subtrees.rs` — `ImperativeSubtrees` with `accept()` (stub)
5. `plugin_calls.rs` — `PluginCalls` with `new_assertion_notify()` (stub)
6. `tables.rs` — `Tables` with `create_table()` (stub)
7. `equations.rs` — `Equations` with `new_at()` (stub)
8. `property_sentences.rs` — `PropertySentences` with `look_for_property_creation()` (stub)

### Task 2: Wire into `assertions/mod.rs`

### Task 3: Add integration test

### Task 4: Verify and commit

- `cargo build` — compiles without errors
- `cargo test` — all tests pass
- `cargo clippy --all-targets` — no new warnings
- `git add -A && git commit -m "PLAN-53: MajorNodes::pre_pass — Pre-pass through major nodes"`

## Success Criteria

- [ ] `MajorNodes::pre_pass()` traverses syntax tree and dispatches by node type
- [ ] `Classifying::sentence()` diagrams sentences via Preform `<sentence-without-occurrences>`
- [ ] SENTENCE_NT nodes get VERB_NT children after pre_pass
- [ ] All stub modules compile and don't panic
- [ ] All existing tests still pass
- [ ] `cargo clippy --all-targets` introduces no new warnings

## Out of Scope

- `MajorNodes::pass_1()` — deferred (needs Refiner, Assertions matrix)
- `MajorNodes::pass_2()` — deferred (needs Assertions matrix, World model)
- `Task::verify` — deferred
- `Tables::traverse_to_stock` — deferred
- Real `Anaphora` implementation — deferred
- Real `ImperativeSubtrees::accept` — deferred
- Real `Tables::create_table` — deferred
- Real `Equations::new_at` — deferred
- Real `PluginCalls` — deferred
- `TRACE_NT` handling — deferred
- `BEGINHERE_NT`/`ENDHERE_NT` extension tracking — deferred
