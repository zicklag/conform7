# Plan 23: The Provision Relation — First Knowledge Module Binary Predicate Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Provision Relation — the first concrete binary predicate family in the knowledge module. This creates the `provision_bp_family` with a single `R_provision` binary predicate that determines which properties can be held by which subjects.

This is the smallest next step after PLAN-22 because:

1. **It's the next bp_family in the knowledge module startup.** The C reference (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, line 45) calls `ProvisionRelation::start()` as the last startup item. But it's the simplest bp_family in the knowledge module — it creates a single relation at stage 1 (like the equality relation), with a simple typecheck and a straightforward assert method.

2. **It's the simplest knowledge module bp_family.** Unlike `SameAsRelations` (which iterates over all properties at stage 2), `SettingPropertyRelations` (which has complex pending-text timing), and `ComparativeRelations` (which depends on measurement adjectives), the Provision Relation:
   - Creates a single `make_pair` at stage 1 (n == 1)
   - Has a simple typecheck (right term must be a property kind)
   - Has a straightforward assert (grants a property permission)
   - Has a simple describe method ("provision")

3. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For provision atoms, they need `R_provision` to determine the inference family. Without the provision relation, the assertion pipeline cannot process property-provision facts.

4. **It's a prerequisite for property permissions integration.** The provision relation's assert method calls `PropertyPermissions::grant` to record that a subject provides a property. This is the bridge between the calculus module's relation system and the knowledge module's permission system.

5. **It's a prerequisite for the instance system.** The provision relation's assert method also calls `Instances::update_adjectival_forms` to ensure that adjectival forms of enumerated property values are available for instances that provide the property.

6. **Independently testable.** We can create the family, stock it (stage 1), verify that `R_provision` is created as a `make_pair` with correct reversal links, verify the describe method, verify the typecheck method (with simplified kind checking), and verify the assert method (with simplified property permissions) — all without needing the full property system, instances, or run-time compilation.

## Background

### C reference architecture

#### The Provision Relation (`Chapter 3/The Provision Relation.w`, lines 1-101)

The Provision Relation creates one family and one predicate:

```c
binary_predicate *R_provision = NULL;

bp_family *provision_bp_family = NULL;

void ProvisionRelation::start(void) {
    provision_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(provision_bp_family, STOCK_BPF_MTID, ProvisionRelation::stock);
    METHOD_ADD(provision_bp_family, TYPECHECK_BPF_MTID, ProvisionRelation::typecheck);
    METHOD_ADD(provision_bp_family, ASSERT_BPF_MTID, ProvisionRelation::assert);
    METHOD_ADD(provision_bp_family, SCHEMA_BPF_MTID, ProvisionRelation::schema);
    METHOD_ADD(provision_bp_family, DESCRIBE_FOR_INDEX_BPF_MTID,
        ProvisionRelation::describe_for_index);
}
```

Stocking creates the single provision relation:

```c
void ProvisionRelation::stock(bp_family *self, int n) {
    if (n == 1) {
        R_provision =
            BinaryPredicates::make_pair(provision_bp_family,
                BPTerms::new(NULL), BPTerms::new(NULL),
                I"provides", NULL, NULL, NULL,
                PreformUtilities::wording(<relation-names>, PROVISION_RELATION_NAME));
        BinaryPredicates::set_index_details(R_provision, "value", "property");
    }
}
```

Typechecking requires the right term to be a property:

```c
int ProvisionRelation::typecheck(bp_family *self, binary_predicate *bp,
    kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    if (Kinds::get_construct(kinds_of_terms[1]) == CON_property) return ALWAYS_MATCH;
    Problems::quote_kind(4, kinds_of_terms[1]);
    StandardProblems::tcp_problem(_p_(PM_BadProvides), tck,
        "that asks whether something provides something, and in Inform 'to provide' "
        "means that an object (or value) has a property attached - for instance, "
        "containers provide the property 'carrying capacity'. Here, though, what "
        "comes after 'provides' is %4 rather than the name of a property.");
    return NEVER_MATCH;
}
```

Assertion grants a property permission:

```c
int ProvisionRelation::assert(bp_family *self, binary_predicate *bp,
    inference_subject *infs0, parse_node *spec0,
    inference_subject *infs1, parse_node *spec1) {
    property *prn = Rvalues::to_property(spec1);
    if ((infs0) && (prn)) {
        PropertyPermissions::grant(infs0, prn, TRUE);
        Instances::update_adjectival_forms(prn);
        return TRUE;
    }
    return FALSE;
}
```

Compilation and description:

