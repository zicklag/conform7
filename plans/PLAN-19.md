# Plan 19: Property Inferences — The Property Inference Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Property Inferences inference family — the concrete `InferenceFamily` that stores facts about properties of subjects. This is the first inference family to be created in the knowledge module startup sequence, and it makes the inference system actually useful for representing real game-world facts.

This is the smallest next step after PLAN-18 because:

1. **It's the first thing called in `KnowledgeModule::start()`.** The C reference (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, line 37) calls `PropertyInferences::start()` before any other startup function. After PLAN-18 implemented the fundamental subjects and kind subjects, the next logical step is to create the inference families that those subjects will use.

2. **It's a concrete `InferenceFamily`.** PLAN-17 created the `InferenceFamily` infrastructure (struct, methods, `create_inference`, `join_inference`, `cmp`). What's missing is a concrete family with real methods. `PropertyInferences` provides the `property_inf` family with `log_details`, `cmp`, and `explain_contradiction` methods.

3. **It bridges the inference system to properties.** `PropertyInferences::new(subj, prn, val)` creates an inference that a subject has a property with a given value. `PropertyInferences::draw(subj, prn, val)` creates and joins the inference in one call. These are the fundamental operations that the assertion pipeline will use to record facts about the game world.

4. **Independently testable.** We can create a `property_inf` family, create property inferences with simplified property references (string names), join them to subjects, compare them, and verify the results — all without needing the full `property` struct, `EitherOrProperties`, or `ValueProperties`.

5. **Prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For property-related atoms, they call `PropertyInferences::draw`. Without the property inference family, the assertion pipeline cannot record property facts.

6. **Prerequisite for model world completion.** `World::stage_IV` (Chapter 5/The Model World.w) uses `PropertyInferences::either_or_state`, `PropertyInferences::verify_prop_states`, and `PropertyInferences::has_or_can_have` to check consistency. These query functions depend on the property inference family.

## Background

### C reference architecture

#### Property Inferences (`Chapter 5/Property Inferences.w`, lines 1-355)

Property inferences say that a subject has a property, which can be either-or or valued:

```c
inference_family *property_inf = NULL;

void PropertyInferences::start(void) {
    property_inf = Inferences::new_family(I"property_inf");
    METHOD_ADD(property_inf, LOG_DETAILS_INF_MTID, PropertyInferences::log_details);
    METHOD_ADD(property_inf, COMPARE_INF_MTID, PropertyInferences::cmp);
    METHOD_ADD(property_inf, EXPLAIN_CONTRADICTION_INF_MTID,
        PropertyInferences::explain_contradiction);
}
```

The inference data stores the property and its value:

```c
typedef struct property_inference_data {
    struct property *inferred_property;   /* property referred to */
    struct parse_node *inferred_property_value; /* and its value, if any */
    CLASS_DEFINITION
} property_inference_data;
```

Key operations:
- `PropertyInferences::new(subj, prn, val)` — creates a new property inference with the prevailing mood (certainty), defaulting to the subject's default certainty if the mood is unknown
- `PropertyInferences::draw(subj, prn, val)` — creates and joins an inference in one call
- `PropertyInferences::draw_from_metadata(subj, prn, val)` — same but marks the inference as metadata-sourced
- `PropertyInferences::draw_negated(subj, prn, val)` — same but negates the certainty
- `PropertyInferences::log_details` — logs the property name and value
- `PropertyInferences::cmp` — compares two property inferences, handling either-or negation pairs
- `PropertyInferences::explain_contradiction` — issues problem messages for contradictory property inferences
- `PropertyInferences::get_property(i)` — returns the property from an inference
- `PropertyInferences::get_value(i)` — returns the value from an inference
- `PropertyInferences::set_value_kind(i, K)` — forces the value's kind (used for bibliographic data)
- `PropertyInferences::either_or_state(subj, prn)` — returns the certainty that a subject has an either-or property (with inheritance)
- `PropertyInferences::either_or_state_without_inheritance(subj, prn, where)` — same but without inheritance
- `PropertyInferences::has_or_can_have(subj, prn)` — checks if a subject has or can have a property
- `PropertyInferences::verify_prop_states(subj)` — catches late-property contradictions

### Key C source files

- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — the full property inference family (355 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `PropertyInferences::start()` (line 37)
- `inform7/knowledge-module/Chapter 5/Inferences.w` — `InferenceFamily` infrastructure, `create_inference`, `join_inference`, `cmp` (used by Property Inferences)
- `inform7/knowledge-module/Chapter 5/The Model World.w` — model completion stages, uses `PropertyInferences::either_or_state`, `verify_prop_states`, `has_or_can_have`
- `inform7/knowledge-module/Chapter 1/Assert Propositions.w` — assertion pipeline, calls `PropertyInferences::draw`
- `inform7/knowledge-module/Chapter 3/Properties.w` — `property` struct (simplified for now)
- `inform7/knowledge-module/Chapter 3/Either-Or Properties.w` — either-or property data (deferred)
- `inform7/knowledge-module/Chapter 3/Valued Properties.w` — valued property data (deferred)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, `InferenceComparison` enum, `JoinResult` enum, `create_inference`, `cmp`, `join_inference`, `render_impossible`.
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, subject hierarchy operations, method dispatch.
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct, `find`, `grant`, accessors.
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()`, fundamental subjects (`model_world`, `global_constants`, `global_variables`).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` family, `from_kind`, `to_kind`, `has_properties`, lattice callbacks.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.

### What's needed

1. **`PropertyInferenceData` struct** — stores the property reference and optional value for a property inference. Simplified: use a string name for the property instead of the full `property` struct.
2. **`PropertyInferences::start()`** — creates the `property_inf` family with methods for `log_details`, `cmp`, and `explain_contradiction`.
3. **`PropertyInferences::new(subj, prn, val)`** — creates a new property inference with the prevailing mood (certainty), defaulting to the subject's default certainty if the mood is unknown.
4. **`PropertyInferences::draw(subj, prn, val)`** — creates and joins an inference in one call.
5. **`PropertyInferences::draw_from_metadata(subj, prn, val)`** — same but marks the inference as metadata-sourced.
6. **`PropertyInferences::draw_negated(subj, prn, val)`** — same but negates the certainty.
7. **Family methods** — `log_details` (logs property name and value), `cmp` (compares two property inferences), `explain_contradiction` (simplified: returns a contradiction description).
8. **Access functions** — `get_property(i)`, `get_value(i)`, `set_value_kind(i, K)`.
9. **Unit tests** — create the family, create property inferences, join them to subjects, compare them, test draw/draw_negated, test access functions.

## Tasks

### 1. Create the `PropertyInferenceData` struct

- [ ] Create `crates/conform7-semantics/src/knowledge/property_inferences.rs` with:

  ```rust
  /// Data stored in a property inference.
  ///
  /// Corresponds to `property_inference_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 23-27).
  ///
  /// Simplified: uses string names for properties instead of the full
  /// `property` struct. The full property struct will be integrated in a
  /// later plan.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct PropertyInferenceData {
      /// The property name (simplified: string instead of `property*`).
      pub property: &'static str,
      /// The property value, if any (simplified: string instead of `parse_node*`).
      pub value: Option<&'static str>,
  }
  ```

- [ ] Add `pub mod property_inferences;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 2. Implement `PropertyInferences::start()` — create the family

