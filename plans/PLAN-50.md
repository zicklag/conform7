# Plan 50: Built-in Infrastructure — Inference Subjects and Kind Constructors

**Status**: In progress
**Target**: 1 day

## Goal

Implement the first two steps of the `BUILT_IN_STUFF_CSEQ` compilation bench: `InferenceSubjects::make_built_in` and `Task::make_built_in_kind_constructors`. These create the fundamental inference subjects (model_world, global_variables, global_constants, relations) and the built-in kind constructors (CON_NIL, CON_TUPLE_ENTRY, CON_INTERMEDIATE, etc.).

This is the **next step** after the `AssertionsModule::start()` sequence (PLAN-49, Complete). The C compilation pipeline (`How To Compile.w`) proceeds from assertions module startup to building built-in infrastructure.

## Background

### C reference architecture

From `gitignore/inform/inform7/core-module/Chapter 1/How To Compile.w`, lines 126-132:

```c
@<Build a rudimentary set of kinds, relations, verbs and inference subjects@> =
    Task::advance_stage_to(BUILT_IN_STUFF_CSEQ, I"Making built in infrastructure",
        -1, debugging, sequence_timer);
    BENCH(InferenceSubjects::make_built_in);
    BENCH(Task::make_built_in_kind_constructors);
    BENCH(BinaryPredicateFamilies::first_stock)
    BENCH(BootVerbs::make_built_in)
```

