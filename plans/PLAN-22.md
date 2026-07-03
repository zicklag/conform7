# Plan 22: The Equality Relation — First Concrete Binary Predicate Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Equality Relation — the first concrete use of the binary predicate system. This creates three `bp_family` instances (equality, spatial, empty) and three `binary_predicate` instances (equality, has, never-holding) that form the foundation of the relation system.

This is the smallest next step after PLAN-21 because:

1. **It's the next thing called in `CalculusModule::start()`.** The C reference (`services/calculus-module/Chapter 1/Calculus Module.w`, line 57) calls `Calculus::Equality::start()` immediately after `KindPredicates::start()`. After PLAN-21 implemented the binary predicate infrastructure (`BpFamily`, `BpTermDetails`, `BinaryPredicate`), the next logical step is to create the first concrete families and predicates that use that infrastructure.

2. **It's the first concrete use of the binary predicate system.** PLAN-21 created the data structures and creation functions (`BpFamily`, `BpTermDetails`, `BinaryPredicate`, `make_equality`, `make_pair`, `make_single`). What's missing is actual families with real methods and actual predicates. The Equality Relation provides:
   - `equality_bp_family` — with `stock`, `describe_for_problems`, `describe_for_index` methods
   - `spatial_bp_family` — with `stock` method (creates the "has" / "is-had-by" pair)
   - `empty_bp_family` — with `stock`, `describe_for_problems`, `describe_for_index` methods
   - `R_equality` — the equality relation (its own reversal)
   - `a_has_b_predicate` — the "has" relation (paired with "is-had-by")
   - `R_empty` — the "never-holding" relation (its own reversal)

3. **It's a prerequisite for all other bp_family implementations.** The four knowledge-module bp_families (`SameAsRelations`, `SettingPropertyRelations`, `ComparativeRelations`, `ProvisionRelation`) all follow the same pattern as the Equality Relation. Having a working example makes those plans straightforward.

4. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For equality atoms, they need `R_equality` to determine the inference family. Without the equality relation, the assertion pipeline cannot process equality facts.

5. **It's a prerequisite for the knowledge module startup.** The knowledge module startup (`KnowledgeModule::start()`) calls `SameAsRelations::start()`, `SettingPropertyRelations::start()`, `ComparativeRelations::start()`, and `ProvisionRelation::start()` — all of which create bp_families. But the calculus module startup (which creates the equality relation) must run first. The equality relation is the first bp_family to be created in the entire system.

6. **Independently testable.** We can create the three families, stock them (stage 1), verify that `R_equality` is its own reversal, verify that `a_has_b_predicate` has a proper reversal pair, verify that `R_empty` is its own reversal, verify the describe methods, and verify the family method dispatch — all without needing properties, instances, or any knowledge module infrastructure.

## Background

### C reference architecture

#### The Equality Relation (`Chapter 3/The Equality Relation.w`, lines 1-99)

The Equality Relation creates three families and three predicates:

```c
bp_family *equality_bp_family = NULL;
bp_family *spatial_bp_family = NULL;
bp_family *empty_bp_family = NULL;

binary_predicate *R_equality = NULL;
binary_predicate *a_has_b_predicate = NULL;
binary_predicate *R_empty = NULL;

void Calculus::Equality::start(void) {
    equality_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(equality_bp_family, STOCK_BPF_MTID,
        Calculus::Equality::stock);
    METHOD_ADD(equality_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID,
        Calculus::Equality::describe_for_problems);
    METHOD_ADD(equality_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID,
        Calculus::Equality::describe_for_index);

    spatial_bp_family = BinaryPredicateFamilies::new();
    #ifndef IF_MODULE
    METHOD_ADD(spatial_bp_family, STOCK_BPF_MTID,
        Calculus::Equality::stock_spatial);
    #endif

    empty_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(empty_bp_family, STOCK_BPF_MTID,
        Calculus::Equality::stock_empty);
    METHOD_ADD(empty_bp_family, DESCRIBE_FOR_PROBLEMS_BPF_MTID,
        Calculus::Equality::describe_empty_for_problems);
    METHOD_ADD(empty_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID,
        Calculus::Equality::describe_empty_for_index);
}
```

Stocking creates the actual predicates:

```c
void Calculus::Equality::stock(bp_family *self, int n) {
    if (n == 1) {
        R_equality = BinaryPredicates::make_equality(equality_bp_family,
            PreformUtilities::wording(<relation-names>, EQUALITY_RELATION_NAME));
        BinaryPredicates::set_index_details(R_equality, "value", "value");
    }
}

void Calculus::Equality::stock_spatial(bp_family *self, int n) {
    if (n == 1) {
        a_has_b_predicate =
            BinaryPredicates::make_pair(spatial_bp_family,
                BPTerms::new_full(NULL, NULL, EMPTY_WORDING, NULL),
                BPTerms::new(NULL),
                I"has", I"is-had-by",
                NULL, NULL,
                PreformUtilities::wording(<relation-names>, POSSESSION_RELATION_NAME));
    }
}

void Calculus::Equality::stock_empty(bp_family *self, int n) {
    if (n == 1) {
        R_empty = BinaryPredicates::make_equality(empty_bp_family,
            PreformUtilities::wording(<relation-names>, EMPTY_RELATION_NAME));
        BinaryPredicates::set_index_details(R_equality, "value", "value");
    }
}
```

Describe methods:

```c
int Calculus::Equality::describe_for_problems(bp_family *self, OUTPUT_STREAM,
    binary_predicate *bp) {
    return FALSE;
}
void Calculus::Equality::describe_for_index(bp_family *self, OUTPUT_STREAM,
    binary_predicate *bp) {
    WRITE("equality");
}
int Calculus::Equality::describe_empty_for_problems(bp_family *self, OUTPUT_STREAM,
    binary_predicate *bp) {
    return FALSE;
}
void Calculus::Equality::describe_empty_for_index(bp_family *self, OUTPUT_STREAM,
    binary_predicate *bp) {
    WRITE("never-holding");
}
```

### Key C source files

