# Plan 25: The Property System — Core Data Structures
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the core property system data structures — the `property` struct, either-or property data, valued property data, and basic creation/accessor functions. This is the foundation for all property-related operations in the knowledge module.

This is the smallest next step after PLAN-24 because:

1. **It's the next fundamental piece of the knowledge module.** The knowledge module startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls several items that depend on the property system:
   - `SameAsRelations::start()` — creates a bp_family that stocks at stage 2 by iterating over all properties
   - `SettingPropertyRelations::start()` — creates a bp_family with one BP per valued property
   - `EitherOrPropertyAdjectives::start()` — creates adjective meanings for either-or properties
   - `MeasurementAdjectives::start()` — creates adjective meanings for measurement properties
   - `ComparativeRelations::start()` — creates comparatives for measurement properties

   All of these depend on the `property` struct and its creation/accessor functions. Without the property system, none of these can be implemented.

2. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For property atoms, they need to look up the `property` struct to determine which inference family to use. Without the property system, the assertion pipeline cannot process property facts.

3. **It's a prerequisite for `PropertyInferences` integration with actual properties.** `PropertyInferences::draw(infs0, prn, spec1)` (Chapter 5/Property Inferences.w, PLAN-19) currently uses a simplified string-based property name. With the property system, we can use the actual `property` struct.

4. **It's a prerequisite for `PropertyPermissions` integration with actual properties.** `PropertyPermissions::grant` and `PropertyPermissions::find` (Chapter 4/Property Permissions.w, PLAN-19) currently use string property names. With the property system, they can use the actual `property` struct.

5. **It's a prerequisite for `RelationSubjects` integration with property permissions.** `RelationSubjects::new` (Chapter 4/Relation Subjects.w, PLAN-24) creates inference subjects for binary predicates. The property system is needed to connect property permissions to actual properties.

6. **Independently testable.** We can create the `property` struct, create either-or and valued property data, test the creation functions (`Properties::obtain`, `Properties::create`), test the kind accessors (`Properties::to_kind`, `Properties::kind_of_contents`), test either-or pair management (`EitherOrProperties::make_pair`, `EitherOrProperties::get_negation`), and test valued property kind management (`ValueProperties::set_kind`, `ValueProperties::kind`) — all without needing the full adjective system, instances, or run-time compilation.

## Background

### C reference architecture

#### Properties (`Chapter 3/Properties.w`, lines 1-605)

The Properties module defines the central `property` struct and creation functions:

```c
typedef struct property {
    struct wording name;                    /* name of property */
    int has_of_in_the_name;                 /* looks like a property test, e.g., "point of view"? */
    int Inter_level_only;                   /* i.e., does not correspond to an I7 property */
    struct parse_node *where_created;

    struct linked_list *permissions;        /* of |property_permission|: who can have this? */

    /* exactly one of these must be non-|NULL|: */
    struct either_or_property_data *either_or_data;  /* for an either/or property */
    struct value_property_data *value_data;          /* for a value property */

    struct property_compilation_data compilation_data;

    struct possession_marker pom;            /* for temporary use when checking implications */

    CLASS_DEFINITION
} property;
```

Key creation functions:

```c
property *Properties::obtain(wording W, int valued) {
    parse_node *p = Lexicon::retrieve(PROPERTY_MC, W);
    property *prn;
    if (p == NULL) {
        prn = Properties::create(W, NULL, NULL, (valued)?FALSE:TRUE, NULL);
        if (valued) ValueProperties::make_setting_bp(prn, W);
    } else {
        prn = Rvalues::to_property(p);
        if ((valued) && (prn->either_or_data))
            internal_error("either/or property made into valued");
        if ((valued == FALSE) && (prn->either_or_data == NULL))
            internal_error("valued property made into either/or");
    }
    return prn;
}

property *Properties::create(wording W, package_request *using_package,
    inter_name *using_iname, int eo, text_stream *translation) {
    W = Articles::remove_article(W);
    // ... name validation ...
    property *prn = CREATE(property);
    // ... initialise the property name structure ...
    // ... check for name clashes with kinds ...
    // ... note significance of special properties ...
    if (eo) {
        prn->either_or_data = EitherOrProperties::new_eo_data(prn);
        prn->value_data = NULL;
    } else {
        prn->either_or_data = NULL;
        prn->value_data = ValueProperties::new_value_data(prn);
    }
    // ... register the property name as a noun ...
    return prn;
}
```

