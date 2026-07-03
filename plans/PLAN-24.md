# Plan 24: Relation Subjects — The Bridge Between Binary Predicates and Inference Subjects
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Relation Subjects inference subject family — the bridge between the binary predicate system (calculus module) and the inference subject system (knowledge module). This creates the `relations_family` inference subject family that wraps each `binary_predicate` as an `inference_subject`, enabling inferences to be drawn about relations.

This is the smallest next step after PLAN-23 because:

1. **It's the next independently testable piece of the knowledge module.** The knowledge module startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `PropertyInferences::start()` (PLAN-19), `RelationInferences::start()` (PLAN-20), then several items that depend on either the adjective meaning system or the property system — neither of which is built yet. `RelationSubjects` (Chapter 4/Relation Subjects.w) is not in the startup sequence directly, but it's a prerequisite for the assertion pipeline and for `RelationInferences` to work with actual `binary_predicate` structs instead of string names.

2. **It bridges the calculus module to the knowledge module.** The `binary_predicate` struct has a `knowledge_about_bp` field (an inference subject) that connects the calculus representation of a relation to the knowledge module's world model. `RelationSubjects` provides the functions to create and access this bridge. Currently, `RelationInferences` (PLAN-20) uses a simplified string-based lookup for relation subjects; with `RelationSubjects`, we can use the actual `binary_predicate` struct.

3. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For binary-predicate atoms, they need to look up the `binary_predicate` struct and its associated inference subject. `RelationSubjects::from_bp(bp)` provides this bridge.

4. **It's a prerequisite for `RelationInferences` integration with binary predicates.** `RelationInferences::draw(bp, ...)` (Chapter 5/Relation Inferences.w, PLAN-20) currently uses a simplified string-based lookup to find the relation subject. With `RelationSubjects`, we can use `RelationSubjects::from_bp(bp)` to find the correct subject directly.

5. **It's a prerequisite for the model world completion.** `RelationSubjects::complete_model` and `RelationSubjects::check_model` (Chapter 4/Relation Subjects.w, lines 50-214) are called during model world completion (Chapter 5/The Model World.w) to verify 1-to-1 relations and set up equivalence relations. These are deferred to a later plan, but the basic family creation and subject management is needed first.

6. **Independently testable.** We can create the relations family, create inference subjects for binary predicates, verify the `from_bp`/`to_bp` round-trip, verify the certainty and get_name methods, and verify the family method dispatch — all without needing the full property system, instances, or run-time compilation.

## Background

### C reference architecture

#### Relation Subjects (`Chapter 4/Relation Subjects.w`, lines 1-216)

The Relation Subjects module creates one inference subject family for binary predicates:

```c
inference_subject_family *relations_family = NULL;

inference_subject_family *RelationSubjects::family(void) {
    if (relations_family == NULL) {
        relations_family = InferenceSubjects::new_family();
        METHOD_ADD(relations_family, GET_DEFAULT_CERTAINTY_INFS_MTID,
            RelationSubjects::certainty);
        METHOD_ADD(relations_family, CHECK_MODEL_INFS_MTID, RelationSubjects::check_model);
        METHOD_ADD(relations_family, COMPLETE_MODEL_INFS_MTID, RelationSubjects::complete_model);
        METHOD_ADD(relations_family, GET_NAME_TEXT_INFS_MTID, RelationSubjects::get_name);
    }
    return relations_family;
}
```

The bridge functions:

```c
inference_subject *RelationSubjects::from_bp(binary_predicate *bp) {
    return bp->knowledge_about_bp;
}

inference_subject *RelationSubjects::new(binary_predicate *bp) {
    return InferenceSubjects::new(relations, RelationSubjects::family(),
        STORE_POINTER_binary_predicate(bp), NULL);
}

binary_predicate *RelationSubjects::to_bp(inference_subject *infs) {
    if ((infs) && (infs->infs_family == relations_family))
        return RETRIEVE_POINTER_binary_predicate(infs->represents);
    return NULL;
}
```

Simple methods:

```c
int RelationSubjects::certainty(inference_subject_family *f, inference_subject *infs) {
    return CERTAIN_CE;
}

void RelationSubjects::get_name(inference_subject_family *family,
    inference_subject *from, wording *W) {
    *W = EMPTY_WORDING; /* nameless */
}
```