**`InferenceSubjects::make_built_in`** (C reference: `inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 99-105):

```c
void InferenceSubjects::make_built_in(void) {
    model_world = InferenceSubjects::new_fundamental(NULL, I"model-world");
    global_variables = InferenceSubjects::new_fundamental(model_world, I"global-variables");
    global_constants = InferenceSubjects::new_fundamental(model_world, I"global-constants");
    relations = InferenceSubjects::new_fundamental(model_world, I"relations");
    PluginCalls::create_inference_subjects();
}
```

Creates 4 fundamental inference subjects in a hierarchy:
```
model_world (root, no broader)
    ├── global_variables
    ├── global_constants
    └── relations
```

**`Task::make_built_in_kind_constructors`** (C reference: `inform7/core-module/Chapter 1/What To Compile.w`, lines 226-229):

```c
void Task::make_built_in_kind_constructors(void) {
    if (inform7_task == NULL) internal_error("there is no current task");
    Projects::load_built_in_kind_constructors(inform7_task->project);
}
```

The C loads kind constructors from project data. Simplified: we create the constructors directly.

### Current Rust state

- **`crates/conform7-semantics/src/knowledge/inference_subjects.rs`** defines:
  - `InferenceSubject` struct with `broader_than`, `infs_family`, `represents`, `inf_list`, `imp_list`, `permissions_list`, `alias_variable`, `log_name`
  - `InferenceSubjectFamily` struct with `name`, `methods`
  - `InferenceSubjectFamilyMethods` with: `get_name_text`, `get_default_certainty`, `new_permission_granted`, `make_adj_const_domain`, `complete_model`, `check_model`
  - `InferenceSubjectFamily::fundamentals()` — creates the fundamentals family with `get_default_certainty` returning `CERTAIN_CE` (3)
  - `InferenceSubject::new(family_idx, broader_than, represents, log_name)` — creates a new subject
  - `InferenceSubject::new_fundamental(broader_than, log_name)` — creates a fundamental subject (family 0)
  - Methods: `is_within`, `is_strictly_within`, `narrowest_broader_subject`, `falls_within`, `get_inferences`, `get_implications`, `get_permissions`, `get_name_text`, `get_default_certainty`, `new_permission_granted`
  - 20+ unit tests covering creation, hierarchy, method dispatch, aliasing

- **`crates/conform7-semantics/src/kind_constructors.rs`** defines:
  - `KindConstructor` struct with `name`, `arity`, `constructs`, `metadata`
  - `KindConstructor::new(name, arity, constructs)` — creates a new constructor
  - `KindConstructor::nil()` — creates the nil constructor
  - `KindConstructor::tuple_entry()` — creates the tuple entry constructor
  - `KindConstructor::intermediate()` — creates the intermediate constructor
  - `KindConstructor::kind_variable()` — creates the kind variable constructor
  - `KindConstructor::rval_function()` — creates the rvalue function constructor
  - `KindConstructor::list_of()` — creates the list-of constructor
  - `KindConstructor::description_of()` — creates the description-of constructor
  - `KindConstructor::table_of()` — creates the table-of constructor
  - `KindConstructor::combination(arity)` — creates a combination constructor
  - `KindConstructor::unchecked()` — creates the unchecked constructor
  - `KindConstructor::named(name)` — creates a named constructor
  - `KindConstructors` struct with `new_family`, `find`, `get_name`
  - 10+ unit tests

- **`crates/conform7-semantics/src/knowledge/mod.rs`** — lists all knowledge modules

- **`crates/conform7-semantics/src/lib.rs`** — crate root, exports all modules

- **All 1444 tests pass; clippy clean.**

### Key gap: `InferenceSubject::new_fundamental` doesn't take a Vec

The current `new_fundamental` creates a standalone subject. For `make_built_in`, we need to push subjects into a Vec and return indices. We need to add a `new_fundamental_in` variant.

### Key gap: No `make_built_in_kind_constructors` function

The kind constructors exist as individual constructors but there's no function that creates all of them in one call. We need to add `KindConstructors::make_built_in` that creates the standard set.

## Decision

### 1. Is Plan 50 the correct next step?

**Yes.** The C compilation pipeline proceeds from assertions module startup to `BUILT_IN_STUFF_CSEQ`. The first two steps (`InferenceSubjects::make_built_in` and `Task::make_built_in_kind_constructors`) are:
- Within the `conform7-semantics` crate
- Use only existing types
- Independently testable
- Create the foundation for later plans (BP first_stock, BootVerbs)

### 2. Is it independently testable?

**Yes.** The foundation consists of:
- Adding `InferenceSubject::new_fundamental_in` that pushes to a Vec and returns the index
- Creating `InferenceSubjects::make_built_in` that creates 4 subjects
- Creating `KindConstructors::make_built_in` that creates the standard kind constructors
- Testing that the correct subjects/constructors are created with correct hierarchy/names

### 3. What is the smallest independently testable subset?

1. Add `InferenceSubject::new_fundamental_in(broader_than_idx, log_name, subjects)` — pushes to Vec, returns index.
2. `InferenceSubjects::make_built_in(subjects)` — creates 4 subjects, returns `[usize; 4]`.
3. `KindConstructors::make_built_in(constructors)` — creates standard kind constructors.
4. All existing tests continue to pass.

### 4. What simplifications are appropriate?

- **No `PluginCalls::create_inference_subjects()`** — plugins are deferred.
- **No `inform7_task` global** — use a static registry or Vec parameter.
- **No project file loading** — create constructors directly.
- **No `current_sentence` tracking** — no parse_node yet.
- **No `BinaryPredicateFamilies::first_stock`** — deferred to Plan 51.
- **No `BootVerbs::make_built_in`** — deferred to Plan 52+.

## Tasks

### Task 1: Add `new_fundamental_in` to `InferenceSubject`

Edit `crates/conform7-semantics/src/knowledge/inference_subjects.rs`.

Add a method that creates a fundamental subject and pushes it into a Vec:

```rust
/// Create a new fundamental inference subject and push it into a Vec.
///
/// Corresponds to `InferenceSubjects::new_fundamental` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 99-105).
///
/// `broader_than_idx` is the index of the broader subject (None for root).
/// `log_name` is the debugging log name.
/// `subjects` is the Vec to push into.
///
/// Returns the index of the newly created subject.
pub fn new_fundamental_in(
    broader_than_idx: Option<usize>,
    log_name: &'static str,
    subjects: &mut Vec<InferenceSubject>,
) -> usize {
    let idx = subjects.len();
    let subject = InferenceSubject::new_fundamental(
        broader_than_idx.map(|b| Box::new(b)),
        log_name,
    );
    subjects.push(subject);
    idx
}
```

**Note**: The current `InferenceSubject::new_fundamental` takes `broader_than: Option<Box<usize>>`. We need to check the exact signature and adapt. If it takes `Option<usize>` directly, we pass it through. If it takes `Option<Box<usize>>`, we wrap.

### Task 2: Add `InferenceSubjects::make_built_in`

Add to `crates/conform7-semantics/src/knowledge/inference_subjects.rs`:

```rust
/// Create the built-in inference subjects.
///
/// Corresponds to `InferenceSubjects::make_built_in` in the C reference
/// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 99-105).
///
/// Creates the hierarchy:
///   model_world (root)
///       global_variables
///       global_constants
///       relations
///
/// Returns the indices of the created subjects in order:
/// [model_world, global_variables, global_constants, relations]
pub fn make_built_in(subjects: &mut Vec<InferenceSubject>) -> [usize; 4] {
    let model_world = Self::new_fundamental_in(None, "model-world", subjects);
    let global_variables = Self::new_fundamental_in(Some(model_world), "global-variables", subjects);
    let global_constants = Self::new_fundamental_in(Some(model_world), "global-constants", subjects);
    let relations = Self::new_fundamental_in(Some(model_world), "relations", subjects);
    [model_world, global_variables, global_constants, relations]
}
```

### Task 3: Add `KindConstructors::make_built_in`

Edit `crates/conform7-semantics/src/kind_constructors.rs`.

Add a function that creates the standard set of built-in kind constructors:

```rust
/// Create the built-in kind constructors.
///
/// Corresponds to `Task::make_built_in_kind_constructors` in the C reference
/// (`inform7/core-module/Chapter 1/What To Compile.w`, lines 226-229).
///
/// Simplified: creates the constructors directly instead of loading from
/// project data.
///
/// Returns the indices of the created constructors in order:
/// [nil, tuple_entry, intermediate, kind_variable, rval_function,
///  list_of, description_of, table_of, combination, unchecked, named]
pub fn make_built_in(constructors: &mut Vec<KindConstructor>) -> [usize; 11] {
    let nil = KindConstructors::new_family(KindConstructor::nil(), constructors);
    let tuple_entry = KindConstructors::new_family(KindConstructor::tuple_entry(), constructors);
    let intermediate = KindConstructors::new_family(KindConstructor::intermediate(), constructors);
    let kind_variable = KindConstructors::new_family(KindConstructor::kind_variable(), constructors);
    let rval_function = KindConstructors::new_family(KindConstructor::rval_function(), constructors);
    let list_of = KindConstructors::new_family(KindConstructor::list_of(), constructors);
    let description_of = KindConstructors::new_family(KindConstructor::description_of(), constructors);
    let table_of = KindConstructors::new_family(KindConstructor::table_of(), constructors);
    let combination = KindConstructors::new_family(KindConstructor::combination(0), constructors);
    let unchecked = KindConstructors::new_family(KindConstructor::unchecked(), constructors);
    let named = KindConstructors::new_family(KindConstructor::named(""), constructors);
    [nil, tuple_entry, intermediate, kind_variable, rval_function,
     list_of, description_of, table_of, combination, unchecked, named]
}
```

**Note**: Check the exact API of `KindConstructors::new_family` — it may take different parameters. The implementer should verify the existing API and adapt.

### Task 4: Add unit tests

Add tests to `crates/conform7-semantics/src/knowledge/inference_subjects.rs`:

```rust
#[test]
fn make_built_in_creates_four_subjects() {
    let mut subjects = Vec::new();
    let indices = InferenceSubjects::make_built_in(&mut subjects);
    assert_eq!(subjects.len(), 4);
    assert_eq!(indices.len(), 4);
}

#[test]
fn make_built_in_hierarchy() {
    let mut subjects = Vec::new();
    let [mw, gv, gc, rel] = InferenceSubjects::make_built_in(&mut subjects);
    // model_world is root (no broader)
    assert!(subjects[mw].broader_than.is_none());
    // global_variables is child of model_world
    assert_eq!(*subjects[gv].broader_than.as_ref().unwrap(), mw);
    // global_constants is child of model_world
    assert_eq!(*subjects[gc].broader_than.as_ref().unwrap(), mw);
    // relations is child of model_world
    assert_eq!(*subjects[rel].broader_than.as_ref().unwrap(), mw);
}

#[test]
fn make_built_in_log_names() {
    let mut subjects = Vec::new();
    let [mw, gv, gc, rel] = InferenceSubjects::make_built_in(&mut subjects);
    assert_eq!(subjects[mw].log_name, "model-world");
    assert_eq!(subjects[gv].log_name, "global-variables");
    assert_eq!(subjects[gc].log_name, "global-constants");
    assert_eq!(subjects[rel].log_name, "relations");
}

#[test]
fn make_built_in_family() {
    let mut subjects = Vec::new();
    let indices = InferenceSubjects::make_built_in(&mut subjects);
    // All subjects should use the fundamentals family (index 0)
    for &idx in &indices {
        assert_eq!(subjects[idx].infs_family, 0,
            "subject '{}' should use fundamentals family",
            subjects[idx].log_name);
    }
}
```

Add tests to `crates/conform7-semantics/src/kind_constructors.rs`:

```rust
#[test]
fn make_built_in_creates_eleven_constructors() {
    let mut constructors = Vec::new();
    let indices = KindConstructors::make_built_in(&mut constructors);
    assert_eq!(constructors.len(), 11);
    assert_eq!(indices.len(), 11);
}

#[test]
fn make_built_in_constructor_names() {
    let mut constructors = Vec::new();
    let indices = KindConstructors::make_built_in(&mut constructors);
    let expected_names = [
        "nil", "tuple_entry", "intermediate", "kind_variable",
        "rval_function", "list_of", "description_of", "table_of",
        "combination", "unchecked", "named",
    ];
    for (i, &idx) in indices.iter().enumerate() {
        assert_eq!(constructors[idx].name, expected_names[i],
            "constructor {} should have name '{}'", i, expected_names[i]);
    }
}
```

### Task 5: Verify

- [ ] `cargo build` — compiles without errors
- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy --all-targets` — no new warnings

## Success Criteria

- [ ] `InferenceSubject::new_fundamental_in` creates a subject and pushes it into a Vec.
- [ ] `InferenceSubjects::make_built_in` creates 4 subjects with correct hierarchy.
- [ ] `KindConstructors::make_built_in` creates 11 kind constructors with correct names.
- [ ] All existing tests still pass.
- [ ] `cargo clippy --all-targets` introduces no new warnings.

## Out of Scope

- **`PluginCalls::create_inference_subjects()`** — plugins are deferred.
- **`BinaryPredicateFamilies::first_stock`** — deferred to Plan 51.
- **`BootVerbs::make_built_in`** — deferred to Plan 52+.
- **Three passes through major nodes** — deferred to later plans.
- **Model world creation** — deferred to later plans.
- **Tables and grammar** — deferred to later plans.
- **Phrases and rules** — deferred to later plans.
- **Inter generation** — deferred to later plans.
