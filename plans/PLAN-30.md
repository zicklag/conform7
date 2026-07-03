# Plan 30: Instance Adjectives — Enumerative Adjective Meaning Family
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the Instance Adjectives system — the second concrete adjective meaning family that uses the adjective meaning infrastructure from PLAN-28. This creates the `enumerative_amf` family with an assert method, enabling instances of enumerated kinds (e.g., "red", "blue", "green" for the colour kind) to be used as adjectives in the model world.

This is the smallest next step after PLAN-29 because:

1. **It's the next item in the knowledge module startup that has no remaining dependencies.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `InstanceAdjectives::start()` at line 39 — before `EitherOrPropertyAdjectives::start()` (PLAN-29, Complete) and `MeasurementAdjectives::start()`. It depends on the adjective meaning system (PLAN-28, Complete), the property system (PLAN-25, Complete), and the `instance` struct (not yet built). `MeasurementAdjectives::start()` (line 41) depends on `measurement_definition` and `Measurements` which are more complex and require grammar parsing.

2. **It's the second simplest adjective meaning family.** At 97 lines of C, `InstanceAdjectives` is slightly larger than `EitherOrPropertyAdjectives` (74 lines) but significantly smaller than `MeasurementAdjectives` (197 lines). It requires the `instance` struct and `InstanceSubjects` family, which are fundamental building blocks needed by many other systems.

3. **It's a prerequisite for the assertion pipeline.** `Assert::inner_slated` (Chapter 1/Assert Propositions.w, line 266) calls `Instances::new()` to create instances from quantifier atoms. Without the `instance` struct, the assertion pipeline cannot create instances. Additionally, `InstanceAdjectives::make_adjectival()` is called by `InstanceSubjects::make_adj_const_domain()` (Chapter 4/Instance Subjects.w, line 65) when a property permission is granted for a kind whose name coincides with a property — this is how "the ball is green" becomes a valid assertion.

4. **It's a prerequisite for `ConditionsOfSubjects`.** `ConditionsOfSubjects::parse()` (Chapter 4/Conditions of Subjects.w, line 71) uses `AdjectiveAmbiguity::has_enumerative_meaning()` to check if an option is already an instance with an adjectival meaning. This requires the enumerative adjective meaning family to exist.

5. **It's a prerequisite for `Instances::register_as_adjectival_constant`.** `Instances::register_as_adjectival_constant()` (Chapter 2/Instances.w, line 218) calls `InferenceSubjects::make_adj_const_domain()` which dispatches to `InstanceSubjects::make_adj_const_domain()` which calls `InstanceAdjectives::make_adjectival()`. This is how instances like "red" become adjectives that set the "colour" property.

6. **It introduces the `instance` struct — a fundamental building block.** The `instance` struct is used throughout the knowledge module: by `InstanceSubjects`, `InstanceAdjectives`, `Instances as Adjectives`, `OrderingInstances`, `Preform for Instances`, and the assertion pipeline. Building it now unlocks all of these downstream systems.

7. **Independently testable.** We can create the `instance` struct, create the `InstanceSubjects` family, create the `enumerative_amf` family, create adjective meanings for instances, test the `make_adjectival` function (which declares an adjective, creates a meaning, adds it to the adjective, and sets the domain), test the `is_enumerative` method, test the assert method (which calls `PropertyInferences::draw` for positive parity and returns FALSE for negative parity), test the `as_adjective` accessor, and test the `as_subject` accessor — all without needing measurement adjectives, comparative relations, or run-time compilation.

## Background

### C reference architecture

#### Instance (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 34-46)

The `instance` struct represents a named constant value of an enumerated kind or an object:

```c
typedef struct instance {
    struct noun *as_noun;                /* the name of the instance */
    struct adjective *as_adjective;      /* if this is a noun used adjectivally, like "red" */
    struct inference_subject *as_subject; /* from which the kind can be deduced */

    struct parse_node *creating_sentence;  /* sentence creating the instance */
    struct parse_node *where_kind_is_set; /* sentence identifying its kind */

    int enumeration_index;                /* within each non-object kind, instances are counted from 1 */

    struct instance_compilation_data compilation_data; /* see //runtime: Instances// */
    CLASS_DEFINITION
} instance;
```

Key functions:

```c
instance *Instances::new(wording W, kind *K) {
    PROTECTED_MODEL_PROCEDURE;
    @<Simplify the initial kind of the instance@>;
    instance *I = CREATE(instance);
    @<Initialise the instance@>;
    @<Add the new instance to its enumeration@>;
    latest_instance = I;
    PluginCalls::new_named_instance_notify(I);
    if (Kinds::eq(K, K_grammatical_gender)) Instances::new_grammatical(I);
    Assertions::Assemblies::satisfies_generalisations(I->as_subject);
    return I;
}
```

```c
wording Instances::get_name(instance *I, int plural) {
    if ((I == NULL) || (I->as_noun == NULL)) return EMPTY_WORDING;
    return Nouns::nominative(I->as_noun, plural);
}

inference_subject *Instances::as_subject(instance *I) {
    if (I == NULL) return NULL;
    return I->as_subject;
}

adjective *Instances::as_adjective(instance *I) {
    if (I == NULL) return NULL;
    return I->as_adjective;
}

kind *Instances::to_kind(instance *I) {
    if (I == NULL) return NULL;
    inference_subject *inherits_from = InferenceSubjects::narrowest_broader_subject(I->as_subject);
    return KindSubjects::to_kind(inherits_from);
}
```

#### Instance Subjects (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 1-67)

The `InstanceSubjects` family bridges instances to the inference subject system:

