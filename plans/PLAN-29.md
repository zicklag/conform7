# Plan 29: Either-Or Property Adjectives — First Adjective Meaning Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Either-Or Property Adjectives system — the first concrete adjective meaning family that uses the adjective meaning infrastructure from PLAN-28. This creates the `either_or_property_amf` family with assert, prepare_schemas, and index methods, enabling either-or properties (e.g., "open", "closed", "transparent") to be used as adjectives in the model world.

This is the smallest next step after PLAN-28 because:

1. **It's the next item in the knowledge module startup that has no remaining dependencies.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `EitherOrPropertyAdjectives::start()` at line 40. It depends on the adjective meaning system (PLAN-28, Complete) and the property system (PLAN-25, Complete) — both done. `InstanceAdjectives::start()` (line 39) depends on the `instance` struct which does not yet exist. `MeasurementAdjectives::start()` (line 41) depends on `measurement_definition` and `Measurements` which do not yet exist.

2. **It's the simplest adjective meaning family.** At 74 lines of C, `EitherOrPropertyAdjectives` is the smallest of the three adjective-dependent startup items:
   - `InstanceAdjectives` (97 lines) — requires the `instance` struct (not yet built)
   - `EitherOrPropertyAdjectives` (74 lines) — depends only on properties and adjectives (both done)
   - `MeasurementAdjectives` (197 lines) — requires `measurement_definition`, `Measurements`, and grammar parsing

3. **It's a prerequisite for the assertion pipeline.** `EitherOrProperties::obtain()` (Chapter 3/Either-Or Properties.w, line 67) calls `EitherOrPropertyAdjectives::create_for_property()` to register each either-or property as an adjective. Without this, either-or properties cannot participate in the model world as adjectives, and assertions like "the door is open" cannot be processed.

4. **It's a prerequisite for `EitherOrProperties::assert`.** `EitherOrProperties::assert()` (Chapter 3/Either-Or Properties.w, lines 159-164) uses `EitherOrProperties::as_adjective()` to look up the adjective for a property and create an adjectival proposition. This requires the either-or property adjectives system to be in place.

5. **It's a prerequisite for `PropertyPermissions` integration with adjectives.** When a property permission is granted to a subject, `EitherOrPropertyAdjectives::create_for_property` is called to register the property as an adjective for that subject's kind. This is how "the door is open" becomes a valid assertion.

6. **Independently testable.** We can create the `either_or_property_amf` family, create adjective meanings for either-or properties, test the `create_for_property` function (which declares an adjective, creates a meaning, adds it to the adjective, and sets the domain), test the assert method (which calls `PropertyInferences::draw` or `PropertyInferences::draw_negated`), test the `is` method (which checks if a meaning belongs to this family), and test the `as_adjective` accessor — all without needing instances, measurement adjectives, or run-time compilation.

## Background

### C reference architecture

#### Either-Or Property Adjectives (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 1-74)

The Either-Or Property Adjectives system creates one family and registers either-or properties as adjectives:

```c
adjective_meaning_family *either_or_property_amf = NULL;

void EitherOrPropertyAdjectives::start(void) {
    either_or_property_amf = AdjectiveMeanings::new_family(1);
    METHOD_ADD(either_or_property_amf, ASSERT_ADJM_MTID,
        EitherOrPropertyAdjectives::assert);
    METHOD_ADD(either_or_property_amf, PREPARE_SCHEMAS_ADJM_MTID,
        EitherOrPropertyAdjectives::prepare_schemas);
    METHOD_ADD(either_or_property_amf, INDEX_ADJM_MTID,
        EitherOrPropertyAdjectives::index);
}
```

The `is` method checks if a meaning belongs to this family:

```c
int EitherOrPropertyAdjectives::is(adjective_meaning *am) {
    if ((am) && (am->family == either_or_property_amf)) return TRUE;
    return FALSE;
}
```

The `create_for_property` function registers an either-or property as an adjective for a given kind:

```c
void EitherOrPropertyAdjectives::create_for_property(property *prn, wording W, kind *K) {
    if ((prn == NULL) || (prn->either_or_data == NULL)) internal_error("not either-or");
    adjective *adj = EitherOrProperties::as_adjective(prn);
    if (adj) {
        if (AdjectiveAmbiguity::can_be_applied_to(adj, K)) return;
    } else {
        adj = Adjectives::declare(W, NULL);
        prn->either_or_data->as_adjective = adj;
    }
    adjective_meaning *am =
        AdjectiveMeanings::new(either_or_property_amf, STORE_POINTER_property(prn), W);
    AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    AdjectiveMeaningDomains::set_from_kind(am, K);
}
```

The assert method calls `PropertyInferences::draw` or `PropertyInferences::draw_negated`:

```c
int EitherOrPropertyAdjectives::assert(adjective_meaning_family *f,
    adjective_meaning *am, inference_subject *infs_to_assert_on, int parity) {
    property *prn = RETRIEVE_POINTER_property(am->family_specific_data);
    if (parity == FALSE) PropertyInferences::draw_negated(infs_to_assert_on, prn, NULL);
    else PropertyInferences::draw(infs_to_assert_on, prn, NULL);
    return TRUE;
}
```

The `prepare_schemas` and `index` methods are simplified (no-op in the Rust version):

```c
void EitherOrPropertyAdjectives::prepare_schemas(adjective_meaning_family *family,
    adjective_meaning *am, int T) {
    property *prn = RETRIEVE_POINTER_property(am->family_specific_data);
    if (am->schemas_prepared == FALSE)
        RTProperties::write_either_or_schemas(am, prn, T);
}

int EitherOrPropertyAdjectives::index(adjective_meaning_family *f, text_stream *OUT,
    adjective_meaning *am) {
    property *prn = RETRIEVE_POINTER_property(am->family_specific_data);
    RTInferences::index_either_or(OUT, prn);
    return TRUE;
}
```

#### Either-Or Properties (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 151-154)

The `as_adjective` accessor retrieves the adjective from either-or property data:

```c
adjective *EitherOrProperties::as_adjective(property *prn) {
    if ((prn == NULL) || (prn->either_or_data == NULL)) internal_error("non-EO property");
    return prn->either_or_data->as_adjective;
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w` — the full either-or property adjectives implementation (74 lines)
- `inform7/knowledge-module/Chapter 3/Either-Or Properties.w` — `EitherOrProperties::as_adjective` (line 151), `EitherOrProperties::obtain` (line 67 calls `create_for_property`)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `EitherOrPropertyAdjectives::start()` (line 40)
- `inform7/assertions-module/Chapter 8/Adjective Meanings.w` — `AdjectiveMeanings::new_family`, `AdjectiveMeanings::new` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Ambiguity.w` — `AdjectiveAmbiguity::add_meaning_to_adjective`, `AdjectiveAmbiguity::can_be_applied_to` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w` — `AdjectiveMeaningDomains::set_from_kind` (PLAN-28)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw`, `PropertyInferences::draw_negated` (PLAN-19)
- `services/linguistics-module/Chapter 2/Adjectives.w` — `Adjectives::declare` (PLAN-28)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/adjectives.rs` — `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, `AdjectiveDomainData` struct, `AdjectiveMeaningFamilyMethods` struct, `AdjectiveMeanings` management functions (`new_family`, `new`, `negate`, `assert`), `AdjectiveAmbiguity` management functions (`add_meaning_to_adjective`, `can_be_applied_to`, `first_meaning`), `AdjectiveMeaningDomains` management functions (`new_from_kind`, `set_from_kind`, `get_kind`, `weak_match`), `Adjectives::declare`, `Adjectives::find`, `Adjectives::get_nominative_singular`, unit tests (PLAN-28, Complete).
- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData` struct (with `as_adjective: Option<usize>` field), `ValuePropertyData` struct, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties::new_eo_data`, `EitherOrProperties::make_pair`, `EitherOrProperties::get_negation`, `ValueProperties` functions, unit tests (PLAN-25, Complete).
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
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod adjectives;` from PLAN-28).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`EitherOrPropertyAdjectives` module** — a new module `either_or_property_adjectives` in the knowledge crate with:
   - `EitherOrPropertyAdjectives::start()` — creates the `either_or_property_amf` family with assert, prepare_schemas, and index methods
   - `EitherOrPropertyAdjectives::is(am_idx, meanings)` — checks if a meaning belongs to the either-or property family
   - `EitherOrPropertyAdjectives::create_for_property(prn_idx, name, kind_idx, adjectives, meanings, families, properties)` — registers an either-or property as an adjective for a given kind:
     - Checks if the property already has an adjective via `EitherOrProperties::as_adjective`
     - If it does, checks if the adjective can already be applied to the kind via `AdjectiveAmbiguity::can_be_applied_to` — if so, returns early
     - If it doesn't, declares a new adjective via `Adjectives::declare` and stores it in `prn.either_or_data.as_adjective`
     - Creates a new adjective meaning via `AdjectiveMeanings::new` with the `either_or_property_amf` family and the property index as family-specific data
     - Adds the meaning to the adjective via `AdjectiveAmbiguity::add_meaning_to_adjective`
     - Sets the domain from the kind via `AdjectiveMeaningDomains::set_from_kind`
   - `EitherOrPropertyAdjectives::assert(am_idx, subj_idx, parity, meanings, subjects, families, properties, inference_families, inferences, data_registry)` — asserts the either-or property on a subject:
     - Retrieves the property index from the meaning's family-specific data
     - If parity is true, calls `PropertyInferences::draw` with the property name
     - If parity is false, calls `PropertyInferences::draw_negated` with the property name
     - Returns true (always handled)
   - `EitherOrPropertyAdjectives::prepare_schemas(am_idx, task)` — simplified: no-op (returns false)
   - `EitherOrPropertyAdjectives::index(am_idx)` — simplified: no-op (returns None)
   - Global constant for the family index