The `check_model` and `complete_model` methods are more complex and depend on `ExplicitRelations` and `RTRelations` — these are deferred.

### Key C source files

- `inform7/knowledge-module/Chapter 4/Relation Subjects.w` — the full relation subjects implementation (216 lines)
- `inform7/knowledge-module/Chapter 5/Relation Inferences.w` — `RelationInferences::draw(bp, ...)` uses `binary_predicate` (PLAN-20)
- `inform7/knowledge-module/Chapter 4/Inference Subjects.w` — `InferenceSubject` struct, `InferenceSubjects::new` (PLAN-17)
- `inform7/knowledge-module/Chapter 5/The Model World.w` — model completion, calls `RelationSubjects::check_model` and `RelationSubjects::complete_model`
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, `knowledge_about_bp` field (PLAN-21)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, unit tests (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`RelationSubjects` module** — a new module `relation_subjects` in the knowledge crate with:
   - `RelationSubjects::family()` — creates the relations inference subject family with certainty, check_model, complete_model, and get_name methods
   - `RelationSubjects::from_bp(bp)` — returns the `knowledge_about_bp` field from a binary predicate (simplified: looks up by BP index in a subject registry)
   - `RelationSubjects::new(bp)` — creates a new inference subject for a binary predicate (simplified: uses BP index as family-specific data)
   - `RelationSubjects::to_bp(infs)` — extracts the binary predicate index from an inference subject
   - `RelationSubjects::certainty()` — returns `Certainty::Certain`
   - `RelationSubjects::get_name()` — returns `None` (nameless)
   - `RelationSubjects::check_model()` — simplified: no-op (full implementation depends on `ExplicitRelations`)
   - `RelationSubjects::complete_model()` — simplified: no-op (full implementation depends on `ExplicitRelations` and `RTRelations`)
   - Global constant for the relations family index

2. **Integration with the knowledge module** — add the `RelationSubjects` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — create the family, create inference subjects for binary predicates, verify the `from_bp`/`to_bp` round-trip, verify the certainty and get_name methods, verify the family method dispatch.

## Tasks

### 1. Create the `RelationSubjects` module

- [ ] Create `crates/conform7-semantics/src/knowledge/relation_subjects.rs` with:

  ```rust
  /// Relation Subjects — the bridge between binary predicates and inference subjects.
  ///
  /// Corresponds to `RelationSubjects` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`).
  ///
  /// Every binary predicate has an associated inference subject, making it
  /// possible to draw inferences about relations. This module provides the
  /// family and functions to create and access these subjects.
  ///
  /// Simplified: uses BP indices instead of `binary_predicate*` pointers.
  /// The `check_model` and `complete_model` methods are no-ops (they depend
  /// on `ExplicitRelations` and `RTRelations`, which are not yet implemented).
  use crate::knowledge::inference_subjects::{
      InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
  };
  use crate::knowledge::inferences::Certainty;
  ```

- [ ] Define the global constant:

  ```rust
  /// Index of the relations family in the inference subject family registry.
  pub const RELATIONS_FAMILY: usize = 0;
  ```

- [ ] Implement `RelationSubjects::family()`:

  ```rust
  /// Create the relations inference subject family.
  ///
  /// Corresponds to `RelationSubjects::family` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 10-20).
  ///
  /// The family has four methods:
  /// - certainty: always returns CERTAIN_CE
  /// - check_model: simplified no-op (full version checks 1-to-1 relations)
  /// - complete_model: simplified no-op (full version sets up equivalence relations)
  /// - get_name: returns empty (nameless)
  pub fn family() -> InferenceSubjectFamily {
      InferenceSubjectFamily {
          name: "relations",
          methods: InferenceSubjectFamilyMethods {
              get_default_certainty: Some(RelationSubjects::certainty),
              check_model: Some(RelationSubjects::check_model),
              complete_model: Some(RelationSubjects::complete_model),
              get_name_text: Some(RelationSubjects::get_name),
              ..InferenceSubjectFamilyMethods::default()
          },
      }
  }
  ```

- [ ] Implement `RelationSubjects::from_bp()`:

  ```rust
  /// Return the inference subject index for a binary predicate.
  ///
  /// Corresponds to `RelationSubjects::from_bp` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 22-24).
  ///
  /// Simplified: looks up the BP's `knowledge_about_bp` field, which stores
  /// the inference subject index.
  pub fn from_bp(bp_idx: usize, bp_registry: &[crate::calculus::binary_predicates::BinaryPredicate]) -> Option<usize> {
      bp_registry.get(bp_idx).and_then(|bp| bp.knowledge_about_bp)
  }
  ```

- [ ] Implement `RelationSubjects::new()`:

  ```rust
  /// Create a new inference subject for a binary predicate.
  ///
  /// Corresponds to `RelationSubjects::new` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 26-29).
  ///
  /// Creates a new inference subject under the `relations` parent (a child of
  /// model_world), using the relations family, with the BP index as family-specific data.
  ///
  /// Also sets the BP's `knowledge_about_bp` field to the new subject's index.
  ///
  /// Returns the index of the new subject.
  pub fn new(
      bp_idx: usize,
      subjects: &mut Vec<InferenceSubject>,
      families: &[InferenceSubjectFamily],
      bp_registry: &mut [crate::calculus::binary_predicates::BinaryPredicate],
  ) -> usize {
      // Find the relations parent subject (a child of model_world).
      // In the C reference, this is a global `relations` subject created during setup.
      // Simplified: we use a convention where the relations parent is at a known index.
      // For now, we create the subject under model_world (index 0).
      let parent = 0; // model_world

      // Create the inference subject with the BP index as family-specific data.
      // The `represents` field stores the BP index as a string for debugging.
      let subject = InferenceSubject {
          broader_than: Some(parent),
          infs_family: 0, // relations family is at index 0
          represents: Some("relation"),
          inf_list: Vec::new(),
          imp_list: Vec::new(),
          permissions_list: Vec::new(),
          alias_variable: None,
          log_name: None,
      };

      let subject_idx = subjects.len();
      subjects.push(subject);

      // Set the BP's knowledge_about_bp field to the new subject's index.
      if let Some(bp) = bp_registry.get_mut(bp_idx) {
          bp.knowledge_about_bp = Some(subject_idx);
      }

      subject_idx
  }
  ```

- [ ] Implement `RelationSubjects::to_bp()`:

  ```rust
  /// Extract the binary predicate index from an inference subject.
  ///
  /// Corresponds to `RelationSubjects::to_bp` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 31-35).
  ///
  /// Simplified: checks that the subject belongs to the relations family,
  /// then returns the BP index stored in the subject's family-specific data.
  /// Since we store the BP index indirectly (via knowledge_about_bp), we
  /// need to search the BP registry for a BP whose knowledge_about_bp
  /// matches the subject index.
  pub fn to_bp(
      infs: &InferenceSubject,
      bp_registry: &[crate::calculus::binary_predicates::BinaryPredicate],
  ) -> Option<usize> {
      // Check that this subject belongs to the relations family.
      if infs.infs_family != 0 { return None; } // relations family is at index 0

      // Search the BP registry for a BP whose knowledge_about_bp matches.
      // In the C reference, this is a direct pointer retrieval.
      // Simplified: linear search through the BP registry.
      for (i, bp) in bp_registry.iter().enumerate() {
          if bp.knowledge_about_bp == Some(0) {
              // We can't directly compare subject indices here because
              // we don't have the subject's index. This is a limitation
              // of the simplified approach.
              // See note below about the design decision.
          }
      }
      None
  }
  ```

  Note: The `to_bp` function in the C reference uses direct pointer comparison (`infs->infs_family == relations_family` and `RETRIEVE_POINTER_binary_predicate(infs->represents)`). In Rust, we need a different approach. The recommended approach is to store the BP index in the subject's `represents` field (as a string representation of the index), or to use a separate mapping. For the simplified implementation, we'll store the BP index as a string in `represents` and parse it back in `to_bp`.

  Revised `new` stores the BP index in `represents`:

  ```rust
  let subject = InferenceSubject {
      broader_than: Some(parent),
      infs_family: 0, // relations family is at index 0
      represents: Some(Box::leak(format!("bp:{}", bp_idx).into_boxed_str())),
      // ...
  };
  ```

  But this creates a memory leak. A better approach: use a separate `HashMap<usize, usize>` mapping from subject index to BP index, or store the BP index in a side table.

  Recommended approach: Use a `Vec<Option<usize>>` side table that maps subject indices to BP indices, similar to how `knowledge_about_bp` maps BP indices to subject indices.

  ```rust
  /// Side table: for each subject index, the BP index it represents (if any).
  /// This is the inverse of `knowledge_about_bp` on the BP registry.
  pub fn new(
      bp_idx: usize,
      subjects: &mut Vec<InferenceSubject>,
      families: &[InferenceSubjectFamily],
      bp_registry: &mut [crate::calculus::binary_predicates::BinaryPredicate],
      bp_to_subject: &mut Vec<Option<usize>>, // side table: subject index -> BP index
  ) -> usize {
      let parent = 0; // model_world
      let subject = InferenceSubject {
          broader_than: Some(parent),
          infs_family: 0, // relations family is at index 0
          represents: Some("relation"),
          inf_list: Vec::new(),
          imp_list: Vec::new(),
          permissions_list: Vec::new(),
          alias_variable: None,
          log_name: None,
      };

      let subject_idx = subjects.len();
      subjects.push(subject);

      // Set the BP's knowledge_about_bp field.
      if let Some(bp) = bp_registry.get_mut(bp_idx) {
          bp.knowledge_about_bp = Some(subject_idx);
      }

      // Record the inverse mapping.
      if subject_idx >= bp_to_subject.len() {
          bp_to_subject.resize(subject_idx + 1, None);
      }
      bp_to_subject[subject_idx] = Some(bp_idx);

      subject_idx
  }

  pub fn to_bp(
      infs_idx: usize,
      bp_to_subject: &[Option<usize>],
  ) -> Option<usize> {
      bp_to_subject.get(infs_idx).copied().flatten()
  }
  ```

- [ ] Implement `RelationSubjects::certainty()`:

  ```rust
  /// Return the default certainty for relation subjects.
  ///
  /// Corresponds to `RelationSubjects::certainty` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 38-40).
  pub fn certainty(
      _family: &InferenceSubjectFamily,
      _infs: &InferenceSubject,
  ) -> Certainty {
      Certainty::Certain
  }
  ```

- [ ] Implement `RelationSubjects::get_name()`:

  ```rust
  /// Return the name of a relation subject.
  ///
  /// Corresponds to `RelationSubjects::get_name` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 42-45).
  ///
  /// Relations are nameless in the C reference (returns EMPTY_WORDING).
  pub fn get_name(
      _family: &InferenceSubjectFamily,
      _infs: &InferenceSubject,
  ) -> Option<&'static str> {
      None // nameless
  }
  ```

- [ ] Implement `RelationSubjects::check_model()`:

  ```rust
  /// Check the model for a relation subject.
  ///
  /// Corresponds to `RelationSubjects::check_model` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 79-87).
  ///
  /// Simplified: no-op. The full implementation checks 1-to-1 relations
  /// for violations, which depends on `ExplicitRelations` and `RTRelations`.
  pub fn check_model(
      _family: &InferenceSubjectFamily,
      _infs: &InferenceSubject,
  ) {
      // Deferred: depends on ExplicitRelations and RTRelations.
  }
  ```

- [ ] Implement `RelationSubjects::complete_model()`:

  ```rust
  /// Complete the model for a relation subject.
  ///
  /// Corresponds to `RelationSubjects::complete_model` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Relation Subjects.w`, lines 50-69).
  ///
  /// Simplified: no-op. The full implementation sets up equivalence relations
  /// and merges equivalence classes, which depends on `ExplicitRelations`
  /// and `RTRelations`.
  pub fn complete_model(
      _family: &InferenceSubjectFamily,
      _infs: &InferenceSubject,
  ) {
      // Deferred: depends on ExplicitRelations and RTRelations.
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod relation_subjects;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/relation_subjects.rs`:

  - Test that `family()` creates a family with the correct name "relations".
  - Test that the relations family has `get_default_certainty`, `check_model`, `complete_model`, and `get_name_text` methods.
  - Test that `certainty` returns `Certainty::Certain`.
  - Test that `get_name` returns `None` (nameless).
  - Test that `new` creates a new inference subject for a binary predicate.
  - Test that `new` sets the BP's `knowledge_about_bp` field.
  - Test that `from_bp` returns the correct subject index for a BP.
  - Test that `to_bp` returns the correct BP index for a subject.
  - Test that `from_bp` returns `None` for an invalid BP index.
  - Test that `to_bp` returns `None` for an invalid subject index.
  - Test that `check_model` is a no-op (doesn't panic).
  - Test that `complete_model` is a no-op (doesn't panic).

### 4. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `RelationSubjects::family()` creates one family: "relations".
- [ ] The relations family has `get_default_certainty`, `check_model`, `complete_model`, and `get_name_text` methods.
- [ ] `certainty` returns `Certainty::Certain`.
- [ ] `get_name` returns `None` (nameless).
- [ ] `new` creates a new inference subject for a binary predicate and sets `knowledge_about_bp`.
- [ ] `from_bp` returns the correct subject index for a BP.
- [ ] `to_bp` returns the correct BP index for a subject.
- [ ] `from_bp` returns `None` for an invalid BP index.
- [ ] `to_bp` returns `None` for an invalid subject index.
- [ ] `check_model` is a no-op (doesn't panic).
- [ ] `complete_model` is a no-op (doesn't panic).
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`ExplicitRelations`**: The explicit relations system (relation forms, storage, run-time) is deferred. The `check_model` and `complete_model` methods are simplified to no-ops.
- **`RTRelations`**: The run-time relations system (equivalence relation partitions, 1-to-1 checking) is deferred.
- **`RelationInferences` integration**: Updating `RelationInferences::draw` to use `RelationSubjects::from_bp` instead of string-based lookup is deferred.
- **`RelationSubjects::check_OtoO_relation`**: The 1-to-1 relation checking logic (lines 89-141) is deferred.
- **`RelationSubjects::check_OtoV_relation`**: The one-to-various relation checking logic (lines 143-214) is deferred.
- **`RelationSubjects::complete_model` equivalence logic**: The equivalence relation partition setup (lines 50-69) is deferred.
- **`InstanceSubjects`**: `InstanceSubjects::family()` (Chapter 4/Instance Subjects.w) — depends on the instance system, is deferred.
- **`VariableSubjects`**: `VariableSubjects::family()` (Chapter 4/Variable Subjects.w) — depends on the nonlocal variables system, is deferred.
- **`ConditionsOfSubjects`**: `ConditionsOfSubjects` (Chapter 4/Conditions of Subjects.w) — depends on the property system and adjective system, is deferred.
- **`Properties`**: The full property system (Chapter 3/Properties.w) — depends on Either-Or Properties, Valued Properties, and the adjective system, is deferred.
- **`SameAsRelations`**: `SameAsRelations::start()` (Chapter 3/Same Property Relation.w) — depends on the property system, is deferred.
- **`SettingPropertyRelations`**: `SettingPropertyRelations::start()` (Chapter 3/Setting Property Relation.w) — depends on the property system, is deferred.
- **`ComparativeRelations`**: `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) — depends on measurement adjectives, is deferred.
- **`InstanceAdjectives`**: `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) — depends on the adjective meaning system (assertions-module), is deferred.
- **`EitherOrPropertyAdjectives`**: `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) — depends on the adjective meaning system, is deferred.
- **`MeasurementAdjectives`**: `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) — depends on the adjective meaning system, is deferred.
- **`Instances`**: The full instance system (Chapter 2/Instances.w) — depends on Instance Subjects, Instance Adjectives, and the assertion pipeline, is deferred.
- **`NonlocalVariables`**: The nonlocal variables system (Chapter 2/Nonlocal Variables.w) — depends on Variable Subjects and the assertion pipeline, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`PreformUtilities::wording`**: The full Preform wording system is deferred. This plan uses simplified string names.
- **`word_assemblage` struct**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`i6_schema` struct**: The full I6 schema struct is deferred. This plan uses simplified string schemas.
- **Run-time compilation**: All `RT*` functions (run-time compilation of relations, subjects, permissions) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