```c
inference_subject_family *instances_family = NULL;

inference_subject_family *InstanceSubjects::family(void) {
    if (instances_family == NULL) {
        instances_family = InferenceSubjects::new_family();
        METHOD_ADD(instances_family,
            GET_DEFAULT_CERTAINTY_INFS_MTID, InstanceSubjects::certainty);
        METHOD_ADD(instances_family,
            GET_NAME_TEXT_INFS_MTID, InstanceSubjects::get_name);
        METHOD_ADD(instances_family,
            MAKE_ADJ_CONST_DOMAIN_INFS_MTID, InstanceSubjects::make_adj_const_domain);
        METHOD_ADD(instances_family,
            NEW_PERMISSION_GRANTED_INFS_MTID, InstanceSubjects::new_permission_granted);
        METHOD_ADD(instances_family,
            EMIT_ALL_INFS_MTID, RTInstances::compile_all);
        METHOD_ADD(instances_family,
            EMIT_ELEMENT_INFS_MTID, RTInstances::emit_element_of_condition);
    }
    return instances_family;
}

inference_subject *InstanceSubjects::new(instance *I, kind *K) {
    return InferenceSubjects::new(KindSubjects::from_kind(K),
        InstanceSubjects::family(), STORE_POINTER_instance(I), NULL);
}

instance *InstanceSubjects::to_instance(inference_subject *infs) {
    if ((infs) && (infs->infs_family == instances_family))
        return RETRIEVE_POINTER_instance(infs->represents);
    return NULL;
}
```

#### Instance Adjectives (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 1-97)

The Instance Adjectives system creates one family and registers instances as adjectives:

```c
adjective_meaning_family *enumerative_amf = NULL;

void InstanceAdjectives::start(void) {
    enumerative_amf = AdjectiveMeanings::new_family(2);
    METHOD_ADD(enumerative_amf, ASSERT_ADJM_MTID, InstanceAdjectives::assert);
}

int InstanceAdjectives::is_enumerative(adjective_meaning *am) {
    if ((am) && (am->family == enumerative_amf)) return TRUE;
    return FALSE;
}
```

The `make_adjectival` function registers an instance as an adjective for a given property:

```c
void InstanceAdjectives::make_adjectival(instance *I, property *P,
    kind *set, instance *singleton) {
    kind *D = NULL;
    @<Find the kind domain within which the adjective applies@>;
    adjective_meaning *am = NULL;
    @<Create the adjective meaning for this use of the instance@>;
    @<Write I6 schemas for asserting and testing this use of the instance@>;
}
```

```c
@<Find the kind domain within which the adjective applies@> =
    if (singleton) D = Instances::to_kind(singleton);
    else if (set) D = set;
    if (D == NULL) internal_error("No adjectival constant domain");

@<Create the adjective meaning for this use of the instance@> =
    wording NW = Instances::get_name(I, FALSE);
    adjective *adj = Adjectives::declare(NW, NULL);
    am = AdjectiveMeanings::new(enumerative_amf, STORE_POINTER_instance(I), NW);
    I->as_adjective = AdjectiveAmbiguity::add_meaning_to_adjective(am, adj);
    if (singleton) AdjectiveMeaningDomains::set_from_instance(am, singleton);
    else if (set) AdjectiveMeaningDomains::set_from_kind(am, set);
```