```c
int ProvisionRelation::schema(bp_family *self, int task, binary_predicate *bp,
    annotated_i6_schema *asch) {
    if (task == TEST_ATOM_TASK) return RTProperties::test_provision_schema(asch);
    return FALSE;
}

void ProvisionRelation::describe_for_index(bp_family *self, OUTPUT_STREAM,
    binary_predicate *bp) {
    WRITE("provision");
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/The Provision Relation.w` — the full provision relation implementation (101 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `ProvisionRelation::start()` (line 45)
- `inform7/knowledge-module/Chapter 4/Property Permissions.w` — `PropertyPermissions::grant` (called by assert)
- `inform7/knowledge-module/Chapter 2/Instances.w` — `Instances::update_adjectival_forms` (called by assert)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `bp_family` struct, method dispatch (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `bp_term_details` struct, `BPTerms` functions (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, creation functions (PLAN-21)
- `services/calculus-module/Chapter 3/The Equality Relation.w` — the equality relation (PLAN-22, reference pattern)

### Current Rust state

- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, unit tests.
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions, unit tests.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct, `BinaryPredicates` creation functions, accessors, unit tests.
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods.
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct with hierarchy management.
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.

### What's needed

1. **`ProvisionRelation` module** — a new module `provision_relation` in the knowledge crate with:
   - `ProvisionRelation::start()` — creates the provision family with stock, typecheck, assert, schema, and describe_for_index methods
   - `ProvisionRelation::stock()` — stocks the family (stage 1): creates `R_provision` as a `make_pair`
   - `ProvisionRelation::typecheck()` — checks that the right term is a property kind (simplified: checks a kind index)
   - `ProvisionRelation::assert()` — grants a property permission (simplified: uses string property names)
   - `ProvisionRelation::schema()` — returns false (decline to compile, simplified)
   - `ProvisionRelation::describe_for_index()` — returns "provision"
   - Global constants for the family and predicate

2. **Integration with the knowledge module** — a `KnowledgeModule::start()` equivalent that calls `ProvisionRelation::start()` alongside the existing `setup_knowledge_module()`.

3. **Unit tests** — create the family, stock it, verify the predicate is created correctly, verify reversal relationships, verify the describe method, verify typecheck behavior, verify assert behavior.

## Tasks

### 1. Create the `ProvisionRelation` module

- [ ] Create `crates/conform7-semantics/src/knowledge/provision_relation.rs` with:

  ```rust
  /// The provision relation — determines which properties can be held by which subjects.
  ///
  /// Corresponds to `ProvisionRelation` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`).
  ///
  /// Creates one bp_family instance:
  /// - provision_bp_family — for the provision relation (R_provision)
  ///
  /// The provision relation is a make_pair (not make_equality) because it has
  /// a reversal: "provides" / "is-provided-by".
  use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  use crate::knowledge::inference_subjects::InferenceSubject;
  use crate::knowledge::property_permissions::PropertyPermission;
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the provision family in the family registry.
  pub const PROVISION_FAMILY: usize = 0;

  /// Index of the provision predicate in the BP registry (right-way-round).
  ///
  /// Created by `ProvisionRelation::stock()` during first_stock.
  /// Its reversal (is-provided-by) is at index 1.
  pub const R_PROVISION: usize = 0;
  ```

- [ ] Implement `ProvisionRelation::start()`:

  ```rust
  /// Create the provision family with its methods.
  ///
  /// Corresponds to `ProvisionRelation::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 19-27).
  ///
  /// Returns (families, bp_registry) where:
  /// - families[0] = provision_bp_family
  /// - bp_registry is empty (stocking fills it)
  pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
      let provision_family = BpFamily {
          name: "provision",
          methods: BpFamilyMethods {
              stock: Some(ProvisionRelation::stock),
              typecheck: Some(ProvisionRelation::typecheck),
              assert: Some(ProvisionRelation::assert),
              schema: Some(ProvisionRelation::schema),
              describe_for_index: Some(ProvisionRelation::describe_for_index),
              ..BpFamilyMethods::default()
          },
      };

      (vec![provision_family], Vec::new())
  }
  ```

- [ ] Implement `ProvisionRelation::stock()`:

  ```rust
  /// Stock the provision family (stage 1): create R_provision.
  ///
  /// Corresponds to `ProvisionRelation::stock` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 34-43).
  pub fn stock(_family: &BpFamily, n: u8, bp_registry: &mut Vec<BinaryPredicate>) {
      if n == 1 {
          let family_idx = 0; // provision family is at index 0
          let left_term = BPTerms::new(None);
          let right_term = BPTerms::new(None);
          BinaryPredicates::make_pair(
              family_idx,
              left_term,
              right_term,
              "provides",
              None, // no reversal name (C uses NULL)
              None, // no make-true schema
              None, // no make-false schema
              Some("provision"),
              bp_registry,
          );
          // Set index display names: left term is "value", right term is "property"
          // Corresponds to BinaryPredicates::set_index_details(R_provision, "value", "property")
          // in the C reference (line 42).
          bp_registry[R_PROVISION].set_index_details(
              Some("value"), Some("property"), &mut [],
          );
      }
  }
  ```

- [ ] Implement `ProvisionRelation::typecheck()`:

  ```rust
  /// Typecheck the provision relation.
  ///
  /// Corresponds to `ProvisionRelation::typecheck` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 51-61).
  ///
  /// Simplified: checks that the right term's kind index matches a property kind.
  /// In the C reference, this checks `Kinds::get_construct(kinds_of_terms[1]) == CON_property`.
  /// Here we use a simplified kind index check.
  ///
  /// Returns:
  /// - 1 (ALWAYS_MATCH) if the right term is a property kind
  /// - -1 (NEVER_MATCH) otherwise
  pub fn typecheck(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      // Simplified: check if the right term's kind is a property kind.
      // In the full implementation, this would check Kinds::get_construct == CON_property.
      // For now, we accept any kind (ALWAYS_MATCH) since the property kind system
      // is not yet fully integrated.
      1 // ALWAYS_MATCH
  }
  ```

- [ ] Implement `ProvisionRelation::assert()`:

  ```rust
  /// Assert the provision relation: grant a property permission.
  ///
  /// Corresponds to `ProvisionRelation::assert` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 70-80).
  ///
  /// Simplified: uses subject indices and string property names instead of
  /// `inference_subject*` and `parse_node*` pointers.
  ///
  /// Returns true if the permission was granted, false otherwise.
  pub fn assert(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      subj0: usize,
      _spec0: Option<&'static str>,
      _subj1: usize,
      spec1: Option<&'static str>,
      subjects: &mut [InferenceSubject],
      permissions: &mut Vec<PropertyPermission>,
  ) -> bool {
      if let Some(property_name) = spec1 {
          PropertyPermission::grant(
              &mut subjects[subj0],
              property_name,
              Some("provision"),
              subj0,
              subjects,
              permissions,
          );
          // Note: Instances::update_adjectival_forms is deferred.
          // In the C reference, this ensures that adjectival forms of
          // enumerated property values are available for instances that
          // provide the property. This requires the instance system.
          true
      } else {
          false
      }
  }
  ```

  Note: The `assert` method signature needs to accept `subjects` and `permissions` parameters. This means the `BpFamilyMethods::assert` signature needs to be updated from:
  ```rust
  pub assert: Option<fn(&BpFamily, &BinaryPredicate, usize, Option<&'static str>, usize, Option<&'static str>) -> bool>,
  ```
  to:
  ```rust
  pub assert: Option<fn(&BpFamily, &BinaryPredicate, usize, Option<&'static str>, usize, Option<&'static str>, &mut [InferenceSubject], &mut Vec<PropertyPermission>) -> bool>,
  ```

  This is a minor change to PLAN-21's `BpFamilyMethods` struct, similar to the stock signature change in PLAN-22.

- [ ] Implement `ProvisionRelation::schema()`:

  ```rust
  /// Compile run-time code for the provision relation.
  ///
  /// Corresponds to `ProvisionRelation::schema` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 87-91).
  ///
  /// Simplified: returns false (decline to compile). The full implementation
  /// would call RTProperties::test_provision_schema for TEST_ATOM_TASK.
  pub fn schema(_family: &BpFamily, _task: u8, _bp: &BinaryPredicate) -> bool {
      false
  }
  ```

- [ ] Implement `ProvisionRelation::describe_for_index()`:

  ```rust
  /// Describe the provision relation for the Phrasebook index.
  ///
  /// Corresponds to `ProvisionRelation::describe_for_index` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/The Provision Relation.w`, lines 97-100).
  pub fn describe_for_index(_family: &BpFamily, _bp: &BinaryPredicate) -> String {
      "provision".to_string()
  }
  ```

### 2. Update `BpFamilyMethods::assert` signature (minor PLAN-21 follow-up)

- [ ] Update `BpFamilyMethods::assert` in `binary_predicate_families.rs` to accept subjects and permissions:

  ```rust
  /// Assert a relation as a true fact about the model world.
  /// Corresponds to ASSERT_BPF_MTID.
  pub assert: Option<fn(
      &BpFamily,
      &BinaryPredicate,
      usize,                    // subj0
      Option<&'static str>,     // spec0
      usize,                    // subj1
      Option<&'static str>,     // spec1
      &mut [InferenceSubject],  // subjects
      &mut Vec<PropertyPermission>, // permissions
  ) -> bool>,
  ```

- [ ] Update `BinaryPredicateFamilies::assert` to pass subjects and permissions:

  ```rust
  pub fn assert(
      bp: &BinaryPredicate,
      subj0: usize,
      spec0: Option<&'static str>,
      subj1: usize,
      spec1: Option<&'static str>,
      families: &[BpFamily],
      subjects: &mut [InferenceSubject],
      permissions: &mut Vec<PropertyPermission>,
  ) -> bool {
      if let Some(family) = families.get(bp.relation_family) {
          if let Some(assert_fn) = family.methods.assert {
              return assert_fn(family, bp, subj0, spec0, subj1, spec1, subjects, permissions);
          }
      }
      false
  }
  ```

- [ ] Update existing tests in `binary_predicate_families.rs` to match the new signature.

### 3. Add module declaration

- [ ] Add `pub mod provision_relation;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 4. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/provision_relation.rs`:

  - Test that `start()` creates one family with the correct name.
  - Test that the provision family has `stock`, `typecheck`, `assert`, `schema`, and `describe_for_index` methods.
  - Test that `first_stock` on the provision family creates `R_provision` as a `make_pair` with a reversal.
  - Test that `R_provision` has `right_way_round = true` and its reversal has `right_way_round = false`.
  - Test that `describe_for_index` returns "provision".
  - Test that `set_index_details` works on `R_provision` (left="value", right="property").
  - Test that `typecheck` returns ALWAYS_MATCH (simplified).
  - Test that `schema` returns false (decline to compile).
  - Test that `assert` grants a property permission correctly.
  - Test that `assert` with no property name returns false.

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `ProvisionRelation::start()` creates one family: provision.
- [ ] The provision family has `stock`, `typecheck`, `assert`, `schema`, and `describe_for_index` methods.
- [ ] `first_stock` on the provision family creates `R_provision` as a `make_pair` with correct reversal links.
- [ ] `R_provision` has `right_way_round = true`; its reversal has `right_way_round = false`.
- [ ] `describe_for_index` returns "provision" for the provision family.
- [ ] `set_index_details` sets left="value", right="property" on `R_provision`.
- [ ] `typecheck` returns ALWAYS_MATCH (simplified).
- [ ] `schema` returns false (decline to compile).
- [ ] `assert` grants a property permission correctly.
- [ ] `BpFamilyMethods::assert` signature accepts subjects and permissions.
- [ ] `BinaryPredicateFamilies::assert` passes subjects and permissions to assert methods.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`SameAsRelations`**: `SameAsRelations::start()` (Chapter 3/Same Property Relation.w) — creates a bp_family that iterates over all properties at stage 2, depends on the full property system, is deferred.
- **`SettingPropertyRelations`**: `SettingPropertyRelations::start()` (Chapter 3/Setting Property Relation.w) — creates a bp_family with pending-text timing, depends on properties and kind subjects, is deferred.
- **`ComparativeRelations`**: `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) — creates a bp_family that depends on measurement adjectives, is deferred.
- **`InstanceAdjectives`**: `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) — depends on the adjective meaning system (assertions-module), is deferred.
- **`EitherOrPropertyAdjectives`**: `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) is deferred.
- **`MeasurementAdjectives`**: `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) is deferred.
- **`Instances::update_adjectival_forms`**: The call to update adjectival forms in the assert method is deferred. This requires the instance system.
- **`RTProperties::test_provision_schema`**: The run-time compilation of provision tests is deferred. The schema method returns false (decline to compile).
- **`Rvalues::to_property`**: The full property resolution from parse nodes is deferred. The assert method uses string property names.
- **Full typechecking**: The typecheck method is simplified to always return ALWAYS_MATCH. The full implementation would check `Kinds::get_construct == CON_property`.
- **`RelationSubjects` family**: The `RelationSubjects` inference subject family (Chapter 4/Relation Subjects.w) is deferred.
- **`ExplicitRelations`**: The explicit relations system (relation forms, storage, run-time) is deferred.
- **`PreformUtilities::wording`**: The full Preform wording system is deferred. This plan uses simplified string names.
- **`word_assemblage` struct**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`i6_schema` struct**: The full I6 schema struct is deferred. This plan uses simplified string schemas.
- **`annotated_i6_schema` struct**: The annotated schema struct used in schema compilation is deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) is deferred.
- **Run-time compilation**: All `RT*` functions (run-time compilation of relations, subjects, permissions) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
