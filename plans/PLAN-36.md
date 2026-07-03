# Plan 36: Quasinumeric Relations — The Numerical Comparison Binary Predicate Family
**Status**: Complete
**Target**: 1 day

## Goal

Implement the Quasinumeric Relations system — a binary predicate family for numerical comparisons (`>`, `<`, `>=`, `<=`). This creates the `quasinumeric_bp_family` with its associated stock, typecheck, assert, schema, and describe methods, and the four global binary predicates `R_numerically_greater_than`, `R_numerically_less_than`, `R_numerically_greater_than_or_equal_to`, and `R_numerically_less_than_or_equal_to`.

This is the smallest next step after PLAN-35 because:

1. **It's the next module in the assertions module startup sequence.** The assertions module startup (`inform7/assertions-module/Chapter 1/Assertions Module.w`, lines 25-37) calls `Calculus::QuasinumericRelations::start()` at line 27 — right after `CreationPredicates::start()` at line 26 (PLAN-35, Complete). This is the natural next step in the startup sequence.

2. **It depends only on calculus infrastructure that's already built.** The `QuasinumericRelations` module uses `BinaryPredicateFamilies::new()` (PLAN-21, Complete), `BinaryPredicates::make_pair()` (PLAN-21, Complete), `BPTerms::new()` (PLAN-21, Complete), and `KindSubjects::from_kind()` (knowledge module, Complete) — all of which already exist. No knowledge module dependencies, no Preform grammar, no run-time compilation.

3. **It's the smallest remaining independent module in the startup sequence.** At only ~180 lines of C (`Quasinumeric Relations.w`), this is one of the simplest bp_family modules not yet implemented. It creates one family with four binary predicates and straightforward methods.

4. **It's a prerequisite for the rest of the assertions module startup.** The `Relations::Universal::start()` (line 28), `ExplicitRelations::start()` (line 29), `EqualityDetails::start()` (line 30), and `KindPredicatesRevisited::start()` (line 31) all follow the same bp_family pattern. Implementing QuasinumericRelations first establishes the pattern for the remaining startup items.

5. **It's independently testable without grammar parsing, the knowledge module, or run-time compilation.** We can create the family via `QuasinumericRelations::start()`, stock the four BPs, test the typecheck method (simplified: always returns ALWAYS_MATCH), test the assert method (returns FALSE), test the describe methods, and verify the BP properties — all programmatically. No Preform grammar, no kind system compatibility checks, no run-time compilation.

6. **It introduces the numerical comparison pattern — a fundamental relation family.** The four inequality relations (`>`, `<`, `>=`, `<=`) are used throughout Inform for numerical comparisons. Implementing them now establishes the pattern for the remaining bp_family modules in the startup sequence.

## Background

### C reference architecture

#### Quasinumeric Relations (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 1-178)

The Quasinumeric Relations system provides four binary predicates for numerical comparisons:

```c
binary_predicate *R_numerically_greater_than = NULL;
binary_predicate *R_numerically_less_than = NULL;
binary_predicate *R_numerically_greater_than_or_equal_to = NULL;
binary_predicate *R_numerically_less_than_or_equal_to = NULL;
```

The `start()` function creates the family and adds methods:

```c
bp_family *quasinumeric_bp_family = NULL;

void Calculus::QuasinumericRelations::start(void) {
    quasinumeric_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(quasinumeric_bp_family, STOCK_BPF_MTID, Calculus::QuasinumericRelations::stock);
    METHOD_ADD(quasinumeric_bp_family, TYPECHECK_BPF_MTID, Calculus::QuasinumericRelations::typecheck);
    METHOD_ADD(quasinumeric_bp_family, ASSERT_BPF_MTID, Calculus::QuasinumericRelations::assert);
    METHOD_ADD(quasinumeric_bp_family, SCHEMA_BPF_MTID, Calculus::QuasinumericRelations::schema);
    METHOD_ADD(quasinumeric_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID, Calculus::QuasinumericRelations::describe_for_problems);
    METHOD_ADD(quasinumeric_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID, Calculus::QuasinumericRelations::describe_for_index);
}
```

The stock method creates four binary predicates with number term details and test schemas:

```c
void Calculus::QuasinumericRelations::stock(bp_family *self, int n) {
    if (n == 1) {
        bp_term_details number_term = BPTerms::new(KindSubjects::from_kind(K_number));
        R_numerically_greater_than =
            BinaryPredicates::make_pair(quasinumeric_bp_family,
                number_term, number_term,
                I"greater-than", NULL, NULL, Calculus::Schemas::new("*1 > *2"),
                PreformUtilities::wording(<relation-names>, GT_RELATION_NAME));
        R_numerically_less_than =
            BinaryPredicates::make_pair(quasinumeric_bp_family,
                number_term, number_term,
                I"less-than", NULL, NULL, Calculus::Schemas::new("*1 < *2"),
                PreformUtilities::wording(<relation-names>, LT_RELATION_NAME));
        R_numerically_greater_than_or_equal_to =
            BinaryPredicates::make_pair(quasinumeric_bp_family,
                number_term, number_term,
                I"at-least", NULL, NULL, Calculus::Schemas::new("*1 >= *2"),
                PreformUtilities::wording(<relation-names>, GE_RELATION_NAME));
        R_numerically_less_than_or_equal_to =
            BinaryPredicates::make_pair(quasinumeric_bp_family,
                number_term, number_term,
                I"at-most", NULL, NULL, Calculus::Schemas::new("*1 <= *2"),
                PreformUtilities::wording(<relation-names>, LE_RELATION_NAME));
        BinaryPredicates::set_index_details(R_numerically_greater_than,
            "arithmetic value", "arithmetic value");
        BinaryPredicates::set_index_details(R_numerically_less_than,
            "arithmetic value", "arithmetic value");
        BinaryPredicates::set_index_details(R_numerically_greater_than_or_equal_to,
            "arithmetic value", "arithmetic value");
        BinaryPredicates::set_index_details(R_numerically_less_than_or_equal_to,
            "arithmetic value", "arithmetic value");
    }
}
```

The typecheck method checks that the two kinds are compatible:

```c
int Calculus::QuasinumericRelations::typecheck(bp_family *self, binary_predicate *bp,
        kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    if ((Kinds::compatible(kinds_of_terms[0], kinds_of_terms[1]) == NEVER_MATCH) &&
        (Kinds::compatible(kinds_of_terms[1], kinds_of_terms[0]) == NEVER_MATCH)) {
        if (tck->log_to_I6_text)
            LOG("Unable to apply inequality of %u and %u\n", kinds_of_terms[0], kinds_of_terms[1]);
        Problems::quote_kind(4, kinds_of_terms[0]);
        Problems::quote_kind(5, kinds_of_terms[1]);
        StandardProblems::tcp_problem(_p_(PM_InequalityFailed), tck,
            "that would mean comparing two kinds of value which cannot mix - "
            "%4 and %5 - so this must be incorrect.");
        return NEVER_MATCH;
    }
    return ALWAYS_MATCH;
}
```

The assert method returns FALSE (these relations cannot be asserted):

```c
int Calculus::QuasinumericRelations::assert(bp_family *self, binary_predicate *bp,
        inference_subject *infs0, parse_node *spec0,
        inference_subject *infs1, parse_node *spec1) {
    return FALSE;
}
```

The schema method handles integer/real number comparisons (simplified: returns FALSE for default schema):

```c
int Calculus::QuasinumericRelations::schema(bp_family *self, int task, binary_predicate *bp, annotated_i6_schema *asch) {
    // ... handles floating-point promotion and comparison routines ...
    return FALSE;
}
```

The describe methods:

```c
int Calculus::QuasinumericRelations::describe_for_problems(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    return FALSE;
}
void Calculus::QuasinumericRelations::describe_for_index(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    WRITE("numeric");
}
```

### Key C source files

