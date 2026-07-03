# Plan 37: The Universal Relation — The Universal and Meaning Binary Predicate Family
**Status**: Complete
**Target**: 1 day

## Goal

Implement the Universal Relation system — a binary predicate family for the universal relation ("relates") and the meaning relation ("means"). This creates the `universal_bp_family` with its associated stock, typecheck, assert, schema, and describe_for_problems methods, and the two global binary predicates `R_universal` and `R_meaning`.

This is the smallest next step after PLAN-36 because:

1. **It's the next module in the assertions module startup sequence.** The assertions module startup (`inform7/assertions-module/Chapter 1/Assertions Module.w`, lines 25-37) calls `Relations::Universal::start()` at line 29 — right after `Calculus::QuasinumericRelations::start()` at line 28 (PLAN-36, Complete). This is the natural next step in the startup sequence.

2. **It depends only on calculus infrastructure that's already built.** The `Relations::Universal` module uses `BinaryPredicateFamilies::new()` (PLAN-21, Complete), `BinaryPredicates::make_pair()` (PLAN-21, Complete), and `BPTerms::new()` (PLAN-21, Complete) — all of which already exist. No knowledge module dependencies, no Preform grammar, no run-time compilation.

3. **It's the smallest remaining independent module in the startup sequence.** At only ~160 lines of C (`The Universal Relation.w`), this is one of the simplest bp_family modules not yet implemented. It creates one family with two binary predicates and straightforward methods.

4. **It's a prerequisite for the rest of the assertions module startup.** The `ExplicitRelations::start()` (line 30), `EqualityDetails::start()` (line 31), and `KindPredicatesRevisited::start()` (line 32) all follow the same bp_family pattern. Implementing the Universal Relation first establishes the pattern for the remaining startup items.

5. **It's independently testable without grammar parsing, the knowledge module, or run-time compilation.** We can create the family via `Relations::Universal::start()`, stock the two BPs, test the typecheck method (simplified: always returns ALWAYS_MATCH), test the assert method (returns FALSE), test the schema method (returns FALSE), and test the describe methods — all programmatically. No Preform grammar, no kind system compatibility checks, no run-time compilation.

6. **It introduces the universal relation pattern — a fundamental relation family.** The universal relation (`R_universal`) is the most general relation: it can apply to any two things and subsumes all other relations. The meaning relation (`R_meaning`) relates a verb to its meaning. These are used throughout Inform for relation-level operations.

## Background

### C reference architecture

#### The Universal Relation (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 1-158)

The Universal Relation system provides two binary predicates for relation-level operations:

```c
binary_predicate *R_universal = NULL;
binary_predicate *R_meaning = NULL;
```

The `start()` function creates the family and adds methods:

```c
bp_family *universal_bp_family = NULL;

void Relations::Universal::start(void) {
    universal_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(universal_bp_family, STOCK_BPF_MTID, Relations::Universal::stock);
    METHOD_ADD(universal_bp_family, TYPECHECK_BPF_MTID, Relations::Universal::typecheck);
    METHOD_ADD(universal_bp_family, ASSERT_BPF_MTID, Relations::Universal::assert);
    METHOD_ADD(universal_bp_family, SCHEMA_BPF_MTID, Relations::Universal::schema);
    METHOD_ADD(universal_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID, Relations::Universal::describe_for_problems);
}
```

The stock method creates two binary predicates with open term details (no kind restriction):

```c
void Relations::Universal::stock(bp_family *self, int n) {
    if (n == 1) {
        R_universal =
            BinaryPredicates::make_pair(universal_bp_family,
                BPTerms::new(NULL), BPTerms::new(NULL),
                I"relates", NULL, NULL, NULL,
                PreformUtilities::wording(<relation-names>, UNIVERSAL_RELATION_NAME));
        R_meaning =
            BinaryPredicates::make_pair(universal_bp_family,
                BPTerms::new(NULL), BPTerms::new(NULL),
                I"means", NULL, NULL, NULL,
                PreformUtilities::wording(<relation-names>, MEANING_RELATION_NAME));
    }
}
```

The typecheck method checks kinds for both BPs:

```c
int Relations::Universal::typecheck(bp_family *self, binary_predicate *bp,
        kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    if (bp == R_meaning) {
        if (Kinds::eq(kinds_of_terms[0], K_verb) == FALSE) {
            // Problem: first term must be a verb
            return NEVER_MATCH;
        }
        if (Kinds::get_construct(kinds_of_terms[1]) != CON_relation) {
            // Problem: second term must be a relation
            return NEVER_MATCH;
        }
    } else {
        if (Kinds::get_construct(kinds_of_terms[0]) != CON_relation) {
            // Problem: first term must be a relation
            return NEVER_MATCH;
        }
        if (Kinds::get_construct(kinds_of_terms[1]) != CON_combination) {
            // Problem: second term must be a combination
            return NEVER_MATCH;
        }
        // Check that the relation's domain matches the combination's components
        kind *rleft = NULL, *rright = NULL;
        Kinds::binary_construction_material(kinds_of_terms[0], &rleft, &rright);
        kind *cleft = NULL, *cright = NULL;
        Kinds::binary_construction_material(kinds_of_terms[1], &cleft, &cright);
        if (Kinds::compatible(cleft, rleft) == NEVER_MATCH) {
            return NEVER_MATCH;
        }
        if (Kinds::compatible(cright, rright) == NEVER_MATCH) {
            return NEVER_MATCH;
        }
    }
    return ALWAYS_MATCH;
}
```

The assert method returns FALSE (these relations cannot be asserted):

```c
int Relations::Universal::assert(bp_family *self, binary_predicate *bp,
        inference_subject *infs0, parse_node *spec0,
        inference_subject *infs1, parse_node *spec1) {
    return FALSE;
}
```

The schema method handles run-time compilation tasks:

```c
int Relations::Universal::schema(bp_family *self, int task, binary_predicate *bp,
        annotated_i6_schema *asch) {
    if (bp == R_meaning) {
        switch(task) {
            case TEST_ATOM_TASK:
                Calculus::Schemas::modify(asch->schema, "*=-(BlkValueCompare(*1(CV_MEANING), *2)==0)");
                return TRUE;
        }
    } else {
        switch(task) {
            case TEST_ATOM_TASK:
                Calculus::Schemas::modify(asch->schema, "*=-((RlnGetF(*1, RR_HANDLER))(*1, RELS_TEST, *&))");
                return TRUE;
            case NOW_ATOM_TRUE_TASK:
                Calculus::Schemas::modify(asch->schema, "*=-((RlnGetF(*1, RR_HANDLER))(*1, RELS_ASSERT_TRUE, *&))");
                return TRUE;
            case NOW_ATOM_FALSE_TASK:
                Calculus::Schemas::modify(asch->schema, "*=-((RlnGetF(*1, RR_HANDLER))(*1, RELS_ASSERT_FALSE, *&))");
                return TRUE;
        }
    }
    return FALSE;
}
```

The describe method:

```c
int Relations::Universal::describe_for_problems(bp_family *self, OUTPUT_STREAM, binary_predicate *bp) {
    return FALSE;
}
```

### Key C source files

