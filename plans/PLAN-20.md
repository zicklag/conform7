# Plan 20: Relation Inferences — The Relation Inference Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Relation Inferences inference family — the concrete `InferenceFamily` that stores facts about relations holding between two subjects or values. This is the second inference family to be created in the knowledge module startup sequence, and it enables the knowledge module to represent facts about relationships in the game world (e.g., "Charles knows Sebastian", "the box contains the key").

This is the smallest next step after PLAN-19 because:

1. **It's the second thing called in `KnowledgeModule::start()`.** The C reference (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, line 38) calls `RelationInferences::start()` immediately after `PropertyInferences::start()`. After PLAN-19 implemented the property inference family, the next logical step is to create the relation inference family.

2. **It's a concrete `InferenceFamily`.** PLAN-17 created the `InferenceFamily` infrastructure (struct, methods, `create_inference`, `join_inference`, `cmp`). PLAN-19 created the first concrete family (`property_inf`). `RelationInferences` provides the `relation_inf` family with `log_details` and `cmp` methods — a second concrete family that exercises the same infrastructure in a different way.

3. **It bridges the inference system to relations.** `RelationInferences::new(subj0, subj1, val0, val1)` creates an inference that a relation holds between two terms. `RelationInferences::draw(rel, subj0, subj1)` creates and joins the inference to a relation subject in one call. These are the fundamental operations that the assertion pipeline will use to record relational facts about the game world.

4. **Independently testable.** We can create a `relation_inf` family, create relation inferences with simplified relation references (string names) and term references (subject indices or value strings), join them to subjects, compare them, and verify the results — all without needing the full `binary_predicate` struct, `RelationSubjects`, or `ExplicitRelations`.

5. **Prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For binary-predicate atoms, they call `RelationInferences::draw`. Without the relation inference family, the assertion pipeline cannot record relational facts.

6. **Prerequisite for model world completion.** `World::stage_IV` (Chapter 5/The Model World.w) and `RelationSubjects::check_model` use `RelationInferences::get_term_subjects` and `RelationInferences::get_term_specs` to verify relation consistency (e.g., one-to-one relation violations). These query functions depend on the relation inference family.

## Background

### C reference architecture

#### Relation Inferences (`Chapter 5/Relation Inferences.w`, lines 1-121)

Relation inferences say that a relation holds between two subjects or values:

```c
inference_family *relation_inf = NULL;

void RelationInferences::start(void) {
    relation_inf = Inferences::new_family(I"relation_inf");
    METHOD_ADD(relation_inf, LOG_DETAILS_INF_MTID, RelationInferences::log_details);
    METHOD_ADD(relation_inf, COMPARE_INF_MTID, RelationInferences::cmp);
}
```

The inference data stores two terms, which can be either subjects or values (but not both):

```c
typedef struct relation_inference_data {
    struct inference_subject *terms_as_subjects[2];
    struct parse_node *terms_as_values[2];
    CLASS_DEFINITION
} relation_inference_data;
```

Key operations:
- `RelationInferences::new(subj0, subj1, val0, val1)` — creates a new relation inference with the prevailing mood (certainty). Terms can be given either as subjects or as arbitrary values. Exactly one pair of pointers is non-NULL.
- `RelationInferences::draw(bp, subj0, subj1)` — creates and joins an inference to the relation subject for a binary predicate (via `RelationSubjects::from_bp(bp)`)
- `RelationInferences::draw_spec(bp, val0, val1)` — same but with values instead of subjects
- `RelationInferences::log_details` — logs the two terms
- `RelationInferences::cmp` — compares two relation inferences, comparing terms in order (subject 1, subject 0, value 1, value 0). Returns `CI_DIFFER_IN_TOPIC` if any term differs, `CI_DIFFER_IN_COPY_ONLY` if only the inference identity differs, `CI_IDENTICAL` if the same inference.
- `RelationInferences::get_term_subjects(i, subj1, subj2)` — returns the subject terms from an inference
- `RelationInferences::get_term_specs(i, val1, val2)` — returns the value terms from an inference

### Key C source files

