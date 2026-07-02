# Plan 18: Knowledge Module Setup and Kind Subjects
**Status**: Complete
**Target**: 1-2 days

## Goal

Implement the knowledge module setup (fundamental subjects) and Kind Subjects — the bridge between the kind system and the knowledge module. This creates the fundamental inference subjects (`model_world`, `global_constants`, `global_variables`) and gives every base kind its own inference subject, making it possible to draw inferences about kinds and connect the kind lattice to the subject hierarchy.

This is the smallest next step after PLAN-17 because:

1. **The knowledge module foundation is complete.** PLAN-17 implemented `InferenceSubject`, `InferenceSubjectFamily`, `Inference`, `InferenceFamily`, `Certainty`, and `PropertyPermission` with all core operations. What's missing is the specific families that connect these data structures to the rest of the system.

2. **Fundamental subjects are needed first.** Before any specific family (kinds, instances, variables, relations) can create subjects, the fundamental subjects must exist. `model_world` is the root of the subject hierarchy. `global_constants` and `global_variables` are used by `KindSubjects::new` and `PropertyPermissions::grant`. Without them, no subject can be created.

3. **Kind Subjects is the next family in the dependency chain.** The architecture is: kinds → calculus → knowledge → kind subjects → instances/properties. `KindSubjects` bridges the kind system to the knowledge module by creating an inference subject for each base kind. This is a prerequisite for both `Instances` (which calls `KindSubjects::from_kind`) and `Properties` (which grants permissions on kind subjects).

4. **Independently testable.** We can create fundamental subjects, build the kind subject hierarchy, and verify that the subject hierarchy mirrors the kind lattice. We can test `KindSubjects::from_kind`, `KindSubjects::to_kind`, `KindSubjects::has_properties`, and the callback functions that connect the lattice to the hierarchy.

5. **Prerequisite for instances.** `Instances::new` (Chapter 2/Instances.w) calls `InstanceSubjects::new(I, K)` which calls `KindSubjects::from_kind(K)`. `Instances::to_kind(I)` calls `KindSubjects::to_kind(inherits_from)`. Without Kind Subjects, we can't create instances.

6. **Prerequisite for properties.** `Properties::create` (Chapter 3/Properties.w) grants permissions on `model_world`, `global_constants`, and `global_variables` — all fundamental subjects. Without the knowledge module setup, we can't create properties.

## Background

### C reference architecture

#### Knowledge Module Setup (`Chapter 1/Knowledge Module.w`, lines 1-66)

The knowledge module setup creates fundamental subjects and registers writers/logging aspects:

```c
void KnowledgeModule::start(void) {
    PropertyInferences::start();
    RelationInferences::start();
    InstanceAdjectives::start();
    EitherOrPropertyAdjectives::start();
    MeasurementAdjectives::start();
    SameAsRelations::start();
    SettingPropertyRelations::start();
    ComparativeRelations::start();
    ProvisionRelation::start();
    REGISTER_WRITER('b', RuleBookings::log);
    REGISTER_WRITER('I', Inferences::log);
    REGISTER_WRITER('j', InferenceSubjects::log);
    REGISTER_WRITER('K', Rulebooks::log);
    REGISTER_WRITER('n', Inferences::log_family)
    REGISTER_WRITER('Y', Properties::log);
    Log::declare_aspect(ACTIVITY_CREATIONS_DA, L"activity creations", FALSE, FALSE);
    Log::declare_aspect(INFERENCES_DA, L"inferences", FALSE, TRUE);
    // ... more aspect declarations
}
```

The module also defines hooks that connect the calculus module to the knowledge module:

```c
#define TERM_DOMAIN_CALCULUS_TYPE struct inference_subject
#define TERM_DOMAIN_WORDING_FUNCTION InferenceSubjects::get_name_text
#define TERM_DOMAIN_TO_KIND_FUNCTION KindSubjects::to_kind
#define TERM_DOMAIN_FROM_KIND_FUNCTION KindSubjects::from_kind
```

