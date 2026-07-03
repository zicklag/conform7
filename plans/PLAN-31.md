# Plan 31: Measurement Definitions — The `measurement_definition` Struct and `Measurements` Management Functions
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the `measurement_definition` struct and `Measurements` management functions — the data structure and core operations for measurement-based adjectives (e.g., "roomy", "tall", "large") that compare a property value against a threshold. This is the foundation for both `MeasurementAdjectives` (the adjective meaning family) and `ComparativeRelations` (the comparative binary predicate family).

This is the smallest next step after PLAN-30 because:

1. **It's the next item in the knowledge module startup that has no remaining dependencies.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `MeasurementAdjectives::start()` at line 41 — after `InstanceAdjectives::start()` (PLAN-30, In progress) and `EitherOrPropertyAdjectives::start()` (PLAN-29, Complete). `MeasurementAdjectives::start()` creates the `measurement_amf` family, and its `claim_definition` method creates `measurement_definition` structs via `Measurements::new()`. The `measurement_definition` struct is also used by `ComparativeRelations::start()` (line 44) via `Measurements::create_comparatives()`.

2. **It's the data structure that both `MeasurementAdjectives` and `ComparativeRelations` depend on.** The `measurement_definition` struct (92 lines of C in `Measurements.w`) stores the headword, property, threshold, region shape, and compilation data for a measurement adjective. `MeasurementAdjectives::claim_definition` creates them, `MeasurementAdjectives::assert` validates and uses them, and `ComparativeRelations::stock` calls `Measurements::create_comparatives()` to iterate over them. Building this struct now unlocks both downstream systems.

3. **It's independently testable without grammar parsing.** The `measurement_definition` struct and `Measurements` management functions (`new`, `validate`, `is_valid`, `retrieve`, `read_property_details`, `weak_comparison_bp`, `strict_comparison`) can be tested programmatically — creating measurement definitions, validating them, retrieving them by property and shape, and testing the comparison operator mappings — all without needing the Preform grammar for `<measurement-adjective-definition>`, `<measurement-range>`, or `<s-literal>`.

4. **It's a prerequisite for `MeasurementAdjectives::assert`.** `MeasurementAdjectives::assert` (Chapter 3/Measurement Adjectives.w, lines 172-185) calls `Measurements::validate(mdef)` and `Measurements::is_valid(mdef)` before drawing a property inference. Without the `measurement_definition` struct and validation functions, the assert method cannot function.

5. **It's a prerequisite for `ComparativeRelations`.** `ComparativeRelations::stock` (Chapter 3/Comparative Relations.w, line 36) calls `Measurements::create_comparatives()` at stage 2, which iterates over all `measurement_definition` structs, validates them, and creates comparative binary predicates. Without the measurement definitions, comparative relations have nothing to work with.

6. **It introduces the region shape constants — a fundamental concept.** The three region shapes (`MEASURE_T_OR_LESS = -1`, `MEASURE_T_EXACTLY = 0`, `MEASURE_T_OR_MORE = 1`) define how a measurement adjective compares a property value against a threshold. These constants are used by `MeasurementAdjectives`, `ComparativeRelations`, and the run-time compilation system. Defining them now establishes the shared vocabulary.

7. **Independently testable.** We can create `MeasurementDefinition` structs, test `Measurements::new` (creates a definition with the right headword, property, shape, and threshold text), test `Measurements::validate` (fills in missing property name and threshold), test `Measurements::is_valid` (checks if a definition is fully validated), test `Measurements::retrieve` (finds a definition by property and shape), test `Measurements::read_property_details` (extracts property and shape), test `Measurements::weak_comparison_bp` (maps shape to comparison operator), and test `Measurements::strict_comparison` (maps shape to comparison operator string) — all without needing grammar parsing, the adjective meaning system, or run-time compilation.

## Background

### C reference architecture

#### Measurement Definition (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92)

The `measurement_definition` struct stores everything needed for a measurement adjective:

```c
typedef struct measurement_definition {
    struct parse_node *measurement_node; /* where the actual definition is */

    struct wording headword; /* adjective being defined (must be single word) */
    struct adjective_meaning *headword_as_adjective; /* which adjective meaning */
    struct wording superlative; /* its superlative form */

    struct property *prop; /* the property being compared, if any */
    struct wording name_of_property_to_compare; /* and its name */

    int region_shape; /* one of the |MEASURE_T_*| constants */
    int region_threshold; /* numerical value of threshold (if any) */
    struct kind *region_kind; /* of this value */
    int region_threshold_evaluated; /* have we evaluated this one yet? */
    struct wording region_threshold_text; /* text of threshold value */

    struct measurement_compilation_data compilation_data;
    CLASS_DEFINITION
} measurement_definition;
```