- `services/calculus-module/Chapter 3/The Equality Relation.w` — the full equality relation implementation (99 lines)
- `services/calculus-module/Chapter 1/Calculus Module.w` — module startup, calls `Calculus::Equality::start()` (line 57)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `bp_family` struct, method dispatch (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `bp_term_details` struct, `BPTerms` functions (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, creation functions (PLAN-21)

### Current Rust state

- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, unit tests.
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions, unit tests.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct, `BinaryPredicates` creation functions (`make_equality`, `make_pair`, `make_single`), accessors, unit tests.
- `crates/conform7-semantics/src/calculus/kind_predicates.rs` — `KindPredicates` module, `KIND_UP_FAMILY` static, `KindPredicates::start()`.
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules.

### What's needed

1. **`EqualityRelation` module** — a new module `equality_relation` in the calculus crate with:
   - `EqualityRelation::start()` — creates the three families (equality, spatial, empty) with their methods
   - `EqualityRelation::stock()` — stocks the equality family (stage 1): creates `R_equality`
   - `EqualityRelation::stock_spatial()` — stocks the spatial family (stage 1): creates `a_has_b_predicate`
   - `EqualityRelation::stock_empty()` — stocks the empty family (stage 1): creates `R_empty`
   - `EqualityRelation::describe_for_problems()` — returns `false` (no special problem description)
   - `EqualityRelation::describe_for_index()` — returns "equality"
   - `EqualityRelation::describe_empty_for_problems()` — returns `false`
   - `EqualityRelation::describe_empty_for_index()` — returns "never-holding"
   - Global constants for the three families and three predicates (or a registry-based approach)

2. **Integration with the calculus module** — a `CalculusModule::start()` equivalent that calls `KindPredicates::start()` and `EqualityRelation::start()`.

3. **Unit tests** — create the families, stock them, verify predicates are created correctly, verify reversal relationships, verify describe methods, verify family method dispatch.

## Tasks

### 1. Create the `EqualityRelation` module

- [ ] Create `crates/conform7-semantics/src/calculus/equality_relation.rs` with:

  ```rust
  /// The equality relation and related families.
  ///
  /// Corresponds to `Calculus::Equality` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`).
  ///
  /// Creates three bp_family instances:
  /// - equality_bp_family — for the equality relation (R_equality)
  /// - spatial_bp_family — for the "has" / "is-had-by" pair (a_has_b_predicate)
  /// - empty_bp_family — for the "never-holding" relation (R_empty)
  ///
  /// These are the first concrete uses of the binary predicate system.
  use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods, BinaryPredicateFamilies};
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::{BpTermDetails, BPTerms};
  ```

- [ ] Define global constants for the three families:

  ```rust
  /// Index of the equality family in the family registry.
  pub const EQUALITY_FAMILY: usize = 0;
  /// Index of the spatial family in the family registry.
  pub const SPATIAL_FAMILY: usize = 1;
  /// Index of the empty family in the family registry.
  pub const EMPTY_FAMILY: usize = 2;

  /// Index of the equality predicate in the BP registry.
  pub const R_EQUALITY: usize = 0;
  /// Index of the "has" predicate in the BP registry.
  pub const A_HAS_B_PREDICATE: usize = 0;
  /// Index of the "never-holding" predicate in the BP registry.
  pub const R_EMPTY: usize = 1;
  ```

  Note: The spatial family creates a pair (has + is-had-by), so `A_HAS_B_PREDICATE` is at index 0 in the BP registry (the right-way-round BP), and its reversal is at index 1. `R_EQUALITY` is at index 0 (equality family creates first), `R_EMPTY` is at index 1 (empty family creates second). The actual indices depend on the order of `stock` calls.

- [ ] Implement `EqualityRelation::start()`:

  ```rust
  /// Create the three families with their methods.
  ///
  /// Corresponds to `Calculus::Equality::start` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 24-46).
  ///
  /// Returns (families, bp_registry) where:
  /// - families[0] = equality_bp_family
  /// - families[1] = spatial_bp_family
  /// - families[2] = empty_bp_family
  /// - bp_registry is empty (stocking fills it)
  pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
      let equality_family = BpFamily {
          name: "equality",
          methods: BpFamilyMethods {
              stock: Some(EqualityRelation::stock),
              describe_for_problems: Some(EqualityRelation::describe_for_problems),
              describe_for_index: Some(EqualityRelation::describe_for_index),
              ..BpFamilyMethods::default()
          },
      };

      let spatial_family = BpFamily {
          name: "spatial",
          methods: BpFamilyMethods {
              stock: Some(EqualityRelation::stock_spatial),
              ..BpFamilyMethods::default()
          },
      };

      let empty_family = BpFamily {
          name: "empty",
          methods: BpFamilyMethods {
              stock: Some(EqualityRelation::stock_empty),
              describe_for_problems: Some(EqualityRelation::describe_empty_for_problems),
              describe_for_index: Some(EqualityRelation::describe_empty_for_index),
              ..BpFamilyMethods::default()
          },
      };

      (vec![equality_family, spatial_family, empty_family], Vec::new())
  }
  ```

- [ ] Implement `EqualityRelation::stock()`:

  ```rust
  /// Stock the equality family (stage 1): create R_equality.
  ///
  /// Corresponds to `Calculus::Equality::stock` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 52-58).
  pub fn stock(family: &BpFamily, n: u8) {
      if n == 1 {
          // Simplified: use a string name instead of PreformUtilities::wording
          BinaryPredicates::make_equality(
              // Find the family index by name
              0, // equality family is at index 0
              "equality",
              // We need a mutable BP registry — this is a limitation of the
              // simplified approach. The stock function receives &BpFamily,
              // so we need a way to access the BP registry.
              // See note below about the design decision.
          );
          // set_index_details would be called here
      }
  }
  ```

  Note: The stock functions in the C reference take `bp_family *self` and modify global variables (`R_equality`, `a_has_b_predicate`, `R_empty`). In Rust, we have two design options:
  - **Option A (global statics)**: Use `LazyLock<Mutex<...>>` or `std::sync::OnceLock` for the three predicates, similar to how `KIND_UP_FAMILY` uses `LazyLock<UpFamily>`.
  - **Option B (registry-based)**: Pass mutable references to the family registry and BP registry through the method dispatch.

  Option A is simpler and closer to the C pattern. The stock functions would be closures that capture the registries, or we can use a different approach where `start()` returns the registries and `stock()` is called externally.

  Recommended approach: Make `start()` return the families and an empty BP registry. Then `first_stock` is called separately, which iterates over families and calls their stock methods. The stock methods need access to the BP registry, so we pass it as a parameter.

  Revised design:

  ```rust
  /// Stock the equality family (stage 1): create R_equality.
  ///
  /// Corresponds to `Calculus::Equality::stock` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 52-58).
  pub fn stock(family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>) {
      if n == 1 {
          let family_idx = 0; // equality family is at index 0
          let idx = BinaryPredicates::make_equality(
              family_idx, "equality", bp_registry,
          );
          BinaryPredicates::set_index_details(
              &mut bp_registry[idx], Some("value"), Some("value"),
          );
      }
  }
  ```

  This means the `BpFamilyMethods::stock` signature needs to change from `fn(&BpFamily, u8)` to `fn(&BpFamily, u8, &mut Vec<BinaryPredicate>)`. This is a minor change to PLAN-21's `BpFamilyMethods` struct.

- [ ] Implement `EqualityRelation::stock_spatial()`:

  ```rust
  /// Stock the spatial family (stage 1): create a_has_b_predicate.
  ///
  /// Corresponds to `Calculus::Equality::stock_spatial` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 60-70).
  pub fn stock_spatial(family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>) {
      if n == 1 {
          let family_idx = 1; // spatial family is at index 1
          let left_term = BPTerms::new_full(None, None, None, None);
          let right_term = BPTerms::new(None);
          BinaryPredicates::make_pair(
              family_idx, left_term, right_term,
              "has", "is-had-by",
              None, None, Some("possession"),
              bp_registry,
          );
      }
  }
  ```

- [ ] Implement `EqualityRelation::stock_empty()`:

  ```rust
  /// Stock the empty family (stage 1): create R_empty.
  ///
  /// Corresponds to `Calculus::Equality::stock_empty` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 72-78).
  pub fn stock_empty(family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>) {
      if n == 1 {
          let family_idx = 2; // empty family is at index 2
          let idx = BinaryPredicates::make_equality(
              family_idx, "never-holding", bp_registry,
          );
          BinaryPredicates::set_index_details(
              &mut bp_registry[idx], Some("value"), Some("value"),
          );
      }
  }
  ```

- [ ] Implement describe methods:

  ```rust
  /// Describe the equality relation for problem messages.
  ///
  /// Corresponds to `Calculus::Equality::describe_for_problems` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 83-86).
  pub fn describe_for_problems(family: &BpFamily, bp: &BinaryPredicate) -> String {
      String::new() // returns empty string (FALSE in C)
  }

  /// Describe the equality relation for the Phrasebook index.
  ///
  /// Corresponds to `Calculus::Equality::describe_for_index` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 87-90).
  pub fn describe_for_index(family: &BpFamily, bp: &BinaryPredicate) -> String {
      "equality".to_string()
  }

  /// Describe the empty relation for problem messages.
  ///
  /// Corresponds to `Calculus::Equality::describe_empty_for_problems` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 91-94).
  pub fn describe_empty_for_problems(family: &BpFamily, bp: &BinaryPredicate) -> String {
      String::new() // returns empty string (FALSE in C)
  }

  /// Describe the empty relation for the Phrasebook index.
  ///
  /// Corresponds to `Calculus::Equality::describe_empty_for_index` in the C reference
  /// (`services/calculus-module/Chapter 3/The Equality Relation.w`, lines 95-98).
  pub fn describe_empty_for_index(family: &BpFamily, bp: &BinaryPredicate) -> String {
      "never-holding".to_string()
  }
  ```

### 2. Update `BpFamilyMethods::stock` signature (minor PLAN-21 follow-up)

- [ ] Update `BpFamilyMethods::stock` in `binary_predicate_families.rs` to accept `&mut Vec<BinaryPredicate>`:

  ```rust
  /// Stock up on relations (stage 1: built-in essentials; stage 2: one per value property).
  /// Corresponds to STOCK_BPF_MTID.
  pub stock: Option<fn(&BpFamily, u8, &mut Vec<BinaryPredicate>)>,
  ```

- [ ] Update `BinaryPredicateFamilies::first_stock` and `second_stock` to pass the BP registry:

  ```rust
  pub fn first_stock(families: &mut [BpFamily], bp_registry: &mut Vec<BinaryPredicate>) {
      for family in families.iter() {
          if let Some(stock) = family.methods.stock {
              stock(family, 1, bp_registry);
          }
      }
  }

  pub fn second_stock(families: &mut [BpFamily], bp_registry: &mut Vec<BinaryPredicate>) {
      for family in families.iter() {
          if let Some(stock) = family.methods.stock {
              stock(family, 2, bp_registry);
          }
      }
  }
  ```

- [ ] Update existing tests in `binary_predicate_families.rs` to match the new signature.

### 3. Add module declaration

- [ ] Add `pub mod equality_relation;` to `crates/conform7-semantics/src/calculus/mod.rs`.

### 4. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/calculus/equality_relation.rs`:

  - Test that `start()` creates three families with the correct names.
  - Test that the equality family has `stock`, `describe_for_problems`, and `describe_for_index` methods.
  - Test that the spatial family has only a `stock` method.
  - Test that the empty family has `stock`, `describe_for_problems`, and `describe_for_index` methods.
  - Test that `first_stock` on the equality family creates `R_equality` as its own reversal.
  - Test that `first_stock` on the spatial family creates a pair (`a_has_b_predicate` and its reversal).
  - Test that `a_has_b_predicate` has `right_way_round = true` and its reversal has `right_way_round = false`.
  - Test that `first_stock` on the empty family creates `R_empty` as its own reversal.
  - Test that `describe_for_problems` returns an empty string.
  - Test that `describe_for_index` returns "equality".
  - Test that `describe_empty_for_problems` returns an empty string.
  - Test that `describe_empty_for_index` returns "never-holding".
  - Test that `set_index_details` works on `R_equality`.
  - Test that the spatial pair has the correct term details (left term has `new_full`, right term has `new`).

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `EqualityRelation::start()` creates three families: equality, spatial, empty.
- [ ] The equality family has `stock`, `describe_for_problems`, and `describe_for_index` methods.
- [ ] The spatial family has only a `stock` method.
- [ ] The empty family has `stock`, `describe_for_problems`, and `describe_for_index` methods.
- [ ] `first_stock` on the equality family creates `R_equality` as its own reversal.
- [ ] `first_stock` on the spatial family creates a pair with correct reversal links.
- [ ] `a_has_b_predicate` has `right_way_round = true`; its reversal has `right_way_round = false`.
- [ ] `first_stock` on the empty family creates `R_empty` as its own reversal.
- [ ] `describe_for_problems` returns an empty string for both equality and empty families.
- [ ] `describe_for_index` returns "equality" for the equality family.
- [ ] `describe_for_index` returns "never-holding" for the empty family.
- [ ] `BpFamilyMethods::stock` signature accepts `&mut Vec<BinaryPredicate>`.
- [ ] `BinaryPredicateFamilies::first_stock` and `second_stock` pass the BP registry to stock methods.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`SameAsRelations`**: `SameAsRelations::start()` (Chapter 3/Same Property Relation.w) — the second bp_family, which depends on properties and the full property system, is deferred.
- **`SettingPropertyRelations`**: `SettingPropertyRelations::start()` (Chapter 3/Setting Property Relation.w) — depends on properties, kind subjects, and the full typechecking system, is deferred.
- **`ComparativeRelations`**: `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) — depends on measurement adjectives and properties, is deferred.
- **`ProvisionRelation`**: `ProvisionRelation::start()` (Chapter 3/The Provision Relation.w) — depends on properties, instances, and property permissions, is deferred.
- **`InstanceAdjectives`**: `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) — depends on the adjective meaning system (assertions-module), is deferred.
- **`EitherOrPropertyAdjectives`**: `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) is deferred.
- **`MeasurementAdjectives`**: `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) is deferred.
- **`RelationSubjects` family**: The `RelationSubjects` inference subject family (Chapter 4/Relation Subjects.w) is deferred.
- **`ExplicitRelations`**: The explicit relations system (relation forms, storage, run-time) is deferred.
- **`PreformUtilities::wording`**: The full Preform wording system is deferred. This plan uses simplified string names.
- **`word_assemblage` struct**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`i6_schema` struct**: The full I6 schema struct is deferred. This plan uses simplified string schemas.
- **`annotated_i6_schema` struct**: The annotated schema struct used in schema compilation is deferred.
- **`IF_MODULE` conditional**: The C code conditionally excludes the spatial family when `IF_MODULE` is defined. This plan always creates the spatial family (simplified).
- **Knowledge module integration**: The knowledge module startup (`KnowledgeModule::start()`) and its inference families are deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) is deferred.
- **Run-time compilation**: All `RT*` functions (run-time compilation of relations, subjects, permissions) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