#### Fundamental Subjects (`Chapter 4/Inference Subjects.w`, lines 35-55)

The fundamental subjects are created in `InferenceSubjects::start()`:

```c
inference_subject *model_world = NULL;
inference_subject *global_constants = NULL;
inference_subject *global_variables = NULL;

void InferenceSubjects::start(void) {
    model_world = InferenceSubjects::new_fundamental(NULL, I"model world");
    global_constants = InferenceSubjects::new_fundamental(model_world, I"global constants");
    global_variables = InferenceSubjects::new_fundamental(model_world, I"global variables");
}
```

The hierarchy is:
```
model_world (root)
  ├── global_constants
  └── global_variables
```

#### Kind Subjects (`Chapter 4/Kind Subjects.w`, lines 1-205)

Every base kind gets its own inference subject, making it possible to draw inferences from sentences such as:

> A scene has a number called the witness count. The witness count of a scene is usually 4.

```c
inference_subject_family *kinds_family = NULL;

inference_subject_family *KindSubjects::family(void) {
    if (kinds_family == NULL) {
        kinds_family = InferenceSubjects::new_family();
        METHOD_ADD(kinds_family, GET_DEFAULT_CERTAINTY_INFS_MTID,
            KindSubjects::certainty);
        METHOD_ADD(kinds_family, GET_NAME_TEXT_INFS_MTID,
            KindSubjects::get_name_text);
        METHOD_ADD(kinds_family, MAKE_ADJ_CONST_DOMAIN_INFS_MTID,
            KindSubjects::make_adj_const_domain);
        METHOD_ADD(kinds_family, NEW_PERMISSION_GRANTED_INFS_MTID,
            KindSubjects::new_permission_granted);
        METHOD_ADD(kinds_family, EMIT_ELEMENT_INFS_MTID, RTKindIDs::emit_element_of_condition);
    }
    return kinds_family;
}
```

Key operations:
- `KindSubjects::new(con)` — creates an inference subject for a base kind constructor, storing it in `con->base_as_infs`
- `KindSubjects::from_kind(K)` — returns the inference subject for a kind (via `K->construct->base_as_infs`)
- `KindSubjects::to_kind(infs)` — returns the kind for an inference subject (via `RETRIEVE_POINTER_kind_constructor(infs->represents)`)
- `KindSubjects::to_nonobject_kind(infs)` — same but returns NULL for object subkinds
- `KindSubjects::has_properties(K)` — true if K is an enumeration or object kind
- `KindSubjects::renew(K, super, W)` — matches an early inference subject to a newly created kind
- `KindSubjects::super(K)` — callback for the lattice: returns the superkind via the subject hierarchy
- `KindSubjects::move_within(sub, super)` — callback for the lattice: moves a kind within the subject hierarchy
- `KindSubjects::allow_sometimes(from)` — callback for the lattice: checks if a kind is within the object hierarchy
- `KindSubjects::new_base_kind_notify(K, super, d, W)` — callback called when a new base kind is created
- `KindSubjects::set_subkind_notify(sub, super)` — callback for subkind setting

The family methods:
- `KindSubjects::certainty` — returns `LIKELY_CE` (inferences about kinds are likely, not certain)
- `KindSubjects::get_name_text` — returns the kind's name
- `KindSubjects::new_permission_granted` — allocates run-time storage for the permission
- `KindSubjects::make_adj_const_domain` — registers an adjectival constant for the kind

### Key C source files

- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module setup, fundamental subject creation, calculus hooks
- `inform7/knowledge-module/Chapter 4/Inference Subjects.w` — `InferenceSubjects::start`, fundamental subject declarations (lines 35-55)
- `inform7/knowledge-module/Chapter 4/Kind Subjects.w` — `KindSubjects` family, `from_kind`, `to_kind`, callbacks (205 lines)
- `services/kinds-module/Chapter 2/The Lattice of Kinds.w` — `HIERARCHY_GET_SUPER_KINDS_CALLBACK`, `HIERARCHY_ALLOWS_SOMETIMES_MATCH_KINDS_CALLBACK`, `HIERARCHY_MOVE_KINDS_CALLBACK` (lines 89-119)
- `services/kinds-module/Chapter 2/Kinds.w` — `Kinds::base_construction`, `Kinds::new_base` (lines 77-95)
- `services/kinds-module/Chapter 4/Kind Constructors.w` — `kind_constructor` struct, `base_as_infs` field