- [ ] Implement `PropertyInferences::start()` that creates the `property_inf` family:

  ```rust
  /// Create the property inference family.
  ///
  /// Corresponds to `PropertyInferences::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 12-18).
  ///
  /// Returns the `property_inf` family with methods for logging, comparison,
  /// and contradiction explanation.
  pub fn start() -> InferenceFamily {
      InferenceFamily {
          name: "property_inf",
          methods: InferenceFamilyMethods {
              log_details: |inf| {
                  // Simplified: log the property name and value
                  if let Some(data) = inf.data {
                      format!("({})", data)
                  } else {
                      String::new()
                  }
              },
              compare: |a, b| {
                  // Compare two property inferences
                  // Returns 0 for identical, non-zero for different
                  if a.data == b.data {
                      0
                  } else {
                      1 // CI_DIFFER_IN_TOPIC
                  }
              },
              explain_contradiction: |a, b, _similarity, _subject| {
                  // Simplified: return a description of the contradiction
                  format!(
                      "Contradiction: property inference {:?} vs {:?}",
                      a.data, b.data
                  )
              },
          },
      }
  }
  ```

  Note: The method implementations are simplified for now. `log_details` returns a formatted string instead of writing to a log stream. `compare` uses simple data comparison (the full C version handles either-or negation pairs). `explain_contradiction` returns a description string instead of issuing problem messages. These will be refined when the full property system is integrated.

### 3. Implement `PropertyInferences::new()` and `PropertyInferences::draw()`

- [ ] Implement `PropertyInferences::new(subj, prn, val, families, inferences) -> usize`:

  ```rust
  /// Create a new property inference.
  ///
  /// Corresponds to `PropertyInferences::new` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 29-40).
  ///
  /// Creates a `PropertyInferenceData` with the given property and value,
  /// then creates an `Inference` with the prevailing mood (defaulting to
  /// the subject's default certainty if the mood is unknown).
  ///
  /// Returns the index of the new inference in the inferences registry.
  pub fn new(
      subj: &InferenceSubject,
      prn: &'static str,
      val: Option<&'static str>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
  ) -> usize {
      // Find the property_inf family index
      let family_idx = families.iter().position(|f| f.name == "property_inf")
          .expect("PropertyInferences::start must be called first");

      // Determine certainty: use the subject's default certainty
      // (simplified: default to LIKELY_CE for now)
      let certainty = Certainty::Likely;

      // Create the inference data
      let data = PropertyInferenceData {
          property: prn,
          value: val,
      };

      // Store the data and create the inference
      let idx = inferences.len();
      inferences.push(Inference {
          family: family_idx,
          data: Some(prn), // simplified: store property name as data
          certainty,
          inferred_from: None,
          drawn_during_stage: 0,
          drawn_from_metadata: false,
      });
      idx
  }
  ```

  Note: The simplified version stores the property name as `inf.data` instead of using a separate data registry. The full C version uses `CREATE(property_inference_data)` and `STORE_POINTER_property_inference_data(data)`. We'll refine this when we integrate the full property struct.

- [ ] Implement `PropertyInferences::draw(subj, prn, val, families, inferences, subjects) -> JoinResult`:

  ```rust
  /// Create a property inference and join it to a subject.
  ///
  /// Corresponds to `PropertyInferences::draw` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 45-48).
  pub fn draw(
      subj_idx: usize,
      prn: &'static str,
      val: Option<&'static str>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      subjects: &mut [InferenceSubject],
  ) -> JoinResult {
      let inf_idx = PropertyInferences::new(
          &subjects[subj_idx], prn, val, families, inferences,
      );
      inferences[inf_idx].join(&mut subjects[subj_idx], families, inferences)
  }
  ```

- [ ] Implement `PropertyInferences::draw_from_metadata(subj, prn, val, ...)`:

  ```rust
  /// Create a property inference from metadata and join it to a subject.
  ///
  /// Corresponds to `PropertyInferences::draw_from_metadata` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 50-55).
  pub fn draw_from_metadata(
      subj_idx: usize,
      prn: &'static str,
      val: Option<&'static str>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      subjects: &mut [InferenceSubject],
  ) -> JoinResult {
      let inf_idx = PropertyInferences::new(
          &subjects[subj_idx], prn, val, families, inferences,
      );
      inferences[inf_idx].drawn_from_metadata = true;
      inferences[inf_idx].join(&mut subjects[subj_idx], families, inferences)
  }
  ```

- [ ] Implement `PropertyInferences::draw_negated(subj, prn, val, ...)`:

  ```rust
  /// Create a negated property inference and join it to a subject.
  ///
  /// Corresponds to `PropertyInferences::draw_negated` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 57-61).
  pub fn draw_negated(
      subj_idx: usize,
      prn: &'static str,
      val: Option<&'static str>,
      families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      subjects: &mut [InferenceSubject],
  ) -> JoinResult {
      let inf_idx = PropertyInferences::new(
          &subjects[subj_idx], prn, val, families, inferences,
      );
      // Negate the certainty
      inferences[inf_idx].certainty = match inferences[inf_idx].certainty {
          Certainty::Impossible => Certainty::Certain,
          Certainty::Unlikely => Certainty::Likely,
          Certainty::Likely => Certainty::Unlikely,
          Certainty::Certain => Certainty::Impossible,
          other => other,
      };
      inferences[inf_idx].join(&mut subjects[subj_idx], families, inferences)
  }
  ```

### 4. Implement access functions

- [ ] Implement `PropertyInferences::get_property(inf) -> Option<&'static str>`:

  ```rust
  /// Get the property name from a property inference.
  ///
  /// Corresponds to `PropertyInferences::get_property` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 188-192).
  pub fn get_property(inf: &Inference) -> Option<&'static str> {
      inf.data
  }
  ```

