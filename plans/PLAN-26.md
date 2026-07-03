# Plan 26: Same Property Relation — First Property-Dependent Binary Predicate Family
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the Same Property Relation — the first concrete binary predicate family that depends on the property system. This creates the `same_property_bp_family` with one `make_pair` binary predicate per valued property, enabling comparisons like "the same height as" and "the same carrying capacity as".

This is the smallest next step after PLAN-25 because:

1. **It's the next item in the knowledge module startup that depends on the property system.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `SameAsRelations::start()` after `MeasurementAdjectives::start()`. But `MeasurementAdjectives` depends on the adjective meaning system (not yet built), while `SameAsRelations` depends only on the property system (PLAN-25, Complete). This makes it the next independently implementable item.

2. **It's simpler than `SettingPropertyRelations`.** The other property-dependent startup item (`SettingPropertyRelations::start()`) has a complex pending-text timing problem, a multi-stage typecheck, and assert/schema methods. `SameAsRelations` has:
   - A single family with only stock and typecheck methods
   - Stocking at stage 2 that iterates over all valued properties
   - A typecheck that simply returns `DECLINE_TO_MATCH`
   - No assert, schema, or describe methods needed for the simplified version

3. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For same-property-value-as atoms, they need the `same_property_bp_family` and its predicates. Without the same property relation, the assertion pipeline cannot process "the same X as" facts.

4. **It's a prerequisite for `RelationSubjects` integration with property-based BPs.** `RelationSubjects::new(bp)` (Chapter 4/Relation Subjects.w, PLAN-24) creates inference subjects for binary predicates. The same property relation predicates need inference subjects to participate in the model world. This plan creates the predicates; `RelationSubjects` integration is already in place.

5. **It's a prerequisite for `PropertyInferences` integration with actual properties.** `PropertyInferences::draw(infs0, prn, spec1)` (Chapter 5/Property Inferences.w, PLAN-19) currently uses a simplified string-based property name. The same property relation stores the property index in family-specific data, providing a pattern for connecting BPs to their associated properties.

6. **Independently testable.** We can create the family, stock it at stage 2 with a registry of valued properties, verify that one `make_pair` is created per valued property, verify the reversal relationships, verify the typecheck method returns `DECLINE_TO_MATCH`, and verify the family-specific data stores the correct property index — all without needing the full adjective system, instances, or run-time compilation.

## Background

### C reference architecture

#### Same Property Relation (`Chapter 3/Same Property Relation.w`, lines 1-121)

The Same Property Relation creates one family and one predicate per valued property:

```c
bp_family *same_property_bp_family = NULL;

void SameAsRelations::start(void) {
    same_property_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(same_property_bp_family, STOCK_BPF_MTID, SameAsRelations::stock);
    METHOD_ADD(same_property_bp_family, TYPECHECK_BPF_MTID, SameAsRelations::typecheck);
}
```

Stocking at stage 2 iterates over all valued properties:

```c
void SameAsRelations::stock(bp_family *self, int n) {
    if (n == 2) {
        property *prn;
        LOOP_OVER(prn, property) {
            if ((Properties::is_value_property(prn)) &&
                (Wordings::nonempty(prn->name))) {
                vocabulary_entry *rel_name;
                inter_name *i6_pname = RTProperties::iname(prn);
                @<Work out the name for the same-property-value-as relation@>;

                TEMPORARY_TEXT(relname)
                WRITE_TO(relname, "%V", rel_name);
                binary_predicate *bp =
                    BinaryPredicates::make_pair(same_property_bp_family,
                        BPTerms::new(NULL), BPTerms::new(NULL),
                        relname, NULL,
                        Calculus::Schemas::new("*1.%n = *2.%n", i6_pname, i6_pname),
                        Calculus::Schemas::new("*1.%n == *2.%n", i6_pname, i6_pname),
                        WordAssemblages::lit_1(rel_name));
                DISCARD_TEXT(relname)
                bp->family_specific = STORE_POINTER_property(prn);
                SameAsRelations::register_same_property_as(bp,
                    Properties::get_name(prn));
            }
        }
    }
}
```

The family-specific data stores a pointer to the property:

```c
property *SameAsRelations::bp_get_same_as_property(binary_predicate *bp) {
    if (bp->relation_family != same_property_bp_family) return NULL;
    if (bp->right_way_round == FALSE) return NULL;
    return RETRIEVE_POINTER_property(bp->family_specific);
}
```

