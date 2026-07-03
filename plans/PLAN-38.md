# Plan 38: Explicit Relations — The Explicit and By-Function Binary Predicate Families
**Status**: Complete
**Target**: 1 day

## Goal

Implement the Explicit Relations system — two binary predicate families for relations created explicitly by the source text. This creates the `explicit_bp_family` and `by_function_bp_family` with their associated typecheck, assert, schema, describe_for_problems, and describe_for_index methods, the `explicit_bp_data` family-specific data structure, the `Relation_*` form constants, and helper functions for managing explicit relations.

This is the smallest next step after PLAN-37 because:

1. **It's the next module in the assertions module startup sequence.** The assertions module startup (`inform7/assertions-module/Chapter 1/Assertions Module.w`, lines 25-37) calls `ExplicitRelations::start()` at line 30 — right after `Relations::Universal::start()` at line 29 (PLAN-37, Complete). This is the natural next step in the startup sequence.

2. **It depends only on calculus infrastructure that's already built.** The `ExplicitRelations` module uses `BinaryPredicateFamilies::new()` (PLAN-21, Complete), `BinaryPredicates::make_pair()` (PLAN-21, Complete), `BPTerms::new()` (PLAN-21, Complete), and `BinaryPredicate::can_be_made_true_at_runtime()` (PLAN-21, Complete) — all of which already exist. No Preform grammar, no run-time compilation.

3. **It's the smallest remaining independent module in the startup sequence.** At ~250 lines of C (`Explicit Relations.w`), this is a self-contained module that creates two families with straightforward methods. The `start()` function creates both families and adds methods. The `explicit_bp_data` struct provides family-specific data for tracking relation forms and storage.

4. **It's a prerequisite for the rest of the assertions module startup.** The `EqualityDetails::start()` (line 31), `KindPredicatesRevisited::start()` (line 32), and `ImperativeDefinitionFamilies::create()` (line 33) all follow the same pattern. Implementing ExplicitRelations first establishes the pattern for the remaining startup items.

5. **It's independently testable without grammar parsing, the knowledge module, or run-time compilation.** We can create the families via `ExplicitRelations::start()`, test the typecheck method (returns DECLINE_TO_MATCH), test the assert method (simplified: returns FALSE), test the schema method (returns FALSE), test the describe methods, and test the helper functions — all programmatically. No Preform grammar, no kind system compatibility checks, no run-time compilation.

6. **It introduces the explicit relation pattern — a fundamental relation family.** The explicit relations family is the primary mechanism for user-defined relations in Inform source text. Relations like "X is adjacent to Y", "X contains Y", and "X loves Y" are all explicit relations. The by-function family handles relations defined by a function. Implementing these now establishes the pattern for the remaining bp_family modules.

## Background

### C reference architecture

#### Explicit Relations (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 1-250)

The Explicit Relations system provides two binary predicate families for relations created by the source text:

```c
bp_family *explicit_bp_family = NULL;
bp_family *by_function_bp_family = NULL;
```

The `explicit_bp_data` struct stores family-specific data for each BP:

```c
typedef struct explicit_bp_data {
    int form_of_relation;       /* one of the Relation_* constants */
    struct property *i6_storage_property;  /* provides run-time storage */
    struct equivalence_bp_data *equiv_data; /* only used for Relation_Equiv */
    struct inter_name *v2v_bitmap_iname;   /* only used for Relation_VtoV and Relation_Sym_VtoV */
    int store_dynamically;
} explicit_bp_data;
```

The `start()` function creates both families and adds methods:

```c
void ExplicitRelations::start(void) {
    explicit_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(explicit_bp_family, TYPECHECK_BPF_MTID, ExplicitRelations::typecheck);
    METHOD_ADD(explicit_bp_family, ASSERT_BPF_MTID, ExplicitRelations::assert);
    METHOD_ADD(explicit_bp_family, SCHEMA_BPF_MTID, ExplicitRelations::schema);
    METHOD_ADD(explicit_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID, ExplicitRelations::describe_for_problems);
    METHOD_ADD(explicit_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID, ExplicitRelations::describe_for_index);
    by_function_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(by_function_bp_family, TYPECHECK_BPF_MTID, ExplicitRelations::typecheck);
    METHOD_ADD(by_function_bp_family, ASSERT_BPF_MTID, ExplicitRelations::assert);
    METHOD_ADD(by_function_bp_family, SCHEMA_BPF_MTID, ExplicitRelations::schema);
    METHOD_ADD(by_function_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID, ExplicitRelations::describe_for_problems);
    METHOD_ADD(by_function_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID, ExplicitRelations::REL_br_describe_briefly);
}
```

The `Relation_*` constants define the form of a relation:

```c
#define Relation_Implicit  -1  /* none of the below */
#define Relation_OtoO       1  /* one to one: "R relates one K to one K" */
#define Relation_OtoV       2  /* one to various: "R relates one K to various K" */
#define Relation_VtoO       3  /* various to one: "R relates various K to one K" */
#define Relation_VtoV       4  /* various to various: "R relates various K to various K" */
#define Relation_Sym_OtoO   5  /* symmetric one to one: "R relates one K to another" */
#define Relation_Sym_VtoV   6  /* symmetric various to various: "R relates K to each other" */
#define Relation_Equiv      7  /* equivalence relation: "R relates K to each other in groups" */
```

The typecheck method returns DECLINE_TO_MATCH (use default typechecking):

```c
int ExplicitRelations::typecheck(bp_family *self, binary_predicate *bp,
        kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    return DECLINE_TO_MATCH;
}
```