- `inform7/assertions-module/Chapter 8/The Universal Relation.w` — `Relations::Universal` module, `universal_bp_family`, `R_universal`, `R_meaning`, `start`, `stock`, `typecheck`, `assert`, `schema`, `describe_for_problems` (158 lines)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `BinaryPredicateFamilies::new`, `BpFamily` struct, method dispatch (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `BinaryPredicates::make_pair`, `BinaryPredicate` struct (PLAN-21, Complete)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `BPTerms::new`, `BpTermDetails` struct (PLAN-21, Complete)
- `inform7/assertions-module/Chapter 1/Assertions Module.w` — module startup, calls `Relations::Universal::start()` (line 29)

### Current Rust state

- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct (with `stock`, `typecheck`, `assert`, `schema`, `describe_for_problems`, `describe_for_index` methods), `BinaryPredicateFamilies::new()`, `BinaryPredicateFamilies::stock()`, `BinaryPredicateFamilies::typecheck()`, `BinaryPredicateFamilies::assert()`, `BinaryPredicateFamilies::schema()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `relation_family`, `family_specific`, `relation_name`, `debugging_log_name`, `term_details`, `reversal`, `right_way_round`, `task_functions` fields; `BinaryPredicates::make_pair()`, `BinaryPredicates::make_single()`, `BinaryPredicates::make_equality()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct with `implies_infs`, `implies_kind`, `called_name`, `function_of_other`, `index_term_as` fields; `BPTerms::new()`, `BPTerms::new_kind()`, `BPTerms::new_full()`, unit tests (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families (equality, spatial, empty), `EQUALITY_FAMILY`, `SPATIAL_FAMILY`, `EMPTY_FAMILY` constants, `R_EQUALITY`, `A_HAS_B_PREDICATE`, `R_EMPTY` constants, `EqualityRelation::start()`, `EqualityRelation::stock()`, `EqualityRelation::stock_spatial()`, `EqualityRelation::stock_empty()`, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/calculus/quasinumeric_relations.rs` — `QuasinumericRelations` module, `QUASINUMERIC_FAMILY`, `R_NUMERICALLY_GREATER_THAN`, `R_NUMERICALLY_LESS_THAN`, `R_NUMERICALLY_GREATER_THAN_OR_EQUAL_TO`, `R_NUMERICALLY_LESS_THAN_OR_EQUAL_TO` constants, `QuasinumericRelations::start()`, `QuasinumericRelations::stock()`, `QuasinumericRelations::typecheck()`, `QuasinumericRelations::assert()`, `QuasinumericRelations::schema()`, `QuasinumericRelations::describe_for_problems()`, `QuasinumericRelations::describe_for_index()`, `QuasinumericRelations::is_quasinumeric_bp()`, unit tests (PLAN-36, Complete).
- `crates/conform7-semantics/src/calculus/creation_predicates.rs` — `CreationPredicates` module, `CALLING_FAMILY`, `IS_A_VAR_FAMILY`, `IS_A_CONST_FAMILY`, `IS_A_KIND_FAMILY` constants, `CreationPredicates::start()`, helper functions, unit tests (PLAN-35, Complete).
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules (includes `pub mod quasinumeric_relations;` from PLAN-36).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects::from_kind()` returns the inference subject index for a kind (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/familiar_kinds.rs` — `K_number`, `K_object`, `K_real_number`, `K_text`, `K_truth_state`, `K_table`, `CON_NUMBER`, `CON_OBJECT`, `CON_REAL_NUMBER`, etc. (Complete).
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with `base_construction()`, `unary_con()`, `binary_con()`, `get_construct()`, `arity()`, etc. (Complete).

### What's needed

1. **`UniversalRelation` module** — a new module `universal_relation` in the calculus crate with:
   - `UniversalRelation::start(families, bp_registry)` — creates the universal bp_family with methods:
     - Stock method creates 2 binary predicates at stock stage 1
     - Typecheck method (simplified: always returns ALWAYS_MATCH)
     - Assert method returns FALSE (cannot be asserted)
     - Schema method returns FALSE (use default schema)
     - describe_for_problems returns FALSE (no special problem description)
   - Global constants for the family and predicate indices:
     - `UNIVERSAL_FAMILY` — index of the universal family in the family registry
     - `R_UNIVERSAL` — index of the universal relation BP
     - `R_MEANING` — index of the meaning relation BP
   - Helper functions:
     - `UniversalRelation::is_universal_bp(bp_idx, bp_registry)` — checks if a BP belongs to the universal family
     - `UniversalRelation::is_meaning_bp(bp_idx, bp_registry)` — checks if a BP is the meaning relation
   - Simplified: no `PreformUtilities::wording` (uses string names for relation names)
   - Simplified: no `Kinds::eq` (typecheck always returns ALWAYS_MATCH)
   - Simplified: no `Kinds::get_construct` (typecheck always returns ALWAYS_MATCH)
   - Simplified: no `Kinds::binary_construction_material` (typecheck always returns ALWAYS_MATCH)
   - Simplified: no `Kinds::compatible` (typecheck always returns ALWAYS_MATCH)
   - Simplified: no `Problems::quote_kind` (no problem messages)
   - Simplified: no `StandardProblems::tcp_problem` (no problem messages)
   - Simplified: no `Calculus::Schemas::modify` (schema returns FALSE)

2. **Integration with the calculus module** — add the `universal_relation` module declaration to the calculus module's `mod.rs`.

3. **Unit tests** — test `UniversalRelation::start()` (creates the family with correct methods), test stock (creates 2 BPs with correct names), test typecheck (returns ALWAYS_MATCH), test assert (returns FALSE), test schema (returns FALSE), test describe_for_problems (returns FALSE), test `is_universal_bp` (returns true for universal BPs, false otherwise), test `is_meaning_bp` (returns true for meaning BP, false otherwise).

## Tasks

### 1. Create the `UniversalRelation` module

- [ ] Create `crates/conform7-semantics/src/calculus/universal_relation.rs` with:

  ```rust
  /// The Universal Relation system — binary predicates for the universal
  /// and meaning relations.
  ///
  /// Corresponds to `Relations::Universal` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`).
  ///
  /// Creates a universal bp_family with two binary predicates:
  /// - R_universal — the universal relation ("relates"), which can apply
  ///   to any two things and subsumes all other relations
  /// - R_meaning — the meaning relation ("means"), which relates a verb
  ///   to its meaning (a relation)
  ///
  /// The universal relation is the most general relation in Inform. It is
  /// used for relation-level operations: testing whether a relation applies
  /// between two values, and asserting/retracting relation facts at run-time.
  ///
  /// The meaning relation is used to associate verbs with their semantic
  /// meaning (a relation). For example, the verb "to love" might mean the
  /// "loves" relation.
  ///
  /// Simplified:
  /// - No PreformUtilities::wording (uses string names)
  /// - No Kinds::eq (typecheck always returns ALWAYS_MATCH)
  /// - No Kinds::get_construct (typecheck always returns ALWAYS_MATCH)
  /// - No Kinds::binary_construction_material (typecheck always returns ALWAYS_MATCH)
  /// - No Kinds::compatible (typecheck always returns ALWAYS_MATCH)
  /// - No Problems::quote_kind (no problem messages)
  /// - No StandardProblems::tcp_problem (no problem messages)
  /// - No Calculus::Schemas::modify (schema returns FALSE)
  use crate::calculus::binary_predicate_families::{
      BpFamily, BpFamilyMethods, BinaryPredicateFamilies,
  };
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the universal family in the family registry.
  ///
  /// This assumes the equality relation (PLAN-22) has already created families 0-2,
  /// and the quasinumeric relation (PLAN-36) has created family 3.
  /// The universal family is family 4.
  pub const UNIVERSAL_FAMILY: usize = 4;

  /// Index of the universal relation predicate in the BP registry.
  ///
  /// Created by `UniversalRelation::stock()` during first stock.
  /// This assumes the equality relation (PLAN-22) has already created BPs 0-3,
  /// and the quasinumeric relation (PLAN-36) has created BPs 4-11.
  pub const R_UNIVERSAL: usize = 12;

  /// Index of the meaning relation predicate in the BP registry.
  ///
  /// Created by `UniversalRelation::stock()` during first stock.
  pub const R_MEANING: usize = 14;
  ```

  Note: These constants assume the equality relation (PLAN-22) has already created families 0-2 and BPs 0-3, and the quasinumeric relation (PLAN-36) has created family 3 and BPs 4-11. The universal family is family 4, and its BPs start at index 12. Each `make_pair` creates 2 BPs (original + reversal), so R_universal is at 12 and R_meaning is at 14.

- [ ] Define the `UniversalRelation` struct:

  ```rust
  /// The universal relation module.
  ///
  /// Corresponds to `Relations::Universal` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`).
  pub struct UniversalRelation;
  ```

- [ ] Implement `UniversalRelation::start()`:

  ```rust
  /// Create the universal bp_family with its methods.
  ///
  /// Corresponds to `Relations::Universal::start` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 19-26).
  ///
  /// # Arguments
  ///
  /// * `families` - The family registry to add to.
  /// * `bp_registry` - The BP registry to add to.
  ///
  /// # Returns
  ///
  /// The index of the created family in the registry.
  pub fn start(
      families: &mut Vec<BpFamily>,
      _bp_registry: &mut Vec<BinaryPredicate>,
  ) -> usize {
      let family_idx = families.len();
      let family = BpFamily {
          name: "universal",
          methods: BpFamilyMethods {
              stock: Some(UniversalRelation::stock),
              typecheck: Some(UniversalRelation::typecheck),
              assert: Some(UniversalRelation::assert),
              schema: Some(UniversalRelation::schema),
              describe_for_problems: Some(UniversalRelation::describe_for_problems),
          },
      };
      families.push(family);
      family_idx
  }
  ```

- [ ] Implement `UniversalRelation::stock()`:

  ```rust
  /// Stock the universal family (stage 1): create two binary predicates.
  ///
  /// Corresponds to `Relations::Universal::stock` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 32-45).
  ///
  /// Creates two binary predicates at stock stage 1:
  /// - R_universal — the universal relation ("relates")
  /// - R_meaning — the meaning relation ("means")
  ///
  /// Both use open term details (BPTerms::new(NULL)) — no kind restriction.
  ///
  /// Simplified:
  /// - No PreformUtilities::wording (uses string names)
  /// - No Calculus::Schemas::new (uses None for test schemas)
  pub fn stock(
      _family: &BpFamily,
      n: u8,
      bp_registry: &mut Vec<BinaryPredicate>,
      _property_registry: &[()],
  ) {
      if n == 1 {
          // Create open term details (no kind restriction).
          // Corresponds to BPTerms::new(NULL) in the C reference.
          let open_term = BPTerms::new(None);

          // R_universal: the universal relation ("relates")
          // Corresponds to lines 34-38 of the C reference.
          BinaryPredicates::make_pair(
              UNIVERSAL_FAMILY,
              open_term.clone(),
              open_term.clone(),
              "relates",
              "relates-rev",
              None,
              None,
              Some("relates"),
              bp_registry,
          );

          // R_meaning: the meaning relation ("means")
          // Corresponds to lines 39-43 of the C reference.
          BinaryPredicates::make_pair(
              UNIVERSAL_FAMILY,
              open_term.clone(),
              open_term,
              "means",
              "means-rev",
              None,
              None,
              Some("means"),
              bp_registry,
          );
      }
  }
  ```

- [ ] Implement `UniversalRelation::typecheck()`:

  ```rust
  /// Typecheck the terms of a universal relation.
  ///
  /// Corresponds to `Relations::Universal::typecheck` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 51-110).
  ///
  /// Simplified: always returns ALWAYS_MATCH. The full C implementation
  /// checks that:
  /// - For R_meaning: first term must be K_verb, second must be CON_relation
  /// - For R_universal: first term must be CON_relation, second must be CON_combination,
  ///   and the relation's domain must match the combination's components
  pub fn typecheck(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      1 // ALWAYS_MATCH
  }
  ```

- [ ] Implement `UniversalRelation::assert()`:

  ```rust
  /// Assert a universal relation.
  ///
  /// Corresponds to `Relations::Universal::assert` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 116-120).
  ///
  /// These relations cannot be asserted — they are for run-time use only.
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

- [ ] Implement `UniversalRelation::schema()`:

  ```rust
  /// Compile run-time code for a universal relation.
  ///
  /// Corresponds to `Relations::Universal::schema` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 127-149).
  ///
  /// Simplified: returns FALSE (use default schema). The full implementation
  /// handles TEST_ATOM_TASK, NOW_ATOM_TRUE_TASK, and NOW_ATOM_FALSE_TASK
  /// with I6 schema modifications.
  pub fn schema(
      _family: &BpFamily,
      _task: u8,
      _bp: &BinaryPredicate,
  ) -> bool {
      false
  }
  ```

- [ ] Implement `UniversalRelation::describe_for_problems()`:

  ```rust
  /// Describe the relation in problem messages.
  ///
  /// Corresponds to `Relations::Universal::describe_for_problems` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Universal Relation.w`, lines 155-157).
  ///
  /// Returns an empty string (equivalent to FALSE in C — no special problem description).
  pub fn describe_for_problems(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
  ) -> String {
      String::new()
  }
  ```

- [ ] Implement helper functions:

  ```rust
  /// Check if a binary predicate belongs to the universal family.
  ///
  /// Corresponds to checking `bp->relation_family == universal_bp_family`
  /// in the C reference.
  pub fn is_universal_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
      bp_registry
          .get(bp_idx)
          .map(|bp| bp.relation_family == UNIVERSAL_FAMILY)
          .unwrap_or(false)
  }

  /// Check if a binary predicate is the meaning relation.
  ///
  /// Corresponds to checking `bp == R_meaning` in the C reference.
  pub fn is_meaning_bp(bp_idx: usize, bp_registry: &[BinaryPredicate]) -> bool {
      bp_registry
          .get(bp_idx)
          .map(|bp| {
              bp.relation_family == UNIVERSAL_FAMILY
                  && bp.relation_name.as_deref() == Some("means")
          })
          .unwrap_or(false)
  }
  ```

- [ ] Add unit tests:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;

      /// Test that `start()` creates the universal family with all five methods.
      #[test]
      fn test_start_creates_family() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          let idx = UniversalRelation::start(&mut families, &mut bp_registry);

          assert_eq!(idx, UNIVERSAL_FAMILY);
          assert!(idx < families.len());
          assert_eq!(families[idx].name, "universal");
          assert!(families[idx].methods.stock.is_some());
          assert!(families[idx].methods.typecheck.is_some());
          assert!(families[idx].methods.assert.is_some());
          assert!(families[idx].methods.schema.is_some());
          assert!(families[idx].methods.describe_for_problems.is_some());
          // No describe_for_index method
          assert!(families[idx].methods.describe_for_index.is_none());
      }

      /// Test that stock creates two binary predicates at stage 1.
      #[test]
      fn test_stock_creates_bps() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);

          // Stock stage 1
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          // Should have created 4 BPs (2 pairs = 2 originals + 2 reversals)
          assert_eq!(bp_registry.len(), 4);

          // Check R_universal
          let universal = &bp_registry[R_UNIVERSAL];
          assert_eq!(universal.relation_family, UNIVERSAL_FAMILY);
          assert_eq!(universal.debugging_log_name.as_deref(), Some("relates"));
          assert!(universal.right_way_round);

          // Check R_meaning
          let meaning = &bp_registry[R_MEANING];
          assert_eq!(meaning.relation_family, UNIVERSAL_FAMILY);
          assert_eq!(meaning.debugging_log_name.as_deref(), Some("means"));
          assert!(meaning.right_way_round);
      }

      /// Test that stock at stage 2 does nothing.
      #[test]
      fn test_stock_stage_2_noop() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);

          // Stock stage 1
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);
          let count_after_stage_1 = bp_registry.len();

          // Stock stage 2 should do nothing
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 2, &mut bp_registry, &[]);
          assert_eq!(bp_registry.len(), count_after_stage_1);
      }

      /// Test that typecheck always returns ALWAYS_MATCH.
      #[test]
      fn test_typecheck_always_matches() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          let result = UniversalRelation::typecheck(
              &families[UNIVERSAL_FAMILY],
              &bp_registry[R_UNIVERSAL],
              &[],
              &[],
          );
          assert_eq!(result, 1); // ALWAYS_MATCH
      }

      /// Test that assert returns FALSE.
      #[test]
      fn test_assert_returns_false() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          let result = UniversalRelation::assert(
              &families[UNIVERSAL_FAMILY],
              &bp_registry[R_UNIVERSAL],
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
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          let result = UniversalRelation::schema(
              &families[UNIVERSAL_FAMILY],
              0,
              &bp_registry[R_UNIVERSAL],
          );
          assert!(!result);
      }

      /// Test that describe_for_problems returns an empty string.
      #[test]
      fn test_describe_for_problems_returns_empty() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          let result = UniversalRelation::describe_for_problems(
              &families[UNIVERSAL_FAMILY],
              &bp_registry[R_UNIVERSAL],
          );
          assert_eq!(result, "");
      }

      /// Test is_universal_bp returns true for universal BPs.
      #[test]
      fn test_is_universal_bp_true() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          assert!(UniversalRelation::is_universal_bp(R_UNIVERSAL, &bp_registry));
          assert!(UniversalRelation::is_universal_bp(R_MEANING, &bp_registry));
      }

      /// Test is_universal_bp returns false for non-universal BPs.
      #[test]
      fn test_is_universal_bp_false() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          // A non-existent BP index should return false
          assert!(!UniversalRelation::is_universal_bp(99, &bp_registry));
      }

      /// Test is_meaning_bp returns true for the meaning BP.
      #[test]
      fn test_is_meaning_bp_true() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          assert!(UniversalRelation::is_meaning_bp(R_MEANING, &bp_registry));
      }

      /// Test is_meaning_bp returns false for the universal BP.
      #[test]
      fn test_is_meaning_bp_false_for_universal() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          assert!(!UniversalRelation::is_meaning_bp(R_UNIVERSAL, &bp_registry));
      }

      /// Test is_meaning_bp returns false for non-existent BPs.
      #[test]
      fn test_is_meaning_bp_false_for_nonexistent() {
          let mut families = Vec::new();
          let mut bp_registry = Vec::new();
          UniversalRelation::start(&mut families, &mut bp_registry);
          UniversalRelation::stock(&families[UNIVERSAL_FAMILY], 1, &mut bp_registry, &[]);

          assert!(!UniversalRelation::is_meaning_bp(99, &bp_registry));
      }
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod universal_relation;` to `crates/conform7-semantics/src/calculus/mod.rs`.

### 3. Update the module map

- [ ] Add the `universal_relation` entry to the module map in `crates/conform7-semantics/src/calculus/mod.rs`:

  ```rust
  //! | [`universal_relation`] | `Chapter 8/The Universal Relation.w` | Universal and meaning binary predicates |
  ```

## Success Criteria

- [ ] `UniversalRelation::start()` creates the universal bp_family with all five methods (stock, typecheck, assert, schema, describe_for_problems).
- [ ] `UniversalRelation::stock()` creates 2 binary predicates (R_universal and R_meaning) at stock stage 1, each with a reversal.
- [ ] `UniversalRelation::stock()` does nothing at stock stage 2.
- [ ] `UniversalRelation::typecheck()` returns ALWAYS_MATCH (simplified).
- [ ] `UniversalRelation::assert()` returns FALSE.
- [ ] `UniversalRelation::schema()` returns FALSE (simplified).
- [ ] `UniversalRelation::describe_for_problems()` returns an empty string.
- [ ] `UniversalRelation::is_universal_bp()` returns true for universal BPs and false otherwise.
- [ ] `UniversalRelation::is_meaning_bp()` returns true for the meaning BP and false otherwise.
- [ ] The module is declared in `calculus/mod.rs`.
- [ ] All unit tests pass with `cargo test -p conform7-semantics`.

## Out of Scope

- **Kind checking in typecheck**: The full C implementation checks `Kinds::eq`, `Kinds::get_construct`, `Kinds::binary_construction_material`, and `Kinds::compatible` to validate kinds. This is deferred until the kind system's compatibility checking is fully implemented.
- **Problem messages**: The full C implementation issues problem messages for typecheck failures. This is deferred.
- **`Calculus::Schemas::modify`**: The full schema system with I6 schema modifications is not yet implemented. We return FALSE (use default schema).
- **`PreformUtilities::wording`**: The C reference uses Preform grammar for relation names. This is deferred — we use string names instead.
- **`BinaryPredicates::set_index_details`**: The C reference may set index details for each BP. This is deferred.
- **Other assertions module startup items**: `ExplicitRelations::start()` (line 30), `EqualityDetails::start()` (line 31), `KindPredicatesRevisited::start()` (line 32), `ImperativeDefinitionFamilies::create()` (line 33), and the adjective-by-* modules (lines 34-37) are all deferred to future plans.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