Kind accessors:

```c
kind *Properties::to_kind(property *prn) {
    if (prn == NULL) internal_error("took kind of null property");
    return Kinds::unary_con(CON_property, Properties::kind_of_contents(prn));
}

kind *Properties::kind_of_contents(property *prn) {
    if (prn == NULL) internal_error("took kind of null property");
    if (prn->either_or_data) return K_truth_state;
    return prn->value_data->property_value_kind;
}
```

#### Either-Or Properties (`Chapter 3/Either-Or Properties.w`, lines 1-165)

Either-or properties have a negation pair and an optional adjective:

```c
typedef struct either_or_property_data {
    struct property *negation;              /* the other, if it's one of a pair */
    struct adjective *as_adjective;         /* if it is adjectivally used */
    // ...
} either_or_property_data;

either_or_property_data *EitherOrProperties::new_eo_data(property *prn) {
    either_or_property_data *eod = CREATE(either_or_property_data);
    eod->negation = NULL;
    eod->as_adjective = NULL;
    return eod;
}
```

Pair management:

```c
void EitherOrProperties::make_pair(property *prn, property *neg) {
    // ... validation ...
    prn->either_or_data->negation = neg;
    neg->either_or_data->negation = prn;
}

property *EitherOrProperties::get_negation(property *prn) {
    if ((prn == NULL) || (prn->either_or_data == NULL)) return NULL;
    return prn->either_or_data->negation;
}
```

#### Valued Properties (`Chapter 3/Valued Properties.w`, lines 1-218)

Valued properties have a value kind and an optional setting BP:

```c
typedef struct value_property_data {
    struct kind *property_value_kind;       /* what kind of value does it hold? */
    struct binary_predicate *setting_bp;    /* which relation sets it? */
    struct binary_predicate *relation_whose_state_this_stores;
    struct condition_of_subject *as_condition_of_subject;
    int name_coincides_with_kind;
} value_property_data;

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

Kind management:

```c
kind *ValueProperties::kind(property *prn) {
    if ((prn == NULL) || (prn->either_or_data)) return NULL;
    return prn->value_data->property_value_kind;
}