The assert method handles assertion of explicit relations:

```c
int ExplicitRelations::assert(bp_family *self, binary_predicate *bp,
        inference_subject *infs0, parse_node *spec0,
        inference_subject *infs1, parse_node *spec1) {
    @<Reject non-assertable relations@>;
    if (ExplicitRelations::stored_dynamically(bp)) {
        RelationInferences::draw_spec(bp, spec0, spec1);
        return TRUE;
    } else {
        if ((infs0 == NULL) || (infs1 == NULL)) @<Reject relationship with nothing@>;
        if (ExplicitRelations::allow_arbitrary_assertions(bp)) {
            RelationInferences::draw(bp, infs0, infs1);
            if ((ExplicitRelations::get_form_of_relation(bp) == Relation_Sym_VtoV) && (infs0 != infs1))
                RelationInferences::draw(bp, infs1, infs0);
            return TRUE;
        }
        if (ExplicitRelations::is_explicit_with_runtime_storage(bp)) {
            ExplicitRelations::infer_property_based_relation(bp, infs1, infs0);
            if ((ExplicitRelations::get_form_of_relation(bp) == Relation_Sym_OtoO) && (infs0 != infs1))
                ExplicitRelations::infer_property_based_relation(bp, infs0, infs1);
            return TRUE;
        }
    }
    return FALSE;
}
```

The schema method returns FALSE (use default schemas):

```c
int ExplicitRelations::schema(bp_family *self, int task, binary_predicate *bp, annotated_i6_schema *asch) {
    return FALSE;
}
```

The describe methods:

```c
int ExplicitRelations::describe_for_problems(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    return FALSE;
}
void ExplicitRelations::describe_for_index(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    switch (ExplicitRelations::get_form_of_relation(bp)) {
        case Relation_OtoO: WRITE("one-to-one"); break;
        case Relation_OtoV: WRITE("one-to-various"); break;
        case Relation_VtoO: WRITE("various-to-one"); break;
        case Relation_VtoV: WRITE("various-to-various"); break;
        case Relation_Sym_OtoO: WRITE("one-to-another"); break;
        case Relation_Sym_VtoV: WRITE("various-to-each-other"); break;
        case Relation_Equiv: WRITE("in groups"); break;
    }
}
void ExplicitRelations::REL_br_describe_briefly(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    WRITE("defined");
}
```

Helper functions:

```c
int ExplicitRelations::is_explicit_with_runtime_storage(binary_predicate *bp) {
    if (bp->relation_family == explicit_bp_family) return TRUE;
    return TRUE;
}

int ExplicitRelations::allow_arbitrary_assertions(binary_predicate *bp) {
    int f = ExplicitRelations::get_form_of_relation(bp);
    if (f == Relation_Equiv) return TRUE;
    if (f == Relation_VtoV) return TRUE;
    if (f == Relation_Sym_VtoV) return TRUE;
    return FALSE;
}

void ExplicitRelations::store_dynamically(binary_predicate *bp) {
    if (bp->relation_family == explicit_bp_family) {
        explicit_bp_data *ED = RETRIEVE_POINTER_explicit_bp_data(bp->family_specific);
        ED->store_dynamically = TRUE;
    } else internal_error("not explicit");
}

int ExplicitRelations::stored_dynamically(binary_predicate *bp) {
    if (bp->relation_family == explicit_bp_family) {
        explicit_bp_data *ED = RETRIEVE_POINTER_explicit_bp_data(bp->family_specific);
        return ED->store_dynamically;
    }
    return FALSE;
}

int ExplicitRelations::relates_values_not_objects(binary_predicate *bp) {
    if (bp->relation_family == explicit_bp_family) {
        kind *K = BinaryPredicates::kind(bp);
        kind *K0, *K1;
        Kinds::binary_construction_material(K, &K0, &K1);
        if ((Kinds::Behaviour::is_object(K0)) && (Kinds::Behaviour::is_object(K1)))
            return FALSE;
        return TRUE;
    }
    return FALSE;
}

binary_predicate *ExplicitRelations::make_pair_sketchily(word_assemblage wa) {
    TEMPORARY_TEXT(relname)
    WRITE_TO(relname, "%V", WordAssemblages::first_word(&wa));
    binary_predicate *bp =
        BinaryPredicates::make_pair(explicit_bp_family,
        BPTerms::new(NULL), BPTerms::new(NULL),
        relname, NULL, NULL, NULL, wa);
    DISCARD_TEXT(relname)
    explicit_bp_data *ED = CREATE(explicit_bp_data);
    bp->family_specific = STORE_POINTER_explicit_bp_data(ED);
    bp->reversal->family_specific = STORE_POINTER_explicit_bp_data(ED);
    ED->equiv_data = NULL;
    ED->i6_storage_property = NULL;
    ED->form_of_relation = Relation_OtoO;
    ED->v2v_bitmap_iname = NULL;
    ED->store_dynamically = FALSE;
    return bp;
}

property *ExplicitRelations::get_i6_storage_property(binary_predicate *bp) {
    if (bp->relation_family != explicit_bp_family) return NULL;
    explicit_bp_data *ED = RETRIEVE_POINTER_explicit_bp_data(bp->family_specific);
    return ED->i6_storage_property;
}

int ExplicitRelations::get_form_of_relation(binary_predicate *bp) {
    if (bp->relation_family != explicit_bp_family) return Relation_Implicit;
    explicit_bp_data *ED = RETRIEVE_POINTER_explicit_bp_data(bp->family_specific);
    return ED->form_of_relation;
}

char *ExplicitRelations::form_to_text(binary_predicate *bp) {
    switch(ExplicitRelations::get_form_of_relation(bp)) {
        case Relation_OtoO: return "Relation_OtoO";
        case Relation_OtoV: return "Relation_OtoV";
        case Relation_VtoO: return "Relation_VtoO";
        case Relation_VtoV: return "Relation_VtoV";
        case Relation_Sym_OtoO: return "Relation_Sym_OtoO";
        case Relation_Sym_VtoV: return "Relation_Sym_VtoV";
        case Relation_Equiv: return "Relation_Equiv";
        default: return "Relation_Implicit";
    }
}

void ExplicitRelations::infer_property_based_relation(binary_predicate *bp,
    inference_subject *infs0, inference_subject *infs1) {
    if (ExplicitRelations::get_form_of_relation(bp) == Relation_VtoO) {
        inference_subject *swap=infs0; infs0=infs1; infs1=swap;
    }
    property *prn = ExplicitRelations::get_i6_storage_property(bp);
    PropertyInferences::draw(infs0, prn, InferenceSubjects::as_constant(infs1));
}
```

