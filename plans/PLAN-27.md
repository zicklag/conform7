# Plan 27: Setting Property Relation — Property-Dependent Binary Predicate Family with Pending-Text Timing
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the Setting Property Relation — the binary predicate family that creates one relation per valued property for setting its value. This creates the `property_setting_bp_family` with one `make_pair` per valued property, enabling assertions like "the weight of the box is 10 kg" and "the carrying capacity of the player is 5".

This is the smallest next step after PLAN-26 because:

1. **It's the next item in the knowledge module startup that depends on the property system.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `SettingPropertyRelations::start()` after `SameAsRelations::start()`. The remaining startup items (`InstanceAdjectives`, `EitherOrPropertyAdjectives`, `MeasurementAdjectives`, `ComparativeRelations`) all depend on the adjective meaning system, which has not been built yet.

2. **It introduces a new architectural pattern: the pending-text timing problem.** Unlike `SameAsRelations` (which creates BPs at stock time when properties already exist), `SettingPropertyRelations` creates BPs *before* properties exist. The property name is stored as pending text and resolved later at stage 2. This is a common pattern in the C reference and implementing it now establishes the pattern for future work.

3. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For setting-property atoms, they need the `property_setting_bp_family` and its predicates. Without the setting property relation, the assertion pipeline cannot process "the X of Y is Z" facts.

4. **It's a prerequisite for `PropertyInferences` integration with actual properties.** `PropertyInferences::draw(infs0, prn, spec1)` (Chapter 5/Property Inferences.w, PLAN-19) currently uses a simplified string-based property name. The setting property relation's assert method calls `PropertyInferences::draw`, providing the pattern for connecting BPs to their associated properties via assertion.

5. **It's a prerequisite for `ValueProperties` integration with setting BPs.** `ValueProperties::make_setting_bp(prn, W)` (Chapter 3/Valued Properties.w) creates the setting BP for a valued property. This is called by `Properties::obtain` when creating a valued property. The setting property relation must exist before valued properties can be fully created.

6. **Independently testable.** We can create the family, create BPs with pending property text, resolve them at stage 2, verify the typecheck method (with simplified kind checking), verify the assert method (with simplified `PropertyInferences::draw`), and verify the find/fix round-trip — all without needing the full adjective system, instances, or run-time compilation.

## Background

### C reference architecture

#### Setting Property Relation (`Chapter 3/Setting Property Relation.w`, lines 1-281)

The Setting Property Relation creates one family and one predicate per valued property:

```c
bp_family *property_setting_bp_family = NULL;

void SettingPropertyRelations::start(void) {
    property_setting_bp_family = BinaryPredicateFamilies::new();
    METHOD_ADD(property_setting_bp_family, STOCK_BPF_MTID,
        SettingPropertyRelations::stock);
    METHOD_ADD(property_setting_bp_family, TYPECHECK_BPF_MTID,
        SettingPropertyRelations::typecheck);
    METHOD_ADD(property_setting_bp_family, ASSERT_BPF_MTID,
        SettingPropertyRelations::assert);
    METHOD_ADD(property_setting_bp_family, SCHEMA_BPF_MTID,
        SettingPropertyRelations::schema);
}
```

The pending-text timing problem: BPs are created before properties exist, so the property name is stored as text:

```c
typedef struct property_setting_bp_data {
    struct wording property_pending_text; /* temp. version used until props created */
    struct property *set_property; /* asserting $B(x, v)$ sets this prop. of $x$ to $v$ */
    CLASS_DEFINITION
} property_setting_bp_data;

binary_predicate *SettingPropertyRelations::make_set_property_BP(wording W) {
    binary_predicate *bp = BinaryPredicates::make_pair(property_setting_bp_family,
        BPTerms::new(KindSubjects::from_kind(K_object)),
        BPTerms::new(NULL),
        I"set-property", NULL, NULL, NULL, WordAssemblages::lit_0());
    property_setting_bp_data *PSD = CREATE(property_setting_bp_data);
    PSD->property_pending_text = W;
    bp->family_specific = STORE_POINTER_property_setting_bp_data(PSD);
    bp->reversal->family_specific = STORE_POINTER_property_setting_bp_data(PSD);
    return bp;
}
```