### Current Rust state

- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct with `new`, `new_fundamental`, `is_within`, `falls_within`, `narrowest_broader_subject`, method dispatch. `InferenceSubjectFamily` struct with `new`, `fundamentals()`.
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `Certainty` enum, `create_inference`, `cmp`, `join`.
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct, `find`, `grant`, accessors.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for `inference_subjects`, `inferences`, `property_permissions`.
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with construction, equality, conformance, compatibility, Display, FromStr.
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct with all Behaviour API fields.
- `crates/conform7-semantics/src/familiar_kinds.rs` — All `K_*` and `CON_*` global constants.
- `crates/conform7-semantics/src/lattice.rs` — `superkind`, `join`, `meet` functions.

### What's needed

1. **Fundamental subject registry** — a way to store and access the fundamental subjects (`model_world`, `global_constants`, `global_variables`) by name.
2. **Knowledge module setup** — `KnowledgeModule::start()` that creates fundamental subjects and registers the calculus hooks.
3. **`KindSubjects` family** — a new `InferenceSubjectFamily` for kinds with methods for certainty (LIKELY_CE), name text, new permission granted, and make adj const domain.
4. **`KindSubjects::new(con)`** — creates an inference subject for a base kind constructor, storing the subject reference on the constructor.
5. **`KindSubjects::from_kind(K)`** — returns the inference subject for a kind via the constructor's stored subject reference.
6. **`KindSubjects::to_kind(infs)`** — returns the kind for an inference subject via the family-specific data.
7. **`KindSubjects::has_properties(K)`** — tests if a kind can have properties (enumeration or object).
8. **Lattice callbacks** — `super`, `move_within`, `allow_sometimes` that connect the kind lattice to the subject hierarchy.
9. **`KindSubjects::new_base_kind_notify`** — callback for when a new base kind is created.
10. **`KindSubjects::renew`** — matches an early inference subject to a newly created kind.
11. **`base_as_infs` field on `KindConstructor`** — stores the inference subject for a base kind constructor.
12. **Unit tests** — create fundamental subjects, build kind subjects, test from_kind/to_kind round-trips, test has_properties, test lattice callbacks.

## Tasks

### 1. Add `base_as_infs` field to `KindConstructor`

- [ ] Add a `base_as_infs: Option<usize>` field to `KindConstructor` in `crates/conform7-semantics/src/kind_constructors.rs`:
  ```rust
  /// The inference subject for this base kind constructor (if any).
  ///
  /// Corresponds to `base_as_infs` in the C reference
  /// (`services/kinds-module/Chapter 4/Kind Constructors.w`).
  /// Only set for base kind constructors (arity 0, not CON_KIND_VARIABLE, not CON_INTERMEDIATE).
  pub base_as_infs: Option<usize>,
  ```
- [ ] Initialize `base_as_infs` to `None` in `KindConstructor::new`.
- [ ] Add a setter method: `KindConstructor::set_base_as_infs(&mut self, infs: usize)`.
- [ ] Add a getter method: `KindConstructor::get_base_as_infs(&self) -> Option<usize>`.
- [ ] Add unit tests:
  - Test that `base_as_infs` defaults to `None`.
  - Test that `set_base_as_infs` and `get_base_as_infs` work correctly.

### 2. Create the knowledge module setup