- `inform7/knowledge-module/Chapter 5/Relation Inferences.w` — the full relation inference family (121 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `RelationInferences::start()` (line 38)
- `inform7/knowledge-module/Chapter 5/Inferences.w` — `InferenceFamily` infrastructure, `create_inference`, `join_inference`, `cmp`, `measure_infs`, `measure_pn` (used by Relation Inferences)
- `inform7/knowledge-module/Chapter 4/Relation Subjects.w` — `RelationSubjects` family (deferred, but `from_bp` is used by `draw`)
- `inform7/knowledge-module/Chapter 1/Assert Propositions.w` — assertion pipeline, calls `RelationInferences::draw`
- `inform7/knowledge-module/Chapter 5/The Model World.w` — model completion stages, uses `RelationInferences::get_term_subjects`

### Current Rust state

- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, `InferenceComparison` enum, `JoinResult` enum, `create_inference`, `cmp`, `join_inference`, `render_impossible`.
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, subject hierarchy operations, method dispatch.
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` family, `PropertyInferenceData` struct, `start()`, `new()`, `draw()`, `draw_from_metadata()`, `draw_negated()`, access functions, unit tests.
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct, `find`, `grant`, accessors.
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()`, fundamental subjects (`model_world`, `global_constants`, `global_variables`).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` family, `from_kind`, `to_kind`, `has_properties`, lattice callbacks.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.

### What's needed

1. **`RelationInferenceData` struct** — stores two term slots (subjects or values) for a relation inference. Simplified: use subject indices and optional value strings instead of `inference_subject*` and `parse_node*` pointers.
2. **`RelationInferences::start()`** — creates the `relation_inf` family with methods for `log_details` and `cmp`.
3. **`RelationInferences::new(subj0, subj1, val0, val1)`** — creates a new relation inference with the prevailing mood (certainty), defaulting to `Certain` if the mood is unknown.
4. **`RelationInferences::draw(rel_name, subj0, subj1, ...)`** — creates and joins an inference to a relation subject (simplified: use string name for the relation, create a lightweight relation subject).
5. **`RelationInferences::draw_spec(rel_name, val0, val1, ...)`** — same but with values instead of subjects.
6. **Family methods** — `log_details` (logs the two terms), `cmp` (compares two relation inferences, comparing terms in order).
7. **Access functions** — `get_term_subjects(i)`, `get_term_specs(i)`.
8. **Unit tests** — create the family, create relation inferences with subject terms, create relation inferences with value terms, join them to subjects, compare them, test draw/draw_spec, test access functions.

## Tasks

### 1. Create the `RelationInferenceData` struct

- [ ] Create `crates/conform7-semantics/src/knowledge/relation_inferences.rs` with:

  ```rust
  /// Data stored in a relation inference.
  ///
  /// Corresponds to `relation_inference_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 28-32).
  ///
  /// Stores two terms, which can be either subjects or values (but not both).
  /// Simplified: uses subject indices and optional value strings instead of
  /// `inference_subject*` and `parse_node*` pointers.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct RelationInferenceData {
      /// The two terms as subject indices (if subject-based).
      /// Both are None if value-based.
      pub terms_as_subjects: [Option<usize>; 2],
      /// The two terms as value strings (if value-based).
      /// Both are None if subject-based.
      pub terms_as_values: [Option<&'static str>; 2],
  }
  ```

- [ ] Add `pub mod relation_inferences;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 2. Implement `RelationInferences::start()` — create the family

- [ ] Implement `RelationInferences::start()` that creates the `relation_inf` family:

  ```rust
  /// Create the relation inference family.
  ///
  /// Corresponds to `RelationInferences::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 14-18).
  ///
  /// Returns the `relation_inf` family with methods for logging and comparison.
  /// Note: the C version does not register an `explain_contradiction` method.
  pub fn start() -> InferenceFamily {
      InferenceFamily {
          name: "relation_inf",
          methods: InferenceFamilyMethods {
              log_details: |inf| {
                  // Simplified: log the data index
                  if let Some(di) = inf.data_index {
                      format!("(relation_inference_data[{}])", di)
                  } else {
                      String::new()
                  }
              },
              compare: |a, b| {
                  // Compare two relation inferences by their data index
                  // (simplified: the full C version compares individual terms)
                  let a_di = a.data_index.unwrap_or(usize::MAX);
                  let b_di = b.data_index.unwrap_or(usize::MAX);
                  if a_di > b_di { 3 } // CI_DIFFER_IN_TOPIC
                  else if a_di < b_di { -3 } // -CI_DIFFER_IN_TOPIC
                  else { 0 } // CI_IDENTICAL
              },
              explain_contradiction: |_a, _b, _similarity, _subject| {
                  // Relation inferences don't register explain_contradiction
                  // in the C reference, but the method table requires it.
                  false
              },
          },
      }
  }
  ```

  Note: The `compare` method is simplified for now. The full C version (`RelationInferences::cmp`, lines 81-101) compares individual terms using `Inferences::measure_infs` and `Inferences::measure_pn`, comparing subject 1 first, then subject 0, then value 1, then value 0. The simplified version uses the data index as a proxy. This will be refined when the full relation subject system is integrated.

### 3. Implement `RelationInferences::new()` and `RelationInferences::draw()`

- [ ] Implement `RelationInferences::new(subj0, subj1, val0, val1, data_registry, families, inferences) -> usize`:

  ```rust
  /// Create a new relation inference.
  ///
  /// Corresponds to `RelationInferences::new` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 34-44).
  ///
  /// Creates a `RelationInferenceData` with the given terms, then creates
  /// an `Inference` with the prevailing mood (defaulting to `Certain` if
  /// the mood is unknown).
  ///
  /// Terms can be given either as subjects or as values, but not both.
  /// Exactly one pair should be non-None.
  ///
  /// Returns the index of the new inference in the inferences registry.
  pub fn new(
      subj0: Option<usize>,
      subj1: Option<usize>,
      val0: Option<&'static str>,
      val1: Option<&'static str>,
      data_registry: &mut Vec<RelationInferenceData>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
  ) -> usize {
      // Find the relation_inf family index
      let family_idx = families.iter().position(|f| f.name == "relation_inf")
          .expect("RelationInferences::start must be called first");

      // Create the inference data
      let data = RelationInferenceData {
          terms_as_subjects: [subj0, subj1],
          terms_as_values: [val0, val1],
      };

      // Store the data and create the inference
      let data_idx = data_registry.len();
      data_registry.push(data);

      let idx = inferences.len();
      inferences.push(Inference {
          family: family_idx,
          data: None, // data is stored in the registry, not inline
          data_index: Some(data_idx),
          certainty: Certainty::Certain,
          inferred_from: None,
          drawn_during_stage: 0,
          drawn_from_metadata: false,
      });
      idx
  }
  ```

- [ ] Implement `RelationInferences::draw(rel_name, subj0, subj1, data_registry, families, inferences, subjects) -> JoinResult`:

  ```rust
  /// Create a relation inference and join it to a relation subject.
  ///
  /// Corresponds to `RelationInferences::draw` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 49-53).
  ///
  /// Simplified: creates a lightweight relation subject from the relation name
  /// if one doesn't exist, then joins the inference to it.
  pub fn draw(
      rel_name: &'static str,
      subj0: Option<usize>,
      subj1: Option<usize>,
      data_registry: &mut Vec<RelationInferenceData>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      subjects: &mut Vec<InferenceSubject>,
      subject_families: &[InferenceSubjectFamily],
  ) -> JoinResult {
      // Find or create a relation subject for this relation name
      let rel_subj_idx = find_or_create_relation_subject(
          rel_name, subjects, subject_families,
      );

      let inf_idx = RelationInferences::new(
          subj0, subj1, None, None, data_registry, families, inferences,
      );
      inferences[inf_idx].join(&mut subjects[rel_subj_idx], families, inferences)
  }
  ```

  Note: `find_or_create_relation_subject` is a helper that looks up an existing relation subject by name, or creates a new one using the fundamentals family (simplified). The full C version uses `RelationSubjects::from_bp(bp)` which retrieves the subject from the `binary_predicate` struct. We'll refine this when `RelationSubjects` and `binary_predicate` are integrated.

- [ ] Implement `RelationInferences::draw_spec(rel_name, val0, val1, ...)`:

  ```rust
  /// Create a relation inference with value terms and join it to a relation subject.
  ///
  /// Corresponds to `RelationInferences::draw_spec` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 55-60).
  pub fn draw_spec(
      rel_name: &'static str,
      val0: &'static str,
      val1: &'static str,
      data_registry: &mut Vec<RelationInferenceData>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      subjects: &mut Vec<InferenceSubject>,
      subject_families: &[InferenceSubjectFamily],
  ) -> JoinResult {
      // Find or create a relation subject for this relation name
      let rel_subj_idx = find_or_create_relation_subject(
          rel_name, subjects, subject_families,
      );

      let inf_idx = RelationInferences::new(
          None, None, Some(val0), Some(val1), data_registry, families, inferences,
      );
      inferences[inf_idx].join(&mut subjects[rel_subj_idx], families, inferences)
  }
  ```

- [ ] Implement the `find_or_create_relation_subject` helper:

  ```rust
  /// Find or create a relation subject for a given relation name.
  ///
  /// Simplified: looks up an existing subject by log_name, or creates a new
  /// fundamental subject under model_world. The full C version uses
  /// `RelationSubjects::from_bp(bp)` and `RelationSubjects::new(bp)`.
  fn find_or_create_relation_subject(
      rel_name: &'static str,
      subjects: &mut Vec<InferenceSubject>,
      families: &[InferenceSubjectFamily],
  ) -> usize {
      // Look for an existing relation subject with this name
      for (i, subj) in subjects.iter().enumerate() {
          if subj.log_name == Some(rel_name) {
              return i;
          }
      }

      // Create a new relation subject under model_world
      let idx = subjects.len();
      subjects.push(InferenceSubject::new(
          0, // fundamentals family (index 0)
          Some(0), // broader_than = model_world
          Some(rel_name), // represents
          Some(rel_name), // log_name
      ));
      idx
  }
  ```

### 4. Implement access functions

- [ ] Implement `RelationInferences::get_term_subjects(inf, data_registry) -> (Option<usize>, Option<usize>)`:

  ```rust
  /// Get the subject terms from a relation inference.
  ///
  /// Corresponds to `RelationInferences::get_term_subjects` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 106-112).
  pub fn get_term_subjects(
      inf: &Inference,
      data_registry: &[RelationInferenceData],
  ) -> (Option<usize>, Option<usize>) {
      if let Some(di) = inf.data_index {
          if let Some(data) = data_registry.get(di) {
              return (data.terms_as_subjects[0], data.terms_as_subjects[1]);
          }
      }
      (None, None)
  }
  ```

- [ ] Implement `RelationInferences::get_term_specs(inf, data_registry) -> (Option<&'static str>, Option<&'static str>)`:

  ```rust
  /// Get the value terms from a relation inference.
  ///
  /// Corresponds to `RelationInferences::get_term_specs` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Relation Inferences.w`, lines 114-120).
  pub fn get_term_specs(
      inf: &Inference,
      data_registry: &[RelationInferenceData],
  ) -> (Option<&'static str>, Option<&'static str>) {
      if let Some(di) = inf.data_index {
          if let Some(data) = data_registry.get(di) {
              return (data.terms_as_values[0], data.terms_as_values[1]);
          }
      }
      (None, None)
  }
  ```

### 5. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/relation_inferences.rs`:

  - Test that `start()` creates a family with name "relation_inf".
  - Test that the relation_inf family's `log_details` method works.
  - Test that `new()` creates an inference with the correct data index.
  - Test that `new()` creates an inference with `Certain` certainty by default.
  - Test that `new()` with subject terms stores them correctly.
  - Test that `new()` with value terms stores them correctly.
  - Test that `draw()` creates and joins an inference to a relation subject.
  - Test that `draw()` returns `Joined` for a new inference.
  - Test that `draw()` returns `DiscardedRedundant` for a duplicate inference.
  - Test that `draw_spec()` creates and joins an inference with value terms.
  - Test that `get_term_subjects()` returns the correct subject terms.
  - Test that `get_term_specs()` returns the correct value terms.
  - Test that two relation inferences with the same data compare as identical.
  - Test that two relation inferences with different data compare as different.
  - Test that the family's `compare` method returns 0 for identical inferences.
  - Test that the family's `compare` method returns non-zero for different inferences.
  - Test that `find_or_create_relation_subject` creates a new subject for an unknown relation.
  - Test that `find_or_create_relation_subject` reuses an existing subject.