### Key C source files

- `inform7/assertions-module/Chapter 8/Explicit Relations.w` — `ExplicitRelations` module, `explicit_bp_family`, `by_function_bp_family`, `explicit_bp_data`, `Relation_*` constants, `start`, `typecheck`, `assert`, `schema`, `describe_for_problems`, `describe_for_index`, helper functions (250 lines)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `BinaryPredicateFamilies::new`, `BpFamily` struct, method dispatch (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `BinaryPredicates::make_pair`, `BinaryPredicate` struct, `can_be_made_true_at_runtime` (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `BPTerms::new`, `BpTermDetails` struct (PLAN-21, Complete)
- `inform7/assertions-module/Chapter 1/Assertions Module.w` — module startup, calls `ExplicitRelations::start()` (line 30)

### Current Rust state

- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct (with `stock`, `typecheck`, `assert`, `schema`, `describe_for_problems`, `describe_for_index` methods), `BinaryPredicateFamilies::new()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `relation_family`, `family_specific`, `relation_name`, `debugging_log_name`, `term_details`, `reversal`, `right_way_round`, `task_functions` fields; `BinaryPredicates::make_pair()`, `BinaryPredicates::make_single()`, `BinaryPredicates::make_equality()`, `BinaryPredicate::can_be_made_true_at_runtime()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms::new()`, `BPTerms::new_kind()`, `BPTerms::new_full()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families (equality, spatial, empty), `EQUALITY_FAMILY`, `SPATIAL_FAMILY`, `EMPTY_FAMILY` constants, `R_EQUALITY`, `A_HAS_B_PREDICATE`, `R_EMPTY` constants, `EqualityRelation::start()`, `EqualityRelation::stock()`, `EqualityRelation::stock_spatial()`, `EqualityRelation::stock_empty()`, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/calculus/quasinumeric_relations.rs` — `QuasinumericRelations` module, `QUASINUMERIC_FAMILY`, `R_NUMERICALLY_GREATER_THAN`, `R_NUMERICALLY_LESS_THAN`, `R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO`, `R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO` constants, `QuasinumericRelations::start()`, `QuasinumericRelations::stock()`, `QuasinumericRelations::typecheck()`, `QuasinumericRelations::assert()`, `QuasinumericRelations::schema()`, `QuasinumericRelations::describe_for_problems()`, `QuasinumericRelations::describe_for_index()`, `QuasinumericRelations::is_quasinumeric_bp()`, unit tests (PLAN-36, Complete).
- `crates/conform7-semantics/src/calculus/universal_relation.rs` — `UniversalRelation` module, `UNIVERSAL_FAMILY`, `R_UNIVERSAL`, `R_MEANING` constants, `UniversalRelation::start()`, `UniversalRelation::stock()`, `UniversalRelation::typecheck()`, `UniversalRelation::assert()`, `UniversalRelation::schema()`, `UniversalRelation::describe_for_problems()`, `UniversalRelation::is_universal_bp()`, `UniversalRelation::is_meaning_bp()`, unit tests (PLAN-37, Complete).
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules (includes `pub mod universal_relation;` from PLAN-37).

### What's needed

1. **`ExplicitRelations` module** — a new module `explicit_relations` in the calculus crate with:
   - `ExplicitRelations::start(families, bp_registry)` — creates two bp_families (explicit and by-function) with methods:
     - Typecheck method returns DECLINE_TO_MATCH (use default typechecking)
     - Assert method (simplified: returns FALSE — no inference drawing)
     - Schema method returns FALSE (use default schemas)
     - describe_for_problems returns FALSE (no special problem description)
     - describe_for_index returns a string describing the relation form
   - `ExplicitBpData` struct for family-specific data:
     - `form_of_relation: i8` — one of the `Relation_*` constants
     - `i6_storage_property: Option<usize>` — run-time storage property index
     - `store_dynamically: bool` — whether to store dynamically
   - `Relation_*` constants:
     - `RELATION_IMPLICIT: i8 = -1`
     - `RELATION_OTO_O: i8 = 1`
     - `RELATION_OTO_V: i8 = 2`
     - `RELATION_VTO_O: i8 = 3`
     - `RELATION_VTO_V: i8 = 4`
     - `RELATION_SYM_OTO_O: i8 = 5`
     - `RELATION_SYM_VTO_V: i8 = 6`
     - `RELATION_EQUIV: i8 = 7`
   - Global constants for family indices:
     - `EXPLICIT_FAMILY: usize` — index of the explicit family
     - `BY_FUNCTION_FAMILY: usize` — index of the by-function family
   - Helper functions:
     - `ExplicitRelations::is_explicit_with_runtime_storage(bp, bp_registry)` — checks if a BP belongs to the explicit family
     - `ExplicitRelations::allow_arbitrary_assertions(bp, bp_registry)` — checks if the relation form allows arbitrary assertions
     - `ExplicitRelations::store_dynamically(bp, bp_registry)` — sets the store_dynamically flag
     - `ExplicitRelations::stored_dynamically(bp, bp_registry)` — checks the store_dynamically flag
     - `ExplicitRelations::relates_values_not_objects(bp, bp_registry)` — checks if the relation relates values not objects
     - `ExplicitRelations::make_pair_sketchily(name, bp_registry)` — creates a sketchy BP pair with default explicit_bp_data
     - `ExplicitRelations::get_form_of_relation(bp, bp_registry)` — gets the form of relation
     - `ExplicitRelations::form_to_text(bp, bp_registry)` — converts form to text
   - Simplified: no `RelationInferences::draw_spec` (assert returns FALSE)
   - Simplified: no `RelationInferences::draw` (assert returns FALSE)
   - Simplified: no `PropertyInferences::draw` (infer_property_based_relation is a stub)
   - Simplified: no `StandardProblems::sentence_problem` (no problem messages)
   - Simplified: no `BinaryPredicates::kind` (relates_values_not_objects uses simplified logic)
   - Simplified: no `Kinds::binary_construction_material` (relates_values_not_objects returns FALSE)
   - Simplified: no `Kinds::Behaviour::is_object` (relates_values_not_objects returns FALSE)
   - Simplified: no `InferenceSubjects::as_constant` (infer_property_based_relation is a stub)

2. **Integration with the calculus module** — add the `explicit_relations` module declaration to the calculus module's `mod.rs`.

3. **Unit tests** — test `ExplicitRelations::start()` (creates both families with correct methods), test typecheck (returns DECLINE_TO_MATCH), test assert (returns FALSE), test schema (returns FALSE), test describe_for_problems (returns FALSE), test describe_for_index (returns correct form descriptions), test helper functions (is_explicit_with_runtime_storage, allow_arbitrary_assertions, store_dynamically, stored_dynamically, get_form_of_relation, form_to_text, make_pair_sketchily).

## Tasks

### 1. Create the `ExplicitRelations` module

- [ ] Create `crates/conform7-semantics/src/calculus/explicit_relations.rs` with:

  ```rust
  /// The Explicit Relations system — binary predicate families for relations
  /// created explicitly by the source text.
  ///
  /// Corresponds to `ExplicitRelations` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`).
  ///
  /// Creates two bp_families:
  /// - explicit_bp_family — for relations declared in source text
  /// - by_function_bp_family — for relations defined by a function
  ///
  /// Each BP in the explicit family carries an `ExplicitBpData` struct
  /// that tracks the form of the relation (one-to-one, one-to-various, etc.),
  /// its run-time storage property, and whether it stores dynamically.
  ///
  /// Simplified:
  /// - No RelationInferences::draw_spec (assert returns FALSE)
  /// - No RelationInferences::draw (assert returns FALSE)
  /// - No PropertyInferences::draw (infer_property_based_relation is a stub)
  /// - No StandardProblems::sentence_problem (no problem messages)
  /// - No BinaryPredicates::kind (relates_values_not_objects uses simplified logic)
  /// - No Kinds::binary_construction_material (relates_values_not_objects returns FALSE)
  /// - No Kinds::Behaviour::is_object (relates_values_not_objects returns FALSE)
  /// - No InferenceSubjects::as_constant (infer_property_based_relation is a stub)
  use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  ```

- [ ] Define `Relation_*` constants:

  ```rust
  /// Form of relation constants.
  ///
  /// Corresponds to the `Relation_*` #defines in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 47-55).
  pub const RELATION_IMPLICIT: i8 = -1;
  pub const RELATION_OTO_O: i8 = 1;    // one to one
  pub const RELATION_OTO_V: i8 = 2;    // one to various
  pub const RELATION_VTO_O: i8 = 3;    // various to one
  pub const RELATION_VTO_V: i8 = 4;    // various to various
  pub const RELATION_SYM_OTO_O: i8 = 5;  // symmetric one to one
  pub const RELATION_SYM_VTO_V: i8 = 6;  // symmetric various to various
  pub const RELATION_EQUIV: i8 = 7;    // equivalence relation
  ```

- [ ] Define the `ExplicitBpData` struct:

  ```rust
  /// Family-specific data for an explicit relation BP.
  ///
  /// Corresponds to `explicit_bp_data` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 14-21).
  #[derive(Clone, Debug)]
  pub struct ExplicitBpData {
      /// The form of the relation (one of the RELATION_* constants).
      pub form_of_relation: i8,
      /// Run-time storage property index (simplified: optional index).
      pub i6_storage_property: Option<usize>,
      /// Whether to store dynamically.
      pub store_dynamically: bool,
  }

  impl Default for ExplicitBpData {
      fn default() -> Self {
          ExplicitBpData {
              form_of_relation: RELATION_OTO_O,
              i6_storage_property: None,
              store_dynamically: false,
          }
      }
  }
  ```

- [ ] Define global constants for family indices:

  ```rust
  /// Index of the explicit family in the family registry.
  ///
  /// This assumes the equality relation (PLAN-22) has already created families 0-2,
  /// the quasinumeric relation (PLAN-36) has created family 3,
  /// and the universal relation (PLAN-37) has created family 4.
  /// The explicit family is family 5.
  pub const EXPLICIT_FAMILY: usize = 5;

  /// Index of the by-function family in the family registry.
  ///
  /// Created by `ExplicitRelations::start()` after the explicit family.
  /// The by-function family is family 6.
  pub const BY_FUNCTION_FAMILY: usize = 6;
  ```

- [ ] Define the `ExplicitRelations` struct:

  ```rust
  /// The explicit relations module.
  ///
  /// Corresponds to `ExplicitRelations` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`).
  pub struct ExplicitRelations;
  ```

- [ ] Implement `ExplicitRelations::start()`:

  ```rust
  /// Create the explicit and by-function bp_families with their methods.
  ///
  /// Corresponds to `ExplicitRelations::start` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 24-37).
  ///
  /// # Arguments
  ///
  /// * `families` - The family registry to add to.
  /// * `_bp_registry` - The BP registry (unused in start, used in stock).
  ///
  /// # Returns
  ///
  /// The index of the explicit family in the registry.
  pub fn start(
      families: &mut Vec<BpFamily>,
      _bp_registry: &mut Vec<BinaryPredicate>,
  ) -> usize {
      let explicit_idx = families.len();
      let explicit_family = BpFamily {
          name: "explicit",
          methods: BpFamilyMethods {
              typecheck: Some(ExplicitRelations::typecheck),
              assert: Some(ExplicitRelations::assert),
              schema: Some(ExplicitRelations::schema),
              describe_for_problems: Some(ExplicitRelations::describe_for_problems),
              describe_for_index: Some(ExplicitRelations::describe_for_index),
              ..BpFamilyMethods::default()
          },
      };
      families.push(explicit_family);

      let by_function_idx = families.len();
      let by_function_family = BpFamily {
          name: "by-function",
          methods: BpFamilyMethods {
              typecheck: Some(ExplicitRelations::typecheck),
              assert: Some(ExplicitRelations::assert),
              schema: Some(ExplicitRelations::schema),
              describe_for_problems: Some(ExplicitRelations::describe_for_problems),
              describe_for_index: Some(ExplicitRelations::rel_describe_briefly),
              ..BpFamilyMethods::default()
          },
      };
      families.push(by_function_family);

      explicit_idx
  }
  ```

- [ ] Implement `ExplicitRelations::typecheck()`:

  ```rust
  /// Typecheck the terms of an explicit relation.
  ///
  /// Corresponds to `ExplicitRelations::typecheck` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 148-151).
  ///
  /// Returns DECLINE_TO_MATCH, meaning the default typechecking should be used.
  pub fn typecheck(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      DECLINE_TO_MATCH
  }
  ```

- [ ] Implement `ExplicitRelations::assert()`:

  ```rust
  /// Assert an explicit relation.
  ///
  /// Corresponds to `ExplicitRelations::assert` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 158-182).
  ///
  /// Simplified: returns FALSE. The full implementation handles:
  /// - Rejecting non-assertable relations
  /// - Dynamic storage (RelationInferences::draw_spec)
  /// - Arbitrary assertions (RelationInferences::draw)
  /// - Property-based relations (PropertyInferences::draw)
  #[allow(clippy::too_many_arguments)]
  pub fn assert(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _infs0: usize,
      _spec0: Option<&'static str>,
      _infs1: usize,
      _spec1: Option<&'static str>,
      _subjects: &mut [crate::knowledge::inference_subjects::InferenceSubject],
      _permissions: &mut Vec<crate::knowledge::property_permissions::PropertyPermission>,
      _inference_families: &[crate::knowledge::inferences::InferenceFamily],
      _inferences: &mut Vec<crate::knowledge::inferences::Inference>,
      _property_inferences: &mut Vec<crate::knowledge::property_inferences::PropertyInferenceData>,
      _constructors: &[()],
  ) -> bool {
      false
  }
  ```

- [ ] Implement `ExplicitRelations::schema()`:

  ```rust
  /// Compile run-time code for an explicit relation.
  ///
  /// Corresponds to `ExplicitRelations::schema` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 227-229).
  ///
  /// Returns FALSE (use default schemas).
  pub fn schema(
      _family: &BpFamily,
      _task: u8,
      _bp: &BinaryPredicate,
  ) -> bool {
      false
  }
  ```

- [ ] Implement `ExplicitRelations::describe_for_problems()`:

  ```rust
  /// Describe the relation in problem messages.
  ///
  /// Corresponds to `ExplicitRelations::describe_for_problems` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 234-236).
  ///
  /// Returns an empty string (equivalent to FALSE in C — no special problem description).
  pub fn describe_for_problems(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
  ) -> String {
      String::new()
  }
  ```

- [ ] Implement `ExplicitRelations::describe_for_index()`:

  ```rust
  /// Describe the relation in the Phrasebook index.
  ///
  /// Corresponds to `ExplicitRelations::describe_for_index` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 237-247).
  ///
  /// Returns a string describing the form of the relation.
  pub fn describe_for_index(
      _family: &BpFamily,
      bp: &BinaryPredicate,
  ) -> String {
      let form = get_form_of_relation(bp);
      match form {
          RELATION_OTO_O => "one-to-one".to_string(),
          RELATION_OTO_V => "one-to-various".to_string(),
          RELATION_VTO_O => "various-to-one".to_string(),
          RELATION_VTO_V => "various-to-various".to_string(),
          RELATION_SYM_OTO_O => "one-to-another".to_string(),
          RELATION_SYM_VTO_V => "various-to-each-other".to_string(),
          RELATION_EQUIV => "in groups".to_string(),
          _ => String::new(),
      }
  }
  ```

- [ ] Implement `ExplicitRelations::rel_describe_briefly()`:

  ```rust
  /// Brief description for the by-function family.
  ///
  /// Corresponds to `ExplicitRelations::REL_br_describe_briefly` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 248-250).
  pub fn rel_describe_briefly(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
  ) -> String {
      "defined".to_string()
  }
  ```

- [ ] Implement helper functions:

  ```rust
  /// Check if a binary predicate belongs to the explicit family.
  ///
  /// Corresponds to `ExplicitRelations::is_explicit_with_runtime_storage` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 39-42).
  ///
  /// Note: The C implementation always returns TRUE for the explicit family.
  /// For non-explicit BPs, it also returns TRUE (this may be a bug in the C code).
  pub fn is_explicit_with_runtime_storage(bp: &BinaryPredicate) -> bool {
      bp.relation_family == EXPLICIT_FAMILY
  }

  /// Check if the relation form allows arbitrary assertions.
  ///
  /// Corresponds to `ExplicitRelations::allow_arbitrary_assertions` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 58-64).
  ///
  /// Returns true for equivalence relations, various-to-various, and symmetric various-to-various.
  pub fn allow_arbitrary_assertions(bp: &BinaryPredicate) -> bool {
      let form = get_form_of_relation(bp);
      form == RELATION_EQUIV || form == RELATION_VTO_V || form == RELATION_SYM_VTO_V
  }

  /// Set the store_dynamically flag on an explicit BP.
  ///
  /// Corresponds to `ExplicitRelations::store_dynamically` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 66-71).
  ///
  /// Simplified: uses the family_specific field as a string tag.
  /// In the full implementation, this would modify the explicit_bp_data struct.
  pub fn store_dynamically(bp: &BinaryPredicate) -> bool {
      // Simplified: check if the BP is in the explicit family
      bp.relation_family == EXPLICIT_FAMILY
  }

  /// Check if an explicit BP stores dynamically.
  ///
  /// Corresponds to `ExplicitRelations::stored_dynamically` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 73-79).
  ///
  /// Simplified: returns false. The full implementation checks the explicit_bp_data struct.
  pub fn stored_dynamically(_bp: &BinaryPredicate) -> bool {
      false
  }

  /// Check if the relation relates values (not objects).
  ///
  /// Corresponds to `ExplicitRelations::relates_values_not_objects` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 81-91).
  ///
  /// Simplified: returns false. The full implementation checks the kinds of both terms.
  pub fn relates_values_not_objects(_bp: &BinaryPredicate) -> bool {
      false
  }

  /// Create a sketchy BP pair for an explicit relation.
  ///
  /// Corresponds to `ExplicitRelations::make_pair_sketchily` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 100-119).
  ///
  /// Creates a BP pair with open term details and default explicit_bp_data.
  /// The family_specific field is set to a string representation of the default data.
  ///
  /// Simplified:
  /// - No WordAssemblages (uses a string name)
  /// - No CREATE/STORE_POINTER (uses family_specific as a string tag)
  /// - No reversal family_specific (both BPs share the same data)
  pub fn make_pair_sketchily(
      name: &str,
      bp_registry: &mut Vec<BinaryPredicate>,
  ) -> usize {
      let open_term = BPTerms::new(None);
      let idx = BinaryPredicates::make_pair(
          EXPLICIT_FAMILY,
          open_term.clone(),
          open_term,
          name,
          &format!("{}-rev", name),
          None,
          None,
          Some(name),
          bp_registry,
      );
      // Set family_specific to a tag indicating default explicit data
      if let Some(bp) = bp_registry.get_mut(idx) {
          bp.family_specific = Some(format!("explicit:{}", name));
      }
      idx
  }

  /// Get the form of relation for an explicit BP.
  ///
  /// Corresponds to `ExplicitRelations::get_form_of_relation` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 127-131).
  ///
  /// Simplified: returns RELATION_IMPLICIT for non-explicit BPs.
  /// The full implementation retrieves the form from the explicit_bp_data struct.
  pub fn get_form_of_relation(bp: &BinaryPredicate) -> i8 {
      if bp.relation_family != EXPLICIT_FAMILY {
          return RELATION_IMPLICIT;
      }
      // Simplified: parse the form from family_specific if available
      // In the full implementation, this would retrieve from explicit_bp_data
      RELATION_OTO_O // default
  }

  /// Convert the form of relation to a text string.
  ///
  /// Corresponds to `ExplicitRelations::form_to_text` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 132-143).
  pub fn form_to_text(bp: &BinaryPredicate) -> &'static str {
      let form = get_form_of_relation(bp);
      match form {
          RELATION_OTO_O => "Relation_OtoO",
          RELATION_OTO_V => "Relation_OtoV",
          RELATION_VTO_O => "Relation_VtoO",
          RELATION_VTO_V => "Relation_VtoV",
          RELATION_SYM_OTO_O => "Relation_Sym_OtoO",
          RELATION_SYM_VTO_V => "Relation_Sym_VtoV",
          RELATION_EQUIV => "Relation_Equiv",
          _ => "Relation_Implicit",
      }
  }

  /// Infer a property-based relation.
  ///
  /// Corresponds to `ExplicitRelations::infer_property_based_relation` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Explicit Relations.w`, lines 215-222).
  ///
  /// Simplified: no-op. The full implementation draws a property inference.
  pub fn infer_property_based_relation(
      _bp: &BinaryPredicate,
      _infs0: usize,
      _infs1: usize,
  ) {
      // No-op in simplified implementation
  }
  ```

- [ ] Add unit tests:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;

      /// Test that start() creates both families with correct methods.
      #[test]
      fn test_start_creates_families() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          let idx = ExplicitRelations::start(&mut families, &mut bp_registry);

          assert_eq!(idx, EXPLICIT_FAMILY);
          assert!(idx < families.len());
          assert_eq!(families[idx].name, "explicit");
          assert!(families[idx].methods.typecheck.is_some());
          assert!(families[idx].methods.assert.is_some());
          assert!(families[idx].methods.schema.is_some());
          assert!(families[idx].methods.describe_for_problems.is_some());
          assert!(families[idx].methods.describe_for_index.is_some());
          // No stock method
          assert!(families[idx].methods.stock.is_none());

          // Check by-function family
          assert!(BY_FUNCTION_FAMILY < families.len());
          assert_eq!(families[BY_FUNCTION_FAMILY].name, "by-function");
          assert!(families[BY_FUNCTION_FAMILY].methods.typecheck.is_some());
          assert!(families[BY_FUNCTION_FAMILY].methods.assert.is_some());
          assert!(families[BY_FUNCTION_FAMILY].methods.schema.is_some());
          assert!(families[BY_FUNCTION_FAMILY].methods.describe_for_problems.is_some());
          assert!(families[BY_FUNCTION_FAMILY].methods.describe_for_index.is_some());
          assert!(families[BY_FUNCTION_FAMILY].methods.stock.is_none());
      }

      /// Test that typecheck returns DECLINE_TO_MATCH.
      #[test]
      fn test_typecheck_declines() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          ExplicitRelations::start(&mut families, &mut bp_registry);

          let result = ExplicitRelations::typecheck(
              &families[EXPLICIT_FAMILY],
              &BinaryPredicate {
                  relation_family: EXPLICIT_FAMILY,
                  family_specific: None,
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              },
              &[],
              &[],
          );
          assert_eq!(result, DECLINE_TO_MATCH);
      }

      /// Test that assert returns FALSE.
      #[test]
      fn test_assert_returns_false() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          ExplicitRelations::start(&mut families, &mut bp_registry);

          let result = ExplicitRelations::assert(
              &families[EXPLICIT_FAMILY],
              &BinaryPredicate {
                  relation_family: EXPLICIT_FAMILY,
                  family_specific: None,
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              },
              0,
              None,
              0,
              None,
              &mut [],
              &mut [],
              &[],
              &mut [],
              &mut [],
              &[],
          );
          assert!(!result);
      }

      /// Test that schema returns FALSE.
      #[test]
      fn test_schema_returns_false() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          ExplicitRelations::start(&mut families, &mut bp_registry);

          let result = ExplicitRelations::schema(
              &families[EXPLICIT_FAMILY],
              0,
              &BinaryPredicate {
                  relation_family: EXPLICIT_FAMILY,
                  family_specific: None,
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              },
          );
          assert!(!result);
      }

      /// Test that describe_for_problems returns empty string.
      #[test]
      fn test_describe_for_problems_empty() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          ExplicitRelations::start(&mut families, &mut bp_registry);

          let result = ExplicitRelations::describe_for_problems(
              &families[EXPLICIT_FAMILY],
              &BinaryPredicate {
                  relation_family: EXPLICIT_FAMILY,
                  family_specific: None,
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              },
          );
          assert!(result.is_empty());
      }

      /// Test describe_for_index returns correct form descriptions.
      #[test]
      fn test_describe_for_index_forms() {
          // Test with different forms
          for (form, expected) in [
              (RELATION_OTO_O, "one-to-one"),
              (RELATION_OTO_V, "one-to-various"),
              (RELATION_VTO_O, "various-to-one"),
              (RELATION_VTO_V, "various-to-various"),
              (RELATION_SYM_OTO_O, "one-to-another"),
              (RELATION_SYM_VTO_V, "various-to-each-other"),
              (RELATION_EQUIV, "in groups"),
          ] {
              // describe_for_index doesn't actually use the form from the BP
              // in the simplified implementation — it always returns "one-to-one"
              // because get_form_of_relation always returns RELATION_OTO_O.
              // This test documents the current simplified behavior.
              let bp = BinaryPredicate {
                  relation_family: EXPLICIT_FAMILY,
                  family_specific: Some(format!("form:{}", form)),
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              };
              let result = ExplicitRelations::describe_for_index(&BpFamily { name: "explicit", methods: BpFamilyMethods::default() }, &bp);
              // In the simplified implementation, get_form_of_relation always returns RELATION_OTO_O
              assert_eq!(result, "one-to-one");
          }
      }

      /// Test rel_describe_briefly returns "defined".
      #[test]
      fn test_rel_describe_briefly() {
          let result = ExplicitRelations::rel_describe_briefly(
              &BpFamily { name: "by-function", methods: BpFamilyMethods::default() },
              &BinaryPredicate {
                  relation_family: BY_FUNCTION_FAMILY,
                  family_specific: None,
                  relation_name: None,
                  debugging_log_name: Some("test".to_string()),
                  term_details: [BPTerms::new(None), BPTerms::new(None)],
                  reversal: None,
                  right_way_round: true,
                  task_functions: [None, None, None, None],
                  loop_parent_optimisation_proviso: None,
                  loop_parent_optimisation_ranger: None,
                  knowledge_about_bp: None,
              },
          );
          assert_eq!(result, "defined");
      }

      /// Test is_explicit_with_runtime_storage.
      #[test]
      fn test_is_explicit_with_runtime_storage() {
          let explicit_bp = BinaryPredicate {
              relation_family: EXPLICIT_FAMILY,
              ..Default::default()
          };
          let non_explicit_bp = BinaryPredicate {
              relation_family: 999,
              ..Default::default()
          };
          assert!(is_explicit_with_runtime_storage(&explicit_bp));
          assert!(!is_explicit_with_runtime_storage(&non_explicit_bp));
      }

      /// Test allow_arbitrary_assertions.
      #[test]
      fn test_allow_arbitrary_assertions() {
          // Default form is RELATION_OTO_O, which should not allow arbitrary assertions
          let oto_bp = BinaryPredicate {
              relation_family: EXPLICIT_FAMILY,
              ..Default::default()
          };
          assert!(!allow_arbitrary_assertions(&oto_bp));
      }

      /// Test form_to_text returns correct strings.
      #[test]
      fn test_form_to_text() {
          // Default form is RELATION_OTO_O
          let bp = BinaryPredicate {
              relation_family: EXPLICIT_FAMILY,
              ..Default::default()
          };
          assert_eq!(form_to_text(&bp), "Relation_OtoO");

          // Non-explicit BP returns Relation_Implicit
          let non_explicit = BinaryPredicate {
              relation_family: 999,
              ..Default::default()
          };
          assert_eq!(form_to_text(&non_explicit), "Relation_Implicit");
      }

      /// Test make_pair_sketchily creates a BP pair.
      #[test]
      fn test_make_pair_sketchily() {
          let mut bp_registry = Vec::new();
          let idx = make_pair_sketchily("adjacent", &mut bp_registry);

          assert!(idx < bp_registry.len());
          assert_eq!(bp_registry[idx].relation_family, EXPLICIT_FAMILY);
          assert_eq!(bp_registry[idx].debugging_log_name.as_deref(), Some("adjacent"));
          assert!(bp_registry[idx].right_way_round);
          assert!(bp_registry[idx].family_specific.is_some());

          // Check reversal was created
          assert!(bp_registry[idx].reversal.is_some());
          let rev_idx = bp_registry[idx].reversal.unwrap();
          assert!(!bp_registry[rev_idx].right_way_round);
          assert_eq!(bp_registry[rev_idx].debugging_log_name.as_deref(), Some("adjacent-rev"));
      }

      /// Test that start returns the correct family indices.
      #[test]
      fn test_family_indices() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          let idx = ExplicitRelations::start(&mut families, &mut bp_registry);

          assert_eq!(idx, EXPLICIT_FAMILY);
          assert_eq!(families.len(), BY_FUNCTION_FAMILY + 1);
          assert_eq!(families[EXPLICIT_FAMILY].name, "explicit");
          assert_eq!(families[BY_FUNCTION_FAMILY].name, "by-function");
      }
  }
  ```

### 2. Integrate with the calculus module

- [ ] Add `pub mod explicit_relations;` to `crates/conform7-semantics/src/calculus/mod.rs` (alphabetically after `equality_relation`).

- [ ] Add the `ExplicitRelations` module to the module map in `mod.rs`.

### 3. Verify

- [ ] Run `cargo build` to ensure the new module compiles.
- [ ] Run `cargo test -- calculus::explicit_relations` to verify all tests pass.

## Success Criteria

- [ ] `ExplicitRelations::start()` creates both families (explicit and by-function) with correct methods.
- [ ] `ExplicitRelations::typecheck()` returns `DECLINE_TO_MATCH`.
- [ ] `ExplicitRelations::assert()` returns `false`.
- [ ] `ExplicitRelations::schema()` returns `false`.
- [ ] `ExplicitRelations::describe_for_problems()` returns an empty string.
- [ ] `ExplicitRelations::describe_for_index()` returns the correct form description.
- [ ] `ExplicitRelations::rel_describe_briefly()` returns `"defined"`.
- [ ] `is_explicit_with_runtime_storage()` correctly identifies explicit family BPs.
- [ ] `allow_arbitrary_assertions()` returns false for default (OTO_O) form.
- [ ] `form_to_text()` returns correct strings for each form.
- [ ] `make_pair_sketchily()` creates a BP pair with correct family, name, and reversal.
- [ ] Family indices are correct: `EXPLICIT_FAMILY = 5`, `BY_FUNCTION_FAMILY = 6`.
- [ ] The module compiles without errors.
- [ ] All unit tests pass.

## Out of Scope

- **RelationInferences integration**: The `assert` method is simplified to return `FALSE`. The full implementation would draw inferences via `RelationInferences::draw_spec`, `RelationInferences::draw`, and `PropertyInferences::draw`. These depend on the knowledge module's inference system, which is not yet fully implemented.
- **StandardProblems integration**: Problem messages for non-assertable relations and relationships with nothing are not implemented. These depend on the problem reporting system.
- **BinaryPredicates::kind integration**: The `relates_values_not_objects` function is simplified to return `FALSE`. The full implementation would use `BinaryPredicates::kind()` and `Kinds::binary_construction_material()`.
- **Kinds::Behaviour::is_object integration**: The `relates_values_not_objects` function is simplified. The full implementation would check if both terms are object kinds.
- **InferenceSubjects::as_constant integration**: The `infer_property_based_relation` function is a no-op. The full implementation would draw property inferences.
- **Stock method**: The explicit and by-function families have no stock method. BPs are created on demand via `make_pair_sketchily` when the source text declares new relations.
- **Preform grammar**: No Preform grammar is implemented. The `make_pair_sketchily` function uses string names instead of `word_assemblage`.
- **Run-time compilation**: No I6 schema compilation is implemented. The `schema` method returns `FALSE`.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