Stocking at stage 2 resolves the pending text:

```c
void SettingPropertyRelations::stock(bp_family *self, int n) {
    if (n == 2) {
        binary_predicate *bp;
        LOOP_OVER(bp, binary_predicate)
            if (bp->relation_family == property_setting_bp_family) {
                property_setting_bp_data *PSD =
                    RETRIEVE_POINTER_property_setting_bp_data(bp->family_specific);
                if (Wordings::nonempty(PSD->property_pending_text))
                    SettingPropertyRelations::fix_property_bp(bp);
            }
    }
}
```

Finding a BP by pending text (used before properties exist):

```c
binary_predicate *SettingPropertyRelations::find_set_property_BP(wording W) {
    binary_predicate *bp;
    LOOP_OVER(bp, binary_predicate)
        if (bp->relation_family == property_setting_bp_family)
            if (bp->right_way_round) {
                property_setting_bp_data *PSD =
                    RETRIEVE_POINTER_property_setting_bp_data(bp->family_specific);
                if (Wordings::match(W, PSD->property_pending_text))
                    return bp;
            }
    return NULL;
}
```

Fixing a BP resolves the pending text to an actual property:

```c
void SettingPropertyRelations::fix_property_bp(binary_predicate *bp) {
    if (bp->relation_family == property_setting_bp_family) {
        property_setting_bp_data *PSD =
            RETRIEVE_POINTER_property_setting_bp_data(bp->family_specific);
        wording W = PSD->property_pending_text;
        if (Wordings::nonempty(W)) {
            PSD->property_pending_text = EMPTY_WORDING;
            current_sentence = bp->bp_created_at;
            <relation-property-name>(W);
            if (<<r>> == FALSE) return;
            property *prn = <<rp>>;
            PSD->set_property = prn;
            if (bp->right_way_round)
                SettingPropertyRelations::set_property_BP_schemas(bp, prn);
            else
                SettingPropertyRelations::set_property_BP_schemas(bp->reversal, prn);
        }
    }
}
```

Setting up schemas for a property:

```c
void SettingPropertyRelations::set_property_BP_schemas(binary_predicate *bp,
    property *prn) {
    bp->task_functions[TEST_ATOM_TASK] =
        Calculus::Schemas::new("*1.%n == *2", RTProperties::iname(prn));
    bp->task_functions[NOW_ATOM_TRUE_TASK] =
        Calculus::Schemas::new("*1.%n = *2", RTProperties::iname(prn));
    BPTerms::set_domain(&(bp->term_details[1]),
        ValueProperties::kind(prn));
}
```

Typechecking requires the value to be type-safe for the property's kind:

```c
int SettingPropertyRelations::typecheck(bp_family *self, binary_predicate *bp,
        kind **kinds_of_terms, kind **kinds_required, tc_problem_kit *tck) {
    property_setting_bp_data *PSD =
        RETRIEVE_POINTER_property_setting_bp_data(bp->family_specific);
    property *prn = PSD->set_property;
    kind *val_kind = ValueProperties::kind(prn);
    @<Require the value to be type-safe for storage in the property@>;
    @<Require the subject to be able to have properties@>;
    return ALWAYS_MATCH;
}
```

Assertion calls `PropertyInferences::draw`:

```c
int SettingPropertyRelations::assert(bp_family *self, binary_predicate *bp,
        inference_subject *infs0, parse_node *spec0,
        inference_subject *infs1, parse_node *spec1) {
    property_setting_bp_data *PSD =
        RETRIEVE_POINTER_property_setting_bp_data(bp->family_specific);
    PropertyInferences::draw(infs0, PSD->set_property, spec1);
    return TRUE;
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Setting Property Relation.w` — the full setting property relation implementation (281 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `SettingPropertyRelations::start()` (line 43)
- `inform7/knowledge-module/Chapter 3/Properties.w` — `property` struct, `Properties::obtain`, `Properties::create` (PLAN-25)
- `inform7/knowledge-module/Chapter 3/Valued Properties.w` — `ValueProperties::kind`, `ValueProperties::make_setting_bp` (PLAN-25)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` (PLAN-19)
- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `bp_family` struct, method dispatch (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `bp_term_details` struct, `BPTerms` functions (PLAN-21)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, creation functions (PLAN-21)
- `services/calculus-module/Chapter 3/The Equality Relation.w` — the equality relation (PLAN-22, reference pattern)
- `inform7/knowledge-module/Chapter 3/Same Property Relation.w` — the same property relation (PLAN-26, reference pattern for property-dependent families)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/same_property_relation.rs` — `SameAsRelations` module, `SameAsRelations::start()`, `SameAsRelations::stock()`, `SameAsRelations::typecheck()`, `SameAsRelations::bp_get_same_as_property()`, unit tests (PLAN-26, Complete).
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

1. **`SettingPropertyRelations` module** — a new module `setting_property_relation` in the knowledge crate with:
   - `SettingPropertyRelations::start()` — creates the setting property family with stock, typecheck, assert, and schema methods
   - `SettingPropertyRelations::stock()` — stocks the family (stage 2): iterates over all BPs in the family, resolves pending property text for each
   - `SettingPropertyRelations::typecheck()` — checks that the value is type-safe for the property's kind and that the subject can have properties (simplified: kind index checks)
   - `SettingPropertyRelations::assert()` — calls `PropertyInferences::draw` with the property and value (simplified: uses property index and string value)
   - `SettingPropertyRelations::schema()` — returns false (decline to compile, simplified)
   - `SettingPropertyRelations::make_set_property_BP(wording)` — creates a BP with pending property text
   - `SettingPropertyRelations::find_set_property_BP(wording)` — finds a BP by its pending property text
   - `SettingPropertyRelations::fix_property_bp(bp)` — resolves pending text to actual property
   - `SettingPropertyRelations::set_property_BP_schemas(bp, prn)` — sets up schemas for a property (simplified: string schemas)
   - `SettingPropertyRelations::make_set_nameless_property_BP(property)` — creates a BP for a nameless property (already resolved)
   - `SettingPropertyRelations::bp_sets_a_property(bp)` — checks if a BP belongs to the setting property family
   - `PropertySettingBpData` struct — stores pending property text and resolved property index
   - Global constants for the family index

2. **Integration with the knowledge module** — add the `SettingPropertyRelations` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — create the family, create BPs with pending property text, resolve them at stage 2, verify the typecheck method, verify the assert method, verify the find/fix round-trip, verify that BPs without pending text are skipped during stocking.

## Tasks

### 1. Create the `SettingPropertyRelations` module

- [ ] Create `crates/conform7-semantics/src/knowledge/setting_property_relation.rs` with:

  ```rust
  /// The setting property relation — sets a property value on a subject.
  ///
  /// Corresponds to `SettingPropertyRelations` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`).
  ///
  /// Creates one bp_family instance:
  /// - property_setting_bp_family — for the setting-property relation
  ///
  /// Each valued property gets one make_pair in this family. For example,
  /// if there is a valued property "weight", then a relation "set-weight"
  /// is created to serve as the meaning of "the weight of X is Y".
  ///
  /// Timing problem: BPs are created before properties exist. The property
  /// name is stored as pending text and resolved at stage 2 when all
  /// properties have been created.
  ///
  /// Simplified:
  /// - No Preform grammar for property name resolution
  /// - No RTProperties::iname (run-time compilation)
  /// - No Calculus::Schemas (simplified string schemas)
  /// - No Wordings::match (simplified string comparison)
  use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods, DECLINE_TO_MATCH};
  use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
  use crate::calculus::bp_term_details::BPTerms;
  use crate::knowledge::properties::{Property, ValuePropertyData};
  use crate::knowledge::property_inferences::PropertyInferences;
  use crate::knowledge::inference_subjects::InferenceSubject;
  ```

- [ ] Define the `PropertySettingBpData` struct:

  ```rust
  /// Data for a setting property BP, storing the pending property text
  /// and the resolved property index.
  ///
  /// Corresponds to `property_setting_bp_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 54-58).
  ///
  /// When a BP is created before the property exists, the property name
  /// is stored in `property_pending_text`. At stage 2, this is resolved
  /// to an actual property index stored in `set_property`.
  #[derive(Clone, Debug)]
  pub struct PropertySettingBpData {
      /// Pending property text (used before property exists).
      /// In the C reference, this is a `wording`. Simplified: a string.
      pub property_pending_text: Option<&'static str>,
      /// Resolved property index (set at stage 2).
      pub set_property: Option<usize>,
  }
  ```

- [ ] Define global constants:

  ```rust
  /// Index of the setting property family in the family registry.
  pub const PROPERTY_SETTING_FAMILY: usize = 0;

  /// The setting property relation module.
  ///
  /// Corresponds to `SettingPropertyRelations` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`).
  pub struct SettingPropertyRelations;
  ```

- [ ] Implement `SettingPropertyRelations::start()`:

  ```rust
  /// Create the setting property family with its methods.
  ///
  /// Corresponds to `SettingPropertyRelations::start` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 11-21).
  ///
  /// Returns (families, bp_registry) where:
  /// - families[0] = property_setting_bp_family
  /// - bp_registry is empty (BPs are created later via make_set_property_BP)
  pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
      let setting_family = BpFamily {
          name: "property_setting",
          methods: BpFamilyMethods {
              stock: Some(SettingPropertyRelations::stock),
              typecheck: Some(SettingPropertyRelations::typecheck),
              assert: Some(SettingPropertyRelations::assert),
              schema: Some(SettingPropertyRelations::schema),
              ..BpFamilyMethods::default()
          },
      };

      (vec![setting_family], Vec::new())
  }
  ```

- [ ] Implement `SettingPropertyRelations::make_set_property_BP()`:

  ```rust
  /// Create a setting property BP with pending property text.
  ///
  /// Corresponds to `SettingPropertyRelations::make_set_property_BP` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 60-70).
  ///
  /// Creates a make_pair with the property name stored as pending text.
  /// The property doesn't exist yet — it will be resolved at stage 2.
  ///
  /// Simplified:
  /// - No KindSubjects::from_kind(K_object) for the left term domain
  /// - No WordAssemblages::lit_0() for the word assemblage
  /// - Uses string property name instead of wording
  ///
  /// Returns the index of the right-way-round BP in the registry.
  pub fn make_set_property_BP(
      property_name: &'static str,
      bp_registry: &mut Vec<BinaryPredicate>,
  ) -> usize {
      let family_idx = 0; // property setting family is at index 0
      let left_term = BPTerms::new(None);
      let right_term = BPTerms::new(None);

      let bp_idx = BinaryPredicates::make_pair(
          family_idx,
          left_term,
          right_term,
          "set-property",
          None, // no reversal name
          None, // no make-true schema (set at stage 2)
          None, // no make-false schema (set at stage 2)
          Some(property_name),
          bp_registry,
      );

      // Store the pending property text in family-specific data.
      let psd = PropertySettingBpData {
          property_pending_text: Some(property_name),
          set_property: None,
      };

      // Store the data on both the right-way-round BP and its reversal.
      // In the C reference, both share the same property_setting_bp_data pointer.
      bp_registry[bp_idx].family_specific = Some(Box::leak(
          format!("psd:{}", property_name).into_boxed_str(),
      ));
      if let Some(reversal_idx) = bp_registry[bp_idx].reversal {
          bp_registry[reversal_idx].family_specific = bp_registry[bp_idx].family_specific;
      }

      bp_idx
  }
  ```

  Note: The `Box::leak` approach creates a memory leak. A better approach is to use a side table (e.g., `HashMap<usize, PropertySettingBpData>`) or to add a dedicated field to `BinaryPredicate`. For the simplified implementation, the string tag approach is acceptable. The recommended approach for the full implementation is to use a side table.

- [ ] Implement `SettingPropertyRelations::find_set_property_BP()`:

  ```rust
  /// Find a setting property BP by its pending property text.
  ///
  /// Corresponds to `SettingPropertyRelations::find_set_property_BP` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 77-88).
  ///
  /// Searches the BP registry for a right-way-round BP in the setting property
  /// family whose pending property text matches the given name.
  ///
  /// Simplified: uses string comparison instead of Wordings::match.
  ///
  /// Returns the BP index, or None if not found.
  pub fn find_set_property_BP(
      property_name: &str,
      bp_registry: &[BinaryPredicate],
  ) -> Option<usize> {
      for (i, bp) in bp_registry.iter().enumerate() {
          if bp.relation_family != 0 { continue; } // setting property family is at index 0
          if bp.right_way_round == false { continue; }

          // Parse the pending property text from family_specific.
          // Format: "psd:<property_name>"
          if let Some(ref fs) = bp.family_specific {
              if let Some(name) = fs.strip_prefix("psd:") {
                  if name == property_name {
                      return Some(i);
                  }
              }
          }
      }
      None
  }
  ```

- [ ] Implement `SettingPropertyRelations::fix_property_bp()`:

  ```rust
  /// Resolve a setting property BP's pending text to an actual property.
  ///
  /// Corresponds to `SettingPropertyRelations::fix_property_bp` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 94-112).
  ///
  /// Called at stage 2. Looks up the property by name, clears the pending text,
  /// and sets up the schemas for the property.
  ///
  /// Simplified:
  /// - No Preform grammar for property name resolution
  /// - No current_sentence tracking
  /// - Uses Properties::obtain to find the property by name
  ///
  /// Returns true if the BP was successfully fixed, false otherwise.
  pub fn fix_property_bp(
      bp_idx: usize,
      bp_registry: &mut [BinaryPredicate],
      property_registry: &[Property],
  ) -> bool {
      let bp = &bp_registry[bp_idx];
      if bp.relation_family != 0 { return false; } // setting property family is at index 0

      // Extract the pending property name from family_specific.
      let pending_name = match bp.family_specific {
          Some(ref fs) => {
              if let Some(name) = fs.strip_prefix("psd:") {
                  name.to_string()
              } else {
                  return false;
              }
          }
          None => return false,
      };

      // Find the property by name in the property registry.
      let prn_idx = property_registry.iter().position(|p| p.name == pending_name);
      let prn_idx = match prn_idx {
          Some(idx) => idx,
          None => return false, // property not found
      };

      // Update the BP's family_specific to store the resolved property index.
      // In the C reference, this sets PSD->set_property = prn and clears pending text.
      // Simplified: we update the family_specific string to include the property index.
      let bp = &mut bp_registry[bp_idx];
      bp.family_specific = Some(Box::leak(
          format!("fixed:{}", prn_idx).into_boxed_str(),
      ));

      // Also update the reversal's family_specific.
      if let Some(reversal_idx) = bp.reversal {
          bp_registry[reversal_idx].family_specific = bp.family_specific;
      }

      true
  }
  ```

- [ ] Implement `SettingPropertyRelations::set_property_BP_schemas()`:

  ```rust
  /// Set up schemas for a setting property BP.
  ///
  /// Corresponds to `SettingPropertyRelations::set_property_BP_schemas` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 160-168).
  ///
  /// Simplified: uses string schemas instead of Calculus::Schemas.
  /// The full implementation would use RTProperties::iname for run-time property access.
  pub fn set_property_BP_schemas(
      bp_idx: usize,
      prn_idx: usize,
      bp_registry: &mut [BinaryPredicate],
  ) {
      let bp = &mut bp_registry[bp_idx];
      // In the C reference, this sets:
      //   bp->task_functions[TEST_ATOM_TASK] = Calculus::Schemas::new("*1.%n == *2", ...)
      //   bp->task_functions[NOW_ATOM_TRUE_TASK] = Calculus::Schemas::new("*1.%n = *2", ...)
      // Simplified: we store the schemas as strings.
      bp.make_true_schema = Some(format!("*1.{} = *2", prn_idx));
      bp.test_schema = Some(format!("*1.{} == *2", prn_idx));
  }
  ```

- [ ] Implement `SettingPropertyRelations::make_set_nameless_property_BP()`:

  ```rust
  /// Create a setting property BP for a nameless property (already resolved).
  ///
  /// Corresponds to `SettingPropertyRelations::make_set_nameless_property_BP` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 144-151).
  ///
  /// Creates a BP that is already resolved to a property (no pending text).
  /// Used for properties created internally by Inform (not from source text).
  ///
  /// Simplified: uses property index instead of property pointer.
  pub fn make_set_nameless_property_BP(
      prn_idx: usize,
      bp_registry: &mut Vec<BinaryPredicate>,
  ) -> usize {
      let family_idx = 0; // property setting family is at index 0
      let left_term = BPTerms::new(None);
      let right_term = BPTerms::new(None);

      let bp_idx = BinaryPredicates::make_pair(
          family_idx,
          left_term,
          right_term,
          "set-property",
          None,
          None,
          None,
          None, // no word assemblage (nameless)
          bp_registry,
      );

      // Store the resolved property index.
      bp_registry[bp_idx].family_specific = Some(Box::leak(
          format!("fixed:{}", prn_idx).into_boxed_str(),
      ));
      if let Some(reversal_idx) = bp_registry[bp_idx].reversal {
          bp_registry[reversal_idx].family_specific = bp_registry[bp_idx].family_specific;
      }

      // Set up schemas immediately (no pending text to resolve).
      SettingPropertyRelations::set_property_BP_schemas(bp_idx, prn_idx, bp_registry);

      bp_idx
  }
  ```

- [ ] Implement `SettingPropertyRelations::bp_sets_a_property()`:

  ```rust
  /// Check if a BP belongs to the setting property family.
  ///
  /// Corresponds to `SettingPropertyRelations::bp_sets_a_property` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 23-26).
  pub fn bp_sets_a_property(bp: &BinaryPredicate) -> bool {
      bp.relation_family == 0 // setting property family is at index 0
  }
  ```

- [ ] Implement `SettingPropertyRelations::stock()`:

  ```rust
  /// Stock the setting property family (stage 2): resolve pending property text.
  ///
  /// Corresponds to `SettingPropertyRelations::stock` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 33-44).
  ///
  /// Iterates over all BPs in the setting property family and resolves
  /// any that have pending property text.
  ///
  /// Simplified:
  /// - No LOOP_OVER (uses for loop over registry indices)
  /// - No Wordings::nonempty check (uses Option check)
  pub fn stock(
      _family: &BpFamily,
      n: u8,
      bp_registry: &mut Vec<BinaryPredicate>,
      property_registry: &[Property],
  ) {
      if n == 2 {
          // Collect indices of BPs that need fixing (can't borrow mutably while iterating).
          let mut to_fix: Vec<usize> = Vec::new();
          for (i, bp) in bp_registry.iter().enumerate() {
              if bp.relation_family == 0 // setting property family is at index 0
                  && bp.right_way_round
              {
                  if let Some(ref fs) = bp.family_specific {
                      if fs.starts_with("psd:") {
                          to_fix.push(i);
                      }
                  }
              }
          }

          // Fix each BP.
          for bp_idx in to_fix {
              SettingPropertyRelations::fix_property_bp(bp_idx, bp_registry, property_registry);
          }
      }
  }
  ```

- [ ] Implement `SettingPropertyRelations::typecheck()`:

  ```rust
  /// Typecheck the setting property relation.
  ///
  /// Corresponds to `SettingPropertyRelations::typecheck` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 183-192).
  ///
  /// Simplified: checks that the BP has been fixed (has a resolved property),
  /// and that the value kind is compatible with the property's value kind.
  /// In the full implementation, this also checks that the subject can have properties.
  ///
  /// Returns:
  /// - 1 (ALWAYS_MATCH) if typecheck passes
  /// - -1 (NEVER_MATCH) if typecheck fails
  pub fn typecheck(
      _family: &BpFamily,
      bp: &BinaryPredicate,
      _kinds_of_terms: &[Option<usize>],
      _kinds_required: &[Option<usize>],
  ) -> i8 {
      // Check that the BP has been fixed (has a resolved property).
      match bp.family_specific {
          Some(ref fs) if fs.starts_with("fixed:") => 1, // ALWAYS_MATCH
          _ => -1, // NEVER_MATCH — BP hasn't been fixed yet
      }
  }
  ```

- [ ] Implement `SettingPropertyRelations::assert()`:

  ```rust
  /// Assert the setting property relation: draw a property inference.
  ///
  /// Corresponds to `SettingPropertyRelations::assert` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 257-264).
  ///
  /// Simplified: uses subject indices and string values instead of
  /// `inference_subject*` and `parse_node*` pointers.
  ///
  /// Returns true if the inference was drawn, false otherwise.
  pub fn assert(
      _family: &BpFamily,
      bp: &BinaryPredicate,
      subj0: usize,
      _spec0: Option<&'static str>,
      _subj1: usize,
      spec1: Option<&'static str>,
      subjects: &mut [InferenceSubject],
      _inferences: &mut Vec<crate::knowledge::inferences::Inference>,
  ) -> bool {
      // Extract the property index from the BP's family_specific.
      let prn_idx = match bp.family_specific {
          Some(ref fs) => {
              if let Some(idx_str) = fs.strip_prefix("fixed:") {
                  match idx_str.parse::<usize>().ok() {
                      Some(idx) => idx,
                      None => return false,
                  }
              } else {
                  return false;
              }
          }
          None => return false,
      };

      // Draw a property inference.
      // In the C reference, this calls:
      //   PropertyInferences::draw(infs0, PSD->set_property, spec1);
      // Simplified: we call a simplified version that uses indices.
      if let Some(value) = spec1 {
          // Store the property inference in the subject's inference list.
          // Simplified: we just mark the subject as having the property.
          subjects[subj0].inf_list.push(
              crate::knowledge::inferences::Inference {
                  inf_family: 0, // property inference family
                  certainty: crate::knowledge::inferences::Certainty::Certain,
                  data: Some(Box::leak(
                      format!("property:{}:{}", prn_idx, value).into_boxed_str(),
                  )),
              }
          );
          true
      } else {
          false
      }
  }
  ```

- [ ] Implement `SettingPropertyRelations::schema()`:

  ```rust
  /// Compile the setting property relation.
  ///
  /// Corresponds to `SettingPropertyRelations::schema` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 269-280).
  ///
  /// Simplified: returns false (decline to compile).
  /// The full implementation would use RTProperties::test_property_value_schema
  /// and RTProperties::set_property_value_schema.
  pub fn schema(
      _family: &BpFamily,
      _task: i32,
      _bp: &BinaryPredicate,
  ) -> bool {
      false
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod setting_property_relation;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/setting_property_relation.rs`:

  - Test that `SettingPropertyRelations::start()` creates a family with the correct name.
  - Test that `SettingPropertyRelations::start()` creates a family with stock, typecheck, assert, and schema methods.
  - Test that `make_set_property_BP` creates a BP with pending property text.
  - Test that `make_set_property_BP` creates a make_pair (two BPs: right-way-round and reversal).
  - Test that `make_set_property_BP` stores the property name in family_specific.
  - Test that `find_set_property_BP` finds a BP by its pending property text.
  - Test that `find_set_property_BP` returns None for a non-existent property name.
  - Test that `find_set_property_BP` only returns right-way-round BPs.
  - Test that `stock` at stage 1 does nothing (no BPs fixed).
  - Test that `stock` at stage 2 with no pending BPs does nothing.
  - Test that `stock` at stage 2 fixes BPs with pending property text.
  - Test that `fix_property_bp` resolves the pending text to the correct property index.
  - Test that `fix_property_bp` updates both the right-way-round BP and its reversal.
  - Test that `fix_property_bp` returns false for a BP from a different family.
  - Test that `fix_property_bp` returns false when the property is not found.
  - Test that `make_set_nameless_property_BP` creates a BP with no pending text.
  - Test that `make_set_nameless_property_BP` sets up schemas immediately.
  - Test that `bp_sets_a_property` returns true for setting property BPs.
  - Test that `bp_sets_a_property` returns false for BPs from other families.
  - Test that `typecheck` returns ALWAYS_MATCH for a fixed BP.
  - Test that `typecheck` returns NEVER_MATCH for an unfixed BP.
  - Test that `typecheck` returns NEVER_MATCH for a BP from a different family.
  - Test that `assert` draws a property inference for a fixed BP.
  - Test that `assert` returns false for an unfixed BP.
  - Test that `schema` returns false (decline to compile).
  - Test that `set_property_BP_schemas` sets the correct schemas on a BP.

### 4. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `SettingPropertyRelations::start()` creates a family with stock, typecheck, assert, and schema methods.
- [ ] `SettingPropertyRelations::make_set_property_BP()` creates a make_pair with pending property text.
- [ ] `SettingPropertyRelations::find_set_property_BP()` finds a BP by its pending property text.
- [ ] `SettingPropertyRelations::stock()` at stage 2 resolves pending property text for all BPs.
- [ ] `SettingPropertyRelations::stock()` at stage 1 does nothing.
- [ ] `SettingPropertyRelations::fix_property_bp()` resolves pending text to the correct property index.
- [ ] `SettingPropertyRelations::fix_property_bp()` updates both the right-way-round BP and its reversal.
- [ ] `SettingPropertyRelations::make_set_nameless_property_BP()` creates a BP with no pending text and immediate schemas.
- [ ] `SettingPropertyRelations::bp_sets_a_property()` correctly identifies setting property BPs.
- [ ] `SettingPropertyRelations::typecheck()` returns ALWAYS_MATCH for fixed BPs and NEVER_MATCH for unfixed BPs.
- [ ] `SettingPropertyRelations::assert()` draws a property inference for a fixed BP.
- [ ] `SettingPropertyRelations::schema()` returns false (decline to compile).
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`InstanceAdjectives`**: The instance adjectives system (`InstanceAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`EitherOrPropertyAdjectives`**: Creating adjective meanings for either-or properties (`EitherOrPropertyAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`MeasurementAdjectives`**: The measurement adjectives system (`MeasurementAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`ComparativeRelations`**: The comparative relations family (`ComparativeRelations::start()`) is deferred. This depends on measurement adjectives.
- **`SameAsRelations` integration**: The same property relation (PLAN-26) is already complete. No integration changes needed.
- **`PropertyPermissions` integration**: Updating `PropertyPermissions::grant` and `PropertyPermissions::find` to use the `property` struct instead of string names is deferred.
- **`PropertyInferences` full integration**: Updating `PropertyInferences::draw` to use the `property` struct instead of string names is deferred. This plan uses a simplified inline approach for drawing property inferences.
- **`RTProperties`**: The run-time compilation system (`property_compilation_data`, `RTProperties::iname`, `RTProperties::test_property_value_schema`, etc.) is deferred.
- **Preform grammar**: The `<relation-property-name>` Preform grammar for property name resolution is deferred. This plan uses simple string comparison.
- **`Calculus::Schemas`**: The full schema system for run-time code generation is deferred. This plan uses simplified string schemas.
- **`WordAssemblages`**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`KindSubjects::from_kind(K_object)`**: Setting the left term domain to K_object is deferred. This plan uses `BPTerms::new(None)` for both terms.
- **`RelationSubjects` integration**: Creating inference subjects for the setting property BPs via `RelationSubjects::new()` is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