- [ ] Implement `PropertyInferences::get_value(inf) -> Option<&'static str>`:

  ```rust
  /// Get the property value from a property inference.
  ///
  /// Corresponds to `PropertyInferences::get_value` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 194-198).
  ///
  /// Simplified: returns the value from the inference data. In the full C
  /// version, this retrieves the value from a `property_inference_data` struct.
  pub fn get_value(inf: &Inference) -> Option<&'static str> {
      // Simplified: the value is not stored separately from the property name
      // in the simplified data model. This will be refined when we integrate
      // the full property struct.
      None
  }
  ```

  Note: `get_value` is a stub in this simplified version because the value is not stored separately from the property name. The full implementation will use a `PropertyInferenceData` registry. This is acceptable because the value is not needed for the core family creation and comparison tests.

### 5. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/property_inferences.rs`:

  - Test that `start()` creates a family with name "property_inf".
  - Test that the property_inf family's `log_details` method works.
  - Test that `new()` creates an inference with the correct property name.
  - Test that `new()` creates an inference with `Likely` certainty by default.
  - Test that `draw()` creates and joins an inference to a subject.
  - Test that `draw()` returns `Joined` for a new inference.
  - Test that `draw()` returns `DiscardedRedundant` for a duplicate inference.
  - Test that `draw_from_metadata()` sets `drawn_from_metadata` to true.
  - Test that `draw_negated()` negates the certainty.
  - Test that `get_property()` returns the correct property name.
  - Test that two property inferences with the same data compare as identical.
  - Test that two property inferences with different data compare as different.
  - Test that the family's `compare` method returns 0 for identical inferences.
  - Test that the family's `compare` method returns non-zero for different inferences.
  - Test that `explain_contradiction` returns a description string.

### 6. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `PropertyInferences::start()` creates an `InferenceFamily` with name "property_inf".
- [ ] The property_inf family has `log_details`, `compare`, and `explain_contradiction` methods.
- [ ] `PropertyInferences::new(subj, prn, val)` creates an inference with the correct property name.
- [ ] `PropertyInferences::new(subj, prn, val)` creates an inference with `Likely` certainty by default.
- [ ] `PropertyInferences::draw(subj, prn, val)` creates and joins an inference to a subject.
- [ ] `PropertyInferences::draw()` returns `Joined` for a new inference.
- [ ] `PropertyInferences::draw()` returns `DiscardedRedundant` for a duplicate inference.
- [ ] `PropertyInferences::draw_from_metadata()` sets `drawn_from_metadata` to true.
- [ ] `PropertyInferences::draw_negated()` negates the certainty of the inference.
- [ ] `PropertyInferences::get_property(inf)` returns the property name from an inference.
- [ ] Two property inferences with the same data compare as identical.
- [ ] Two property inferences with different data compare as different.
- [ ] The family's `compare` method returns 0 for identical inferences.
- [ ] The family's `compare` method returns non-zero for different inferences.
- [ ] `explain_contradiction` returns a description string.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Relation Inferences**: `RelationInferences` (Chapter 5/Relation Inferences.w) — the relation inference family — is deferred. This plan implements Property Inferences only.
- **Full `property` struct**: The full `property` struct with either-or and valued property data is deferred. This plan uses simplified string-based property references.
- **Either-or property data**: `EitherOrProperties` (Chapter 3/Either-Or Properties.w) — either-or property data structures — is deferred.
- **Valued property data**: `ValueProperties` (Chapter 3/Valued Properties.w) — valued property data structures — is deferred.
- **`PropertyInferences::either_or_state`**: The query function that checks if a subject has an either-or property (with inheritance) is deferred. It depends on the full either-or property system.
- **`PropertyInferences::either_or_state_without_inheritance`**: The variant without inheritance is deferred.
- **`PropertyInferences::has_or_can_have`**: The query function that checks if a subject has or can have a property is deferred. It depends on `PropertyPermissions::find` and the full property system.
- **`PropertyInferences::verify_prop_states`**: The late-contradiction checking function is deferred. It depends on the full property kind system.
- **`PropertyInferences::set_value_kind`**: The function that forces a value's kind is deferred. It depends on the full kind system integration.
- **Instance Subjects**: `InstanceSubjects` (Chapter 4/Instance Subjects.w) — the instances family of inference subjects — is deferred.
- **Variable Subjects**: `VariableSubjects` (Chapter 4/Variable Subjects.w) — the variables family of inference subjects — is deferred.
- **Relation Subjects**: `RelationSubjects` (Chapter 4/Relation Subjects.w) — the relations family of inference subjects — is deferred.
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
