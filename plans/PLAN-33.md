# Plan 33: Valued Properties — The `ValueProperties` Management Functions
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the `ValueProperties` management functions — the operations on valued property data that extend the property system from PLAN-25. This creates a `ValueProperties` module with functions for getting and setting the kind of a valued property, making a property name coincide with a kind name, managing the setting binary predicate, and obtaining properties with kind constraints.

This is the smallest next step after PLAN-32 because:

1. **It's the next most independent module in the knowledge module.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) has `ComparativeRelations::start()` at line 44 as the only remaining uncompleted startup item, but it's deferred because it depends on `Grading::make_comparative` and `Grading::make_quiddity` from the linguistics module. `ValueProperties` is not called from the startup sequence — it's a utility module whose functions are called on demand by other systems. It can be built independently now.

2. **The `ValuePropertyData` struct already exists from PLAN-25, but the management functions are missing.** PLAN-25 created the `ValuePropertyData` struct with fields (`property_value_kind`, `setting_bp`, `name_coincides_with_kind`, `as_condition_of_subject`, `relation_whose_state_this_stores`) and the `new_value_data()` constructor. But the management functions that operate on this data — `ValueProperties::kind`, `ValueProperties::set_kind`, `ValueProperties::make_coincide_with_kind`, `ValueProperties::coincides_with_kind`, `ValueProperties::make_setting_bp`, `ValueProperties::get_setting_bp`, `ValueProperties::set_stored_relation`, `ValueProperties::get_stored_relation`, `ValueProperties::obtain`, `ValueProperties::obtain_within_kind` — have not been implemented. These are the functions that make the data structure useful.

3. **It provides functions needed by other systems.** `ValueProperties::kind` is called by `Measurements::validate` (PLAN-31, Complete) to check kind compatibility when validating a measurement definition. `ValueProperties::coincides_with_kind` is called by `ComparativeRelations::typecheck` (deferred) to check if a property name coincides with a kind name. `ValueProperties::get_setting_bp` is called by `SettingPropertyRelations::assert` (PLAN-27, Complete) to get the setting binary predicate for a property. `ValueProperties::make_coincide_with_kind` is called by `ConditionsOfSubjects::parse` (deferred) to make a property coincide with a kind. Building these functions now unblocks downstream work.

4. **It's independently testable without grammar parsing or the assertions module.** The core functions (`kind`, `set_kind`, `coincides_with_kind`, `make_coincide_with_kind`, `make_setting_bp`, `get_setting_bp`, `set_stored_relation`, `get_stored_relation`, `obtain`, `obtain_within_kind`) can be tested programmatically — creating valued properties, setting their kinds, making them coincide with kinds, creating and retrieving setting binary predicates, and obtaining properties with kind constraints — all without needing Preform grammar, the assertions module, or run-time compilation.

5. **It introduces the `obtain_within_kind` pattern — a new property creation pattern.** Unlike `Properties::obtain` (which creates a property without kind constraints), `ValueProperties::obtain_within_kind` creates or retrieves a property while ensuring its value kind is compatible with a given kind. This is a fundamental pattern used by the assertion pipeline when parsing sentences like "A thing has a number called weight."

6. **It introduces the `make_coincide_with_kind` pattern — a bridge between kinds and properties.** When a kind of value is defined (e.g., "weight"), and then a property with the same name is created (e.g., "A thing has a weight"), the property and kind are said to "coincide." This is how Inform 7 enables sentences like "if the ball's weight is 10" — the property "weight" stores values of the kind "weight." Implementing this now establishes the pattern for `ConditionsOfSubjects` and the assertion pipeline.

## Background

### C reference architecture

#### Valued Properties (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 1-218)

The Valued Properties system provides management functions for valued property data:

```c
typedef struct value_property_data {
    struct kind *property_value_kind;
    struct binary_predicate *setting_bp;
    struct binary_predicate *relation_whose_state_this_stores;
    struct condition_of_subject *as_condition_of_subject;
    int name_coincides_with_kind;
    CLASS_DEFINITION
} value_property_data;
```

Key functions:

```c
value_property_data *ValueProperties::new_value_data(property *prn) {
    value_property_data *vod = CREATE(value_property_data);
    vod->property_value_kind = NULL;
    vod->setting_bp = NULL;
    vod->name_coincides_with_kind = FALSE;
    vod->as_condition_of_subject = NULL;
    vod->relation_whose_state_this_stores = NULL;
    return vod;
}
```