2. **`EitherOrProperties::as_adjective()` accessor** — add to `properties.rs`:
   - `EitherOrProperties::as_adjective(prn_idx, properties)` — returns the adjective index from the either-or property data, or None if the property is not either-or

3. **Integration with the knowledge module** — add the `EitherOrPropertyAdjectives` module declaration to the knowledge module's `mod.rs`.

4. **Unit tests** — create the family, create adjective meanings for either-or properties, test the `create_for_property` function (declares adjective, creates meaning, adds to adjective, sets domain), test the `is` method, test the assert method (calls `PropertyInferences::draw` for positive parity, `PropertyInferences::draw_negated` for negative parity), test that `create_for_property` is idempotent (calling it again with the same kind returns early), test that `create_for_property` creates a new meaning for a different kind, test the `as_adjective` accessor.

## Tasks

### 1. Add `EitherOrProperties::as_adjective()` accessor

- [ ] Add to `crates/conform7-semantics/src/knowledge/properties.rs`:

  ```rust
  /// Get the adjective index for an either-or property.
  ///
  /// Corresponds to `EitherOrProperties::as_adjective` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 151-154).
  ///
  /// Returns the adjective index, or None if the property is not either-or
  /// or has no associated adjective.
  pub fn as_adjective(prn_idx: usize, properties: &[Property]) -> Option<usize> {
      properties.get(prn_idx).and_then(|prn| {
          prn.either_or_data.as_ref().and_then(|eod| eod.as_adjective)
      })
  }
  ```

### 2. Create the `EitherOrPropertyAdjectives` module