Key functions:

```c
measurement_definition *Measurements::new(parse_node *q, wording AW, wording THRESW,
    property *prop, int shape, wording PRW) {
    measurement_definition *mdef = CREATE(measurement_definition);
    mdef->measurement_node = q;
    mdef->headword = Wordings::first_word(AW);
    mdef->region_threshold = 0;
    mdef->region_threshold_text = THRESW;
    mdef->region_threshold_evaluated = FALSE;
    mdef->prop = prop;
    mdef->region_shape = shape;
    mdef->name_of_property_to_compare = PRW;
    mdef->superlative = EMPTY_WORDING;
    mdef->headword_as_adjective = NULL;
    mdef->compilation_data = RTAdjectives::new_measurement_compilation_data(mdef);
    return mdef;
}
```

```c
void Measurements::read_property_details(measurement_definition *mdef,
    property **prn, int *shape) {
    if (prn) *prn = mdef->prop;
    if (shape) *shape = mdef->region_shape;
}

measurement_definition *Measurements::retrieve(property *prn, int shape) {
    measurement_definition *mdef;
    LOOP_OVER(mdef, measurement_definition) {
        Measurements::validate(mdef);
        if ((Measurements::is_valid(mdef)) && (mdef->prop == prn) &&
            (mdef->region_shape == shape))
            return mdef;
    }
    return NULL;
}
```

```c
void Measurements::validate(measurement_definition *mdef) {
    if ((mdef->prop == NULL) && (Wordings::nonempty(mdef->name_of_property_to_compare)))
        @<Fill in the missing property name, P@>;
    if (mdef->region_threshold_evaluated == FALSE)
        @<Fill in the missing threshold value, t@>;
}
```

```c
int Measurements::is_valid(measurement_definition *mdef) {
    if ((mdef->prop == NULL) || (mdef->region_threshold_evaluated == FALSE))
        return FALSE;
    return TRUE;
}
```

```c
binary_predicate *Measurements::weak_comparison_bp(int shape) {
    binary_predicate *operator = NULL;
    switch (shape) {
        case MEASURE_T_OR_MORE: operator = R_numerically_greater_than_or_equal_to; break;
        case MEASURE_T_EXACTLY: operator = R_equality; break;
        case MEASURE_T_OR_LESS: operator = R_numerically_less_than_or_equal_to; break;
        default: internal_error("unknown region for weak comparison");
    }
    return operator;
}

char *Measurements::strict_comparison(int shape) {
    char *operator = NULL;
    switch (shape) {
        case MEASURE_T_OR_MORE: operator = ">"; break;
        case MEASURE_T_OR_LESS: operator = "<"; break;
        default: internal_error("unknown region for strict comparison");
    }
    return operator;
}
```

#### Region Shape Constants (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 31-33)

```c
@d MEASURE_T_OR_LESS -1
@d MEASURE_T_EXACTLY 0
@d MEASURE_T_OR_MORE 1
```

#### Measurement Adjectives (`inform7/knowledge-module/Chapter 3/Measurement Adjectives.w`, lines 1-197)

The `MeasurementAdjectives` module creates the `measurement_amf` family and uses `measurement_definition` structs:

```c
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

#### Comparative Relations (`inform7/knowledge-module/Chapter 3/Comparative Relations.w`, lines 1-120)

The `ComparativeRelations` module uses measurement definitions to create comparative binary predicates:

```c
void ComparativeRelations::stock(bp_family *self, int n) {
    if (n == 2) Measurements::create_comparatives();
}
```

```c
void Measurements::create_comparatives(void) {
    measurement_definition *mdef;
    LOOP_OVER(mdef, measurement_definition) {
        Measurements::validate(mdef);
        if ((Measurements::is_valid(mdef)) &&
            (mdef->region_shape != MEASURE_T_EXACTLY)) {
            wording H = mdef->headword;
            wording comparative_form = Grading::make_comparative(H,
                Task::language_of_syntax());
            vocabulary_entry *quiddity =
                Lexer::word(Wordings::first_wn(
                    Grading::make_quiddity(H, Task::language_of_syntax())));
            i6_schema *schema_to_compare_property_values;

            @<Work out property comparison schema@>;
            @<Construct a BP named for the quiddity and tested using the comparative schema@>;
        }
    }
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Measurements.w` — `measurement_definition` struct, `Measurements::new`, `Measurements::validate`, `Measurements::is_valid`, `Measurements::retrieve`, `Measurements::read_property_details`, `Measurements::weak_comparison_bp`, `Measurements::strict_comparison`, `Measurements::validate_definitions`, `Measurements::create_comparatives`, `Measurements::register_comparative` (301 lines)
- `inform7/knowledge-module/Chapter 3/Measurement Adjectives.w` — `MeasurementAdjectives` module, `measurement_amf` family, `claim_definition`, `assert`, `prepare_schemas` (197 lines)
- `inform7/knowledge-module/Chapter 3/Comparative Relations.w` — `ComparativeRelations` module, `property_comparison_bp_family`, `stock`, `typecheck`, `schema`, `initialise` (120 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `MeasurementAdjectives::start()` (line 41), `ComparativeRelations::start()` (line 44)
- `inform7/assertions-module/Chapter 8/Adjective Meanings.w` — `AdjectiveMeanings::new_family`, `AdjectiveMeanings::new` (PLAN-28)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` (PLAN-19)
- `services/linguistics-module/Chapter 2/Adjectives.w` — `Adjectives::declare` (PLAN-28)
- `services/linguistics-module/Chapter 2/Grading.w` — `Grading::make_superlative`, `Grading::make_comparative`, `Grading::make_quiddity` (deferred)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/instances.rs` — `Instance` struct, `Instances` management functions (PLAN-30, In progress).
- `crates/conform7-semantics/src/knowledge/instance_subjects.rs` — `InstanceSubjects` family (PLAN-30, In progress).
- `crates/conform7-semantics/src/knowledge/instance_adjectives.rs` — `InstanceAdjectives` module, `enumerative_amf` family (PLAN-30, In progress).
- `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs` — `EitherOrPropertyAdjectives` module, `EITHER_OR_PROPERTY_FAMILY` constant, `start()`, `is()`, `create_for_property()`, `assert()`, `prepare_schemas()`, `index()`, unit tests (PLAN-29, Complete).
- `crates/conform7-semantics/src/knowledge/adjectives.rs` — `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, `AdjectiveDomainData` struct, `AdjectiveMeaningFamilyMethods` struct, `AdjectiveMeanings` management functions, `AdjectiveAmbiguity` management functions, `AdjectiveMeaningDomains` management functions, `Adjectives::declare`, `Adjectives::find`, `Adjectives::get_nominative_singular`, unit tests (PLAN-28, Complete).
- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData` struct, `ValuePropertyData` struct, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties::new_eo_data`, `EitherOrProperties::make_pair`, `EitherOrProperties::get_negation`, `EitherOrProperties::as_adjective`, `ValueProperties` functions, unit tests (PLAN-25, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, `PropertyInferences::new()`, `PropertyInferences::draw()`, `PropertyInferences::draw_negated()`, `PropertyInferences::draw_from_metadata()`, unit tests (PLAN-19, Complete).
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
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod instances;`, `pub mod instance_subjects;`, `pub mod instance_adjectives;` from PLAN-30).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **Region shape constants** — define the three measurement region shapes:
   - `MEASURE_T_OR_LESS = -1` — property value is less than or equal to the threshold
   - `MEASURE_T_EXACTLY = 0` — property value is exactly equal to the threshold
   - `MEASURE_T_OR_MORE = 1` — property value is greater than or equal to the threshold

2. **`MeasurementDefinition` struct** — a new struct in a new `measurements` module with:
   - `headword` — the adjective being defined (simplified: a string instead of `wording`)
   - `headword_as_adjective` — optional adjective meaning index (set by `MeasurementAdjectives`)
   - `superlative` — optional superlative form string (deferred: set by `Grading::make_superlative`)
   - `prop` — optional property index (the property being compared)
   - `name_of_property_to_compare` — optional string name of the property (used before the property is resolved)
   - `region_shape` — one of the `MEASURE_T_*` constants
   - `region_threshold` — numerical value of the threshold
   - `region_kind` — optional kind index of the threshold value
   - `region_threshold_evaluated` — whether the threshold has been evaluated
   - `region_threshold_text` — optional string text of the threshold value

3. **`Measurements` management functions**:
   - `Measurements::new(headword, prop, shape, threshold_text)` — create a new measurement definition:
     - Stores the headword, property, shape, and threshold text
     - Initialises threshold to 0, evaluated to false
     - Returns the index of the new definition
   - `Measurements::validate(mdef_idx, definitions, properties, kinds)` — validate a measurement definition:
     - If the property is missing but the property name is set, try to resolve it (simplified: check if the property name matches an existing property)
     - If the threshold hasn't been evaluated, try to evaluate it (simplified: parse the threshold text as a number)
     - Sets `region_threshold_evaluated` to true on success
   - `Measurements::is_valid(mdef_idx, definitions)` — check if a measurement definition is fully validated:
     - Returns true if both the property and threshold are resolved
   - `Measurements::read_property_details(mdef_idx, definitions)` — extract the property and shape from a definition:
     - Returns `(Option<usize>, i32)` — the property index and region shape
   - `Measurements::retrieve(prn_idx, shape, definitions)` — find a measurement definition by property and shape:
     - Validates each definition, then checks if it matches the given property and shape
     - Returns the index of the matching definition, or None
   - `Measurements::weak_comparison_bp(shape)` — get the comparison operator for a shape:
     - `MEASURE_T_OR_MORE` → `R_numerically_greater_than_or_equal_to` (simplified: returns a string ">=")
     - `MEASURE_T_EXACTLY` → `R_equality` (simplified: returns a string "==")
     - `MEASURE_T_OR_LESS` → `R_numerically_less_than_or_equal_to` (simplified: returns a string "<=")
   - `Measurements::strict_comparison(shape)` — get the comparison operator string for a shape:
     - `MEASURE_T_OR_MORE` → `">"`
     - `MEASURE_T_OR_LESS` → `"<"`
     - `MEASURE_T_EXACTLY` → panics (exact measurements don't have strict comparisons)
   - `Measurements::validate_definitions(definitions, properties, kinds)` — validate all definitions (simplified: calls `validate` on each)
   - `Measurements::create_comparatives(definitions, ...)` — create comparative forms (simplified: no-op, deferred to ComparativeRelations plan)

4. **Integration with the knowledge module** — add the `measurements` module declaration to the knowledge module's `mod.rs`.

5. **Unit tests** — create measurement definitions, test `Measurements::new` (creates a definition with the right headword, property, shape, and threshold text), test `Measurements::validate` (fills in missing property name and threshold), test `Measurements::is_valid` (checks if a definition is fully validated), test `Measurements::retrieve` (finds a definition by property and shape), test `Measurements::read_property_details` (extracts property and shape), test `Measurements::weak_comparison_bp` (maps shape to comparison operator), test `Measurements::strict_comparison` (maps shape to comparison operator string), test that `retrieve` returns None for non-matching definitions, test that `is_valid` returns false for unvalidated definitions.

## Tasks

### 1. Create the `MeasurementDefinition` struct and `Measurements` management functions

- [ ] Create `crates/conform7-semantics/src/knowledge/measurements.rs` with:

  ```rust
  /// Measurement definitions — adjectives that compare a property value against a threshold.
  ///
  /// Corresponds to `measurement_definition` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92).
  ///
  /// A typical example would be:
  ///
  /// > Definition: A container is roomy if its carrying capacity is 10 or more.
  ///
  /// Here the domain of the definition is "container", and we assign an adjective
  /// meaning for "roomy" which involves the comparison of a property (here "carrying
  /// capacity") against a threshold value t (here, t=10). "roomy" is said to
  /// be the headword; the comparative form would be "roomier", and the superlative
  /// form "roomiest".
  ///
  /// Simplified:
  /// - No `parse_node *` (creation tracking deferred)
  /// - No `measurement_compilation_data` (run-time compilation deferred)
  /// - No `Grading::make_superlative` (superlative form deferred)
  /// - No `Grading::make_comparative` (comparative form deferred)
  /// - No `Grading::make_quiddity` (quiddity form deferred)
  use crate::knowledge::properties::Property;

  /// Region shape constants for measurement definitions.
  ///
  /// Corresponds to `MEASURE_T_*` constants in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 31-33).
  ///
  /// These define how a measurement adjective compares a property value
  /// against a threshold:
  /// - `MEASURE_T_OR_LESS` (-1): property value <= threshold (e.g., "10 or less")
  /// - `MEASURE_T_EXACTLY` (0): property value == threshold (e.g., "exactly 10")
  /// - `MEASURE_T_OR_MORE` (1): property value >= threshold (e.g., "10 or more")
  pub const MEASURE_T_OR_LESS: i32 = -1;
  pub const MEASURE_T_EXACTLY: i32 = 0;
  pub const MEASURE_T_OR_MORE: i32 = 1;

  /// A measurement definition — defines an adjective that compares a property
  /// value against a threshold.
  ///
  /// Corresponds to `measurement_definition` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92).
  ///
  /// Each such definition allows the property value to belong to a "region",
  /// which takes one of three "shapes": or-less, exactly, or or-more.
  #[derive(Clone, Debug)]
  pub struct MeasurementDefinition {
      /// The adjective being defined (headword, must be a single word).
      /// Corresponds to `headword` in the C reference.
      pub headword: String,
      /// The adjective meaning index, set by MeasurementAdjectives.
      /// Corresponds to `headword_as_adjective` in the C reference.
      pub headword_as_adjective: Option<usize>,
      /// The superlative form (e.g., "roomiest").
      /// Corresponds to `superlative` in the C reference.
      /// Deferred: set by Grading::make_superlative.
      pub superlative: Option<String>,
      /// The property being compared, if any.
      /// Corresponds to `prop` in the C reference.
      pub prop: Option<usize>,
      /// The name of the property to compare (used before the property is resolved).
      /// Corresponds to `name_of_property_to_compare` in the C reference.
      pub name_of_property_to_compare: Option<String>,
      /// The region shape: one of MEASURE_T_OR_LESS, MEASURE_T_EXACTLY, MEASURE_T_OR_MORE.
      /// Corresponds to `region_shape` in the C reference.
      pub region_shape: i32,
      /// The numerical value of the threshold.
      /// Corresponds to `region_threshold` in the C reference.
      pub region_threshold: i32,
      /// The kind of the threshold value, if known.
      /// Corresponds to `region_kind` in the C reference.
      pub region_kind: Option<usize>,
      /// Whether the threshold has been evaluated.
      /// Corresponds to `region_threshold_evaluated` in the C reference.
      pub region_threshold_evaluated: bool,
      /// The text of the threshold value (e.g., "10").
      /// Corresponds to `region_threshold_text` in the C reference.
      pub region_threshold_text: Option<String>,
  }

  impl MeasurementDefinition {
      /// Create a new measurement definition with default values.
      pub fn new(headword: &str) -> Self {
          MeasurementDefinition {
              headword: headword.to_string(),
              headword_as_adjective: None,
              superlative: None,
              prop: None,
              name_of_property_to_compare: None,
              region_shape: MEASURE_T_EXACTLY,
              region_threshold: 0,
              region_kind: None,
              region_threshold_evaluated: false,
              region_threshold_text: None,
          }
      }
  }

  /// The Measurements management module.
  ///
  /// Corresponds to `Measurements` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Measurements.w`).
  pub struct Measurements;

  impl Measurements {
      /// Create a new measurement definition.
      ///
      /// Corresponds to `Measurements::new` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 95-110).
      ///
      /// Returns the index of the new definition.
      ///
      /// Simplified:
      /// - No `parse_node *` (creation tracking deferred)
      /// - No `RTAdjectives::new_measurement_compilation_data` (run-time compilation deferred)
      pub fn new(
          headword: &str,
          prop: Option<usize>,
          shape: i32,
          threshold_text: Option<&str>,
          definitions: &mut Vec<MeasurementDefinition>,
      ) -> usize {
          let idx = definitions.len();
          let mut mdef = MeasurementDefinition::new(headword);
          mdef.prop = prop;
          mdef.region_shape = shape;
          mdef.region_threshold_text = threshold_text.map(|s| s.to_string());
          definitions.push(mdef);
          idx
      }

      /// Validate a measurement definition.
      ///
      /// Corresponds to `Measurements::validate` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 150-155).
      ///
      /// This function tries to fill in missing data:
      /// 1. If the property is missing but the property name is set, try to resolve it
      /// 2. If the threshold hasn't been evaluated, try to evaluate it
      ///
      /// Simplified:
      /// - No `<property-name>` grammar parsing (uses string matching)
      /// - No `<s-literal>` grammar parsing (uses simple number parsing)
      /// - No `Rvalues::to_kind` or `Rvalues::to_encoded_notation`
      /// - No `Kinds::Behaviour::is_quasinumerical` check
      /// - No `Kinds::compatible` check
      /// - No problem message generation
      pub fn validate(
          mdef_idx: usize,
          definitions: &mut [MeasurementDefinition],
          properties: &[Property],
      ) {
          if let Some(mdef) = definitions.get_mut(mdef_idx) {
              // Fill in missing property from name
              if mdef.prop.is_none() {
                  if let Some(ref prn_name) = mdef.name_of_property_to_compare {
                      for (i, prn) in properties.iter().enumerate() {
                          if prn.name == prn_name.as_str() {
                              mdef.prop = Some(i);
                              break;
                          }
                      }
                  }
              }

              // Fill in missing threshold value
              if !mdef.region_threshold_evaluated {
                  if let Some(ref threshold_text) = mdef.region_threshold_text {
                      if let Ok(val) = threshold_text.parse::<i32>() {
                          mdef.region_threshold = val;
                          mdef.region_threshold_evaluated = true;
                      }
                  }
              }
          }
      }

      /// Check if a measurement definition is fully validated.
      ///
      /// Corresponds to `Measurements::is_valid` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 220-224).
      ///
      /// Returns true if both the property and threshold are resolved.
      pub fn is_valid(mdef_idx: usize, definitions: &[MeasurementDefinition]) -> bool {
          definitions.get(mdef_idx).is_some_and(|mdef| {
              mdef.prop.is_some() && mdef.region_threshold_evaluated
          })
      }

      /// Extract the property and shape from a measurement definition.
      ///
      /// Corresponds to `Measurements::read_property_details` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 115-119).
      ///
      /// Returns (property_index, region_shape).
      pub fn read_property_details(
          mdef_idx: usize,
          definitions: &[MeasurementDefinition],
      ) -> (Option<usize>, i32) {
          if let Some(mdef) = definitions.get(mdef_idx) {
              (mdef.prop, mdef.region_shape)
          } else {
              (None, MEASURE_T_EXACTLY)
          }
      }

      /// Find a measurement definition by property and shape.
      ///
      /// Corresponds to `Measurements::retrieve` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 124-133).
      ///
      /// Validates each definition, then checks if it matches the given
      /// property and shape. Returns the index of the matching definition,
      /// or None.
      pub fn retrieve(
          prn_idx: usize,
          shape: i32,
          definitions: &mut [MeasurementDefinition],
          properties: &[Property],
      ) -> Option<usize> {
          for i in 0..definitions.len() {
              Measurements::validate(i, definitions, properties);
              if let Some(mdef) = definitions.get(i) {
                  if Measurements::is_valid(i, definitions)
                      && mdef.prop == Some(prn_idx)
                      && mdef.region_shape == shape
                  {
                      return Some(i);
                  }
              }
          }
          None
      }

      /// Get the weak comparison operator for a region shape.
      ///
      /// Corresponds to `Measurements::weak_comparison_bp` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 40-49).
      ///
      /// Returns a string representation of the comparison operator:
      /// - MEASURE_T_OR_MORE -> ">="
      /// - MEASURE_T_EXACTLY -> "=="
      /// - MEASURE_T_OR_LESS -> "<="
      ///
      /// Simplified: returns a string instead of a `binary_predicate *`.
      pub fn weak_comparison_bp(shape: i32) -> &'static str {
          match shape {
              MEASURE_T_OR_MORE => ">=",
              MEASURE_T_EXACTLY => "==",
              MEASURE_T_OR_LESS => "<=",
              _ => panic!("unknown region for weak comparison"),
          }
      }

      /// Get the strict comparison operator string for a region shape.
      ///
      /// Corresponds to `Measurements::strict_comparison` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 51-59).
      ///
      /// Returns a string representation of the comparison operator:
      /// - MEASURE_T_OR_MORE -> ">"
      /// - MEASURE_T_OR_LESS -> "<"
      ///
      /// Panics for MEASURE_T_EXACTLY (exact measurements don't have strict comparisons).
      pub fn strict_comparison(shape: i32) -> &'static str {
          match shape {
              MEASURE_T_OR_MORE => ">",
              MEASURE_T_OR_LESS => "<",
              _ => panic!("unknown region for strict comparison"),
          }
      }

      /// Validate all measurement definitions.
      ///
      /// Corresponds to `Measurements::validate_definitions` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 141-145).
      pub fn validate_definitions(
          definitions: &mut [MeasurementDefinition],
          properties: &[Property],
      ) {
          for i in 0..definitions.len() {
              Measurements::validate(i, definitions, properties);
          }
      }

      /// Create comparative forms for all measurement definitions.
      ///
      /// Corresponds to `Measurements::create_comparatives` in the C reference
      /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 231-249).
      ///
      /// Simplified: no-op. Comparative creation depends on Grading, binary
      /// predicates, and ComparativeRelations — deferred to a later plan.
      pub fn create_comparatives(
          _definitions: &[MeasurementDefinition],
          _properties: &[Property],
      ) {
          // No-op: deferred to ComparativeRelations plan
      }
  }
  ```

### 2. Integrate with the knowledge module

- [ ] Add module declaration to `crates/conform7-semantics/src/knowledge/mod.rs`:

  ```rust
  pub mod measurements;
  ```

### 3. Unit tests

- [ ] Add tests to `crates/conform7-semantics/src/knowledge/measurements.rs`:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::knowledge::properties::{Property, ValuePropertyData};

      #[test]
      fn test_region_shape_constants() {
          assert_eq!(MEASURE_T_OR_LESS, -1);
          assert_eq!(MEASURE_T_EXACTLY, 0);
          assert_eq!(MEASURE_T_OR_MORE, 1);
      }

      #[test]
      fn test_new_creates_definition() {
          let mut definitions = Vec::new();
          let idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

          assert_eq!(idx, 0);
          assert_eq!(definitions[0].headword, "roomy");
          assert_eq!(definitions[0].prop, Some(0));
          assert_eq!(definitions[0].region_shape, MEASURE_T_OR_MORE);
          assert_eq!(definitions[0].region_threshold_text, Some("10".to_string()));
          assert_eq!(definitions[0].region_threshold, 0);
          assert!(!definitions[0].region_threshold_evaluated);
          assert!(definitions[0].headword_as_adjective.is_none());
      }

      #[test]
      fn test_new_without_threshold() {
          let mut definitions = Vec::new();
          let idx = Measurements::new("roomy", Some(0), MEASURE_T_EXACTLY, None, &mut definitions);

          assert_eq!(idx, 0);
          assert!(definitions[0].region_threshold_text.is_none());
      }

      #[test]
      fn test_validate_fills_in_threshold() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          let properties = Vec::new();

          assert!(!definitions[0].region_threshold_evaluated);
          Measurements::validate(0, &mut definitions, &properties);
          assert!(definitions[0].region_threshold_evaluated);
          assert_eq!(definitions[0].region_threshold, 10);
      }

      #[test]
      fn test_validate_fills_in_property_from_name() {
          let mut definitions = Vec::new();
          let mut mdef = MeasurementDefinition::new("roomy");
          mdef.name_of_property_to_compare = Some("carrying capacity".to_string());
          mdef.region_shape = MEASURE_T_OR_MORE;
          mdef.region_threshold_text = Some("10".to_string());
          definitions.push(mdef);

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

          assert!(definitions[0].prop.is_none());
          Measurements::validate(0, &mut definitions, &properties);
          assert_eq!(definitions[0].prop, Some(0));
      }

      #[test]
      fn test_is_valid_returns_true_for_validated() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          let properties = Vec::new();

          assert!(!Measurements::is_valid(0, &definitions));
          Measurements::validate(0, &mut definitions, &properties);
          assert!(Measurements::is_valid(0, &definitions));
      }

      #[test]
      fn test_is_valid_returns_false_for_unvalidated() {
          let definitions = vec![MeasurementDefinition::new("roomy")];
          assert!(!Measurements::is_valid(0, &definitions));
      }

      #[test]
      fn test_is_valid_returns_false_for_missing_property() {
          let mut definitions = Vec::new();
          let mut mdef = MeasurementDefinition::new("roomy");
          mdef.region_threshold_evaluated = true;
          mdef.region_threshold = 10;
          definitions.push(mdef);

          assert!(!Measurements::is_valid(0, &definitions));
      }

      #[test]
      fn test_read_property_details() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(42), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

          let (prop, shape) = Measurements::read_property_details(0, &definitions);
          assert_eq!(prop, Some(42));
          assert_eq!(shape, MEASURE_T_OR_MORE);
      }

      #[test]
      fn test_read_property_details_invalid_index() {
          let definitions = Vec::new();
          let (prop, shape) = Measurements::read_property_details(0, &definitions);
          assert_eq!(prop, None);
          assert_eq!(shape, MEASURE_T_EXACTLY);
      }

      #[test]
      fn test_retrieve_finds_matching_definition() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          Measurements::new("compact", Some(0), MEASURE_T_OR_LESS, Some("5"), &mut definitions);
          let properties = Vec::new();

          // Validate all
          Measurements::validate_definitions(&mut definitions, &properties);

          let found = Measurements::retrieve(0, MEASURE_T_OR_MORE, &mut definitions, &properties);
          assert_eq!(found, Some(0));

          let found = Measurements::retrieve(0, MEASURE_T_OR_LESS, &mut definitions, &properties);
          assert_eq!(found, Some(1));
      }

      #[test]
      fn test_retrieve_returns_none_for_no_match() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          let properties = Vec::new();
          Measurements::validate_definitions(&mut definitions, &properties);

          let found = Measurements::retrieve(1, MEASURE_T_OR_MORE, &mut definitions, &properties);
          assert_eq!(found, None);
      }

      #[test]
      fn test_weak_comparison_bp() {
          assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_OR_MORE), ">=");
          assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_EXACTLY), "==");
          assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_OR_LESS), "<=");
      }

      #[test]
      #[should_panic(expected = "unknown region for weak comparison")]
      fn test_weak_comparison_bp_invalid_shape() {
          Measurements::weak_comparison_bp(42);
      }

      #[test]
      fn test_strict_comparison() {
          assert_eq!(Measurements::strict_comparison(MEASURE_T_OR_MORE), ">");
          assert_eq!(Measurements::strict_comparison(MEASURE_T_OR_LESS), "<");
      }

      #[test]
      #[should_panic(expected = "unknown region for strict comparison")]
      fn test_strict_comparison_exact() {
          Measurements::strict_comparison(MEASURE_T_EXACTLY);
      }

      #[test]
      fn test_validate_definitions_validates_all() {
          let mut definitions = Vec::new();
          Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
          Measurements::new("compact", Some(1), MEASURE_T_OR_LESS, Some("5"), &mut definitions);
          let properties = Vec::new();

          assert!(!definitions[0].region_threshold_evaluated);
          assert!(!definitions[1].region_threshold_evaluated);

          Measurements::validate_definitions(&mut definitions, &properties);

          assert!(definitions[0].region_threshold_evaluated);
          assert_eq!(definitions[0].region_threshold, 10);
          assert!(definitions[1].region_threshold_evaluated);
          assert_eq!(definitions[1].region_threshold, 5);
      }

      #[test]
      fn test_create_comparatives_is_noop() {
          let definitions = Vec::new();
          let properties = Vec::new();
          // Should not panic
          Measurements::create_comparatives(&definitions, &properties);
      }

      #[test]
      fn test_measurement_definition_defaults() {
          let mdef = MeasurementDefinition::new("tall");
          assert_eq!(mdef.headword, "tall");
          assert!(mdef.headword_as_adjective.is_none());
          assert!(mdef.superlative.is_none());
          assert!(mdef.prop.is_none());
          assert!(mdef.name_of_property_to_compare.is_none());
          assert_eq!(mdef.region_shape, MEASURE_T_EXACTLY);
          assert_eq!(mdef.region_threshold, 0);
          assert!(mdef.region_kind.is_none());
          assert!(!mdef.region_threshold_evaluated);
          assert!(mdef.region_threshold_text.is_none());
      }
  }
  ```