```c
property *ValueProperties::obtain(wording W) {
    return Properties::obtain(W, TRUE);
}
```

```c
property *ValueProperties::obtain_within_kind(wording W, kind *K) {
    property *prn = NULL;
    if (K == NULL) K = K_object;
    K = Kinds::weaken(K, K_object);
    if (<property-name>(W)) {
        prn = <<rp>>;
        if (prn->value_data == NULL) @<Issue an incompatible property kind message@>;
        kind *existing_kind = prn->value_data->property_value_kind;
        switch(Kinds::compatible(K, existing_kind)) {
            case SOMETIMES_MATCH:
                if (Kinds::compatible(existing_kind, K) != ALWAYS_MATCH)
                    @<Issue an incompatible property kind message@>;
                prn->value_data->property_value_kind = K;
                break;
            case NEVER_MATCH:
                @<Issue an incompatible property kind message@>;
        }
    } else {
        prn = Properties::obtain(W, TRUE);
        prn->value_data->property_value_kind = K;
    }
    return prn;
}
```

```c
kind *ValueProperties::kind(property *prn) {
    if ((prn == NULL) || (prn->either_or_data)) return NULL;
    return prn->value_data->property_value_kind;
}

void ValueProperties::set_kind(property *prn, kind *K) {
    if (K == NULL) internal_error("tried to set null kind");
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    // ... kind validation and problem messages ...
    prn->value_data->property_value_kind = K;
}
```

```c
void ValueProperties::make_coincide_with_kind(property *prn, kind *K) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    ValueProperties::set_kind(prn, K);
    if (Kinds::eq(K, K_grammatical_gender)) P_grammatical_gender = prn;
    prn->value_data->name_coincides_with_kind = TRUE;
    if (Properties::can_name_coincide_with_kind(K))
        Instances::make_kind_coincident(K, prn);
}

int ValueProperties::coincides_with_kind(property *prn) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    return prn->value_data->name_coincides_with_kind;
}
```

```c
void ValueProperties::make_setting_bp(property *prn, wording W) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    binary_predicate *bp = SettingPropertyRelations::find_set_property_BP(W);
    if (bp == NULL) bp = SettingPropertyRelations::make_set_property_BP(W);
    SettingPropertyRelations::fix_property_bp(bp);
    SettingPropertyRelations::fix_property_bp(BinaryPredicates::get_reversal(bp));
    prn->value_data->setting_bp = bp;
}

binary_predicate *ValueProperties::get_setting_bp(property *prn) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    return prn->value_data->setting_bp;
}
```

```c
void ValueProperties::set_stored_relation(property *prn, binary_predicate *bp) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    prn->value_data->relation_whose_state_this_stores = bp;
}

binary_predicate *ValueProperties::get_stored_relation(property *prn) {
    if ((prn == NULL) || (prn->either_or_data)) internal_error("non-value property");
    return prn->value_data->relation_whose_state_this_stores;
}
```

```c
void ValueProperties::assert(property *prn, inference_subject *owner,
    parse_node *val, int certainty) {
    pcalc_prop *prop = Propositions::Abstract::to_set_property(prn, val);
    Assert::true_about(prop, owner, certainty);
}
```

#### Properties (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 547-551)