- [ ] Create `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs` with:

  ```rust
  /// The Either-Or Property Adjectives system — either-or properties used as adjectives.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`).
  ///
  /// Creates one adjective_meaning_family instance:
  /// - either_or_property_amf — for either-or property adjectives
  ///
  /// Each either-or property can be used as an adjective. For example, the
  /// either-or property "open" can be used as the adjective "open" in
  /// assertions like "the door is open".
  ///
  /// Simplified:
  /// - No RTProperties::write_either_or_schemas (run-time compilation deferred)
  /// - No RTInferences::index_either_or (index generation deferred)
  /// - No Preform grammar for property name resolution
  use crate::knowledge::adjectives::{
      Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
      AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
  };
  use crate::knowledge::inference_subjects::InferenceSubject;
  use crate::knowledge::inferences::{Inference, InferenceFamily};
  use crate::knowledge::properties::{EitherOrProperties, Property, PropertyInferenceData};
  use crate::knowledge::property_inferences::PropertyInferences;
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the either-or property family in the family registry.
  pub const EITHER_OR_PROPERTY_FAMILY: usize = 0;

  /// The either-or property adjectives module.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`).
  pub struct EitherOrPropertyAdjectives;
  ```

- [ ] Implement `EitherOrPropertyAdjectives::start()`:

  ```rust
  /// Create the either-or property family with its methods.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 12-20).
  ///
  /// Returns (families, meanings, adjectives) where:
  /// - families[0] = either_or_property_amf
  /// - meanings is empty (create_for_property fills it)
  /// - adjectives is empty (create_for_property fills it)
  pub fn start() -> (Vec<AdjectiveMeaningFamily>, Vec<AdjectiveMeaning>, Vec<Adjective>) {
      let either_or_property_family = AdjectiveMeaningFamily {
          name: "either_or_property",
          definition_claim_priority: 1,
          methods: AdjectiveMeaningFamilyMethods {
              assert: Some(EitherOrPropertyAdjectives::assert),
              prepare_schemas: Some(EitherOrPropertyAdjectives::prepare_schemas),
              index: Some(EitherOrPropertyAdjectives::index),
              ..AdjectiveMeaningFamilyMethods::default()
          },
      };

      (vec![either_or_property_family], Vec::new(), Vec::new())
  }
  ```

- [ ] Implement `EitherOrPropertyAdjectives::is()`:

  ```rust
  /// Check if an adjective meaning belongs to the either-or property family.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::is` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 22-25).
  ///
  /// Returns true if the meaning belongs to this family, false otherwise.
  pub fn is(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
      meanings.get(am_idx).is_some_and(|am| am.family == EITHER_OR_PROPERTY_FAMILY)
  }
  ```

- [ ] Implement `EitherOrPropertyAdjectives::create_for_property()`:

  ```rust
  /// Register an either-or property as an adjective for a given kind.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::create_for_property` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 35-48).
  ///
  /// This function:
  /// 1. Checks if the property already has an adjective
  /// 2. If it does, checks if the adjective can already be applied to the kind
  /// 3. If it doesn't, declares a new adjective and stores it in the property data
  /// 4. Creates a new adjective meaning for this property-kind pair
  /// 5. Adds the meaning to the adjective
  /// 6. Sets the domain from the kind
  ///
  /// Simplified:
  /// - No internal_error for non-either-or properties (returns silently)
  /// - No Preform grammar for name validation
  ///
  /// Returns the adjective index.
  pub fn create_for_property(
      prn_idx: usize,
      name: &'static str,
      kind_idx: usize,
      adjectives: &mut Vec<Adjective>,
      meanings: &mut Vec<AdjectiveMeaning>,
      families: &[AdjectiveMeaningFamily],
      properties: &mut [Property],
  ) -> usize {
      // Check if the property is either-or.
      let eo_data = properties[prn_idx].either_or_data.as_ref();
      if eo_data.is_none() {
          // Not an either-or property — return a dummy adjective index.
          // In the C reference, this is an internal_error.
          // Simplified: create a placeholder adjective.
          return Adjectives::declare(name, adjectives);
      }

      // Check if the property already has an adjective.
      let adj_idx = EitherOrProperties::as_adjective(prn_idx, properties);

      let adj_idx = if let Some(adj) = adj_idx {
          // Property already has an adjective — check if it can be applied to this kind.
          if AdjectiveAmbiguity::can_be_applied_to(adj, Some(kind_idx), adjectives, meanings) {
              return adj; // Already registered for this kind — no-op.
          }
          adj
      } else {
          // No adjective yet — declare one and store it in the property data.
          let adj = Adjectives::declare(name, adjectives);
          if let Some(eod) = &mut properties[prn_idx].either_or_data {
              eod.as_adjective = Some(adj);
          }
          adj
      };

      // Create a new adjective meaning for this property-kind pair.
      // The family-specific data stores the property index as a string.
      let am = AdjectiveMeanings::new(
          EITHER_OR_PROPERTY_FAMILY,
          Some(Box::leak(format!("property:{}", prn_idx).into_boxed_str())),
          Some(name),
          meanings,
      );

      // Add the meaning to the adjective.
      AdjectiveAmbiguity::add_meaning_to_adjective(am, adj_idx, adjectives, meanings);

      // Set the domain from the kind.
      AdjectiveMeaningDomains::set_from_kind(am, kind_idx, meanings);

      adj_idx
  }
  ```

  Note: The `Box::leak` approach creates a memory leak. A better approach is to store the property index in a side table (e.g., `HashMap<usize, usize>` mapping meaning index to property index), or to add a dedicated field to `AdjectiveMeaning`. For the simplified implementation, the string tag approach is acceptable. The recommended approach for the full implementation is to use a side table.

- [ ] Implement `EitherOrPropertyAdjectives::assert()`:

  ```rust
  /// Assert an either-or property adjective on an inference subject.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::assert` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 53-59).
  ///
  /// For positive parity (e.g., "the door is open"), calls
  /// `PropertyInferences::draw` with the property name.
  ///
  /// For negative parity (e.g., "the door is not open"), calls
  /// `PropertyInferences::draw_negated` with the property name.
  ///
  /// Simplified:
  /// - No RETRIEVE_POINTER_property (uses string tag to look up property name)
  /// - Property name is looked up from the property registry
  ///
  /// Returns true (always handled).
  pub fn assert(
      am_idx: usize,
      subj_idx: usize,
      parity: bool,
      meanings: &mut [AdjectiveMeaning],
      subjects: &mut [InferenceSubject],
      families: &[AdjectiveMeaningFamily],
      properties: &[Property],
      inference_families: &[InferenceFamily],
      inferences: &mut Vec<Inference>,
      data_registry: &mut Vec<PropertyInferenceData>,
  ) -> bool {
      // Retrieve the property index from the meaning's family-specific data.
      let am = &meanings[am_idx];
      let prn_idx = am.family_specific_data
          .and_then(|data| data.strip_prefix("property:"))
          .and_then(|s| s.parse::<usize>().ok());

      let prn_idx = match prn_idx {
          Some(idx) => idx,
          None => return false, // No property data — decline.
      };

      // Get the property name.
      let prn_name = properties.get(prn_idx).map(|p| p.name).unwrap_or("");

      if parity {
          PropertyInferences::draw(
              subj_idx, prn_name, None,
              inference_families, inferences, subjects, data_registry,
          );
      } else {
          PropertyInferences::draw_negated(
              subj_idx, prn_name, None,
              inference_families, inferences, subjects, data_registry,
          );
      }

      true
  }
  ```

- [ ] Implement `EitherOrPropertyAdjectives::prepare_schemas()`:

  ```rust
  /// Prepare I6 schemas for an either-or property adjective.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::prepare_schemas` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 61-66).
  ///
  /// Simplified: no-op. The full implementation would call
  /// `RTProperties::write_either_or_schemas` to generate I6 schemas.
  pub fn prepare_schemas(_am_idx: usize, _task: i32) -> bool {
      false // Not implemented — run-time compilation deferred.
  }
  ```

- [ ] Implement `EitherOrPropertyAdjectives::index()`:

  ```rust
  /// Produce index text for an either-or property adjective.
  ///
  /// Corresponds to `EitherOrPropertyAdjectives::index` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w`, lines 68-73).
  ///
  /// Simplified: no-op. The full implementation would call
  /// `RTInferences::index_either_or` to generate index text.
  pub fn index(_am_idx: usize) -> Option<&'static str> {
      None // Not implemented — index generation deferred.
  }
  ```

### 3. Add module declaration

- [ ] Add `pub mod either_or_property_adjectives;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 4. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs`:

  - Test that `start()` creates a family with the correct name and priority.
  - Test that `start()` creates a family with assert, prepare_schemas, and index methods.
  - Test that `is()` returns true for a meaning in the either-or property family.
  - Test that `is()` returns false for a meaning in a different family.
  - Test that `create_for_property` declares a new adjective for a property without one.
  - Test that `create_for_property` stores the adjective index in the property data.
  - Test that `create_for_property` creates a meaning with the correct family.
  - Test that `create_for_property` adds the meaning to the adjective.
  - Test that `create_for_property` sets the domain from the kind.
  - Test that `create_for_property` is idempotent (calling it again with the same kind returns early).
  - Test that `create_for_property` creates a new meaning for a different kind.
  - Test that `create_for_property` handles non-either-or properties gracefully.
  - Test that `assert` with positive parity calls `PropertyInferences::draw`.
  - Test that `assert` with negative parity calls `PropertyInferences::draw_negated`.
  - Test that `assert` returns true (always handled).
  - Test that `prepare_schemas` returns false (no-op).
  - Test that `index` returns None (no-op).
  - Test that `EitherOrProperties::as_adjective` returns the correct adjective index.
  - Test that `EitherOrProperties::as_adjective` returns None for a property without an adjective.
  - Test that `EitherOrProperties::as_adjective` returns None for a valued property.

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `EitherOrPropertyAdjectives::start()` creates a family with priority 1 and assert, prepare_schemas, and index methods.
- [ ] `EitherOrPropertyAdjectives::is()` correctly identifies meanings in the either-or property family.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` declares a new adjective for a property without one.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` stores the adjective index in the property's `either_or_data.as_adjective` field.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` creates a meaning with the correct family and family-specific data.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` adds the meaning to the adjective via `AdjectiveAmbiguity::add_meaning_to_adjective`.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` sets the domain from the kind via `AdjectiveMeaningDomains::set_from_kind`.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` is idempotent for the same property-kind pair.
- [ ] `EitherOrPropertyAdjectives::create_for_property()` creates a new meaning for a different kind.
- [ ] `EitherOrPropertyAdjectives::assert()` with positive parity calls `PropertyInferences::draw`.
- [ ] `EitherOrPropertyAdjectives::assert()` with negative parity calls `PropertyInferences::draw_negated`.
- [ ] `EitherOrPropertyAdjectives::assert()` returns true (always handled).
- [ ] `EitherOrPropertyAdjectives::prepare_schemas()` returns false (no-op).
- [ ] `EitherOrPropertyAdjectives::index()` returns None (no-op).
- [ ] `EitherOrProperties::as_adjective()` returns the correct adjective index for an either-or property.
- [ ] `EitherOrProperties::as_adjective()` returns None for a property without an adjective.
- [ ] `EitherOrProperties::as_adjective()` returns None for a valued property.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`InstanceAdjectives`**: The instance adjectives system (`InstanceAdjectives::start()`, Chapter 2/Instances as Adjectives.w) is deferred. This depends on the `instance` struct which does not yet exist.
- **`MeasurementAdjectives`**: The measurement adjectives system (`MeasurementAdjectives::start()`, Chapter 3/Measurement Adjectives.w) is deferred. This depends on `measurement_definition` and `Measurements` which do not yet exist.
- **`ComparativeRelations`**: The comparative relations family (`ComparativeRelations::start()`, Chapter 3/Comparative Relations.w) is deferred. This depends on measurement adjectives.
- **`EitherOrProperties::obtain` integration**: The full `EitherOrProperties::obtain` function (which calls `EitherOrPropertyAdjectives::create_for_property`) is deferred. This plan implements `create_for_property` as a standalone function; the integration with `EitherOrProperties::obtain` will happen when the property system is fully integrated with the adjective system.
- **`EitherOrProperties::assert`**: The full `EitherOrProperties::assert` function (which creates adjectival propositions) is deferred. This depends on `AdjectivalPredicates` and `Assert::true_about` which are not yet implemented.
- **`EitherOrProperties::new_nameless`**: The nameless property creation function is deferred. This depends on `Hierarchy` and `RTProperties` which are not yet implemented.
- **`RTProperties::write_either_or_schemas`**: The run-time schema generation for either-or properties is deferred. This plan implements `prepare_schemas` as a no-op.
- **`RTInferences::index_either_or`**: The index generation for either-or properties is deferred. This plan implements `index` as a no-op.
- **`AdjectiveMeaningDomains::determine`**: The full domain determination logic (parsing text-based domains, circularity detection, problem messages) is deferred.
- **`AdjectiveAmbiguity::sort`**: The full meaning sorting into precedence order is deferred.
- **`AdjectiveCompilationData`**: The full compilation data for adjectives (`adjective_compilation_data`, `RTAdjectives`) is deferred.
- **`LexicalCluster`**: The full lexical cluster system for inflected adjective forms is deferred.
- **`LinguisticStock`**: The linguistic stock system for grammatical categories is deferred.
- **`Lexicon` registration**: Registering adjectives with the lexicon module for parsing is deferred.
- **`Preform` grammar**: The `<adjective-name>` Preform grammar for parsing adjective names is deferred.
- **`RTProperties`**: The run-time compilation system for properties is deferred.
- **`Calculus::Schemas`**: The full schema system for run-time code generation is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