- [ ] Create `crates/conform7-semantics/src/knowledge/setup.rs` with:
  ```rust
  /// The root of the inference subject hierarchy.
  ///
  /// Corresponds to `model_world` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 35-55).
  pub const MODEL_WORLD: usize = 0;

  /// Global constants subject (a child of model_world).
  pub const GLOBAL_CONSTANTS: usize = 1;

  /// Global variables subject (a child of model_world).
  pub const GLOBAL_VARIABLES: usize = 2;

  /// Create the fundamental subjects and set up the knowledge module.
  ///
  /// This must be called before any other knowledge module operations.
  ///
  /// Corresponds to `InferenceSubjects::start` and `KnowledgeModule::start`
  /// in the C reference.
  ///
  /// Returns (subjects, families) where:
  /// - subjects[0] = model_world (root)
  /// - subjects[1] = global_constants (child of model_world)
  /// - subjects[2] = global_variables (child of model_world)
  pub fn setup_knowledge_module() -> (Vec<InferenceSubject>, Vec<InferenceSubjectFamily>) {
      let fundamentals = InferenceSubjectFamily::fundamentals();
      let families = vec![fundamentals];

      let model_world = InferenceSubject::new_fundamental(None, "model world");
      let global_constants = InferenceSubject::new_fundamental(Some(0), "global constants");
      let global_variables = InferenceSubject::new_fundamental(Some(0), "global variables");

      let subjects = vec![model_world, global_constants, global_variables];
      (subjects, families)
  }
  ```