```c
int Properties::can_name_coincide_with_kind(kind *K) {
    if (K == NULL) return FALSE;
    return K->construct->can_coincide_with_property;
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Valued Properties.w` — `ValueProperties` module, `value_property_data` struct, `new_value_data`, `obtain`, `obtain_within_kind`, `kind`, `set_kind`, `make_coincide_with_kind`, `coincides_with_kind`, `make_setting_bp`, `get_setting_bp`, `set_stored_relation`, `get_stored_relation`, `assert` (218 lines)
- `inform7/knowledge-module/Chapter 3/Properties.w` — `Properties` struct, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `Properties::property_with_same_name_as`, `Properties::can_name_coincide_with_kind` (551 lines)
- `inform7/knowledge-module/Chapter 3/Setting Property Relation.w` — `SettingPropertyRelations` module, `make_set_property_BP`, `find_set_property_BP`, `fix_property_bp` (PLAN-27, Complete)
- `inform7/knowledge-module/Chapter 3/Either-Or Properties.w` — `EitherOrProperties` module, `either_or_property_data` struct (PLAN-25, Complete)
- `inform7/knowledge-module/Chapter 2/Instances.w` — `Instances::make_kind_coincident` (PLAN-30, Complete)
- `inform7/knowledge-module/Chapter 3/Measurements.w` — `Measurements::validate` uses `ValueProperties::kind` (PLAN-31, Complete)
- `inform7/knowledge-module/Chapter 3/Comparative Relations.w` — `ComparativeRelations::typecheck` uses `ValueProperties::coincides_with_kind` (deferred)
- `inform7/knowledge-module/Chapter 4/Conditions of Subjects.w` — `ConditionsOfSubjects::parse` uses `ValueProperties::make_coincide_with_kind` (deferred)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `ValuePropertyData` struct (with `property_value_kind`, `setting_bp`, `name_coincides_with_kind`, `as_condition_of_subject`, `relation_whose_state_this_stores` fields), `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `Properties::property_with_same_name_as`, `EitherOrProperties` functions, unit tests (PLAN-25, Complete).
- `crates/conform7-semantics/src/knowledge/setting_property_relation.rs` — `SettingPropertyRelations` module, `start()`, `make_set_property_BP()`, `find_set_property_BP()`, `fix_property_bp()`, `bp_get_set_property()`, `bp_get_pending_text()`, `bp_sets_a_property()`, `assert()`, `schema()`, unit tests (PLAN-27, Complete).
- `crates/conform7-semantics/src/knowledge/measurements.rs` — `MeasurementDefinition` struct, `Measurements` management functions, unit tests (PLAN-31, Complete).
- `crates/conform7-semantics/src/knowledge/instances.rs` — `Instance` struct, `Instances` management functions, `Instances::make_kind_coincident` (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/measurement_adjectives.rs` — `MeasurementAdjectives` module, `measurement_amf` family, `start()`, `claim_definition()`, `assert()`, `prepare_schemas()`, unit tests (PLAN-32, Complete).
- `crates/conform7-semantics/src/knowledge/adjectives.rs` — `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, `AdjectiveMeanings` management functions, unit tests (PLAN-28, Complete).
- `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs` — `EitherOrPropertyAdjectives` module, `EITHER_OR_PROPERTY_FAMILY` constant, `start()`, `is()`, `create_for_property()`, `assert()`, `prepare_schemas()`, `index()`, unit tests (PLAN-29, Complete).
- `crates/conform7-semantics/src/knowledge/instance_adjectives.rs` — `InstanceAdjectives` module, `enumerative_amf` family, `start()`, `is_enumerative()`, `make_adjectival()`, `assert()`, unit tests (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, `PropertyInferences::new()`, `PropertyInferences::draw()`, `PropertyInferences::draw_negated()`, `PropertyInferences::draw_from_metadata()`, unit tests (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/same_property_relation.rs` — `SameAsRelations` module, `SameAsRelations::start()`, `SameAsRelations::stock()`, `SameAsRelations::typecheck()`, unit tests (PLAN-26, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, `ProvisionRelation::stock()`, `ProvisionRelation::typecheck()`, `ProvisionRelation::assert()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/relation_subjects.rs` — `RelationSubjects` module, `RelationSubjects::family()`, `RelationSubjects::from_bp()`, `RelationSubjects::new()`, `RelationSubjects::to_bp()`, unit tests (PLAN-24, Complete).
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` module, `KindSubjects::family()`, `KindSubjects::from_kind()`, `KindSubjects::to_kind()`, unit tests (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod measurement_adjectives;` from PLAN-32).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`ValueProperties` module** — a new module `value_properties` in the knowledge crate with:
   - `ValueProperties::kind(prn_idx, properties)` — returns the kind of a valued property:
     - If the property is either-or or has no value data, returns None
     - Returns the `property_value_kind` from the value data
   - `ValueProperties::set_kind(prn_idx, kind_name, properties)` — sets the kind of a valued property:
     - If the property is either-or or has no value data, panics (internal error)
     - Sets the `property_value_kind` field
     - Simplified: no kind validation or problem messages
   - `ValueProperties::make_coincide_with_kind(prn_idx, kind_name, properties, instances)` — makes a property coincide with a kind:
     - If the property is either-or or has no value data, panics (internal error)
     - Calls `set_kind` to set the property's kind
     - Sets `name_coincides_with_kind` to true
     - Simplified: no `P_grammatical_gender` special case
     - Simplified: no `Properties::can_name_coincide_with_kind` check (always calls `Instances::make_kind_coincident`)
   - `ValueProperties::coincides_with_kind(prn_idx, properties)` — checks if a property name coincides with a kind name:
     - If the property is either-or or has no value data, panics (internal error)
     - Returns the `name_coincides_with_kind` field
   - `ValueProperties::make_setting_bp(prn_idx, property_name, bp_registry, properties)` — creates the setting binary predicate for a property:
     - If the property is either-or or has no value data, panics (internal error)
     - Calls `SettingPropertyRelations::find_set_property_BP` to find an existing BP
     - If not found, calls `SettingPropertyRelations::make_set_property_BP` to create one
     - Calls `SettingPropertyRelations::fix_property_bp` on the BP and its reversal
     - Stores the BP index in `value_data.setting_bp`
   - `ValueProperties::get_setting_bp(prn_idx, properties)` — gets the setting binary predicate for a property:
     - If the property is either-or or has no value data, panics (internal error)
     - Returns the `setting_bp` field
   - `ValueProperties::set_stored_relation(prn_idx, bp_idx, properties)` — sets the stored relation for a property:
     - If the property is either-or or has no value data, panics (internal error)
     - Sets the `relation_whose_state_this_stores` field
   - `ValueProperties::get_stored_relation(prn_idx, properties)` — gets the stored relation for a property:
     - If the property is either-or or has no value data, panics (internal error)
     - Returns the `relation_whose_state_this_stores` field
   - `ValueProperties::obtain(name, properties)` — find or create a valued property:
     - Wraps `Properties::obtain(name, true, properties)`
   - `ValueProperties::obtain_within_kind(name, kind_name, properties)` — find or create a valued property with a specific kind:
     - If a property with the name already exists and has value data, checks kind compatibility (simplified: just sets the kind if not already set)
     - If no property exists, creates one and sets its kind
     - Returns the property index
   - `ValueProperties::can_name_coincide_with_kind(kind_name)` — checks if a kind name can coincide with a property name:
     - Simplified: returns true for all kinds (the C reference delegates to `K->construct->can_coincide_with_property` which is a kind system detail)
   - `ValueProperties::assert(prn_idx, owner_idx, val, certainty)` — asserts a property value:
     - Deferred: depends on `Propositions::Abstract::to_set_property` and `Assert::true_about` from the assertions module
     - Returns false (no-op)