## Success Criteria

- [ ] `MEASURE_T_OR_LESS`, `MEASURE_T_EXACTLY`, and `MEASURE_T_OR_MORE` constants defined
- [ ] `MeasurementDefinition` struct exists with `headword`, `headword_as_adjective`, `superlative`, `prop`, `name_of_property_to_compare`, `region_shape`, `region_threshold`, `region_kind`, `region_threshold_evaluated`, and `region_threshold_text` fields
- [ ] `Measurements::new()` creates a measurement definition with the right headword, property, shape, and threshold text
- [ ] `Measurements::validate()` fills in missing property name and threshold
- [ ] `Measurements::is_valid()` correctly identifies validated and unvalidated definitions
- [ ] `Measurements::read_property_details()` extracts property and shape from a definition
- [ ] `Measurements::retrieve()` finds a definition by property and shape, returns None for non-matches
- [ ] `Measurements::weak_comparison_bp()` maps each shape to the correct comparison operator string
- [ ] `Measurements::strict_comparison()` maps each shape to the correct comparison operator string
- [ ] `Measurements::validate_definitions()` validates all definitions
- [ ] `Measurements::create_comparatives()` is a no-op (deferred)
- [ ] Module declaration added to `mod.rs`
- [ ] All unit tests pass with `cargo test`

## Out of Scope

- **Measurement Adjectives** (`Chapter 3/Measurement Adjectives.w`) — the `measurement_amf` family, `claim_definition`, `assert`, and `prepare_schemas` methods; deferred to a later plan after the `measurement_definition` struct is in place
- **Comparative Relations** (`Chapter 3/Comparative Relations.w`) — the `property_comparison_bp_family`, `stock`, `typecheck`, `schema`, and `initialise` methods; deferred until both measurement definitions and measurement adjectives are in place
- **`Grading::make_superlative`** — superlative form generation; deferred (requires the linguistics module)
- **`Grading::make_comparative`** — comparative form generation; deferred (requires the linguistics module)
- **`Grading::make_quiddity`** — quiddity form generation; deferred (requires the linguistics module)
- **Preform grammar** — `<measurement-adjective-definition>`, `<measurement-range>`, `<s-literal>`, `<property-name>` — grammar parsing deferred
- **I6 schema generation** — `RTAdjectives::make_mdef_test_schema`, `RTAdjectives::new_measurement_compilation_data` — run-time compilation deferred
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