void ValueProperties::set_kind(property *prn, kind *K) {
    // ... validation ...
    prn->value_data->property_value_kind = K;
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 3/Properties.w` — the full property system (605 lines)
- `inform7/knowledge-module/Chapter 3/Either-Or Properties.w` — either-or property data (165 lines)
- `inform7/knowledge-module/Chapter 3/Valued Properties.w` — valued property data (218 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `SameAsRelations::start()` etc. (line 42-45)
- `inform7/knowledge-module/Chapter 4/Property Permissions.w` — `PropertyPermissions::grant`, `PropertyPermissions::find` (PLAN-19)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` uses `property` (PLAN-19)
- `inform7/knowledge-module/Chapter 3/Same Property Relation.w` — `SameAsRelations::stock` iterates over properties (stage 2)
- `inform7/knowledge-module/Chapter 3/Setting Property Relation.w` — `SettingPropertyRelations` creates BPs per property
- `inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w` — `EitherOrPropertyAdjectives::create_for_property`
- `inform7/knowledge-module/Chapter 3/Measurement Adjectives.w` — `MeasurementAdjectives` depends on properties
- `inform7/knowledge-module/Chapter 3/Comparative Relations.w` — `ComparativeRelations` depends on measurement adjectives
- `services/kinds-module/Chapter 2/Kinds.w` — `kind` struct, `Kinds::unary_con` (kind constructors)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct (PLAN-21)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, unit tests (PLAN-19, Complete). Uses string property names.
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods (PLAN-19, Complete). Uses string property names.
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` module, `KindSubjects::family()`, `KindSubjects::from_kind()`, `KindSubjects::to_kind()`, unit tests (Complete).
- `crates/conform7-semantics/src/knowledge/relation_subjects.rs` — `RelationSubjects` module, `RelationSubjects::family()`, `RelationSubjects::from_bp()`, `RelationSubjects::new()`, `RelationSubjects::to_bp()`, unit tests (PLAN-24, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, `ProvisionRelation::stock()`, `ProvisionRelation::typecheck()`, `ProvisionRelation::assert()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct, `Kind::unary_con()`, `Kind::binary_con()`, `Compatibility` enum, `FromStr` parser, unit tests (Complete).
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct, `ConstructorGroup` enum, `Variance` enum, unit tests (Complete).

### What's needed

1. **`Property` struct** — the core data structure with:
   - `name` — property name (simplified: a string instead of `wording`)
   - `has_of_in_the_name` — whether the name looks like a property test
   - `inter_level_only` — whether this is an Inter-level-only property
   - `permissions` — list of property permission indices
   - `either_or_data` — optional either-or property data
   - `value_data` — optional valued property data
   - `compilation_data` — simplified: a string tag (full `property_compilation_data` deferred)
   - `pom` — possession marker for temporary use

2. **`EitherOrPropertyData` struct** — either-or property data with:
   - `negation` — optional property index (the other in a pair)
   - `as_adjective` — optional adjective index (deferred: depends on adjective meaning system)

3. **`ValuePropertyData` struct** — valued property data with:
   - `property_value_kind` — optional kind (the kind of value stored)
   - `setting_bp` — optional binary predicate index (the relation that sets this property)
   - `name_coincides_with_kind` — whether the name matches a kind name
   - `as_condition_of_subject` — optional condition of subject index (deferred)

4. **`Properties` creation functions** — `Properties::obtain`, `Properties::create` (simplified):
   - `Properties::obtain(name, valued)` — find or create a property by name
   - `Properties::create(name, eo)` — create a new property struct
   - Simplified: no Preform grammar, no noun registration, no RTProperties

5. **`Properties` kind accessors** — `Properties::to_kind`, `Properties::kind_of_contents`:
   - `Properties::to_kind(prn)` — returns the kind of a property (e.g., "property of numbers")
   - `Properties::kind_of_contents(prn)` — returns the kind of values stored in a property

6. **`EitherOrProperties` functions** — `new_eo_data`, `make_pair`, `get_negation`:
   - `EitherOrProperties::new_eo_data()` — creates new either-or property data
   - `EitherOrProperties::make_pair(prn, neg)` — joins two properties into a negation pair
   - `EitherOrProperties::get_negation(prn)` — returns the negation of a property

7. **`ValueProperties` functions** — `new_value_data`, `kind`, `set_kind`:
   - `ValueProperties::new_value_data()` — creates new valued property data
   - `ValueProperties::kind(prn)` — returns the value kind of a property
   - `ValueProperties::set_kind(prn, K)` — sets the value kind of a property

8. **Integration with the knowledge module** — add the `properties` module declaration to the knowledge module's `mod.rs`.

9. **Unit tests** — create properties, test either-or vs valued, test kind accessors, test pair management, test obtain/create round-trip.

## Tasks

### 1. Create the `Property` struct and `PropertyData` structs

- [ ] Create `crates/conform7-semantics/src/knowledge/properties.rs` with:

  ```rust
  /// A property — a named attribute that subjects can have.
  ///
  /// Corresponds to `property` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 21-38).
  ///
  /// Properties come in two types: either/or (e.g., "open"/"closed") and
  /// valued (e.g., "carrying capacity"). Each property has a list of
  /// permissions saying which subjects can have it.
  ///
  /// Simplified: uses string names instead of `wording`, and a simplified
  /// compilation data field instead of the full `property_compilation_data`.
  #[derive(Clone, Debug)]
  pub struct Property {
      /// Name of the property (simplified: a string instead of `wording`).
      pub name: &'static str,
      /// Whether the name looks like a property test (e.g., "point of view").
      pub has_of_in_the_name: bool,
      /// Whether this is an Inter-level-only property (no I7 source text existence).
      pub inter_level_only: bool,
      /// List of property permission indices: who can have this property.
      pub permissions: Vec<usize>,
      /// Either-or property data, or None if this is a valued property.
      pub either_or_data: Option<EitherOrPropertyData>,
      /// Valued property data, or None if this is an either-or property.
      pub value_data: Option<ValuePropertyData>,
      /// Compilation data (simplified: a string tag).
      /// Full `property_compilation_data` is deferred.
      pub compilation_data: Option<&'static str>,
      /// Possession marker for temporary use when checking implications.
      pub possession_marker: bool,
  }
  ```

- [ ] Define the `EitherOrPropertyData` struct:

  ```rust
  /// Data for an either-or property.
  ///
  /// Corresponds to `either_or_property_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 29-36).
  ///
  /// Either-or properties may come in pairs: "open" and "closed" are a pair,
  /// where each is the negation of the other. Not all either-or properties
  /// are paired; sometimes an author simply says something "can be P".
  #[derive(Clone, Debug)]
  pub struct EitherOrPropertyData {
      /// The negation property index, if this is one of a pair.
      pub negation: Option<usize>,
      /// The adjective index, if this property is adjectivally used.
      /// Deferred: depends on the adjective meaning system.
      pub as_adjective: Option<usize>,
  }
  ```

- [ ] Define the `ValuePropertyData` struct:

  ```rust
  /// Data for a valued property.
  ///
  /// Corresponds to `value_property_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 10-17).
  ///
  /// Valued properties store values of a specific kind. Each valued property
  /// has an associated setting relation (a binary predicate) that sets its value.
  #[derive(Clone, Debug)]
  pub struct ValuePropertyData {
      /// The kind of value stored in this property (simplified: a kind name string).
      /// Corresponds to `property_value_kind` in the C reference.
      pub property_value_kind: Option<&'static str>,
      /// The setting binary predicate index (the relation that sets this property).
      /// Corresponds to `setting_bp` in the C reference.
      /// Simplified: deferred until SettingPropertyRelations is implemented.
      pub setting_bp: Option<usize>,
      /// Whether the property name coincides with a kind name.
      /// Corresponds to `name_coincides_with_kind` in the C reference.
      pub name_coincides_with_kind: bool,
      /// Condition of subject data, if this property is a condition of a subject.
      /// Deferred: depends on ConditionsOfSubjects.
      pub as_condition_of_subject: Option<usize>,
      /// Binary predicate whose state this property stores (if any).
      /// Deferred: depends on relation storage.
      pub relation_whose_state_this_stores: Option<usize>,
  }
  ```

### 2. Implement `Properties` creation functions

- [ ] Define the `Properties` namespace struct:

  ```rust
  /// Creation and accessor functions for properties.
  ///
  /// Corresponds to `Properties` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`).
  pub struct Properties;
  ```

- [ ] Implement `Properties::create()`:

  ```rust
  /// Create a new property.
  ///
  /// Corresponds to `Properties::create` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 66-82).
  ///
  /// Simplified:
  /// - No Preform grammar for name validation
  /// - No noun registration
  /// - No RTProperties::initialise_pcd
  /// - No special property detection (P_description, P_specification, etc.)
  /// - No PluginCalls::new_property_notify
  ///
  /// `eo` is true for either-or properties, false for valued properties.
  ///
  /// Returns the index of the new property in the registry.
  pub fn create(
      name: &'static str,
      eo: bool,
      registry: &mut Vec<Property>,
  ) -> usize {
      let has_of = name.contains(" of ");
      let prn = Property {
          name,
          has_of_in_the_name: has_of,
          inter_level_only: false,
          permissions: Vec::new(),
          either_or_data: if eo {
              Some(EitherOrPropertyData {
                  negation: None,
                  as_adjective: None,
              })
          } else {
              None
          },
          value_data: if eo {
              None
          } else {
              Some(ValuePropertyData {
                  property_value_kind: None,
                  setting_bp: None,
                  name_coincides_with_kind: false,
                  as_condition_of_subject: None,
                  relation_whose_state_this_stores: None,
              })
          },
          compilation_data: None,
          possession_marker: false,
      };
      let idx = registry.len();
      registry.push(prn);
      idx
  }
  ```

- [ ] Implement `Properties::obtain()`:

  ```rust
  /// Find or create a property by name.
  ///
  /// Corresponds to `Properties::obtain` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 47-61).
  ///
  /// Simplified:
  /// - No Lexicon::retrieve (no Preform grammar)
  /// - No Rvalues::to_property
  /// - No internal_error for type mismatches
  /// - No ValueProperties::make_setting_bp
  ///
  /// If a property with the given name already exists, returns its index.
  /// If `valued` is true, the property must be a valued property (not either-or).
  /// If `valued` is false, the property must be an either-or property.
  /// If no property exists with the given name, creates a new one.
  ///
  /// Returns the index of the property.
  pub fn obtain(
      name: &'static str,
      valued: bool,
      registry: &mut Vec<Property>,
  ) -> usize {
      // Check if a property with this name already exists.
      if let Some(idx) = registry.iter().position(|p| p.name == name) {
          // Property exists — verify type consistency.
          // Simplified: no internal_error for mismatches.
          return idx;
      }
      // Create a new property.
      Properties::create(name, !valued, registry)
  }
  ```

### 3. Implement `Properties` kind accessors

- [ ] Implement `Properties::to_kind()`:

  ```rust
  /// Return the kind of a property.
  ///
  /// Corresponds to `Properties::to_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 238-241).
  ///
  /// For an either-or property, returns "truth-state-valued property".
  /// For a valued property, returns "<kind>-valued property".
  ///
  /// Simplified: returns a string representation instead of a `kind*` pointer.
  pub fn to_kind(prn: &Property) -> String {
      if prn.either_or_data.is_some() {
          "truth-state-valued property".to_string()
      } else if let Some(ref vd) = prn.value_data {
          if let Some(k) = vd.property_value_kind {
              format!("{}-valued property", k)
          } else {
              "value-valued property".to_string()
          }
      } else {
          "property".to_string()
      }
  }
  ```

- [ ] Implement `Properties::kind_of_contents()`:

  ```rust
  /// Return the kind of values stored in a property.
  ///
  /// Corresponds to `Properties::kind_of_contents` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 243-247).
  ///
  /// For an either-or property, returns "truth_state".
  /// For a valued property, returns the property's value kind.
  pub fn kind_of_contents(prn: &Property) -> Option<&'static str> {
      if prn.either_or_data.is_some() {
          Some("truth_state")
      } else if let Some(ref vd) = prn.value_data {
          vd.property_value_kind
      } else {
          None
      }
  }
  ```

### 4. Implement `EitherOrProperties` functions

- [ ] Define the `EitherOrProperties` namespace:

  ```rust
  /// Operations on either-or properties.
  ///
  /// Corresponds to `EitherOrProperties` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`).
  pub struct EitherOrProperties;
  ```

- [ ] Implement `EitherOrProperties::new_eo_data()`:

  ```rust
  /// Create new either-or property data.
  ///
  /// Corresponds to `EitherOrProperties::new_eo_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 38-46).
  pub fn new_eo_data() -> EitherOrPropertyData {
      EitherOrPropertyData {
          negation: None,
          as_adjective: None,
      }
  }
  ```

- [ ] Implement `EitherOrProperties::make_pair()`:

  ```rust
  /// Join two either-or properties into a negation pair.
  ///
  /// Corresponds to `EitherOrProperties::make_pair` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 97-128).
  ///
  /// Simplified: no problem message generation for broken pairs.
  /// Returns true if the pair was successfully created, false otherwise.
  pub fn make_pair(
      prn_idx: usize,
      neg_idx: usize,
      registry: &mut [Property],
  ) -> bool {
      let (prn, neg) = get_two_mut(registry, prn_idx, neg_idx);
      // Both must be either-or properties.
      if prn.either_or_data.is_none() || neg.either_or_data.is_none() {
          return false;
      }
      // Check if either already has a negation.
      if let Some(ref eod) = prn.either_or_data {
          if eod.negation.is_some() && eod.negation != Some(neg_idx) {
              return false; // Already paired with someone else.
          }
      }
      if let Some(ref eod) = neg.either_or_data {
          if eod.negation.is_some() && eod.negation != Some(prn_idx) {
              return false; // Already paired with someone else.
          }
      }
      // Set the negation pointers.
      if let Some(ref mut eod) = prn.either_or_data {
          eod.negation = Some(neg_idx);
      }
      if let Some(ref mut eod) = neg.either_or_data {
          eod.negation = Some(prn_idx);
      }
      true
  }
  ```

  Note: `get_two_mut` is a helper function that safely returns mutable references to two distinct elements of a slice.

- [ ] Implement `EitherOrProperties::get_negation()`:

  ```rust
  /// Get the negation of an either-or property.
  ///
  /// Corresponds to `EitherOrProperties::get_negation` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 131-134).
  pub fn get_negation(prn: &Property) -> Option<usize> {
      prn.either_or_data.as_ref().and_then(|eod| eod.negation)
  }
  ```

### 5. Implement `ValueProperties` functions

- [ ] Define the `ValueProperties` namespace:

  ```rust
  /// Operations on valued properties.
  ///
  /// Corresponds to `ValueProperties` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`).
  pub struct ValueProperties;
  ```

- [ ] Implement `ValueProperties::new_value_data()`:

  ```rust
  /// Create new valued property data.
  ///
  /// Corresponds to `ValueProperties::new_value_data` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 19-27).
  pub fn new_value_data() -> ValuePropertyData {
      ValuePropertyData {
          property_value_kind: None,
          setting_bp: None,
          name_coincides_with_kind: false,
          as_condition_of_subject: None,
          relation_whose_state_this_stores: None,
      }
  }
  ```

- [ ] Implement `ValueProperties::kind()`:

  ```rust
  /// Get the value kind of a valued property.
  ///
  /// Corresponds to `ValueProperties::kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 139-142).
  ///
  /// Returns None for either-or properties.
  pub fn kind(prn: &Property) -> Option<&'static str> {
      prn.value_data.as_ref().and_then(|vd| vd.property_value_kind)
  }
  ```

- [ ] Implement `ValueProperties::set_kind()`:

  ```rust
  /// Set the value kind of a valued property.
  ///
  /// Corresponds to `ValueProperties::set_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 144-173).
  ///
  /// Simplified: no kind validation, no problem message generation.
  /// Only works for valued properties (not either-or).
  pub fn set_kind(prn: &mut Property, kind: &'static str) -> bool {
      if let Some(ref mut vd) = prn.value_data {
          vd.property_value_kind = Some(kind);
          true
      } else {
          false // either-or property
      }
  }
  ```

- [ ] Implement `ValueProperties::make_coincide_with_kind()`:

  ```rust
  /// Mark a valued property as having the same name as a kind.
  ///
  /// Corresponds to `ValueProperties::make_coincide_with_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 182-189).
  ///
  /// Simplified: no Instances::make_kind_coincident call.
  pub fn make_coincide_with_kind(prn: &mut Property, kind: &'static str) -> bool {
      if let Some(ref mut vd) = prn.value_data {
          vd.property_value_kind = Some(kind);
          vd.name_coincides_with_kind = true;
          true
      } else {
          false
      }
  }
  ```

- [ ] Implement `ValueProperties::coincides_with_kind()`:

  ```rust
  /// Check if a valued property has the same name as a kind.
  ///
  /// Corresponds to `ValueProperties::coincides_with_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 191-194).
  pub fn coincides_with_kind(prn: &Property) -> bool {
      prn.value_data.as_ref().map_or(false, |vd| vd.name_coincides_with_kind)
  }
  ```

### 6. Add module declaration

- [ ] Add `pub mod properties;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 7. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/properties.rs`:

  - Test that `Properties::create` creates a property with the correct name.
  - Test that `Properties::create` creates an either-or property when `eo=true`.
  - Test that `Properties::create` creates a valued property when `eo=false`.
  - Test that `Properties::create` sets `has_of_in_the_name` correctly for names containing " of ".
  - Test that `Properties::obtain` creates a new property when none exists.
  - Test that `Properties::obtain` returns an existing property when one exists.
  - Test that `Properties::to_kind` returns "truth-state-valued property" for either-or properties.
  - Test that `Properties::to_kind` returns "<kind>-valued property" for valued properties.
  - Test that `Properties::kind_of_contents` returns "truth_state" for either-or properties.
  - Test that `Properties::kind_of_contents` returns the value kind for valued properties.
  - Test that `EitherOrProperties::new_eo_data` creates data with no negation.
  - Test that `EitherOrProperties::make_pair` joins two properties into a pair.
  - Test that `EitherOrProperties::make_pair` returns false for non-either-or properties.
  - Test that `EitherOrProperties::get_negation` returns the negation of a paired property.
  - Test that `EitherOrProperties::get_negation` returns None for an unpaired property.
  - Test that `ValueProperties::new_value_data` creates data with no kind.
  - Test that `ValueProperties::set_kind` sets the value kind of a valued property.
  - Test that `ValueProperties::set_kind` returns false for an either-or property.
  - Test that `ValueProperties::kind` returns the value kind of a valued property.
  - Test that `ValueProperties::kind` returns None for an either-or property.
  - Test that `ValueProperties::make_coincide_with_kind` sets the kind and marks coincidence.
  - Test that `ValueProperties::coincides_with_kind` returns true after `make_coincide_with_kind`.
  - Test that `ValueProperties::coincides_with_kind` returns false for a normal property.

### 8. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `Properties::create` creates a property with the correct name and type (either-or or valued).
- [ ] `Properties::obtain` finds existing properties or creates new ones.
- [ ] `Properties::to_kind` returns the correct kind string for both property types.
- [ ] `Properties::kind_of_contents` returns the correct contents kind for both property types.
- [ ] `EitherOrProperties::new_eo_data` creates data with no negation.
- [ ] `EitherOrProperties::make_pair` correctly joins two either-or properties.
- [ ] `EitherOrProperties::get_negation` returns the negation of a paired property.
- [ ] `ValueProperties::new_value_data` creates data with no kind.
- [ ] `ValueProperties::set_kind` sets the value kind of a valued property.
- [ ] `ValueProperties::kind` returns the value kind of a valued property.
- [ ] `ValueProperties::make_coincide_with_kind` sets the kind and marks coincidence.
- [ ] `ValueProperties::coincides_with_kind` returns the correct coincidence status.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`PropertyPermissions` integration**: Updating `PropertyPermissions::grant` and `PropertyPermissions::find` to use the `property` struct instead of string names is deferred. The existing string-based approach continues to work.
- **`PropertyInferences` integration**: Updating `PropertyInferences::draw` to use the `property` struct instead of string names is deferred.
- **`RTProperties`**: The run-time compilation system (`property_compilation_data`, `RTProperties::initialise_pcd`, `RTProperties::iname`, etc.) is deferred.
- **Noun registration**: Registering property names as nouns in the lexicon (`Nouns::new_proper_noun`) is deferred. This depends on the Preform/Nouns system.
- **Special property detection**: The `P_description`, `P_specification`, `P_indefinite_appearance_text`, `P_variable_initial_value`, and `P_grammatical_gender` special properties are deferred.
- **`PluginCalls::new_property_notify`**: Plugin notification for new properties is deferred.
- **`EitherOrPropertyAdjectives`**: Creating adjective meanings for either-or properties (`EitherOrPropertyAdjectives::create_for_property`) is deferred. This depends on the adjective meaning system (assertions-module).
- **`ValueProperties::make_setting_bp`**: Creating the setting binary predicate for valued properties is deferred. This depends on `SettingPropertyRelations`.
- **`SettingPropertyRelations`**: The setting property relation family (`SettingPropertyRelations::start()`) is deferred. This depends on the property system and will be implemented in a later plan.
- **`SameAsRelations`**: The same-property-value-as relation family (`SameAsRelations::start()`) is deferred. This depends on the property system and will be implemented in a later plan.
- **`ComparativeRelations`**: The comparative relations family (`ComparativeRelations::start()`) is deferred. This depends on measurement adjectives.
- **`MeasurementAdjectives`**: The measurement adjectives system (`MeasurementAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`InstanceAdjectives`**: The instance adjectives system (`InstanceAdjectives::start()`) is deferred. This depends on the adjective meaning system.
- **`ConditionsOfSubjects`**: The conditions of subjects system (`ConditionsOfSubjects`) is deferred. This depends on the property system, instances, and adjectives.
- **`InstanceSubjects`**: `InstanceSubjects::family()` (Chapter 4/Instance Subjects.w) — depends on the instance system, is deferred.
- **`VariableSubjects`**: `VariableSubjects::family()` (Chapter 4/Variable Subjects.w) — depends on the nonlocal variables system, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`PreformUtilities::wording`**: The full Preform wording system is deferred. This plan uses simplified string names.
- **`word_assemblage` struct**: The full word assemblage struct is deferred. This plan uses simplified string names.
- **`i6_schema` struct**: The full I6 schema struct is deferred. This plan uses simplified string schemas.
- **Run-time compilation**: All `RT*` functions (run-time compilation of properties, permissions, relations) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
