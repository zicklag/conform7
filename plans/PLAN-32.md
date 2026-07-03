# Plan 32: Measurement Adjectives — The `measurement_amf` Adjective Meaning Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Measurement Adjectives system — the third concrete adjective meaning family that uses the adjective meaning infrastructure from PLAN-28 and the `measurement_definition` struct from PLAN-31. This creates the `measurement_amf` family with `assert`, `claim_definition`, and `prepare_schemas` methods, enabling measurement-based adjectives (e.g., "roomy", "tall", "large") that compare a property value against a threshold to be used as adjectives in the model world.

This is the smallest next step after PLAN-31 because:

1. **It's the next item in the knowledge module startup that has no remaining dependencies.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `MeasurementAdjectives::start()` at line 41 — after `InstanceAdjectives::start()` (PLAN-30, Complete) and `EitherOrPropertyAdjectives::start()` (PLAN-29, Complete). It depends on the adjective meaning system (PLAN-28, Complete), the property system (PLAN-25, Complete), the `measurement_definition` struct and `Measurements` management functions (PLAN-31, Complete), and `PropertyInferences::draw` (PLAN-19, Complete) — all done. `ComparativeRelations::start()` (line 44) depends on `MeasurementAdjectives` and `Measurements::create_comparatives`, so it must come after.

2. **It's the next simplest step after the `measurement_definition` struct.** The `measurement_definition` struct and `Measurements` management functions (PLAN-31) are the data foundation. The `MeasurementAdjectives` module (197 lines of C in `Measurement Adjectives.w`) is the adjective meaning family that uses that data — creating the `measurement_amf` family, implementing `claim_definition` (which creates `measurement_definition` structs from parsed definitions), `assert` (which validates and draws property inferences), and `prepare_schemas` (which prepares I6 schemas for run-time compilation).

3. **It's a prerequisite for `ComparativeRelations`.** `ComparativeRelations::stock` (Chapter 3/Comparative Relations.w, line 36) calls `Measurements::create_comparatives()` at stage 2, which iterates over all `measurement_definition` structs, validates them, and creates comparative binary predicates. But `create_comparatives` also depends on `Grading::make_comparative` and `Grading::make_quiddity` (linguistics module) and `BinaryPredicateFamilies` — so it's deferred. The `measurement_amf` family itself, however, is independently buildable now.

4. **It's a prerequisite for the assertion pipeline.** `MeasurementAdjectives::claim_definition` is called by the `CLAIM_DEFINITION_SENTENCE_ADJM_MTID` method dispatch when a "Definition:" clause is parsed. Without the `measurement_amf` family, measurement definitions cannot be claimed, and assertions like "Peter is tall" cannot be processed for measurement adjectives.

5. **It introduces the `claim_definition` pattern — a new method type.** Unlike `EitherOrPropertyAdjectives` (which uses `create_for_property` directly) and `InstanceAdjectives` (which uses `make_adjectival` directly), `MeasurementAdjectives` uses the `CLAIM_DEFINITION_SENTENCE_ADJM_MTID` method dispatch. This is a new pattern where the adjective meaning family claims a "Definition:" clause during parsing. Implementing this now establishes the pattern for future `condition_amf` and other definition-claiming families.

6. **Independently testable without grammar parsing.** The `claim_definition` method depends on Preform grammar (`<measurement-adjective-definition>`, `<measurement-range>`, `<s-literal>`) which is deferred. However, we can:
   - Create the `measurement_amf` family with `start()` and test that the family is created with the right methods
   - Implement a simplified `claim_definition` that takes pre-parsed parameters (headword, property, shape, threshold text) and creates a `measurement_definition` + adjective meaning — testable without grammar
   - Test the `assert` method (validates the measurement definition, draws a property inference via `PropertyInferences::draw`)
   - Test the `prepare_schemas` method (no-op, deferred)
   - Test the `is_measurement` method (checks if a meaning belongs to this family)

## Background

### C reference architecture

#### Measurement Adjectives (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 1-197)

The Measurement Adjectives system creates one family and handles definition claiming, assertion, and schema preparation:

```c
adjective_meaning_family *measurement_amf = NULL;

void MeasurementAdjectives::start(void) {
    measurement_amf = AdjectiveMeanings::new_family(3);
    METHOD_ADD(measurement_amf, ASSERT_ADJM_MTID,
        MeasurementAdjectives::assert);
    METHOD_ADD(measurement_amf, PREPARE_SCHEMAS_ADJM_MTID,
        MeasurementAdjectives::prepare_schemas);
    METHOD_ADD(measurement_amf, CLAIM_DEFINITION_SENTENCE_ADJM_MTID,
        MeasurementAdjectives::claim_definition);
}
```

The `claim_definition` method parses a "Definition:" clause and creates a `measurement_definition`:

```c
int MeasurementAdjectives::claim_definition(adjective_meaning_family *f,
    adjective_meaning **result, parse_node *q,
    int sense, wording AW, wording DNW, wording CONW, wording CALLW) {
    if (sense == 0) return FALSE;

    if (<measurement-adjective-definition>(CONW) == FALSE) return FALSE;
    int shape = <<r>>;
    wording PRW = GET_RW(<measurement-adjective-definition>, 1);
    wording THRESW = GET_RW(<measurement-range>, 1);
    property *prop = <<rp>>;

    @<Reject some overly elaborate attempts to define overly elaborate measurements@>;
    @<Allow an exact measurement to be created only if we can already parse the threshold@>;

    measurement_definition *mdef = Measurements::new(q, AW, THRESW, prop, shape, PRW);
    if (shape != MEASURE_T_EXACTLY) @<Create the superlative form@>;
    @<Create the adjectival meaning arising from this measurement@>;
    *result = mdef->headword_as_adjective;
    return TRUE;
}
```

The adjectival meaning creation (lines 160-167):

```c
@<Create the adjectival meaning arising from this measurement@> =
    adjective_meaning *am = AdjectiveMeanings::new(measurement_amf,
        STORE_POINTER_measurement_definition(mdef), Node::get_text(q));
    mdef->headword_as_adjective = am;
    adjective *adj = Adjectives::declare(AW, NULL);
    AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    AdjectiveMeanings::perform_task_via_function(am, TEST_ATOM_TASK);
    AdjectiveMeaningDomains::set_from_text(am, DNW);
```

The `assert` method validates and uses the measurement definition:

```c
int MeasurementAdjectives::assert(adjective_meaning_family *f,
    adjective_meaning *am, inference_subject *infs_to_assert_on, int parity) {
    measurement_definition *mdef =
        RETRIEVE_POINTER_measurement_definition(am->family_specific_data);
    Measurements::validate(mdef);
    if ((Measurements::is_valid(mdef)) && (mdef->prop) && (parity == TRUE)) {
        parse_node *val = NULL;
        if (<s-literal>(mdef->region_threshold_text)) val = <<rp>>;
        else internal_error("literal unreadable");
        PropertyInferences::draw(infs_to_assert_on, mdef->prop, val);
        return TRUE;
    }
    return FALSE;
}
```

The `prepare_schemas` method prepares I6 schemas (deferred):

```c
void MeasurementAdjectives::prepare_schemas(adjective_meaning_family *family,
    adjective_meaning *am, int T) {
    measurement_definition *mdef =
        RETRIEVE_POINTER_measurement_definition(am->family_specific_data);
    if ((mdef->prop) && (mdef->region_threshold_evaluated))
        RTAdjectives::make_mdef_test_schema(mdef, T);
}
```

#### Measurement Definition (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92)

The `measurement_definition` struct (PLAN-31, Complete) stores everything needed for a measurement adjective:

```c
typedef struct measurement_definition {
    struct parse_node *measurement_node;
    struct wording headword;
    struct adjective_meaning *headword_as_adjective;
    struct wording superlative;
    struct property *prop;
    struct wording name_of_property_to_compare;
    int region_shape;
    int region_threshold;
    struct kind *region_kind;
    int region_threshold_evaluated;
    struct wording region_threshold_text;
    struct measurement_compilation_data compilation_data;
    CLASS_DEFINITION
} measurement_definition;
```

#### Comparative Relations (`inform7/knowledge-module/Chapter 3/Comparative Relations.w`, lines 1-120)

The `ComparativeRelations` module uses measurement definitions to create comparative binary predicates — deferred to a later plan:

```c
void ComparativeRelations::stock(bp_family *self, int n) {
    if (n == 2) Measurements::create_comparatives();
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Measurement Adjectives.w` — `MeasurementAdjectives` module, `measurement_amf` family, `claim_definition`, `assert`, `prepare_schemas` (197 lines)
- `inform7/knowledge-module/Chapter 3/Measurements.w` — `measurement_definition` struct, `Measurements::new`, `Measurements::validate`, `Measurements::is_valid`, `Measurements::retrieve`, `Measurements::read_property_details`, `Measurements::weak_comparison_bp`, `Measurements::strict_comparison`, `Measurements::validate_definitions`, `Measurements::create_comparatives` (301 lines, PLAN-31)
- `inform7/knowledge-module/Chapter 3/Comparative Relations.w` — `ComparativeRelations` module, `property_comparison_bp_family`, `stock`, `typecheck`, `schema`, `initialise` (120 lines, deferred)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `MeasurementAdjectives::start()` (line 41)
- `inform7/assertions-module/Chapter 8/Adjective Meanings.w` — `AdjectiveMeanings::new_family`, `AdjectiveMeanings::new` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Ambiguity.w` — `AdjectiveAmbiguity::add_meaning_to_adjective` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w` — `AdjectiveMeaningDomains::set_from_text`, `AdjectiveMeaningDomains::set_from_kind` (PLAN-28)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` (PLAN-19)
- `services/linguistics-module/Chapter 2/Adjectives.w` — `Adjectives::declare` (PLAN-28)
- `services/linguistics-module/Chapter 2/Grading.w` — `Grading::make_superlative`, `Grading::make_comparative`, `Grading::make_quiddity` (deferred)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/measurements.rs` — `MeasurementDefinition` struct, `Measurements` management functions (`new`, `validate`, `is_valid`, `read_property_details`, `retrieve`, `weak_comparison_bp`, `strict_comparison`, `validate_definitions`, `create_comparatives`), `MEASURE_T_OR_LESS`, `MEASURE_T_EXACTLY`, `MEASURE_T_OR_MORE` constants, unit tests (PLAN-31, Complete).
- `crates/conform7-semantics/src/knowledge/adjectives.rs` — `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, `AdjectiveDomainData` struct, `AdjectiveMeaningFamilyMethods` struct, `AdjectiveMeanings` management functions (`new_family`, `new`, `negate`, `assert`), `AdjectiveAmbiguity` management functions (`add_meaning_to_adjective`, `can_be_applied_to`, `first_meaning`), `AdjectiveMeaningDomains` management functions (`new_from_kind`, `set_from_kind`, `set_from_text`, `get_kind`, `weak_match`), `Adjectives::declare`, `Adjectives::find`, `Adjectives::get_nominative_singular`, unit tests (PLAN-28, Complete).
- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData` struct, `ValuePropertyData` struct, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties::new_eo_data`, `EitherOrProperties::make_pair`, `EitherOrProperties::get_negation`, `EitherOrProperties::as_adjective`, `ValueProperties` functions, unit tests (PLAN-25, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, `PropertyInferences::new()`, `PropertyInferences::draw()`, `PropertyInferences::draw_negated()`, `PropertyInferences::draw_from_metadata()`, unit tests (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs` — `EitherOrPropertyAdjectives` module, `EITHER_OR_PROPERTY_FAMILY` constant, `start()`, `is()`, `create_for_property()`, `assert()`, `prepare_schemas()`, `index()`, unit tests (PLAN-29, Complete).
- `crates/conform7-semantics/src/knowledge/instances.rs` — `Instance` struct, `Instances` management functions (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/instance_subjects.rs` — `InstanceSubjects` family (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/instance_adjectives.rs` — `InstanceAdjectives` module, `enumerative_amf` family (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/same_property_relation.rs` — `SameAsRelations` module, `SameAsRelations::start()`, `SameAsRelations::stock()`, `SameAsRelations::typecheck()`, unit tests (PLAN-26, Complete).
- `crates/conform7-semantics/src/knowledge/setting_property_relation.rs` — `SettingPropertyRelations` module, `SettingPropertyRelations::start()`, `SettingPropertyRelations::stock()`, `SettingPropertyRelations::typecheck()`, `SettingPropertyRelations::assert()`, `SettingPropertyRelations::schema()`, unit tests (PLAN-27, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, `ProvisionRelation::stock()`, `ProvisionRelation::typecheck()`, `ProvisionRelation::assert()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/relation_subjects.rs` — `RelationSubjects` module, `RelationSubjects::family()`, `RelationSubjects::from_bp()`, `RelationSubjects::new()`, `RelationSubjects::to_bp()`, unit tests (PLAN-24, Complete).
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` module, `KindSubjects::family()`, `KindSubjects::from_kind()`, `KindSubjects::to_kind()`, unit tests (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod measurements;` from PLAN-31).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`MeasurementAdjectives` module** — a new module `measurement_adjectives` in the knowledge crate with:
   - `MeasurementAdjectives::start()` — creates the `measurement_amf` family with assert, prepare_schemas, and claim_definition methods
   - `MeasurementAdjectives::is_measurement(am_idx, meanings)` — checks if a meaning belongs to the measurement family
   - `MeasurementAdjectives::claim_definition(headword, prop, shape, threshold_text, domain_text, definitions, adjectives, meanings, families, properties)` — simplified version that takes pre-parsed parameters and creates a measurement definition + adjective meaning:
     - Creates a `measurement_definition` via `Measurements::new`
     - Creates an adjective meaning via `AdjectiveMeanings::new` with the `measurement_amf` family and the measurement definition index as family-specific data
     - Declares a new adjective via `Adjectives::declare`
     - Adds the meaning to the adjective via `AdjectiveAmbiguity::add_meaning_to_adjective`
     - Sets the domain from text via `AdjectiveMeaningDomains::set_from_text`
     - Stores the adjective meaning in `mdef.headword_as_adjective`
     - Returns the measurement definition index
   - `MeasurementAdjectives::assert(am_idx, subj_idx, parity, meanings, subjects, families, properties, definitions, inference_families, inferences, data_registry)` — asserts the measurement adjective on a subject:
     - Retrieves the measurement definition index from the meaning's family-specific data
     - Validates the measurement definition via `Measurements::validate`
     - If valid and parity is true, draws a property inference via `PropertyInferences::draw` with the property and threshold value
     - Returns true if asserted, false otherwise
   - `MeasurementAdjectives::prepare_schemas(am_idx, task)` — simplified: no-op (returns false)
   - Global constant for the family index

2. **Integration with the knowledge module** — add the `measurement_adjectives` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — create the family, create measurement definitions via `claim_definition`, test the `is_measurement` method, test the `assert` method (validates and draws property inference for positive parity, returns false for negative parity), test that `claim_definition` creates a measurement definition with the right headword, property, shape, and threshold text, test that `claim_definition` creates an adjective meaning with the right family and family-specific data, test that `claim_definition` declares an adjective and adds the meaning to it, test that `claim_definition` sets the domain from text, test the `prepare_schemas` method (no-op).

## Tasks

### 1. Create the `MeasurementAdjectives` module

- [ ] Create `crates/conform7-semantics/src/knowledge/measurement_adjectives.rs` with:

  ```rust
  /// The Measurement Adjectives system — measurement-based adjectives used as adjectives.
  ///
  /// Corresponds to `MeasurementAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`).
  ///
  /// Creates one adjective_meaning_family instance:
  /// - measurement_amf — for measurement-based adjectives
  ///
  /// Measurement adjectives compare a property value against a threshold. For example,
  /// the definition "Definition: A container is roomy if its carrying capacity is 10 or more"
  /// creates the adjective "roomy" which is true when carrying_capacity >= 10.
  ///
  /// Simplified:
  /// - No Preform grammar parsing (<measurement-adjective-definition>, <measurement-range>)
  /// - No Grading::make_superlative (superlative form deferred)
  /// - No Grading::make_comparative (comparative form deferred)
  /// - No Grading::make_quiddity (quiddity form deferred)
  /// - No RTAdjectives::make_mdef_test_schema (run-time compilation deferred)
  /// - No <s-literal> grammar parsing (uses simple number parsing)
  /// - No problem message generation
  use crate::knowledge::adjectives::{
      Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
      AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
  };
  use crate::knowledge::inference_subjects::InferenceSubject;
  use crate::knowledge::inferences::{Inference, InferenceFamily};
  use crate::knowledge::measurements::{
      MeasurementDefinition, Measurements, MEASURE_T_EXACTLY, MEASURE_T_OR_LESS, MEASURE_T_OR_MORE,
  };
  use crate::knowledge::properties::Property;
  use crate::knowledge::property_inferences::PropertyInferences;
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the measurement family in the family registry.
  pub const MEASUREMENT_FAMILY: usize = 2;

  /// The measurement adjectives module.
  ///
  /// Corresponds to `MeasurementAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`).
  pub struct MeasurementAdjectives;
  ```

- [ ] Implement `MeasurementAdjectives::start()`:

  ```rust
  /// Create the measurement family with its methods.
  ///
  /// Corresponds to `MeasurementAdjectives::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 10-18).
  ///
  /// Returns (families, meanings, adjectives, definitions) where:
  /// - families[MEASUREMENT_FAMILY] = measurement_amf
  /// - meanings is empty (claim_definition fills it)
  /// - adjectives is empty (claim_definition fills it)
  /// - definitions is empty (claim_definition fills it)
  pub fn start() -> (
      Vec<AdjectiveMeaningFamily>,
      Vec<AdjectiveMeaning>,
      Vec<Adjective>,
      Vec<MeasurementDefinition>,
  ) {
      let measurement_family = AdjectiveMeaningFamily {
          name: "measurement",
          definition_claim_priority: 3,
          methods: AdjectiveMeaningFamilyMethods {
              assert: Some(MeasurementAdjectives::assert),
              prepare_schemas: Some(MeasurementAdjectives::prepare_schemas),
              claim_definition: Some(MeasurementAdjectives::claim_definition),
              ..AdjectiveMeaningFamilyMethods::default()
          },
      };

      (
          vec![
              AdjectiveMeaningFamily::default(), // index 0: either_or_property (from PLAN-29)
              AdjectiveMeaningFamily::default(), // index 1: enumerative (from PLAN-30)
              measurement_family,                // index 2: measurement
          ],
          Vec::new(),
          Vec::new(),
          Vec::new(),
      )
  }
  ```

  Note: The returned families vector must include placeholders for families created by earlier plans (either_or_property at index 0, enumerative at index 1) so that the measurement family is at index 2. Alternatively, if the knowledge module startup is refactored to merge families from multiple `start()` calls, this function should return only the measurement family and the merge logic should handle indexing. The exact approach depends on how the knowledge module startup is structured — the implementer should check `setup.rs` and `mod.rs` for the current startup pattern.

- [ ] Implement `MeasurementAdjectives::is_measurement()`:

  ```rust
  /// Check if an adjective meaning belongs to the measurement family.
  ///
  /// Corresponds to checking `am->family == measurement_amf` in the C reference.
  ///
  /// Returns true if the meaning belongs to this family, false otherwise.
  pub fn is_measurement(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
      meanings.get(am_idx).is_some_and(|am| am.family == MEASUREMENT_FAMILY)
  }
  ```

- [ ] Implement `MeasurementAdjectives::claim_definition()`:

  ```rust
  /// Claim a definition as a measurement adjective.
  ///
  /// Corresponds to `MeasurementAdjectives::claim_definition` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 56-75).
  ///
  /// This is a simplified version that takes pre-parsed parameters instead of
  /// parsing them from grammar. In the C reference, the Preform grammar
  /// `<measurement-adjective-definition>` and `<measurement-range>` parse the
  /// definition clause. Here, the caller provides the parsed values directly.
  ///
  /// Parameters:
  /// - headword: the adjective being defined (e.g., "roomy")
  /// - prop: optional property index (the property being compared)
  /// - shape: the region shape (MEASURE_T_OR_LESS, MEASURE_T_EXACTLY, MEASURE_T_OR_MORE)
  /// - threshold_text: optional text of the threshold value (e.g., "10")
  /// - domain_text: optional text of the domain (e.g., "container")
  /// - definitions: mutable vector of measurement definitions
  /// - adjectives: mutable vector of adjectives
  /// - meanings: mutable vector of adjective meanings
  /// - families: slice of adjective meaning families
  /// - properties: slice of properties
  ///
  /// Returns the index of the new measurement definition, or None if creation failed.
  ///
  /// Simplified:
  /// - No `<measurement-adjective-definition>` grammar parsing
  /// - No `<measurement-range>` grammar parsing
  /// - No rejection of overly elaborate definitions (multi-word headwords, callings, unless)
  /// - No exact measurement threshold pre-parsing check
  /// - No Grading::make_superlative (superlative form deferred)
  /// - No AdjectiveMeanings::perform_task_via_function (TEST_ATOM_TASK deferred)
  pub fn claim_definition(
      headword: &str,
      prop: Option<usize>,
      shape: i32,
      threshold_text: Option<&str>,
      domain_text: Option<&str>,
      definitions: &mut Vec<MeasurementDefinition>,
      adjectives: &mut Vec<Adjective>,
      meanings: &mut Vec<AdjectiveMeaning>,
      families: &[AdjectiveMeaningFamily],
      properties: &[Property],
  ) -> Option<usize> {
      // Create the measurement definition
      let mdef_idx = Measurements::new(headword, prop, shape, threshold_text, definitions);

      // Create the adjective meaning
      let am_idx = AdjectiveMeanings::new(
          MEASUREMENT_FAMILY,
          mdef_idx, // family_specific_data: the measurement definition index
          headword,
          meanings,
          families,
      );

      // Declare the adjective
      let adj_idx = Adjectives::declare(headword, None, adjectives);

      // Add the meaning to the adjective
      AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, meanings, adjectives);

      // Set the domain from text
      if let Some(domain) = domain_text {
          AdjectiveMeaningDomains::set_from_text(am_idx, domain, meanings, families);
      }

      // Store the adjective meaning in the measurement definition
      if let Some(mdef) = definitions.get_mut(mdef_idx) {
          mdef.headword_as_adjective = Some(am_idx);
      }

      Some(mdef_idx)
  }
  ```

- [ ] Implement `MeasurementAdjectives::assert()`:

  ```rust
  /// Assert a measurement adjective on a subject.
  ///
  /// Corresponds to `MeasurementAdjectives::assert` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 172-185).
  ///
  /// This function:
  /// 1. Retrieves the measurement definition from the meaning's family-specific data
  /// 2. Validates the measurement definition
  /// 3. If valid and parity is true, draws a property inference with the threshold value
  /// 4. Returns true if asserted, false otherwise
  ///
  /// Simplified:
  /// - No `<s-literal>` grammar parsing (uses the threshold value directly)
  /// - No `Rvalues::from_encoded_notation` (uses the threshold value directly)
  /// - No `internal_error` on unreadable literal (returns false instead)
  pub fn assert(
      am_idx: usize,
      subj_idx: usize,
      parity: bool,
      meanings: &[AdjectiveMeaning],
      subjects: &[InferenceSubject],
      families: &[InferenceFamily],
      properties: &[Property],
      definitions: &mut [MeasurementDefinition],
      inference_families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      data_registry: &mut Vec<Vec<u8>>,
  ) -> bool {
      // Get the measurement definition index from the meaning's family-specific data
      let mdef_idx = match meanings.get(am_idx) {
          Some(am) => am.family_specific_data,
          None => return false,
      };

      // Validate the measurement definition
      Measurements::validate(mdef_idx, definitions, properties);

      // Check if valid and parity is true
      if Measurements::is_valid(mdef_idx, definitions) && parity {
          if let Some(mdef) = definitions.get(mdef_idx) {
              if let Some(prn_idx) = mdef.prop {
                  // Draw a property inference with the threshold value
                  // Simplified: use the threshold value directly instead of parsing
                  // it from text via <s-literal>
                  PropertyInferences::draw(
                      subj_idx,
                      prn_idx,
                      mdef.region_threshold,
                      subjects,
                      families,
                      properties,
                      inference_families,
                      inferences,
                      data_registry,
                  );
                  return true;
              }
          }
      }

      false
  }
  ```

- [ ] Implement `MeasurementAdjectives::prepare_schemas()`:

  ```rust
  /// Prepare I6 schemas for a measurement adjective.
  ///
  /// Corresponds to `MeasurementAdjectives::prepare_schemas` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 190-196).
  ///
  /// Simplified: no-op. Run-time compilation is deferred.
  pub fn prepare_schemas(_am_idx: usize, _task: usize) -> bool {
      false
  }
  ```

### 2. Integrate with the knowledge module

- [ ] Add module declaration to `crates/conform7-semantics/src/knowledge/mod.rs`:

  ```rust
  pub mod measurement_adjectives;
  ```

### 3. Unit tests

- [ ] Add tests to `crates/conform7-semantics/src/knowledge/measurement_adjectives.rs`:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::knowledge::adjectives::{
          Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
          AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
      };
      use crate::knowledge::inference_subjects::InferenceSubject;
      use crate::knowledge::inferences::{Inference, InferenceFamily};
      use crate::knowledge::measurements::{
          MeasurementDefinition, Measurements, MEASURE_T_EXACTLY, MEASURE_T_OR_LESS, MEASURE_T_OR_MORE,
      };
      use crate::knowledge::properties::{Property, ValuePropertyData};
      use crate::knowledge::property_inferences::PropertyInferences;

      #[test]
      fn test_start_creates_family() {
          let (families, meanings, adjectives, definitions) = MeasurementAdjectives::start();

          assert_eq!(families.len(), 3);
          assert_eq!(families[MEASUREMENT_FAMILY].name, "measurement");
          assert!(families[MEASUREMENT_FAMILY].methods.assert.is_some());
          assert!(families[MEASUREMENT_FAMILY].methods.prepare_schemas.is_some());
          assert!(families[MEASUREMENT_FAMILY].methods.claim_definition.is_some());
          assert!(meanings.is_empty());
          assert!(adjectives.is_empty());
          assert!(definitions.is_empty());
      }

      #[test]
      fn test_is_measurement_returns_true_for_measurement_meaning() {
          let (families, mut meanings, _, _) = MeasurementAdjectives::start();
          let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, 0, "roomy", &mut meanings, &families);

          assert!(MeasurementAdjectives::is_measurement(am_idx, &meanings));
      }

      #[test]
      fn test_is_measurement_returns_false_for_other_meaning() {
          let (families, mut meanings, _, _) = MeasurementAdjectives::start();
          // Create a meaning with a different family
          let am_idx = AdjectiveMeanings::new(0, 0, "open", &mut meanings, &families);

          assert!(!MeasurementAdjectives::is_measurement(am_idx, &meanings));
      }

      #[test]
      fn test_claim_definition_creates_measurement_definition() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "roomy",
              Some(0),
              MEASURE_T_OR_MORE,
              Some("10"),
              Some("container"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          );

          assert!(mdef_idx.is_some());
          let mdef_idx = mdef_idx.unwrap();
          assert_eq!(definitions[mdef_idx].headword, "roomy");
          assert_eq!(definitions[mdef_idx].prop, Some(0));
          assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_MORE);
          assert_eq!(definitions[mdef_idx].region_threshold_text, Some("10".to_string()));
          assert!(definitions[mdef_idx].headword_as_adjective.is_some());
      }

      #[test]
      fn test_claim_definition_creates_adjective_meaning() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "roomy",
              Some(0),
              MEASURE_T_OR_MORE,
              Some("10"),
              Some("container"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          ).unwrap();

          let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
          assert_eq!(meanings[am_idx].family, MEASUREMENT_FAMILY);
          assert_eq!(meanings[am_idx].family_specific_data, mdef_idx);
      }

      #[test]
      fn test_claim_definition_declares_adjective() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "roomy",
              Some(0),
              MEASURE_T_OR_MORE,
              Some("10"),
              Some("container"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          ).unwrap();

          // Check that an adjective was declared
          assert!(!adjectives.is_empty());
          let adj = &adjectives[0];
          assert_eq!(adj.nominative_singular, "roomy");

          // Check that the meaning was added to the adjective
          let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
          assert!(adj.meaning_indices.contains(&am_idx));
      }

      #[test]
      fn test_claim_definition_sets_domain() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "roomy",
              Some(0),
              MEASURE_T_OR_MORE,
              Some("10"),
              Some("container"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          ).unwrap();

          let am_idx = definitions[mdef_idx].headword_as_adjective.unwrap();
          // The domain should be set from text "container"
          // (The exact assertion depends on how AdjectiveMeaningDomains::set_from_text works)
          assert!(meanings[am_idx].domain.is_some());
      }

      #[test]
      fn test_assert_valid_measurement_draws_inference() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = vec![Property {
              name: "carrying capacity",
              has_of_in_the_name: false,
              inter_level_only: false,
              permissions: Vec::new(),
              either_or_data: None,
              value_data: Some(ValuePropertyData {
                  property_value_kind: Some("number"),
                  setting_bp: None,
                  name_coincides_with_kind: false,
                  as_condition_of_subject: None,
                  relation_whose_state_this_stores: None,
              }),
              compilation_data: None,
              possession_marker: false,
          }];

          // Create a measurement definition with a known property and threshold
          let mdef_idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

          // Create an adjective meaning for it
          let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, mdef_idx, "roomy", &mut meanings, &families);

          // Set up inference infrastructure
          let (inference_families, mut inferences, mut data_registry) = PropertyInferences::start();
          let (subjects, _) = crate::knowledge::inference_subjects::InferenceSubjects::start();

          // Create a subject to assert on
          let subj_idx = 0; // simplified: use a dummy subject index

          // Assert with positive parity
          let result = MeasurementAdjectives::assert(
              am_idx, subj_idx, true,
              &meanings, &subjects, &families,
              &properties, &mut definitions,
              &inference_families, &mut inferences, &mut data_registry,
          );

          assert!(result);
          // Should have drawn a property inference
          assert!(!inferences.is_empty());
      }

      #[test]
      fn test_assert_negative_parity_returns_false() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = vec![Property {
              name: "carrying capacity",
              has_of_in_the_name: false,
              inter_level_only: false,
              permissions: Vec::new(),
              either_or_data: None,
              value_data: Some(ValuePropertyData {
                  property_value_kind: Some("number"),
                  setting_bp: None,
                  name_coincides_with_kind: false,
                  as_condition_of_subject: None,
                  relation_whose_state_this_stores: None,
              }),
              compilation_data: None,
              possession_marker: false,
          }];

          let mdef_idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, mdef_idx, "roomy", &mut meanings, &families);

          let (inference_families, mut inferences, mut data_registry) = PropertyInferences::start();
          let (subjects, _) = crate::knowledge::inference_subjects::InferenceSubjects::start();

          // Assert with negative parity
          let result = MeasurementAdjectives::assert(
              am_idx, 0, false,
              &meanings, &subjects, &families,
              &properties, &mut definitions,
              &inference_families, &mut inferences, &mut data_registry,
          );

          assert!(!result);
          // Should NOT have drawn a property inference
          assert!(inferences.is_empty());
      }

      #[test]
      fn test_assert_unvalidated_definition_returns_false() {
          let (families, mut meanings, _, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          // Create a measurement definition with no property and no threshold text
          let mdef_idx = Measurements::new("roomy", None, MEASURE_T_OR_MORE, None, &mut definitions);
          let am_idx = AdjectiveMeanings::new(MEASUREMENT_FAMILY, mdef_idx, "roomy", &mut meanings, &families);

          let (inference_families, mut inferences, mut data_registry) = PropertyInferences::start();
          let (subjects, _) = crate::knowledge::inference_subjects::InferenceSubjects::start();

          let result = MeasurementAdjectives::assert(
              am_idx, 0, true,
              &meanings, &subjects, &families,
              &properties, &mut definitions,
              &inference_families, &mut inferences, &mut data_registry,
          );

          assert!(!result);
      }

      #[test]
      fn test_prepare_schemas_is_noop() {
          // Should not panic
          let result = MeasurementAdjectives::prepare_schemas(0, 0);
          assert!(!result);
      }

      #[test]
      fn test_claim_definition_without_domain() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "tall",
              Some(0),
              MEASURE_T_OR_MORE,
              Some("68"),
              None, // no domain text
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          );

          assert!(mdef_idx.is_some());
          let mdef_idx = mdef_idx.unwrap();
          assert_eq!(definitions[mdef_idx].headword, "tall");
          assert_eq!(definitions[mdef_idx].prop, Some(0));
          assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_MORE);
          assert_eq!(definitions[mdef_idx].region_threshold_text, Some("68".to_string()));
      }

      #[test]
      fn test_claim_definition_exact_measurement() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "handy",
              Some(0),
              MEASURE_T_EXACTLY,
              Some("7"),
              Some("person"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          );

          assert!(mdef_idx.is_some());
          let mdef_idx = mdef_idx.unwrap();
          assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_EXACTLY);
      }

      #[test]
      fn test_claim_definition_or_less_measurement() {
          let (families, mut meanings, mut adjectives, mut definitions) = MeasurementAdjectives::start();
          let properties = Vec::new();

          let mdef_idx = MeasurementAdjectives::claim_definition(
              "compact",
              Some(0),
              MEASURE_T_OR_LESS,
              Some("5"),
              Some("container"),
              &mut definitions,
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
          );

          assert!(mdef_idx.is_some());
          let mdef_idx = mdef_idx.unwrap();
          assert_eq!(definitions[mdef_idx].region_shape, MEASURE_T_OR_LESS);
      }
  }
  ```

## Success Criteria

- [ ] `MEASUREMENT_FAMILY` constant defined (index 2)
- [ ] `MeasurementAdjectives::start()` creates the `measurement_amf` family with assert, prepare_schemas, and claim_definition methods
- [ ] `MeasurementAdjectives::is_measurement()` correctly identifies measurement family meanings
- [ ] `MeasurementAdjectives::claim_definition()` creates a measurement definition with the right headword, property, shape, and threshold text
- [ ] `MeasurementAdjectives::claim_definition()` creates an adjective meaning with the measurement family and the definition index as family-specific data
- [ ] `MeasurementAdjectives::claim_definition()` declares an adjective and adds the meaning to it
- [ ] `MeasurementAdjectives::claim_definition()` sets the domain from text
- [ ] `MeasurementAdjectives::claim_definition()` stores the adjective meaning in `mdef.headword_as_adjective`
- [ ] `MeasurementAdjectives::assert()` validates the measurement definition and draws a property inference for positive parity
- [ ] `MeasurementAdjectives::assert()` returns false for negative parity
- [ ] `MeasurementAdjectives::assert()` returns false for unvalidated definitions
- [ ] `MeasurementAdjectives::prepare_schemas()` is a no-op (returns false)
- [ ] Module declaration added to `mod.rs`
- [ ] All unit tests pass with `cargo test`

## Out of Scope

- **Preform grammar** (`<measurement-adjective-definition>`, `<measurement-range>`, `<s-literal>`, `<property-name>`) — grammar parsing deferred; `claim_definition` takes pre-parsed parameters
- **`Grading::make_superlative`** — superlative form generation; deferred (requires the linguistics module)
- **`Grading::make_comparative`** — comparative form generation; deferred (requires the linguistics module)
- **`Grading::make_quiddity`** — quiddity form generation; deferred (requires the linguistics module)
- **`ComparativeRelations`** (`Chapter 3/Comparative Relations.w`) — the `property_comparison_bp_family`, `stock`, `typecheck`, `schema`, and `initialise` methods; deferred until both measurement adjectives and `Measurements::create_comparatives` are in place
- **`Measurements::create_comparatives`** — comparative creation; deferred (depends on `Grading`, `BinaryPredicateFamilies`, and `ComparativeRelations`)
- **I6 schema generation** — `RTAdjectives::make_mdef_test_schema`, `RTAdjectives::new_measurement_compilation_data` — run-time compilation deferred
- **`AdjectiveMeanings::perform_task_via_function`** — `TEST_ATOM_TASK` dispatch; deferred
- **Problem message generation** — `PM_GradingMisphrased`, `PM_MultiwordGrading`, `PM_GradingCalled`, `PM_GradingUnless`, `PM_GradingUnknownProperty`, `PM_GradingNonarithmeticKOV`, `PM_GradingWrongKOV`, `PM_GradingNonLiteral` — problem messages deferred
- **`ConditionsOfSubjects`** (`Chapter 4/Conditions of Subjects.w`) — depends on instances and enumerative adjectives; deferred
- **`NonlocalVariables`** (`Chapter 2/Nonlocal Variables.w`) — depends on `VariableSubjects`; deferred
- **`VariableSubjects`** (`Chapter 4/Variable Subjects.w`) — depends on `NonlocalVariables`; deferred
- **`OrderingInstances`** (`Chapter 2/Ordering Instances.w`) — ordering instances for compilation; deferred
- **`Preform for Instances`** (`Chapter 2/Preform for Instances.w`) — grammar for parsing instance names; deferred
- **`The Model World`** (`Chapter 5/The Model World.w`) — world-building stages; deferred
- **`The Naming Thicket`** (`Chapter 5/The Naming Thicket.w`) — naming resolution; deferred
- **`Indefinite Appearance`** (`Chapter 5/Indefinite Appearance.w`) — indefinite descriptions; deferred
- **`Assert Propositions`** (`Chapter 1/Assert Propositions.w`) — the assertion pipeline; deferred until more infrastructure is in place
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