- [ ] Add `pub mod setup;` to `crates/conform7-semantics/src/knowledge/mod.rs`.
- [ ] Add unit tests:
  - Test that `setup_knowledge_module` creates three subjects.
  - Test that `model_world` has no broader subject (it's the root).
  - Test that `global_constants` and `global_variables` have `model_world` as their broader subject.
  - Test that all three subjects use the fundamentals family (index 0).
  - Test that `is_within` works for the fundamental hierarchy.

### 3. Create the `KindSubjects` family

- [ ] Create `crates/conform7-semantics/src/knowledge/kind_subjects.rs` with:

  ```rust
  /// The kinds family of inference subjects.
  ///
  /// Every base kind gets its own inference subject, making it possible to
  /// draw inferences from sentences such as:
  /// "A scene has a number called the witness count. The witness count of a
  /// scene is usually 4."
  ///
  /// Corresponds to `KindSubjects` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`).
  use crate::kind_constructors::KindConstructor;
  use crate::kinds::Kind;
  use crate::knowledge::inference_subjects::{
      InferenceSubject, InferenceSubjectFamily, InferenceSubjectFamilyMethods,
  };
  ```

- [ ] Implement `KindSubjects::family()` — creates the kinds family with method dispatch:
  ```rust
  pub fn family() -> InferenceSubjectFamily {
      InferenceSubjectFamily {
          name: "kinds",
          methods: InferenceSubjectFamilyMethods {
              get_name_text: |subject| {
                  // Retrieve the kind constructor from subject.represents
                  // and return its name
                  subject.log_name
              },
              get_default_certainty: |_| 1, // LIKELY_CE
              new_permission_granted: |_, _| {},
              make_adj_const_domain: |_, _, _| {},
              complete_model: |_| {},
              check_model: |_| {},
          },
      }
  }
  ```

  Note: The method implementations are simplified for now. `get_name_text` uses `log_name` as a stand-in. `new_permission_granted` and `make_adj_const_domain` are stubs (they need run-time compilation and instance adjectives respectively, which are out of scope).

- [ ] Implement `KindSubjects::new(con, subjects, families) -> usize` — creates an inference subject for a base kind constructor:
  ```rust
  /// Create an inference subject for a base kind constructor.
  ///
  /// Corresponds to `KindSubjects::new` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 43-52).
  ///
  /// Only creates subjects for base constructors (arity 0, not CON_KIND_VARIABLE,
  /// not CON_INTERMEDIATE). Returns None for other constructors.
  pub fn new(
      con: &KindConstructor,
      subjects: &mut Vec<InferenceSubject>,
      families: &[InferenceSubjectFamily],
  ) -> Option<usize> {
      // Only create subjects for base constructors (arity 0) that are not
      // kind variables or intermediate results.
      if con.arity != 0 || con.name == "kind variable" || con.name == "intermediate" {
          return None;
      }

      // Find the kinds family index
      let family_idx = families.iter().position(|f| f.name == "kinds")?;

      // The broader subject is global_constants (index 1 in the fundamental setup)
      let subject = InferenceSubject::new(
          family_idx,
          Some(1), // global_constants
          None,    // represents (simplified: no pointer storage yet)
          Some(con.name),
      );
      let idx = subjects.len();
      subjects.push(subject);
      Some(idx)
  }
  ```

- [ ] Implement `KindSubjects::from_kind(K, constructors) -> Option<usize>` — returns the inference subject index for a kind:
  ```rust
  /// Return the inference subject for a kind.
  ///
  /// Corresponds to `KindSubjects::from_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 58-61).
  pub fn from_kind(kind: &Kind, constructors: &[KindConstructor]) -> Option<usize> {
      constructors[kind.construct_id].base_as_infs
  }
  ```

  Note: This requires `Kind` to have a `construct_id` field (or we use the constructor's index). We'll need to adjust the approach based on how `Kind` stores its constructor reference.

- [ ] Implement `KindSubjects::to_kind(subject_idx, subjects, constructors) -> Option<Kind>` — returns the kind for an inference subject:
  ```rust
  /// Return the kind for an inference subject.
  ///
  /// Corresponds to `KindSubjects::to_kind` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 63-70).
  pub fn to_kind(
      subject_idx: usize,
      subjects: &[InferenceSubject],
      constructors: &[KindConstructor],
  ) -> Option<Kind> {
      let subject = &subjects[subject_idx];
      // Find the constructor whose base_as_infs matches this subject
      for (i, con) in constructors.iter().enumerate() {
          if con.base_as_infs == Some(subject_idx) {
              return Some(Kind::base_construction(i));
          }
      }
      None
  }
  ```

- [ ] Implement `KindSubjects::has_properties(kind, constructors) -> bool`:
  ```rust
  /// Test if a kind can have properties.
  ///
  /// Corresponds to `KindSubjects::has_properties` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 102-107).
  pub fn has_properties(kind: &Kind, constructors: &[KindConstructor]) -> bool {
      let con = &constructors[kind.construct_id];
      con.enumeration || con.object_kind
  }
  ```

- [ ] Implement `KindSubjects::superkind(subject_idx, subjects, constructors) -> Option<usize>` — callback for the lattice:
  ```rust
  /// Return the superkind inference subject for a kind's subject.
  ///
  /// Corresponds to `KindSubjects::super` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 159-162).
  pub fn superkind(
      subject_idx: usize,
      subjects: &[InferenceSubject],
      constructors: &[KindConstructor],
  ) -> Option<usize> {
      let subject = &subjects[subject_idx];
      subject.narrowest_broader_subject()
  }
  ```

- [ ] Implement `KindSubjects::move_within(subject_idx, super_subject_idx, subjects)` — callback for the lattice:
  ```rust
  /// Move a kind within the subject hierarchy.
  ///
  /// Corresponds to `KindSubjects::move_within` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 164-168).
  pub fn move_within(
      subject_idx: usize,
      super_subject_idx: usize,
      subjects: &mut [InferenceSubject],
  ) {
      subjects[subject_idx].falls_within(super_subject_idx);
  }
  ```

- [ ] Implement `KindSubjects::allow_sometimes(subject_idx, subjects, constructors) -> bool`:
  ```rust
  /// Check if a kind is within the object hierarchy.
  ///
  /// Corresponds to `KindSubjects::allow_sometimes` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`, lines 170-176).
  pub fn allow_sometimes(
      subject_idx: usize,
      subjects: &[InferenceSubject],
      constructors: &[KindConstructor],
  ) -> bool {
      // Walk up the subject hierarchy looking for K_object
      let mut current = Some(subject_idx);
      while let Some(idx) = current {
          if let Some(k) = KindSubjects::to_kind(idx, subjects, constructors) {
              if constructors[k.construct_id].name == "object" {
                  return true;
              }
          }
          current = subjects[idx].narrowest_broader_subject();
      }
      false
  }
  ```

- [ ] Add unit tests:
  - Test that `KindSubjects::family()` creates a family with name "kinds".
  - Test that the kinds family's `get_default_certainty` returns 1 (LIKELY_CE).
  - Test that `KindSubjects::new` creates a subject for a base constructor.
  - Test that `KindSubjects::new` returns None for kind variable and intermediate constructors.
  - Test that `KindSubjects::from_kind` returns the correct subject for a kind.
  - Test that `KindSubjects::to_kind` returns the correct kind for a subject.
  - Test `from_kind`/`to_kind` round-trip.
  - Test that `KindSubjects::has_properties` returns true for enumeration and object kinds.
  - Test that `KindSubjects::has_properties` returns false for non-enumeration, non-object kinds.
  - Test that `KindSubjects::superkind` follows the subject hierarchy.
  - Test that `KindSubjects::move_within` correctly moves a subject.
  - Test that `KindSubjects::allow_sometimes` returns true for object subkinds.

### 4. Wire up the lattice callbacks

- [ ] Add a `construct_id` field to `Kind` in `crates/conform7-semantics/src/kinds.rs`:
  ```rust
  /// Index into the constructor registry for this kind's constructor.
  pub construct_id: usize,
  ```
  This replaces or supplements the `construct: &'static KindConstructor` field, allowing us to look up constructors by index in a registry.

  Note: If `Kind` currently uses `&'static KindConstructor` (pointer), we may need to add `construct_id` alongside it, or switch to index-based lookup. The simplest approach is to add `construct_id` as an additional field and keep the pointer for backward compatibility.