The assert method calls `PropertyInferences::draw` for positive parity, and returns FALSE for negative parity (refusing to assert falseness since it's unclear what to infer):

```c
int InstanceAdjectives::assert(adjective_meaning_family *f, adjective_meaning *am,
    inference_subject *infs_to_assert_on, int parity) {
    if (parity == FALSE) return FALSE;
    instance *I = RETRIEVE_POINTER_instance(am->family_specific_data);
    property *P = Properties::property_with_same_name_as(Instances::to_kind(I));
    if (P == NULL) internal_error("enumerative adjective on non-property");
    PropertyInferences::draw(infs_to_assert_on, P, Rvalues::from_instance(I));
    return TRUE;
}
```

### Key C source files

- `inform7/knowledge-module/Chapter 2/Instances.w` — the `instance` struct, `Instances::new`, `Instances::get_name`, `Instances::as_subject`, `Instances::as_adjective`, `Instances::to_kind`, `Instances::register_as_adjectival_constant` (403 lines)
- `inform7/knowledge-module/Chapter 4/Instance Subjects.w` — `InstanceSubjects` family, `InstanceSubjects::new`, `InstanceSubjects::to_instance`, `InstanceSubjects::make_adj_const_domain` (67 lines)
- `inform7/knowledge-module/Chapter 2/Instances as Adjectives.w` — `InstanceAdjectives` module, `enumerative_amf` family, `make_adjectival`, `assert` (97 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `InstanceAdjectives::start()` (line 39)
- `inform7/assertions-module/Chapter 8/Adjective Meanings.w` — `AdjectiveMeanings::new_family`, `AdjectiveMeanings::new` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Ambiguity.w` — `AdjectiveAmbiguity::add_meaning_to_adjective` (PLAN-28)
- `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w` — `AdjectiveMeaningDomains::set_from_kind`, `AdjectiveMeaningDomains::set_from_instance` (PLAN-28)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` (PLAN-19)
- `services/linguistics-module/Chapter 2/Adjectives.w` — `Adjectives::declare` (PLAN-28)
- `inform7/knowledge-module/Chapter 4/Kind Subjects.w` — `KindSubjects::to_kind`, `KindSubjects::from_kind` (used by `Instances::to_kind`)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/either_or_property_adjectives.rs` — `EitherOrPropertyAdjectives` module, `EITHER_OR_PROPERTY_FAMILY` constant, `start()`, `is()`, `create_for_property()`, `assert()`, `prepare_schemas()`, `index()`, unit tests (PLAN-29, Complete).
- `crates/conform7-semantics/src/knowledge/adjectives.rs` — `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, `AdjectiveDomainData` struct, `AdjectiveMeaningFamilyMethods` struct, `AdjectiveMeanings` management functions (`new_family`, `new`, `negate`, `assert`), `AdjectiveAmbiguity` management functions (`add_meaning_to_adjective`, `can_be_applied_to`, `first_meaning`), `AdjectiveMeaningDomains` management functions (`new_from_kind`, `set_from_kind`, `get_kind`, `weak_match`), `Adjectives::declare`, `Adjectives::find`, `Adjectives::get_nominative_singular`, unit tests (PLAN-28, Complete).
- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData` struct (with `as_adjective: Option<usize>` field), `ValuePropertyData` struct, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties::new_eo_data`, `EitherOrProperties::make_pair`, `EitherOrProperties::get_negation`, `EitherOrProperties::as_adjective`, `ValueProperties` functions, unit tests (PLAN-25, Complete).
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
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod either_or_property_adjectives;` from PLAN-29).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`Instance` struct** — a new struct in a new `instances` module (or in `instance_subjects.rs`) with:
   - `name` — the instance name (simplified: a string instead of `noun *`)
   - `as_adjective` — optional adjective index (set by `InstanceAdjectives::make_adjectival`)
   - `as_subject` — inference subject index (set by `InstanceSubjects::new`)
   - `enumeration_index` — integer index within the kind (0 for objects, 1+ for enumerated values)
   - `Instances::new(name, kind_idx, subjects, families)` — create a new instance:
     - Creates an inference subject via `InstanceSubjects::new`
     - Stores the instance in the subjects list
     - Returns the instance index
   - `Instances::get_name(inst_idx, instances)` — return the instance's name
   - `Instances::as_subject(inst_idx, instances)` — return the instance's subject index
   - `Instances::as_adjective(inst_idx, instances)` — return the instance's adjective index
   - `Instances::to_kind(inst_idx, instances, subjects, families)` — deduce the kind from the subject hierarchy
   - `Instances::get_numerical_value(inst_idx, instances)` — return the enumeration index

2. **`InstanceSubjects` family** — a new module `instance_subjects` with:
   - `InstanceSubjects::family()` — returns the instances family (lazily created)
   - `InstanceSubjects::new(inst_idx, kind_idx, subjects, families)` — creates a new inference subject for an instance
   - `InstanceSubjects::to_instance(subj_idx, subjects)` — extracts the instance index from a subject
   - `InstanceSubjects::certainty()` — returns `CERTAIN_CE`
   - `InstanceSubjects::get_name()` — returns the instance's name
   - `InstanceSubjects::make_adj_const_domain()` — calls `InstanceAdjectives::make_adjectival` (simplified: no-op, since `InstanceAdjectives` is the next step)
   - `InstanceSubjects::new_permission_granted()` — simplified: no-op (RT compilation deferred)

3. **`InstanceAdjectives` module** — a new module `instance_adjectives` with:
   - `InstanceAdjectives::start()` — creates the `enumerative_amf` family with the assert method
   - `InstanceAdjectives::is_enumerative(am_idx, meanings)` — checks if a meaning belongs to the enumerative family
   - `InstanceAdjectives::make_adjectival(inst_idx, prn_idx, set_kind_idx, singleton_idx, adjectives, meanings, families, properties, instances, subjects)` — registers an instance as an adjective for a given property:
     - Finds the kind domain (from singleton or set)
     - Gets the instance name
     - Declares a new adjective via `Adjectives::declare`
     - Creates a new adjective meaning via `AdjectiveMeanings::new` with the `enumerative_amf` family and the instance index as family-specific data
     - Adds the meaning to the adjective via `AdjectiveAmbiguity::add_meaning_to_adjective`
     - Stores the adjective index in `instance.as_adjective`
     - Sets the domain from the kind or instance via `AdjectiveMeaningDomains::set_from_kind` or `set_from_instance`
   - `InstanceAdjectives::assert(am_idx, subj_idx, parity, meanings, subjects, families, properties, instances, inference_families, inferences, data_registry)` — asserts the instance as a property value on a subject:
     - If parity is false, returns FALSE (refuses to assert falseness)
     - Retrieves the instance index from the meaning's family-specific data
     - Finds the property with the same name as the instance's kind via `Properties::property_with_same_name_as`
     - Calls `PropertyInferences::draw` with the property and instance
     - Returns TRUE
   - Global constant for the family index

4. **Integration with the knowledge module** — add the `instances`, `instance_subjects`, and `instance_adjectives` module declarations to the knowledge module's `mod.rs`.

5. **Unit tests** — create instances, create the `InstanceSubjects` family, create the `enumerative_amf` family, test the `make_adjectival` function (declares adjective, creates meaning, adds to adjective, sets domain, stores in instance), test the `is_enumerative` method, test the assert method (calls `PropertyInferences::draw` for positive parity, returns FALSE for negative parity), test the `as_subject` and `as_adjective` accessors, test `Instances::to_kind` via the subject hierarchy, test `Instances::get_name`.

## Tasks

### 1. Create the `Instance` struct and `Instances` management functions

- [ ] Create `crates/conform7-semantics/src/knowledge/instances.rs` with:

  ```rust
  /// An instance — a named constant value of an enumerated kind or an object.
  ///
  /// Corresponds to `instance` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 34-46).
  ///
  /// Instances are named constants giving a finite range of possible values of
  /// a kind. For example, "red", "blue" and "green" are instances of the kind
  /// "colour". Objects are instances too: "Peter" and "blue ball" are instances
  /// of the kind "object".
  ///
  /// Simplified:
  /// - No `noun *` (simplified to a string name)
  /// - No `parse_node *` (creation tracking deferred)
  /// - No `instance_compilation_data` (run-time compilation deferred)
  /// - No `PROTECTED_MODEL_PROCEDURE` guard
  /// - No `PluginCalls` notifications
  /// - No `Assertions::Assemblies` generalisations
  use crate::knowledge::inference_subjects::InferenceSubject;
  use crate::knowledge::kind_subjects::KindSubjects;

  /// An instance — a named constant value of an enumerated kind or an object.
  #[derive(Clone, Debug)]
  pub struct Instance {
      /// The name of the instance.
      /// Corresponds to `as_noun` in the C reference (simplified: string instead of `noun *`).
      pub name: String,
      /// The adjective index, if this instance is used adjectivally (like "red").
      /// Corresponds to `as_adjective` in the C reference.
      pub as_adjective: Option<usize>,
      /// The inference subject index, from which the kind can be deduced.
      /// Corresponds to `as_subject` in the C reference.
      pub as_subject: Option<usize>,
      /// Within each non-object kind, instances are counted from 1.
      /// Corresponds to `enumeration_index` in the C reference.
      pub enumeration_index: usize,
  }

  impl Instance {
      /// Create a new instance with default values.
      pub fn new(name: &str) -> Self {
          Instance {
              name: name.to_string(),
              as_adjective: None,
              as_subject: None,
              enumeration_index: 0,
          }
      }
  }

  /// The Instances management module.
  ///
  /// Corresponds to `Instances` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Instances.w`).
  pub struct Instances;

  impl Instances {
      /// Create a new instance of the given kind.
      ///
      /// Corresponds to `Instances::new` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 60-76).
      ///
      /// Returns the index of the new instance.
      ///
      /// Simplified:
      /// - No `PROTECTED_MODEL_PROCEDURE` guard
      /// - No kind simplification (weaken to K_object)
      /// - No `PluginCalls` notifications
      /// - No `Assertions::Assemblies` generalisations
      /// - No `Preform` noun creation
      /// - No grammatical gender handling
      pub fn new(
          name: &str,
          kind_idx: Option<usize>,
          subjects: &mut Vec<InferenceSubject>,
          families: &[InferenceSubjectFamily],
      ) -> usize {
          let inst_idx = instances.len();
          let mut instance = Instance::new(name);

          // Create the inference subject for this instance
          let subj_idx = InstanceSubjects::new(inst_idx, kind_idx, subjects, families);
          instance.as_subject = Some(subj_idx);

          instances.push(instance);
          inst_idx
      }

      /// Get the name of an instance.
      ///
      /// Corresponds to `Instances::get_name` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 121-124).
      pub fn get_name(inst_idx: usize, instances: &[Instance]) -> &str {
          &instances[inst_idx].name
      }

      /// Get the inference subject index for an instance.
      ///
      /// Corresponds to `Instances::as_subject` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 156-159).
      pub fn as_subject(inst_idx: usize, instances: &[Instance]) -> Option<usize> {
          instances.get(inst_idx).and_then(|i| i.as_subject)
      }

      /// Get the adjective index for an instance.
      ///
      /// Corresponds to `Instances::as_adjective` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 161-164).
      pub fn as_adjective(inst_idx: usize, instances: &[Instance]) -> Option<usize> {
          instances.get(inst_idx).and_then(|i| i.as_adjective)
      }

      /// Get the kind of an instance by examining its subject hierarchy.
      ///
      /// Corresponds to `Instances::to_kind` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 240-244).
      ///
      /// The kind is not stored directly in the instance; it is deduced from
      /// the position of the instance's subject in the subjects hierarchy.
      pub fn to_kind(
          inst_idx: usize,
          instances: &[Instance],
          subjects: &[InferenceSubject],
          families: &[InferenceSubjectFamily],
      ) -> Option<usize> {
          let subj_idx = instances.get(inst_idx)?.as_subject?;
          let inherits_from = InferenceSubjects::narrowest_broader_subject(subj_idx, subjects)?;
          KindSubjects::to_kind(inherits_from, subjects, families)
      }

      /// Get the numerical value (enumeration index) of an instance.
      ///
      /// Corresponds to `Instances::get_numerical_value` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances.w`, lines 136-138).
      pub fn get_numerical_value(inst_idx: usize, instances: &[Instance]) -> usize {
          instances[inst_idx].enumeration_index
      }
  }
  ```

### 2. Create the `InstanceSubjects` family

- [ ] Create `crates/conform7-semantics/src/knowledge/instance_subjects.rs` with:

  ```rust
  /// The instances family of inference subjects.
  ///
  /// Corresponds to `InstanceSubjects` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
  ///
  /// This family bridges instances to the inference subject system, allowing
  /// instances to have properties and inferences drawn about them.
  ///
  /// Simplified:
  /// - No RTInstances::compile_all (run-time compilation deferred)
  /// - No RTInstances::emit_element_of_condition (run-time compilation deferred)
  /// - No RTPropertyPermissions::new_storage (run-time compilation deferred)
  /// - `make_adj_const_domain` is a no-op (InstanceAdjectives is the next step)
  use crate::knowledge::inference_subjects::{
      InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
  };
  use crate::knowledge::instances::Instance;

  /// Index of the instances family in the family registry.
  pub const INSTANCES_FAMILY: usize = 0;

  /// The InstanceSubjects management module.
  ///
  /// Corresponds to `InstanceSubjects` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`).
  pub struct InstanceSubjects;

  impl InstanceSubjects {
      /// Get or create the instances family.
      ///
      /// Corresponds to `InstanceSubjects::family` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 11-29).
      pub fn family(families: &mut Vec<InferenceSubjectFamily>) -> usize {
          if families.len() <= INSTANCES_FAMILY {
              let family = InferenceSubjectFamily {
                  name: "instances",
                  methods: InferenceSubjectFamilyMethods {
                      get_default_certainty: Some(InstanceSubjects::certainty),
                      get_name_text: Some(InstanceSubjects::get_name),
                      make_adj_const_domain: Some(InstanceSubjects::make_adj_const_domain),
                      new_permission_granted: Some(InstanceSubjects::new_permission_granted),
                      ..InferenceSubjectFamilyMethods::default()
                  },
              };
              families.push(family);
          }
          INSTANCES_FAMILY
      }

      /// Create a new inference subject for an instance.
      ///
      /// Corresponds to `InstanceSubjects::new` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 35-38).
      pub fn new(
          inst_idx: usize,
          kind_idx: Option<usize>,
          subjects: &mut Vec<InferenceSubject>,
          families: &[InferenceSubjectFamily],
      ) -> usize {
          let parent = kind_idx.and_then(|k| KindSubjects::from_kind(k, subjects, families));
          let subj = InferenceSubject::new(
              parent,
              INSTANCES_FAMILY,
              Some(inst_idx),
          );
          let subj_idx = subjects.len();
          subjects.push(subj);
          subj_idx
      }

      /// Extract the instance index from an inference subject.
      ///
      /// Corresponds to `InstanceSubjects::to_instance` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 40-44).
      pub fn to_instance(subj_idx: usize, subjects: &[InferenceSubject]) -> Option<usize> {
          let subj = subjects.get(subj_idx)?;
          if subj.family == INSTANCES_FAMILY {
              subj.represents
          } else {
              None
          }
      }

      /// Get the default certainty for instance subjects.
      ///
      /// Corresponds to `InstanceSubjects::certainty` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 31-33).
      fn certainty(
          _family_idx: usize,
          _subj_idx: usize,
          _subjects: &[InferenceSubject],
      ) -> i32 {
          2 // CERTAIN_CE
      }

      /// Get the name of an instance subject.
      ///
      /// Corresponds to `InstanceSubjects::get_name` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 52-56).
      fn get_name(
          _family_idx: usize,
          subj_idx: usize,
          subjects: &[InferenceSubject],
          instances: &[Instance],
      ) -> String {
          if let Some(inst_idx) = InstanceSubjects::to_instance(subj_idx, subjects) {
              if let Some(inst) = instances.get(inst_idx) {
                  return inst.name.clone();
              }
          }
          String::new()
      }

      /// Handle a new permission granted for an instance subject.
      ///
      /// Corresponds to `InstanceSubjects::new_permission_granted` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 58-61).
      ///
      /// Simplified: no-op (RT compilation deferred).
      fn new_permission_granted(
          _family_idx: usize,
          _subj_idx: usize,
          _subjects: &[InferenceSubject],
      ) {
          // No-op: RTPropertyPermissions::new_storage deferred
      }

      /// Make an instance into an adjectival constant for a property domain.
      ///
      /// Corresponds to `InstanceSubjects::make_adj_const_domain` in the C reference
      /// (`inform7/knowledge-module/Chapter 4/Instance Subjects.w`, lines 63-66).
      ///
      /// Simplified: no-op (InstanceAdjectives is the next step).
      fn make_adj_const_domain(
          _family_idx: usize,
          _subj_idx: usize,
          _subjects: &[InferenceSubject],
          _instances: &[Instance],
          _properties: &[Property],
      ) {
          // No-op: InstanceAdjectives::make_adjectival deferred
      }
  }
  ```

### 3. Create the `InstanceAdjectives` module

- [ ] Create `crates/conform7-semantics/src/knowledge/instance_adjectives.rs` with:

  ```rust
  /// The Instance Adjectives system — instances used as adjectives.
  ///
  /// Corresponds to `InstanceAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`).
  ///
  /// Creates one adjective_meaning_family instance:
  /// - enumerative_amf — for enumerative (instance-based) adjectives
  ///
  /// When instances of a kind whose name coincides with a property are used
  /// adjectivally, they set that property's value. For example, if "colour"
  /// is a kind of value with instances "red", "blue", "green", and a door
  /// has a colour, then "the door is red" asserts that the door's colour
  /// property is set to the "red" instance.
  ///
  /// Simplified:
  /// - No I6 schemas for asserting and testing (run-time compilation deferred)
  /// - No Preform grammar for instance name resolution
  use crate::knowledge::adjectives::{
      Adjective, AdjectiveAmbiguity, AdjectiveMeaning, AdjectiveMeaningDomains,
      AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods, AdjectiveMeanings, Adjectives,
  };
  use crate::knowledge::inference_subjects::InferenceSubject;
  use crate::knowledge::inferences::{Inference, InferenceFamily};
  use crate::knowledge::instances::{Instance, Instances};
  use crate::knowledge::properties::{Properties, Property};
  use crate::knowledge::property_inferences::{PropertyInferenceData, PropertyInferences};

  /// Index of the enumerative family in the family registry.
  pub const ENUMERATIVE_FAMILY: usize = 0;

  /// The InstanceAdjectives management module.
  ///
  /// Corresponds to `InstanceAdjectives` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`).
  pub struct InstanceAdjectives;

  impl InstanceAdjectives {
      /// Create the enumerative family with its methods.
      ///
      /// Corresponds to `InstanceAdjectives::start` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 23-26).
      ///
      /// Returns (families, meanings, adjectives) where:
      /// - families[0] = enumerative_amf
      /// - meanings is empty (make_adjectival fills it)
      /// - adjectives is empty (make_adjectival fills it)
      pub fn start() -> (Vec<AdjectiveMeaningFamily>, Vec<AdjectiveMeaning>, Vec<Adjective>) {
          let enumerative_family = AdjectiveMeaningFamily {
              name: "enumerative",
              definition_claim_priority: 2,
              methods: AdjectiveMeaningFamilyMethods {
                  assert: Some(InstanceAdjectives::assert),
                  ..AdjectiveMeaningFamilyMethods::default()
              },
          };

          (vec![enumerative_family], Vec::new(), Vec::new())
      }

      /// Check if an adjective meaning belongs to the enumerative family.
      ///
      /// Corresponds to `InstanceAdjectives::is_enumerative` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 28-31).
      ///
      /// Returns true if the meaning belongs to this family, false otherwise.
      pub fn is_enumerative(am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
          meanings.get(am_idx).is_some_and(|am| am.family == ENUMERATIVE_FAMILY)
      }

      /// Register an instance as an adjective for a given property.
      ///
      /// Corresponds to `InstanceAdjectives::make_adjectival` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 48-55).
      ///
      /// This function:
      /// 1. Finds the kind domain (from singleton or set)
      /// 2. Gets the instance name
      /// 3. Declares a new adjective
      /// 4. Creates a new adjective meaning for this instance-property pair
      /// 5. Adds the meaning to the adjective
      /// 6. Stores the adjective index in the instance
      /// 7. Sets the domain from the kind or instance
      ///
      /// Simplified:
      /// - No I6 schema writing (run-time compilation deferred)
      /// - No Preform grammar for instance name resolution
      pub fn make_adjectival(
          inst_idx: usize,
          _prn_idx: usize,
          set_kind_idx: Option<usize>,
          singleton_idx: Option<usize>,
          adjectives: &mut Vec<Adjective>,
          meanings: &mut Vec<AdjectiveMeaning>,
          families: &[AdjectiveMeaningFamily],
          properties: &[Property],
          instances: &[Instance],
          subjects: &[InferenceSubject],
          subject_families: &[InferenceSubjectFamily],
      ) {
          // Find the kind domain
          let domain_kind = if let Some(singleton) = singleton_idx {
              Instances::to_kind(singleton, instances, subjects, subject_families)
          } else {
              set_kind_idx
          };

          if domain_kind.is_none() {
              panic!("No adjectival constant domain");
          }

          // Get the instance name
          let name = Instances::get_name(inst_idx, instances);

          // Declare a new adjective
          let adj_idx = Adjectives::declare(name, adjectives);

          // Create a new adjective meaning
          let am_idx = AdjectiveMeanings::new(
              ENUMERATIVE_FAMILY,
              Some(inst_idx), // family_specific_data = instance index
              name,
              meanings,
              families,
          );

          // Add the meaning to the adjective
          AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx, adjectives, meanings);

          // Store the adjective index in the instance
          // (This requires mutable access to instances, which we handle via a callback
          //  or by returning the mapping. For now, we use a simple approach.)
          // Note: instances are passed as &mut via the caller

          // Set the domain
          if let Some(singleton) = singleton_idx {
              AdjectiveMeaningDomains::set_from_instance(am_idx, singleton, meanings, instances, subjects, subject_families);
          } else if let Some(kind) = domain_kind {
              AdjectiveMeaningDomains::set_from_kind(am_idx, kind, meanings, subjects, subject_families);
          }
      }

      /// Assert an enumerative adjective on a subject.
      ///
      /// Corresponds to `InstanceAdjectives::assert` in the C reference
      /// (`inform7/knowledge-module/Chapter 2/Instances as Adjectives.w`, lines 88-96).
      ///
      /// For positive parity, asserts that the property with the same name as
      /// the instance's kind has the instance as its value. For negative parity,
      /// returns FALSE (refusing to assert falseness since it's unclear what
      /// to infer from, e.g., "the ball is not green").
      pub fn assert(
          _family_idx: usize,
          am_idx: usize,
          subj_idx: usize,
          parity: bool,
          meanings: &[AdjectiveMeaning],
          subjects: &[InferenceSubject],
          _families: &[AdjectiveMeaningFamily],
          properties: &[Property],
          instances: &[Instance],
          _inference_families: &[InferenceFamily],
          inferences: &mut Vec<Inference>,
          _data_registry: &mut Vec<PropertyInferenceData>,
      ) -> bool {
          if !parity {
              return false; // Refuse to assert falseness
          }

          // Retrieve the instance from the meaning's family-specific data
          let inst_idx = meanings.get(am_idx)
              .and_then(|am| am.family_specific_data)
              .expect("enumerative adjective meaning without instance data");

          // Find the property with the same name as the instance's kind
          let kind_idx = Instances::to_kind(inst_idx, instances, subjects, &[])
              .expect("instance without kind");
          let prn_idx = Properties::property_with_same_name_as(kind_idx, properties)
              .expect("enumerative adjective on non-property");

          // Draw the inference
          PropertyInferences::draw(subj_idx, prn_idx, Some(inst_idx), subjects, inferences, _data_registry);
          true
      }
  }
  ```

### 4. Integrate with the knowledge module

- [ ] Add module declarations to `crates/conform7-semantics/src/knowledge/mod.rs`:

  ```rust
  pub mod instances;
  pub mod instance_subjects;
  pub mod instance_adjectives;
  ```

### 5. Unit tests

- [ ] Add tests to `crates/conform7-semantics/src/knowledge/instance_adjectives.rs`:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::knowledge::adjectives::{
          Adjective, AdjectiveMeaning, AdjectiveMeaningFamily, AdjectiveMeaningFamilyMethods,
          AdjectiveMeanings, AdjectiveAmbiguity, AdjectiveMeaningDomains, Adjectives,
      };
      use crate::knowledge::inference_subjects::{
          InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
      };
      use crate::knowledge::inferences::{Inference, InferenceFamily, Certainty};
      use crate::knowledge::instances::{Instance, Instances};
      use crate::knowledge::instance_subjects::InstanceSubjects;
      use crate::knowledge::properties::{Property, EitherOrPropertyData, ValuePropertyData, Properties};
      use crate::knowledge::property_inferences::{PropertyInferenceData, PropertyInferences};
      use crate::knowledge::kind_subjects::KindSubjects;

      // Helper to create a minimal kind subject hierarchy
      fn setup_kind_subjects() -> (Vec<InferenceSubject>, Vec<InferenceSubjectFamily>) {
          let mut families = Vec::new();
          let mut subjects = Vec::new();

          // Create the fundamentals family
          let fundamentals = InferenceSubjectFamily::fundamentals();
          families.push(fundamentals);

          // Create model_world (root)
          let model_world = InferenceSubject::new_fundamental(None, "model world");
          subjects.push(model_world);

          // Create a kind subject for "colour" kind
          let colour_kind = InferenceSubject::new(Some(0), 0, None);
          let colour_kind_idx = subjects.len();
          subjects.push(colour_kind);

          (subjects, families, colour_kind_idx)
      }

      #[test]
      fn test_start_creates_enumerative_family() {
          let (families, meanings, adjectives) = InstanceAdjectives::start();
          assert_eq!(families.len(), 1);
          assert_eq!(families[0].name, "enumerative");
          assert_eq!(families[0].definition_claim_priority, 2);
          assert!(families[0].methods.assert.is_some());
          assert!(meanings.is_empty());
          assert!(adjectives.is_empty());
      }

      #[test]
      fn test_is_enumerative() {
          let (families, mut meanings, _) = InstanceAdjectives::start();
          let am_idx = AdjectiveMeanings::new(ENUMERATIVE_FAMILY, Some(0), "red", &mut meanings, &families);
          assert!(InstanceAdjectives::is_enumerative(am_idx, &meanings));

          // A meaning from a different family should not be enumerative
          let other_family = AdjectiveMeaningFamily {
              name: "other",
              definition_claim_priority: 0,
              methods: AdjectiveMeaningFamilyMethods::default(),
          };
          let other_idx = AdjectiveMeanings::new(1, Some(0), "test", &mut meanings, &[other_family]);
          assert!(!InstanceAdjectives::is_enumerative(other_idx, &meanings));
      }

      #[test]
      fn test_make_adjectival_creates_adjective_and_meaning() {
          let (mut families, mut meanings, mut adjectives) = InstanceAdjectives::start();
          let (mut subjects, mut subject_families, colour_kind_idx) = setup_kind_subjects();

          // Create a property "colour" that coincides with the kind name
          let mut properties = vec![Property {
              name: Some("colour".to_string()),
              either_or_data: None,
              value_data: Some(ValuePropertyData {
                  kind: Some(0),
                  ..ValuePropertyData::default()
              }),
          }];

          // Create an instance "red" of kind "colour"
          let mut instances = vec![Instance {
              name: "red".to_string(),
              as_adjective: None,
              as_subject: None,
              enumeration_index: 1,
          }];

          // Call make_adjectival
          InstanceAdjectives::make_adjectival(
              0, // inst_idx
              0, // prn_idx
              Some(colour_kind_idx), // set_kind_idx
              None, // singleton_idx
              &mut adjectives,
              &mut meanings,
              &families,
              &properties,
              &instances,
              &subjects,
              &subject_families,
          );

          // Check that an adjective was created
          assert_eq!(adjectives.len(), 1);
          assert_eq!(adjectives[0].name, "red");

          // Check that a meaning was created
          assert_eq!(meanings.len(), 1);
          assert_eq!(meanings[0].family, ENUMERATIVE_FAMILY);
          assert_eq!(meanings[0].family_specific_data, Some(0)); // instance index
      }

      #[test]
      fn test_assert_positive_parity_calls_draw() {
          let (families, mut meanings, mut adjectives) = InstanceAdjectives::start();
          let (mut subjects, mut subject_families, colour_kind_idx) = setup_kind_subjects();
          let mut inference_families = vec![InferenceFamily {
              name: "property",
              methods: InferenceFamilyMethods {
                  log: None,
                  ..InferenceFamilyMethods::default()
              },
          }];
          let mut inferences = Vec::new();
          let mut data_registry = Vec::new();

          // Create a property "colour"
          let mut properties = vec![Property {
              name: Some("colour".to_string()),
              either_or_data: None,
              value_data: Some(ValuePropertyData {
                  kind: Some(colour_kind_idx),
                  ..ValuePropertyData::default()
              }),
          }];

          // Create an instance "red" of kind "colour"
          let mut instances = vec![Instance {
              name: "red".to_string(),
              as_adjective: None,
              as_subject: None,
              enumeration_index: 1,
          }];

          // Create the adjective meaning
          let am_idx = AdjectiveMeanings::new(
              ENUMERATIVE_FAMILY,
              Some(0), // instance index
              "red",
              &mut meanings,
              &families,
          );

          // Create a subject to assert on
          let door_subj = InferenceSubject::new(Some(0), 0, None);
          let subj_idx = subjects.len();
          subjects.push(door_subj);

          // Assert with positive parity
          let result = InstanceAdjectives::assert(
              0, am_idx, subj_idx, true,
              &meanings, &subjects, &families,
              &properties, &instances,
              &inference_families, &mut inferences, &mut data_registry,
          );

          assert!(result);
          // Should have drawn an inference
          assert!(!inferences.is_empty());
      }

      #[test]
      fn test_assert_negative_parity_returns_false() {
          let (families, mut meanings, _) = InstanceAdjectives::start();
          let (subjects, _, _) = setup_kind_subjects();
          let properties = Vec::new();
          let instances = vec![Instance::new("red")];
          let inference_families = Vec::new();
          let mut inferences = Vec::new();
          let mut data_registry = Vec::new();

          let am_idx = AdjectiveMeanings::new(
              ENUMERATIVE_FAMILY,
              Some(0),
              "red",
              &mut meanings,
              &families,
          );

          let result = InstanceAdjectives::assert(
              0, am_idx, 0, false,
              &meanings, &subjects, &families,
              &properties, &instances,
              &inference_families, &mut inferences, &mut data_registry,
          );

          assert!(!result);
          assert!(inferences.is_empty());
      }

      #[test]
      fn test_instances_new_creates_subject() {
          let mut subjects = Vec::new();
          let mut families = Vec::new();
          let fundamentals = InferenceSubjectFamily::fundamentals();
          families.push(fundamentals);

          let model_world = InferenceSubject::new_fundamental(None, "model world");
          subjects.push(model_world);

          let mut instances = Vec::new();
          let inst_idx = Instances::new("red", Some(0), &mut subjects, &mut families, &mut instances);

          assert_eq!(inst_idx, 0);
          assert_eq!(instances[0].name, "red");
          assert!(instances[0].as_subject.is_some());
          assert!(instances[0].as_adjective.is_none());
      }

      #[test]
      fn test_instances_get_name() {
          let instances = vec![Instance::new("red")];
          assert_eq!(Instances::get_name(0, &instances), "red");
      }

      #[test]
      fn test_instances_as_subject() {
          let mut instances = Vec::new();
          let mut subjects = vec![InferenceSubject::new_fundamental(None, "model world")];
          let families = vec![InferenceSubjectFamily::fundamentals()];

          let inst_idx = Instances::new("red", None, &mut subjects, &mut families, &mut instances);
          let subj_idx = Instances::as_subject(inst_idx, &instances);
          assert!(subj_idx.is_some());
      }

      #[test]
      fn test_instances_as_adjective() {
          let instances = vec![Instance {
              name: "red".to_string(),
              as_adjective: Some(42),
              as_subject: None,
              enumeration_index: 1,
          }];
          assert_eq!(Instances::as_adjective(0, &instances), Some(42));
      }

      #[test]
      fn test_instances_as_adjective_none() {
          let instances = vec![Instance::new("red")];
          assert_eq!(Instances::as_adjective(0, &instances), None);
      }

      #[test]
      fn test_instance_subjects_family() {
          let mut families = Vec::new();
          let family_idx = InstanceSubjects::family(&mut families);
          assert_eq!(family_idx, 0);
          assert_eq!(families[0].name, "instances");
          assert!(families[0].methods.get_default_certainty.is_some());
          assert!(families[0].methods.get_name_text.is_some());
      }

      #[test]
      fn test_instance_subjects_to_instance() {
          let mut subjects = vec![InferenceSubject::new_fundamental(None, "model world")];
          let mut families = Vec::new();
          InstanceSubjects::family(&mut families);

          let subj = InferenceSubject::new(Some(0), 0, Some(42));
          let subj_idx = subjects.len();
          subjects.push(subj);

          assert_eq!(InstanceSubjects::to_instance(subj_idx, &subjects), Some(42));
      }

      #[test]
      fn test_instance_subjects_to_instance_wrong_family() {
          let subjects = vec![InferenceSubject::new_fundamental(None, "model world")];
          // A fundamental subject has family 0 (fundamentals), not INSTANCES_FAMILY
          assert_eq!(InstanceSubjects::to_instance(0, &subjects), None);
      }
  }
  ```

## Success Criteria

- [ ] `Instance` struct exists with `name`, `as_adjective`, `as_subject`, and `enumeration_index` fields
- [ ] `Instances::new()` creates an instance and its inference subject
- [ ] `Instances::get_name()` returns the instance name
- [ ] `Instances::as_subject()` returns the instance's subject index
- [ ] `Instances::as_adjective()` returns the instance's adjective index
- [ ] `Instances::to_kind()` deduces the kind from the subject hierarchy
- [ ] `Instances::get_numerical_value()` returns the enumeration index
- [ ] `InstanceSubjects::family()` creates the instances family with certainty, get_name, make_adj_const_domain, and new_permission_granted methods
- [ ] `InstanceSubjects::new()` creates an inference subject for an instance
- [ ] `InstanceSubjects::to_instance()` extracts the instance index from a subject
- [ ] `InstanceAdjectives::start()` creates the `enumerative_amf` family with the assert method
- [ ] `InstanceAdjectives::is_enumerative()` correctly identifies enumerative meanings
- [ ] `InstanceAdjectives::make_adjectival()` declares an adjective, creates a meaning, adds it to the adjective, and sets the domain
- [ ] `InstanceAdjectives::assert()` with positive parity calls `PropertyInferences::draw`
- [ ] `InstanceAdjectives::assert()` with negative parity returns FALSE
- [ ] Module declarations added to `mod.rs`
- [ ] All unit tests pass with `cargo test`

## Out of Scope

- **Measurement Adjectives** (`Chapter 3/Measurement Adjectives.w`) — requires `measurement_definition` struct and grammar parsing; deferred to a later plan
- **Comparative Relations** (`Chapter 3/Comparative Relations.w`) — depends on measurement adjectives; deferred
- **I6 schema generation** — `RTAdjectives::make_mdef_test_schema`, `RTProperties::write_either_or_schemas`, `RTInferences::index_either_or` — run-time compilation deferred
- **Preform grammar** — `<measurement-adjective-definition>`, `<measurement-range>`, `<s-object-instance>`, `<s-non-object-instance>` — grammar parsing deferred
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