2. **Integration with the knowledge module** — add the `value_properties` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — test `ValueProperties::kind` (returns kind for valued property, returns None for either-or property), test `ValueProperties::set_kind` (sets the kind, panics for either-or), test `ValueProperties::make_coincide_with_kind` (sets kind and name_coincides_with_kind, calls make_kind_coincident), test `ValueProperties::coincides_with_kind` (returns true after make_coincide_with_kind, false otherwise, panics for either-or), test `ValueProperties::make_setting_bp` (creates BP and stores in value_data), test `ValueProperties::get_setting_bp` (returns stored BP, panics for either-or), test `ValueProperties::set_stored_relation` and `get_stored_relation` (round-trip), test `ValueProperties::obtain` (creates valued property, returns existing), test `ValueProperties::obtain_within_kind` (creates property with kind, returns existing with compatible kind), test `ValueProperties::can_name_coincide_with_kind` (returns true for all kinds).

## Tasks

### 1. Create the `ValueProperties` module

- [ ] Create `crates/conform7-semantics/src/knowledge/value_properties.rs` with:

  ```rust
  /// The Valued Properties system — management functions for valued property data.
  ///
  /// Corresponds to `ValueProperties` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`).
  ///
  /// Valued properties store values of a specific kind. Each valued property
  /// has an associated setting relation (a binary predicate) that sets its value.
  ///
  /// Simplified:
  /// - No Preform grammar for property name parsing
  /// - No kind validation or problem messages
  /// - No P_grammatical_gender special case
  /// - No Properties::can_name_coincide_with_kind check (always calls make_kind_coincident)
  /// - No Propositions::Abstract::to_set_property (assert deferred)
  /// - No Assert::true_about (assert deferred)
  /// - No RTProperties::dont_show_in_index (new_nameless deferred)
  /// - No nameless property creation (new_nameless deferred)
  use crate::knowledge::properties::{Property, ValuePropertyData};
  use crate::knowledge::setting_property_relation::SettingPropertyRelations;
  use crate::knowledge::instances::Instances;
  use crate::calculus::binary_predicates::BinaryPredicate;
  ```

- [ ] Define the `ValueProperties` struct:

  ```rust
  /// The valued properties module.
  ///
  /// Corresponds to `ValueProperties` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`).
  pub struct ValueProperties;
  ```

- [ ] Implement `ValueProperties::kind`:

  ```rust
  /// Return the kind of a valued property.
  ///
  /// Corresponds to `ValueProperties::kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 139-142).
  ///
  /// Returns None for either-or properties or properties without value data.
  pub fn kind(prn_idx: usize, properties: &[Property]) -> Option<&'static str> {
      let prn = &properties[prn_idx];
      if prn.either_or_data.is_some() {
          return None;
      }
      prn.value_data.as_ref()?.property_value_kind
  }
  ```

- [ ] Implement `ValueProperties::set_kind`:

  ```rust
  /// Set the kind of a valued property.
  ///
  /// Corresponds to `ValueProperties::set_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 144-173).
  ///
  /// Simplified:
  /// - No kind validation (definite check, problem messages)
  /// - No RTProperties::can_be_compiled check
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn set_kind(prn_idx: usize, kind_name: &'static str, properties: &mut [Property]) {
      let prn = &mut properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      let vd = prn.value_data.as_mut().expect("non-value property");
      vd.property_value_kind = Some(kind_name);
  }
  ```

- [ ] Implement `ValueProperties::make_coincide_with_kind`:

  ```rust
  /// Make a property name coincide with a kind name.
  ///
  /// Corresponds to `ValueProperties::make_coincide_with_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 182-189).
  ///
  /// Simplified:
  /// - No P_grammatical_gender special case
  /// - No Properties::can_name_coincide_with_kind check (always calls make_kind_coincident)
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn make_coincide_with_kind(
      prn_idx: usize,
      kind_name: &'static str,
      properties: &mut [Property],
      instances: &mut [crate::knowledge::instances::Instance],
  ) {
      Self::set_kind(prn_idx, kind_name, properties);
      let prn = &mut properties[prn_idx];
      let vd = prn.value_data.as_mut().expect("non-value property");
      vd.name_coincides_with_kind = true;
      // Simplified: always calls make_kind_coincident (no can_name_coincide_with_kind check)
      Instances::make_kind_coincident(kind_name, prn_idx, instances);
  }
  ```

- [ ] Implement `ValueProperties::coincides_with_kind`:

  ```rust
  /// Check if a property name coincides with a kind name.
  ///
  /// Corresponds to `ValueProperties::coincides_with_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 191-194).
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn coincides_with_kind(prn_idx: usize, properties: &[Property]) -> bool {
      let prn = &properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      prn.value_data.as_ref().expect("non-value property").name_coincides_with_kind
  }
  ```

- [ ] Implement `ValueProperties::make_setting_bp`:

  ```rust
  /// Create the setting binary predicate for a valued property.
  ///
  /// Corresponds to `ValueProperties::make_setting_bp` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 121-128).
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn make_setting_bp(
      prn_idx: usize,
      property_name: &str,
      bp_registry: &mut Vec<BinaryPredicate>,
      properties: &mut [Property],
  ) {
      let prn = &properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      assert!(prn.value_data.is_some(), "non-value property");

      // Find or create the setting BP.
      let bp_idx = match SettingPropertyRelations::find_set_property_BP(property_name, bp_registry) {
          Some(idx) => idx,
          None => SettingPropertyRelations::make_set_property_BP(property_name, bp_registry),
      };

      // Fix the BP and its reversal.
      SettingPropertyRelations::fix_property_bp(bp_idx, bp_registry, properties);
      if let Some(rev_idx) = bp_registry[bp_idx].reversal {
          SettingPropertyRelations::fix_property_bp(rev_idx, bp_registry, properties);
      }

      // Store the BP index in the property's value data.
      let prn = &mut properties[prn_idx];
      let vd = prn.value_data.as_mut().expect("non-value property");
      vd.setting_bp = Some(bp_idx);
  }
  ```

- [ ] Implement `ValueProperties::get_setting_bp`:

  ```rust
  /// Get the setting binary predicate for a valued property.
  ///
  /// Corresponds to `ValueProperties::get_setting_bp` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 130-133).
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn get_setting_bp(prn_idx: usize, properties: &[Property]) -> Option<usize> {
      let prn = &properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      prn.value_data.as_ref().expect("non-value property").setting_bp
  }
  ```

- [ ] Implement `ValueProperties::set_stored_relation` and `get_stored_relation`:

  ```rust
  /// Set the stored relation for a valued property.
  ///
  /// Corresponds to `ValueProperties::set_stored_relation` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 200-203).
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn set_stored_relation(prn_idx: usize, bp_idx: usize, properties: &mut [Property]) {
      let prn = &mut properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      let vd = prn.value_data.as_mut().expect("non-value property");
      vd.relation_whose_state_this_stores = Some(bp_idx);
  }

  /// Get the stored relation for a valued property.
  ///
  /// Corresponds to `ValueProperties::get_stored_relation` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 205-208).
  ///
  /// Panics if the property is either-or or has no value data.
  pub fn get_stored_relation(prn_idx: usize, properties: &[Property]) -> Option<usize> {
      let prn = &properties[prn_idx];
      assert!(prn.either_or_data.is_none(), "non-value property");
      prn.value_data.as_ref().expect("non-value property").relation_whose_state_this_stores
  }
  ```

- [ ] Implement `ValueProperties::obtain`:

  ```rust
  /// Find or create a valued property by name.
  ///
  /// Corresponds to `ValueProperties::obtain` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 35-37).
  ///
  /// Wraps `Properties::obtain` with `valued = true`.
  pub fn obtain(name: &'static str, properties: &mut Vec<Property>) -> usize {
      crate::knowledge::properties::Properties::obtain(name, true, properties)
  }
  ```

- [ ] Implement `ValueProperties::obtain_within_kind`:

  ```rust
  /// Find or create a valued property with a specific kind.
  ///
  /// Corresponds to `ValueProperties::obtain_within_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 44-66).
  ///
  /// Simplified:
  /// - No Preform grammar for property name parsing
  /// - No kind compatibility checking (SOMETIMES_MATCH, NEVER_MATCH)
  /// - No problem messages for incompatible kinds
  /// - No Kinds::weaken
  ///
  /// If a property with the given name already exists and has value data,
  /// sets its kind if not already set. If no property exists, creates one
  /// and sets its kind.
  pub fn obtain_within_kind(
      name: &'static str,
      kind_name: &'static str,
      properties: &mut Vec<Property>,
  ) -> usize {
      // Check if a property with this name already exists.
      if let Some(idx) = properties.iter().position(|p| p.name == name) {
          // Property exists — set the kind if not already set.
          if let Some(ref mut vd) = properties[idx].value_data {
              if vd.property_value_kind.is_none() {
                  vd.property_value_kind = Some(kind_name);
              }
          }
          return idx;
      }
      // Create a new valued property with the given kind.
      let idx = Properties::obtain(name, true, properties);
      if let Some(ref mut vd) = properties[idx].value_data {
          vd.property_value_kind = Some(kind_name);
      }
      idx
  }
  ```

- [ ] Implement `ValueProperties::can_name_coincide_with_kind`:

  ```rust
  /// Check if a kind name can coincide with a property name.
  ///
  /// Corresponds to `Properties::can_name_coincide_with_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 548-551).
  ///
  /// Simplified: returns true for all kinds (the C reference delegates to
  /// `K->construct->can_coincide_with_property` which is a kind system detail).
  pub fn can_name_coincide_with_kind(_kind_name: &str) -> bool {
      true
  }
  ```

- [ ] Implement `ValueProperties::assert` (deferred, no-op):

  ```rust
  /// Assert a property value for a subject.
  ///
  /// Corresponds to `ValueProperties::assert` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 213-217).
  ///
  /// Deferred: depends on `Propositions::Abstract::to_set_property` and
  /// `Assert::true_about` from the assertions module.
  ///
  /// Returns false (no-op).
  pub fn assert(
      _prn_idx: usize,
      _owner_idx: usize,
      _val: &str,
      _certainty: i32,
  ) -> bool {
      false
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod value_properties;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Write unit tests

- [ ] Test `ValueProperties::kind`:
  - Returns the kind for a valued property with a kind set
  - Returns None for a valued property without a kind
  - Returns None for an either-or property
- [ ] Test `ValueProperties::set_kind`:
  - Sets the kind on a valued property
  - Panics for an either-or property
- [ ] Test `ValueProperties::make_coincide_with_kind`:
  - Sets the kind and name_coincides_with_kind on a valued property
  - Calls make_kind_coincident on the instances
  - Panics for an either-or property
- [ ] Test `ValueProperties::coincides_with_kind`:
  - Returns true after make_coincide_with_kind
  - Returns false by default
  - Panics for an either-or property
- [ ] Test `ValueProperties::make_setting_bp`:
  - Creates a setting BP and stores it in the property's value data
  - Reuses an existing setting BP if one exists
- [ ] Test `ValueProperties::get_setting_bp`:
  - Returns the stored setting BP
  - Returns None if no setting BP has been created
  - Panics for an either-or property
- [ ] Test `ValueProperties::set_stored_relation` and `get_stored_relation`:
  - Round-trip: set and get a stored relation
  - Panics for an either-or property
- [ ] Test `ValueProperties::obtain`:
  - Creates a new valued property
  - Returns an existing valued property
- [ ] Test `ValueProperties::obtain_within_kind`:
  - Creates a new valued property with the given kind
  - Returns an existing valued property and sets its kind if not already set
- [ ] Test `ValueProperties::can_name_coincide_with_kind`:
  - Returns true for any kind name

## Success Criteria

- [ ] `crates/conform7-semantics/src/knowledge/value_properties.rs` exists with all the functions listed above
- [ ] `crates/conform7-semantics/src/knowledge/mod.rs` includes `pub mod value_properties;`
- [ ] All unit tests pass: `cargo test -p conform7-semantics --lib knowledge::value_properties`
- [ ] The existing test suite still passes: `cargo test -p conform7-semantics`
- [ ] `ValueProperties::kind` returns the correct kind for valued properties and None for either-or properties
- [ ] `ValueProperties::set_kind` correctly sets the kind on a valued property
- [ ] `ValueProperties::make_coincide_with_kind` correctly sets kind and name_coincides_with_kind, and calls make_kind_coincident
- [ ] `ValueProperties::coincides_with_kind` correctly reports whether a property coincides with a kind
- [ ] `ValueProperties::make_setting_bp` correctly creates or finds a setting BP and stores it in the property's value data
- [ ] `ValueProperties::get_setting_bp` correctly retrieves the stored setting BP
- [ ] `ValueProperties::set_stored_relation` and `get_stored_relation` correctly round-trip a stored relation
- [ ] `ValueProperties::obtain` correctly creates or finds a valued property
- [ ] `ValueProperties::obtain_within_kind` correctly creates or finds a valued property with a specific kind
- [ ] `ValueProperties::can_name_coincide_with_kind` returns true for any kind name
- [ ] `ValueProperties::assert` is a no-op (returns false)

## Out of Scope

- **`ValueProperties::new_nameless`** — creating nameless properties (depends on `RTProperties::dont_show_in_index`, `Hierarchy::completion_package`, and run-time compilation). Deferred.
- **`ValueProperties::new_nameless_using`** — creating nameless properties with a specific iname (same dependencies). Deferred.
- **`ValueProperties::assert`** — asserting a property value (depends on `Propositions::Abstract::to_set_property` and `Assert::true_about` from the assertions module). Deferred.
- **Kind validation in `set_kind`** — the C reference validates that the kind is definite and that the property can be compiled. Simplified: no validation.
- **Kind compatibility checking in `obtain_within_kind`** — the C reference checks `Kinds::compatible` with `SOMETIMES_MATCH` and `NEVER_MATCH` outcomes. Simplified: just sets the kind if not already set.
- **`P_grammatical_gender` special case in `make_coincide_with_kind`** — the C reference sets `P_grammatical_gender = prn` when the kind is `K_grammatical_gender`. Simplified: no special case.
- **`Properties::can_name_coincide_with_kind` check in `make_coincide_with_kind`** — the C reference checks if the kind's construct allows coincidence. Simplified: always calls `Instances::make_kind_coincident`.
- **Problem messages** — the C reference issues problem messages for incompatible kinds, non-arithmetic KOVs, and other errors. Simplified: no problem messages.
- **`ConditionsOfSubjects`** — the module that uses `ValueProperties::make_coincide_with_kind` for parsing condition sentences. Deferred to a later plan.
- **`ComparativeRelations`** — the module that uses `ValueProperties::coincides_with_kind` for typechecking. Deferred (depends on Grading/linguistics).
- **`NonlocalVariables`** — the module that uses `ValueProperties` for variable initial values. Deferred.
- **`VariableSubjects`** — the inference subject family for variables. Deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