- `inform7/assertions-module/Chapter 8/Quasinumeric Relations.w` — `Calculus::QuasinumericRelations` module, `quasinumeric_bp_family`, `R_numerically_greater_than`, `R_numerically_less_than`, `R_numerically_greater_than_or_equal_to`, `R_numerically_less_than_or_equal_to`, `start`, `stock`, `typecheck`, `assert`, `schema`, `describe_for_problems`, `describe_for_index` (178 lines)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `BinaryPredicateFamilies::new`, `BpFamily` struct, method dispatch (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `BinaryPredicates::make_pair`, `BinaryPredicate` struct (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `BPTerms::new`, `BpTermDetails` struct (PLAN-21, Complete)
- `inform7/knowledge-module/Chapter 4/Kind Subjects.w` — `KindSubjects::from_kind` (Complete)
- `inform7/assertions-module/Chapter 1/Assertions Module.w` — module startup, calls `Calculus::QuasinumericRelations::start()` (line 27)

### Current Rust state

- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct (with `stock`, `typecheck`, `assert`, `schema`, `describe_for_problems`, `describe_for_index` methods), `BinaryPredicateFamilies::new()`, `BinaryPredicateFamilies::stock()`, `BinaryPredicateFamilies::typecheck()`, `BinaryPredicateFamilies::assert()`, `BinaryPredicateFamilies::schema()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `relation_family`, `family_specific`, `relation_name`, `debugging_log_name`, `term_details`, `reversal`, `right_way_round`, `task_functions` fields; `BinaryPredicates::make_pair()`, `BinaryPredicates::make_single()`, `BinaryPredicates::make_equality()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct with `implies_infs`, `implies_kind`, `called_name`, `function_of_other`, `index_term_as` fields; `BPTerms::new()`, `BPTerms::new_kind()`, `BPTerms::new_full()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families (equality, spatial, empty), `EQUALITY_FAMILY`, `SPATIAL_FAMILY`, `EMPTY_FAMILY` constants, `R_EQUALITY`, `A_HAS_B_PREDICATE`, `R_EMPTY` constants, `EqualityRelation::start()`, `EqualityRelation::stock()`, `EqualityRelation::stock_spatial()`, `EqualityRelation::stock_empty()`, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/calculus/creation_predicates.rs` — `CreationPredicates` module, `CALLING_UP_FAMILY`, `IS_A_VAR_UP_FAMILY`, `IS_A_CONST_UP_FAMILY`, `IS_A_KIND_UP_FAMILY` statics, `CreationPredicates::is_calling_up_atom()`, `CreationPredicates::calling_up()`, `CreationPredicates::get_calling_name()`, `CreationPredicates::what_kind_of_calling()`, `CreationPredicates::is_a_var_up()`, `CreationPredicates::is_a_const_up()`, `CreationPredicates::is_a_kind_up()`, `CreationPredicates::what_kind()`, `CreationPredicates::contains_callings()`, unit tests (PLAN-35, Complete).
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules (includes `pub mod creation_predicates;` from PLAN-35).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects::from_kind()` returns the inference subject index for a kind (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/familiar_kinds.rs` — `K_number`, `K_object`, `K_real_number`, `K_text`, `K_truth_state`, `K_table`, `CON_NUMBER`, `CON_OBJECT`, `CON_REAL_NUMBER`, etc. (Complete).
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with `base_construction()`, `unary_con()`, `binary_con()`, `get_construct()`, `arity()`, etc. (Complete).

### What's needed

1. **`QuasinumericRelations` module** — a new module `quasinumeric_relations` in the calculus crate with:
   - `QuasinumericRelations::start(families, bp_registry, constructors)` — creates the quasinumeric bp_family with methods:
     - Stock method creates 4 binary predicates at stock stage 1
     - Typecheck method (simplified: always returns ALWAYS_MATCH)
     - Assert method returns FALSE (cannot be asserted)
     - Schema method returns FALSE (use default schema)
     - describe_for_problems returns FALSE (no special problem description)
     - describe_for_index returns "numeric"
   - Global constants for the family and predicate indices:
     - `QUASINUMERIC_FAMILY` — index of the quasinumeric family in the family registry
     - `R_NUMERICALLY_GREATER_THAN` — index of the greater-than BP
     - `R_NUMERICALLY_LESS_THAN` — index of the less-than BP
     - `R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO` — index of the greater-than-or-equal-to BP
     - `R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO` — index of the less-than-or-equal-to BP
   - Helper functions:
     - `QuasinumericRelations::is_quasinumeric_bp(bp_idx, bp_registry)` — checks if a BP belongs to the quasinumeric family
   - Simplified: no `PreformUtilities::wording` (uses string names for relation names)
   - Simplified: no `Kinds::compatible` (typecheck always returns ALWAYS_MATCH)
   - Simplified: no `Kinds::FloatingPoint::uses_floating_point` (schema returns FALSE)
   - Simplified: no `Kinds::Behaviour::get_comparison_routine` (schema returns FALSE)
   - Simplified: no `Problems::quote_kind` (no problem messages)
   - Simplified: no `StandardProblems::tcp_problem` (no problem messages)
   - Simplified: no `BinaryPredicates::set_index_details` (index details deferred)
   - Simplified: no `Calculus::Schemas::new` (uses `Option<&str>` in `make_pair`)

2. **Integration with the calculus module** — add the `quasinumeric_relations` module declaration to the calculus module's `mod.rs`.

3. **Unit tests** — test `QuasinumericRelations::start()` (creates the family with correct methods), test stock (creates 4 BPs with correct names and test schemas), test typecheck (returns ALWAYS_MATCH), test assert (returns FALSE), test schema (returns FALSE), test describe_for_problems (returns FALSE), test describe_for_index (returns "numeric"), test `is_quasinumeric_bp` (returns true for quasinumeric BPs, false otherwise).

## Tasks

### 1. Create the `QuasinumericRelations` module

- [ ] Create `crates/conform7-semantics/src/calculus/quasinumeric_relations.rs` with:

  ```rust
  /// The Quasinumeric Relations system — binary predicates for numerical comparisons.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`).
  ///
  /// Creates a quasinumeric bp_family with four binary predicates:
  /// - R_numerically_greater_than (>)
  /// - R_numerically_less_than (<)
  /// - R_numerically_greater_than_or_equal_to (>=)
  /// - R_numerically_less_than_or_equal_to (<=)
  ///
  /// These relations can be applied not only to numbers but also to units
  /// (height, length, etc.). The inequality relations are used throughout
  /// Inform for numerical comparisons.
  ///
  /// Simplified:
  /// - No PreformUtilities::wording (uses string names)
  /// - No Kinds::compatible (typecheck always returns ALWAYS_MATCH)
  /// - No Kinds::FloatingPoint::uses_floating_point (schema returns FALSE)
  /// - No Kinds::Behaviour::get_comparison_routine (schema returns FALSE)
  /// - No Problems::quote_kind (no problem messages)
  /// - No StandardProblems::tcp_problem (no problem messages)
  /// - No BinaryPredicates::set_index_details (deferred)
  /// - No Calculus::Schemas::new (uses Option<&str> in make_pair)
  use crate::calculus::binary_predicate_families::{
      BpFamily, BpFamilyMethods, BinaryPredicateFamilies,
  };
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  use crate::knowledge::kind_subjects;
  use crate::kinds::Kind;
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the quasinumeric family in the family registry.
  pub const QUASINUMERIC_FAMILY: usize = 3;

  /// Index of the greater-than predicate in the BP registry.
  ///
  /// Created by `QuasinumericRelations::stock()` during first stock.
  pub const R_NUMERICALLY_GREATER_THAN: usize = 4;

  /// Index of the less-than predicate in the BP registry.
  ///
  /// Created by `QuasinumericRelations::stock()` during first stock.
  pub const R_NUMERICALLY_LESS_THAN: usize = 5;

  /// Index of the greater-than-or-equal-to predicate in the BP registry.
  ///
  /// Created by `QuasinumericRelations::stock()` during first stock.
  pub const R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO: usize = 6;

  /// Index of the less-than-or-equal-to predicate in the BP registry.
  ///
  /// Created by `QuasinumericRelations::stock()` during first stock.
  pub const R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO: usize = 7;
  ```

  Note: These constants assume the equality relation (PLAN-22) has already created families 0-2 and BPs 0-3. The quasinumeric family is family 3, and its BPs start at index 4.

- [ ] Define the `QuasinumericRelations` struct:

  ```rust
  /// The quasinumeric relations module.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`).
  pub struct QuasinumericRelations;
  ```

- [ ] Implement `QuasinumericRelations::start()`:

  ```rust
  /// Create the quasinumeric bp_family with its methods.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::start` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 29-37).
  ///
  /// # Arguments
  ///
  /// * `families` - The family registry to add to.
  /// * `bp_registry` - The BP registry to add to.
  /// * `constructors` - The kind constructors (for KindSubjects::from_kind).
  ///
  /// # Returns
  ///
  /// The index of the created family in the registry.
  pub fn start(
      families: &mut Vec<BpFamily>,
      bp_registry: &mut Vec<BinaryPredicate>,
      constructors: &[crate::kind_constructors::KindConstructor],
  ) -> usize {
      let family_idx = families.len();
      let family = BpFamily {
          name: "quasinumeric",
          methods: BpFamilyMethods {
              stock: Some(QuasinumericRelations::stock),
              typecheck: Some(QuasinumericRelations::typecheck),
              assert: Some(QuasinumericRelations::assert),
              schema: Some(QuasinumericRelations::schema),
              describe_for_problems: Some(QuasinumericRelations::describe_for_problems),
              describe_for_index: Some(QuasinumericRelations::describe_for_index),
              ..BpFamilyMethods::default()
          },
      };
      families.push(family);

      // Stock the four BPs at stage 1
      QuasinumericRelations::stock(&families[family_idx], 1, bp_registry, constructors);

      family_idx
  }
  ```

- [ ] Implement `QuasinumericRelations::stock()`:

  ```rust
  /// Stock the four quasinumeric binary predicates.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::stock` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 43-75).
  ///
  /// Creates four binary predicates at stock stage 1:
  /// - R_numerically_greater_than with test schema "*1 > *2"
  /// - R_numerically_less_than with test schema "*1 < *2"
  /// - R_numerically_greater_than_or_equal_to with test schema "*1 >= *2"
  /// - R_numerically_less_than_or_equal_to with test schema "*1 <= *2"
  ///
  /// Simplified:
  /// - No PreformUtilities::wording (uses string names)
  /// - No BinaryPredicates::set_index_details (deferred)
  pub fn stock(
      _family: &BpFamily,
      n: u8,
      bp_registry: &mut Vec<BinaryPredicate>,
      constructors: &[crate::kind_constructors::KindConstructor],
  ) {
      if n == 1 {
          // Create term details for the number kind
          let k_number = Kind::base_construction(
              &crate::familiar_kinds::CON_NUMBER,
          );
          let number_infs = kind_subjects::from_kind(&k_number, constructors);
          let number_term = BPTerms::new(number_infs);

          // R_numerically_greater_than: *1 > *2
          BinaryPredicates::make_pair(
              QUASINUMERIC_FAMILY,
              number_term.clone(),
              number_term.clone(),
              "greater-than",
              "greater-than-rev",
              None,
              Some("*1 > *2"),
              Some("greater-than"),
              bp_registry,
          );

          // R_numerically_less_than: *1 < *2
          BinaryPredicates::make_pair(
              QUASINUMERIC_FAMILY,
              number_term.clone(),
              number_term.clone(),
              "less-than",
              "less-than-rev",
              None,
              Some("*1 < *2"),
              Some("less-than"),
              bp_registry,
          );

          // R_numerically_greater_than_or_equal_to: *1 >= *2
          BinaryPredicates::make_pair(
              QUASINUMERIC_FAMILY,
              number_term.clone(),
              number_term.clone(),
              "at-least",
              "at-least-rev",
              None,
              Some("*1 >= *2"),
              Some("at-least"),
              bp_registry,
          );

          // R_numerically_less_than_or_equal_to: *1 <= *2
          BinaryPredicates::make_pair(
              QUASINUMERIC_FAMILY,
              number_term,
              number_term,
              "at-most",
              "at-most-rev",
              None,
              Some("*1 <= *2"),
              Some("at-most"),
              bp_registry,
          );
      }
  }
  ```

- [ ] Implement `QuasinumericRelations::typecheck()`:

  ```rust
  /// Typecheck the terms of a quasinumeric relation.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::typecheck` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 80-94).
  ///
  /// Simplified: always returns ALWAYS_MATCH (no kind compatibility checking yet).
  pub fn typecheck(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      1 // ALWAYS_MATCH
  }
  ```

- [ ] Implement `QuasinumericRelations::assert()`:

  ```rust
  /// Assert a quasinumeric relation.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::assert` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 100-104).
  ///
  /// These relations cannot be asserted — they are for run-time testing only.
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

- [ ] Implement `QuasinumericRelations::schema()`:

  ```rust
  /// Compile run-time code for a quasinumeric relation.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::schema` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 112-167).
  ///
  /// Simplified: returns FALSE (use default schema). The full implementation
  /// handles floating-point promotion and comparison routines.
  pub fn schema(
      _family: &BpFamily,
      _task: u8,
      _bp: &BinaryPredicate,
  ) -> bool {
      false
  }
  ```

- [ ] Implement `QuasinumericRelations::describe_for_problems()`:

  ```rust
  /// Describe the relation in problem messages.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::describe_for_problems` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 172-174).
  ///
  /// Returns FALSE (no special problem description needed).
  pub fn describe_for_problems(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
  ) -> String {
      String::new()
  }
  ```

- [ ] Implement `QuasinumericRelations::describe_for_index()`:

  ```rust
  /// Describe the relation in the Phrasebook index.
  ///
  /// Corresponds to `Calculus::QuasinumericRelations::describe_for_index` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Quasinumeric Relations.w`, lines 175-177).
  pub fn describe_for_index(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
  ) -> String {
      "numeric".to_string()
  }
  ```

- [ ] Implement `QuasinumericRelations::is_quasinumeric_bp()`:

  ```rust
  /// Check if a binary predicate belongs to the quasinumeric family.
  ///
  /// Corresponds to checking `bp->relation_family == quasinumeric_bp_family`
  /// in the C reference.
  pub fn is_quasinumeric_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
      bp_registry
          .get(bp_idx)
          .map(|bp| bp.relation_family == QUASINUMERIC_FAMILY)
          .unwrap_or(false)
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod quasinumeric_relations;` to `crates/conform7-semantics/src/calculus/mod.rs`.

### 3. Unit tests

- [ ] Test `QuasinumericRelations::start()` — creates the family with correct methods (stock, typecheck, assert, schema, describe_for_problems, describe_for_index).
- [ ] Test stock creates 4 BPs — verify `R_NUMERICALLY_GREATER_THAN`, `R_NUMERICALLY_LESS_THAN`, `R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO`, `R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO` are created with correct names and test schemas.
- [ ] Test each BP's test schema — `R_numerically_greater_than` has `"*1 > *2"`, `R_numerically_less_than` has `"*1 < *2"`, `R_numerically_greater_than_or_equal_to` has `"*1 >= *2"`, `R_numerically_less_than_or_equal_to` has `"*1 <= *2"`.
- [ ] Test each BP's term details — all four BPs have number kind term details.
- [ ] Test each BP's reversal — each BP has a reversal with swapped term details.
- [ ] Test typecheck returns ALWAYS_MATCH (1).
- [ ] Test assert returns FALSE.
- [ ] Test schema returns FALSE.
- [ ] Test describe_for_problems returns empty string.
- [ ] Test describe_for_index returns "numeric".
- [ ] Test `is_quasinumeric_bp` — returns true for quasinumeric BPs, false for non-quasinumeric BPs.
- [ ] Test that stock at stage 2 does nothing (no additional BPs created).

## Success Criteria

- [ ] `QuasinumericRelations::start()` creates the quasinumeric bp_family with all six methods (stock, typecheck, assert, schema, describe_for_problems, describe_for_index).
- [ ] Stock creates exactly 4 binary predicates with correct names and test schemas.
- [ ] Each BP has the correct term details (number kind domain).
- [ ] Each BP has a reversal with swapped term details.
- [ ] Typecheck returns ALWAYS_MATCH (1) for all inputs.
- [ ] Assert returns FALSE (cannot be asserted).
- [ ] Schema returns FALSE (use default schema).
- [ ] Describe_for_problems returns empty string.
- [ ] Describe_for_index returns "numeric".
- [ ] `is_quasinumeric_bp` correctly identifies quasinumeric BPs.
- [ ] All unit tests pass.
- [ ] The module is declared in `calculus/mod.rs`.

## Out of Scope

- **Kind compatibility checking in typecheck**: The full C implementation checks `Kinds::compatible` to reject incompatible kind comparisons. This is deferred until the kind system's compatibility checking is implemented.
- **Floating-point schema handling**: The full C implementation handles floating-point promotion and comparison routines. This is deferred until the floating-point kind system is implemented.
- **`BinaryPredicates::set_index_details`**: The C reference sets index details for each BP. This is deferred.
- **`PreformUtilities::wording`**: The C reference uses Preform grammar for relation names. This is deferred — we use string names instead.
- **Problem messages**: The full C implementation issues problem messages for typecheck failures. This is deferred.
- **`Calculus::Schemas` module**: The full schema system with `i6_schema` structs and format-string processing is not yet implemented. We use `Option<&str>` in `make_pair` for test schemas.
- **Other assertions module startup items**: `Relations::Universal::start()`, `ExplicitRelations::start()`, `EqualityDetails::start()`, `KindPredicatesRevisited::start()`, `ImperativeDefinitionFamilies::create()`, and the adjective-by-* modules are all deferred to future plans.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