### 6. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `RelationInferences::start()` creates an `InferenceFamily` with name "relation_inf".
- [ ] The relation_inf family has `log_details`, `compare`, and `explain_contradiction` methods.
- [ ] `RelationInferences::new(subj0, subj1, val0, val1)` creates an inference with the correct data.
- [ ] `RelationInferences::new(subj0, subj1, val0, val1)` creates an inference with `Certain` certainty by default.
- [ ] `RelationInferences::new()` with subject terms stores them in the data registry.
- [ ] `RelationInferences::new()` with value terms stores them in the data registry.
- [ ] `RelationInferences::draw(rel_name, subj0, subj1)` creates and joins an inference to a relation subject.
- [ ] `RelationInferences::draw()` returns `Joined` for a new inference.
- [ ] `RelationInferences::draw()` returns `DiscardedRedundant` for a duplicate inference.
- [ ] `RelationInferences::draw_spec(rel_name, val0, val1)` creates and joins an inference with value terms.
- [ ] `RelationInferences::get_term_subjects(inf)` returns the correct subject terms.
- [ ] `RelationInferences::get_term_specs(inf)` returns the correct value terms.
- [ ] Two relation inferences with the same data compare as identical.
- [ ] Two relation inferences with different data compare as different.
- [ ] The family's `compare` method returns 0 for identical inferences.
- [ ] The family's `compare` method returns non-zero for different inferences.
- [ ] `find_or_create_relation_subject` creates a new subject for an unknown relation.
- [ ] `find_or_create_relation_subject` reuses an existing subject.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Full `binary_predicate` struct**: The full `binary_predicate` struct with relation metadata is deferred. This plan uses simplified string names for relations.
- **`RelationSubjects` family**: The `RelationSubjects` inference subject family (Chapter 4/Relation Subjects.w) is deferred. This plan uses lightweight fundamental subjects as stand-ins for relation subjects.
- **`ExplicitRelations`**: The explicit relations system (relation forms, storage, run-time) is deferred.
- **`InstanceAdjectives`**: `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) — the next call in the startup sequence — is deferred.
- **`EitherOrPropertyAdjectives`**: `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) is deferred.
- **`MeasurementAdjectives`**: `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) is deferred.
- **`SameAsRelations`**: `SameAsRelations::start()` (Chapter 3/Same Property Relation.w) is deferred.
- **`SettingPropertyRelations`**: `SettingPropertyRelations::start()` (Chapter 3/Setting Property Relation.w) is deferred.
- **`ComparativeRelations`**: `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) is deferred.
- **`ProvisionRelation`**: `ProvisionRelation::start()` (Chapter 3/The Provision Relation.w) is deferred.
- **Instance Subjects**: `InstanceSubjects` (Chapter 4/Instance Subjects.w) — the instances family of inference subjects — is deferred.
- **Variable Subjects**: `VariableSubjects` (Chapter 4/Variable Subjects.w) — the variables family of inference subjects — is deferred.
- **Conditions of Subjects**: `Conditions of Subjects` (Chapter 4/Conditions of Subjects.w) — conditions on subjects — is deferred.
- **Instances**: `Instances` (Chapter 2/Instances.w) — the instance creation and management system — is deferred.
- **Properties**: `Properties` (Chapter 3/Properties.w) — the property creation and management system — is deferred.
- **Nonlocal variables**: `NonlocalVariables` (Chapter 2/Nonlocal Variables.w) — global variable management — is deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline that converts propositions to inferences — is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the model world construction stages — is deferred.
- **The naming thicket**: `The Naming Thicket` (Chapter 5/The Naming Thicket.w) — naming system — is deferred.
- **Indefinite appearance**: `Indefinite Appearance` (Chapter 5/Indefinite Appearance.w) — indefinite appearance text — is deferred.
- **Plugin system**: The plugin attachment system (`additional_data_for_plugins`, `PluginCalls`) is deferred.
- **Run-time compilation**: All `RT*` functions (run-time compilation of subjects, permissions, instances, properties) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