Typechecking simply declines to match (letting the standard machinery handle it):

```c
int SameAsRelations::typecheck(bp_family *self, binary_predicate *bp,
        kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    return DECLINE_TO_MATCH;
}
```

The relation name is derived from the property name, with spaces replaced by hyphens:

```c
@<Work out the name for the same-property-value-as relation@> =
    TEMPORARY_TEXT(i7_name)
    WRITE_TO(i7_name, "same-%<W-as", prn->name);
    LOOP_THROUGH_TEXT(pos, i7_name)
        if (Str::get(pos) == ' ') Str::put(pos, '-');
    wording I7W = Feeds::feed_text_expanding_strings(i7_name);
    rel_name = Lexer::word(Wordings::first_wn(I7W));
    DISCARD_TEXT(i7_name)
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Same Property Relation.w` — the full same property relation implementation (121 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `SameAsRelations::start()` (line 42)
- `inform7/knowledge-module/Chapter 3/Properties.w` — `property` struct, `Properties::is_value_property`, `Properties::get_name` (PLAN-25)
- `inform7/knowledge-module/Chapter 3/Valued Properties.w` — `ValueProperties::kind` (PLAN-25)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `bp_family` struct, method dispatch (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `bp_term_details` struct, `BPTerms` functions (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, creation functions (PLAN-21)
- `services/calculus-module/Chapter 3/The Equality Relation.w` — the equality relation (PLAN-22, reference pattern)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData`, `ValuePropertyData`, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties`, `ValueProperties`, unit tests (PLAN-25, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, `ProvisionRelation::stock()`, `ProvisionRelation::typecheck()`, `ProvisionRelation::assert()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/relation_subjects.rs` — `RelationSubjects` module, `RelationSubjects::family()`, `RelationSubjects::from_bp()`, `RelationSubjects::new()`, `RelationSubjects::to_bp()`, unit tests (PLAN-24, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, unit tests (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` module, `KindSubjects::family()`, `KindSubjects::from_kind()`, `KindSubjects::to_kind()`, unit tests (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`SameAsRelations` module** — a new module `same_property_relation` in the knowledge crate with:
   - `SameAsRelations::start()` — creates the same property family with stock and typecheck methods
   - `SameAsRelations::stock()` — stocks the family (stage 2): iterates over all valued properties, creates one `make_pair` per property
   - `SameAsRelations::typecheck()` — returns `DECLINE_TO_MATCH` (let standard machinery handle it)
   - `SameAsRelations::bp_get_same_as_property(bp_idx)` — retrieves the property index from a BP's family-specific data
   - Helper function to derive the relation name from the property name (e.g., "height" → "same-height-as")
   - Global constants for the family index

2. **Integration with the knowledge module** — add the `SameAsRelations` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — create the family, stock it with a property registry containing valued properties, verify one `make_pair` is created per valued property, verify reversal relationships, verify the typecheck method returns `DECLINE_TO_MATCH`, verify `bp_get_same_as_property` returns the correct property index, verify that either-or properties are skipped during stocking.

## Tasks

### 1. Create the `SameAsRelations` module

- [ ] Create `crates/conform7-semantics/src/knowledge/same_property_relation.rs` with:

  ```rust
  /// The same property relation — compares a property value between two owners.
  ///
  /// Corresponds to `SameAsRelations` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`).
  ///
  /// Creates one bp_family instance:
  /// - same_property_bp_family — for the same-property-value-as relation
  ///
  /// Each valued property gets one make_pair in this family. For example,
  /// if there is a valued property "height", then a relation "same-height-as"
  /// is created to serve as the meaning of "the same height as".
  ///
  /// Simplified:
  /// - No Preform grammar for relation name construction
  /// - No preposition registration (SameAsRelations::register_same_property_as)
  /// - No RTProperties::iname (run-time compilation)
  /// - No Calculus::Schemas (simplified string schemas)
  use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods, DECLINE_TO_MATCH};
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  use crate::knowledge::properties::{Property, ValuePropertyData};
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the same property family in the family registry.
  pub const SAME_PROPERTY_FAMILY: usize = 0;

  /// The same property relation module.
  ///
  /// Corresponds to `SameAsRelations` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`).
  pub struct SameAsRelations;
  ```

- [ ] Implement `SameAsRelations::start()`:

  ```rust
  /// Create the same property family with its methods.
  ///
  /// Corresponds to `SameAsRelations::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 11-15).
  ///
  /// Returns (families, bp_registry) where:
  /// - families[0] = same_property_bp_family
  /// - bp_registry is empty (stocking fills it)
  pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
      let same_property_family = BpFamily {
          name: "same_property",
          methods: BpFamilyMethods {
              stock: Some(SameAsRelations::stock),
              typecheck: Some(SameAsRelations::typecheck),
              ..BpFamilyMethods::default()
          },
      };

      (vec![same_property_family], Vec::new())
  }
  ```

- [ ] Implement the relation name helper:

  ```rust
  /// Derive the same-property-value-as relation name from a property name.
  ///
  /// Corresponds to the `<same-property-as-construction>` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 74-76).
  ///
  /// For a property named "height", returns "same-height-as".
  /// For a property named "carrying capacity", returns "same-carrying-capacity-as".
  /// For a property named "point of view", returns "same-point-of-view-as".
  fn derive_relation_name(property_name: &str) -> String {
      format!("same-{}-as", property_name.replace(' ', "-"))
  }
  ```

- [ ] Implement `SameAsRelations::stock()`:

  ```rust
  /// Stock the same property family (stage 2): create one make_pair per valued property.
  ///
  /// Corresponds to `SameAsRelations::stock` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 33-59).
  ///
  /// Simplified:
  /// - No Wordings::nonempty check (all properties have names in our simplified model)
  /// - No RTProperties::iname (run-time compilation deferred)
  /// - No Calculus::Schemas (simplified string schemas)
  /// - No SameAsRelations::register_same_property_as (preposition registration deferred)
  ///
  /// Only valued properties (those with `value_data.is_some()`) get a same-property
  /// relation. Either-or properties are skipped.
  pub fn stock(
      _family: &BpFamily,
      n: u8,
      bp_registry: &mut Vec<BinaryPredicate>,
      property_registry: &[Property],
  ) {
      if n == 2 {
          let family_idx = 0; // same property family is at index 0

          for (prn_idx, prn) in property_registry.iter().enumerate() {
              // Only valued properties get a same-property relation.
              if prn.value_data.is_none() {
                  continue;
              }

              // Derive the relation name from the property name.
              let rel_name = derive_relation_name(prn.name);

              // Create a make_pair for this property.
              // In the C reference, the schemas use RTProperties::iname(prn) for
              // run-time property access. Simplified: we use string schemas with
              // the property name as a placeholder.
              let left_term = BPTerms::new(None);
              let right_term = BPTerms::new(None);

              let bp_idx = BinaryPredicates::make_pair(
                  family_idx,
                  left_term,
                  right_term,
                  &rel_name,
                  None, // no reversal name
                  Some(&format!("*1.{} = *2.{}", prn.name, prn.name)), // make-true schema
                  Some(&format!("*1.{} == *2.{}", prn.name, prn.name)), // test schema
                  Some(&rel_name),
                  bp_registry,
              );

              // Store the property index in family-specific data.
              // In the C reference, this is STORE_POINTER_property(prn).
              // Simplified: we store the property index as a string.
              bp_registry[bp_idx].family_specific = Some(Box::leak(
                  format!("property:{}", prn_idx).into_boxed_str(),
              ));
          }
      }
  }
  ```

  Note: The `Box::leak` approach creates a memory leak. A better approach is to store the property index in a side table (e.g., `HashMap<usize, usize>` mapping BP index to property index), or to add a dedicated field to `BinaryPredicate`. For the simplified implementation, the string tag approach is acceptable. The recommended approach for the full implementation is to use a side table.

- [ ] Implement `SameAsRelations::typecheck()`:

  ```rust
  /// Typecheck the same property relation.
  ///
  /// Corresponds to `SameAsRelations::typecheck` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 117-120).
  ///
  /// Returns DECLINE_TO_MATCH, letting the standard machinery handle typechecking.
  pub fn typecheck(
      _family: &BpFamily,
      _bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      DECLINE_TO_MATCH
  }
  ```

- [ ] Implement `SameAsRelations::bp_get_same_as_property()`:

  ```rust
  /// Retrieve the property index from a same-property BP's family-specific data.
  ///
  /// Corresponds to `SameAsRelations::bp_get_same_as_property` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Same Property Relation.w`, lines 64-68).
  ///
  /// Returns None if the BP is not from the same property family, or if it's
  /// a reversal (not right-way-round).
  ///
  /// Simplified: parses the property index from the family_specific string tag.
  pub fn bp_get_same_as_property(
      bp: &BinaryPredicate,
  ) -> Option<usize> {
      if bp.relation_family != 0 { return None; } // same property family is at index 0
      if bp.right_way_round == false { return None; }

      // Parse the property index from the family_specific string.
      // Format: "property:<index>"
      if let Some(ref fs) = bp.family_specific {
          if let Some(idx_str) = fs.strip_prefix("property:") {
              return idx_str.parse::<usize>().ok();
            }
        }
        None
    }
    ```

### 2. Add module declaration

- [ ] Add `pub mod same_property_relation;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/same_property_relation.rs`:

  - Test that `SameAsRelations::start()` creates a family with the correct name.
  - Test that `SameAsRelations::start()` creates a family with stock and typecheck methods.
  - Test that `derive_relation_name` converts "height" to "same-height-as".
  - Test that `derive_relation_name` converts "carrying capacity" to "same-carrying-capacity-as".
  - Test that `derive_relation_name` converts "point of view" to "same-point-of-view-as".
  - Test that `stock` at stage 1 does nothing (no predicates created).
  - Test that `stock` at stage 2 with no properties creates no predicates.
  - Test that `stock` at stage 2 with one valued property creates one make_pair (two BPs: right-way-round and reversal).
  - Test that `stock` at stage 2 with multiple valued properties creates one make_pair per property.
  - Test that `stock` at stage 2 skips either-or properties.
  - Test that `stock` at stage 2 creates BPs with the correct relation names.
  - Test that `typecheck` returns `DECLINE_TO_MATCH`.
  - Test that `bp_get_same_as_property` returns the correct property index for a right-way-round BP.
  - Test that `bp_get_same_as_property` returns None for a reversal BP.
  - Test that `bp_get_same_as_property` returns None for a BP from a different family.
  - Test that the reversal relationship is correctly set up (reversal.reversal == original).

### 4. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `SameAsRelations::start()` creates a family with stock and typecheck methods.
- [ ] `SameAsRelations::stock()` at stage 2 creates one `make_pair` per valued property.
- [ ] `SameAsRelations::stock()` skips either-or properties.
- [ ] `SameAsRelations::stock()` at stage 1 does nothing.
- [ ] `SameAsRelations::typecheck()` returns `DECLINE_TO_MATCH`.
- [ ] `SameAsRelations::bp_get_same_as_property()` returns the correct property index for right-way-round BPs.
- [ ] `SameAsRelations::bp_get_same_as_property()` returns None for reversal BPs.
- [ ] `derive_relation_name()` correctly converts property names to relation names.
- [ ] Reversal relationships are correctly set up for each make_pair.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`SettingPropertyRelations`**: The setting property relation family (`SettingPropertyRelations::start()`) is deferred. This has a more complex pending-text timing problem and will be implemented in a later plan.
- **`EitherOrPropertyAdjectives`**: Creating adjective meanings for either-or properties (`EitherOrPropertyAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`MeasurementAdjectives`**: The measurement adjectives system (`MeasurementAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`ComparativeRelations`**: The comparative relations family (`ComparativeRelations::start()`) is deferred. This depends on measurement adjectives.
- **`InstanceAdjectives`**: The instance adjectives system (`InstanceAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`PropertyPermissions` integration**: Updating `PropertyPermissions::grant` and `PropertyPermissions::find` to use the `property` struct instead of string names is deferred.
- **`PropertyInferences` integration**: Updating `PropertyInferences::draw` to use the `property` struct instead of string names is deferred.
- **`RTProperties`**: The run-time compilation system (`property_compilation_data`, `RTProperties::iname`, `RTProperties::test_property_value_schema`, etc.) is deferred.
- **Preposition registration**: `SameAsRelations::register_same_property_as` (creating prepositional forms like "the same height as") is deferred. This depends on the Preform/Verbs system.
- **`Calculus::Schemas`**: The full schema system for run-time code generation is deferred. This plan uses simplified string schemas.
- **`WordAssemblages`**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`PreformUtilities::wording`**: The full Preform wording system is deferred. This plan uses simplified string names.
- **`RelationSubjects` integration**: Creating inference subjects for the same-property BPs via `RelationSubjects::new()` is deferred. The `RelationSubjects` module (PLAN-24) is already in place, but integration with property-based BPs will happen in a later plan.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