- [ ] Update `Kind::base_construction`, `Kind::unary_con`, `Kind::binary_con`, etc. to set `construct_id` from the constructor's position in the registry.

- [ ] Add a `KindConstructorRegistry` type or use `&[KindConstructor]` slices to pass constructor registries through the API.

- [ ] Add unit tests:
  - Test that `Kind::construct_id` is set correctly after construction.
  - Test that the constructor registry lookup works.

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `base_as_infs: Option<usize>` field exists on `KindConstructor` with getter and setter.
- [ ] `setup_knowledge_module()` creates three fundamental subjects: `model_world` (root), `global_constants`, `global_variables`.
- [ ] Fundamental subjects form the correct hierarchy: `model_world` → `global_constants`, `model_world` → `global_variables`.
- [ ] `KindSubjects::family()` creates a family with `get_default_certainty` returning `LIKELY_CE` (1).
- [ ] `KindSubjects::new(con)` creates an inference subject for base kind constructors (arity 0, not variable/intermediate).
- [ ] `KindSubjects::new(con)` stores the subject index in `con.base_as_infs`.
- [ ] `KindSubjects::new(con)` returns `None` for kind variable and intermediate constructors.
- [ ] `KindSubjects::from_kind(K)` returns the correct subject index for a kind.
- [ ] `KindSubjects::to_kind(subject)` returns the correct kind for a subject.
- [ ] `from_kind`/`to_kind` round-trips correctly.
- [ ] `KindSubjects::has_properties(K)` returns `true` for enumeration and object kinds.
- [ ] `KindSubjects::has_properties(K)` returns `false` for non-enumeration, non-object kinds.
- [ ] `KindSubjects::superkind(subject)` follows the subject hierarchy to find the superkind.
- [ ] `KindSubjects::move_within(subject, super_subject)` correctly moves a subject in the hierarchy.
- [ ] `KindSubjects::allow_sometimes(subject)` returns `true` for object subkinds.
- [ ] `Kind` has a `construct_id` field for constructor registry lookup.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Instance Subjects**: `InstanceSubjects` (knowledge-module/Chapter 4/Instance Subjects.w) — the instances family of inference subjects — is deferred. This plan implements Kind Subjects only.
- **Variable Subjects**: `VariableSubjects` (knowledge-module/Chapter 4/Variable Subjects.w) — the variables family of inference subjects — is deferred.
- **Relation Subjects**: `RelationSubjects` (knowledge-module/Chapter 4/Relation Subjects.w) — the relations family of inference subjects — is deferred.
- **Conditions of Subjects**: `Conditions of Subjects` (knowledge-module/Chapter 4/Conditions of Subjects.w) — conditions on subjects — is deferred.
- **Instances**: `Instances` (knowledge-module/Chapter 2/Instances.w) — the instance creation and management system — is deferred. This plan provides the Kind Subjects prerequisite.
- **Properties**: `Properties` (knowledge-module/Chapter 3/Properties.w) — the property creation and management system — is deferred. This plan provides the fundamental subjects prerequisite.
- **Either-or properties**: `EitherOrProperties` (knowledge-module/Chapter 3/Either-Or Properties.w) — either/or property data — is deferred.
- **Valued properties**: `ValueProperties` (knowledge-module/Chapter 3/Valued Properties.w) — valued property data — is deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (knowledge-module/Chapter 1/Assert Propositions.w) — the assertion pipeline that converts propositions to inferences — is deferred.
- **The model world**: `The Model World` (knowledge-module/Chapter 5/The Model World.w) — the model world construction stages — is deferred.
- **Property inferences**: `PropertyInferences` (knowledge-module/Chapter 5/Property Inferences.w) — the property inference family — is deferred.
- **Relation inferences**: `RelationInferences` (knowledge-module/Chapter 5/Relation Inferences.w) — the relation inference family — is deferred.
- **The naming thicket**: `The Naming Thicket` (knowledge-module/Chapter 5/The Naming Thicket.w) — naming system — is deferred.
- **Indefinite appearance**: `Indefinite Appearance` (knowledge-module/Chapter 5/Indefinite Appearance.w) — indefinite appearance text — is deferred.
- **Nonlocal variables**: `NonlocalVariables` (knowledge-module/Chapter 2/Nonlocal Variables.w) — global variable management — is deferred.
- **Instance adjectives**: `Instances as Adjectives` (knowledge-module/Chapter 2/Instances as Adjectives.w) — adjectival forms of instances — is deferred.
- **Preform for instances**: `Preform for Instances` (knowledge-module/Chapter 2/Preform for Instances.w) — Preform grammar for instance names — is deferred.
- **Ordering instances**: `Ordering Instances` (knowledge-module/Chapter 2/Ordering Instances.w) — instance ordering — is deferred.
- **Measurement adjectives**: `MeasurementAdjectives` (knowledge-module/Chapter 3/Measurement Adjectives.w) — measurement adjectives — is deferred.
- **Measurements**: `Measurements` (knowledge-module/Chapter 3/Measurements.w) — measurement system — is deferred.
- **Comparative relations**: `ComparativeRelations` (knowledge-module/Chapter 3/Comparative Relations.w) — comparative relations — is deferred.
- **The provision relation**: `The Provision Relation` (knowledge-module/Chapter 3/The Provision Relation.w) — provision relation — is deferred.
- **Setting property relation**: `Setting Property Relation` (knowledge-module/Chapter 3/Setting Property Relation.w) — setting property relation — is deferred.
- **Same property relation**: `Same Property Relation` (knowledge-module/Chapter 3/Same Property Relation.w) — same property relation — is deferred.
- **Either-or property adjectives**: `Either-Or Property Adjectives` (knowledge-module/Chapter 3/Either-Or Property Adjectives.w) — either-or property adjectives — is deferred.
- **Plugin system**: The plugin attachment system (`additional_data_for_plugins`, `PluginCalls`) is deferred.
- **Run-time compilation**: All `RT*` functions (run-time compilation of subjects, permissions, instances, properties) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
