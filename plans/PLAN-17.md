# Plan 17: Knowledge Module Foundation — Inference Subjects, Inferences, and Property Permissions
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the core data structures of the knowledge module — the world model that stores and reconciles facts about the game world. This includes inference subjects (the unified representation for anything a proposition can discuss), inference subject families (method dispatch for different subject types), inferences (single facts with certainty levels), inference families (method dispatch for different inference types), and property permissions (the bridge between subjects and properties).

This is the smallest next step after PLAN-16 because:

1. **The calculus module is complete.** PLAN-16 implemented `PcalcTerm`, `PcalcFunc`, `AtomElement`, `PcalcProp`, `PredicateRef`, `QuantifierRef`, proposition operations (conjunction, negation, quantification, validity, copy), `UnaryPredicate`, `UpFamily`, and `KindPredicates`. The predicate calculus engine is ready for downstream consumers.

2. **The knowledge module is the next layer in the dependency chain.** The architecture is: kinds → calculus → knowledge. The knowledge module (`inform7/knowledge-module/`) builds on the calculus module to represent facts about the game world. The central data structure is the `inference_subject` — anything in the model that a proposition can discuss (kinds, instances, variables, relations). Before we can implement instances, properties, or the assertion pipeline, we need the inference subject and inference data structures.

3. **Independently testable.** The core data structures — `InferenceSubject`, `InferenceSubjectFamily`, `Inference`, `InferenceFamily`, `PropertyPermission` — can be constructed and tested in isolation. We can create subjects, build the subject hierarchy, draw inferences with varying certainty, compare inferences, and grant property permissions, all without needing the full assertion pipeline.

4. **Prerequisite for instances.** `Instances` (knowledge-module/Chapter 2/Instances.w) creates an `inference_subject` for each instance via `InstanceSubjects::new`. The instance struct has an `as_subject` field. Without inference subjects, we can't create instances.

5. **Prerequisite for properties.** `Properties` (knowledge-module/Chapter 3/Properties.w) uses `property_permission` structures to track which subjects can have which properties. `PropertyPermissions::find` and `PropertyPermissions::grant` are the core operations. Without property permissions, we can't attach properties to subjects.

6. **Prerequisite for kind subjects.** `KindSubjects` (knowledge-module/Chapter 4/Kind Subjects.w) creates inference subjects for base kinds, bridging the kind system to the knowledge module. This is how kinds get properties and inferences.

## Background

### C reference architecture

The knowledge module is defined across several files in `inform7/knowledge-module/`:

#### Inference Subjects (`Chapter 4/Inference Subjects.w`, lines 1-551)

The central data structure of the knowledge module. An "inference subject" is anything about which an inference can be drawn — kinds, instances, variables, and relations. Subjects form a hierarchy (a DAG) where narrower subjects inherit from broader ones.

```c
typedef struct inference_subject {
    struct inference_subject *broader_than; /* going up in the hierarchy */
    struct inference_subject_family *infs_family;
    struct general_pointer represents; /* family-specific data */
    void *additional_data_for_plugins[MAX_PLUGINS];
    struct linked_list *inf_list; /* contingently true: each inference drawn about this */
    struct linked_list *imp_list; /* necessarily true: each implication applying to this */
    struct linked_list *permissions_list; /* of property_permission */
    struct assemblies_data assemblies; /* what generalisations have been made about this? */
    struct nonlocal_variable *alias_variable; /* in the way that "player" aliases "yourself" */
    struct parse_node *infs_created_at; /* which sentence created this */
    char *infs_name_in_log; /* solely to make the debugging log more legible */
    CLASS_DEFINITION
} inference_subject;
```

Key operations:
- `InferenceSubjects::new` — creates a new subject with a family, broader subject, and family-specific data
- `InferenceSubjects::new_fundamental` — creates a fundamental subject (model_world, global_variables, etc.)
- `InferenceSubjects::is_within` — tests if one subject is within (contained by) another
- `InferenceSubjects::falls_within` — demotes a subject to a new broader subject
- `InferenceSubjects::narrowest_broader_subject` — returns the immediate broader subject
- `InferenceSubjects::get_inferences` / `get_implications` / `get_permissions` — access the subject's lists
- `InferenceSubjects::get_name_text` — gets the name of a subject via family method call
- `InferenceSubjects::get_default_certainty` — gets the default certainty for a subject via family method call

#### Inference Subject Families (`Chapter 4/Inference Subjects.w`, lines 14-24)

Each subject belongs to a family that provides method dispatch:

```c
typedef struct inference_subject_family {
    struct method_set *methods;
    CLASS_DEFINITION
} inference_subject_family;
```

Methods (defined as method IDs):
- `GET_NAME_TEXT_INFS_MTID` — fill in a wording with the subject's name
- `GET_DEFAULT_CERTAINTY_INFS_MTID` — return the default certainty level
- `NEW_PERMISSION_GRANTED_INFS_MTID` — called when a new property permission is granted
- `MAKE_ADJ_CONST_DOMAIN_INFS_MTID` — called to make a subject the domain of an adjectival constant
- `COMPLETE_MODEL_INFS_MTID` — called during model completion
- `CHECK_MODEL_INFS_MTID` — called during model checking
- `EMIT_ELEMENT_INFS_MTID` — compile run-time code to test if a value is a constant of this subject
- `EMIT_ALL_INFS_MTID` / `EMIT_ONE_INFS_MTID` — compile all/one subject's run-time data

#### Inferences (`Chapter 5/Inferences.w`, lines 1-402)

A lightweight structure representing a single fact about the world model:

```c
typedef struct inference {
    struct inference_family *family; /* see above */
    general_pointer data; /* details specific to the family */
    int certainty; /* any *_CE value other than UNKNOWN_CE */
    struct parse_node *inferred_from; /* from what sentence was this drawn? */
    int drawn_during_stage; /* or was this drawn during the model completion stage? */
    int drawn_from_metadata; /* or from the project's metadata file? */
    CLASS_DEFINITION
} inference;
```

Certainty levels:
- `IMPOSSIBLE_CE` (-2) — known to be false
- `UNLIKELY_CE` (-1) — unlikely to be true
- `UNKNOWN_CE` (0) — no information
- `LIKELY_CE` (1) — likely to be true
- `INITIALLY_CE` (2) — initially true (for start-of-play state)
- `CERTAIN_CE` (3) — certainly true

Key operations:
- `Inferences::create_inference` — creates a new inference with a family, data, and certainty
- `Inferences::join_inference` — adds an inference to a subject's list, handling contradictions and duplicates
- `Inferences::cmp` — compares two inferences for sorting (returns CI_* values)
- `Inferences::get_certainty` / `where_inferred` / `get_inference_type` — accessors
- `Inferences::render_impossible` — marks an inference as impossible

#### Inference Families (`Chapter 5/Inferences.w`, lines 340-401)

Every inference belongs to a family with method dispatch:

```c
typedef struct inference_family {
    struct method_set *methods;
    struct text_stream *log_name;
    CLASS_DEFINITION
} inference_family;
```

Methods:
- `LOG_DETAILS_INF_MTID` — log family-specific details
- `COMPARE_INF_MTID` — family-specific comparison for sorting
- `EXPLAIN_CONTRADICTION_INF_MTID` — issue a better-phrased contradiction problem message

#### Property Permissions (`Chapter 4/Property Permissions.w`, lines 1-146)

The bridge between subjects and properties — records which subjects can have which properties:

```c
typedef struct property_permission {
    struct inference_subject *property_owner; /* to whom permission is granted */
    struct property *property_granted; /* which property is permitted */
    struct parse_node *where_granted; /* sentence granting the permission */
    struct general_pointer pp_storage_data; /* how we'll compile this at run-time */
    void *plugin_pp[MAX_PLUGINS]; /* storage for plugins to attach */
    struct property_permission_compilation_data compilation_data;
    CLASS_DEFINITION
} property_permission;
```

Key operations:
- `PropertyPermissions::find` — finds a permission for a subject/property pair (with inheritance)
- `PropertyPermissions::grant` — grants a new permission, creating it if it doesn't exist
- `PropertyPermissions::get_property` / `get_subject` / `get_storage_data` / `where_granted` — accessors

### Key C source files

- `inform7/knowledge-module/Chapter 4/Inference Subjects.w` — `inference_subject` struct, creation, hierarchy, methods, logging
- `inform7/knowledge-module/Chapter 5/Inferences.w` — `inference` struct, creation, joining, comparison, families
- `inform7/knowledge-module/Chapter 4/Property Permissions.w` — `property_permission` struct, find, grant, accessors
- `inform7/knowledge-module/Preliminaries/What This Module Does.w` — overview of the knowledge module
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module setup, startup
- `inform7/knowledge-module/Chapter 1/Assert Propositions.w` — how propositions become inferences (future work)

### Current Rust state

- `crates/conform7-semantics/src/calculus/` — Complete calculus module with terms, atoms, propositions, unary predicates, families, and kind predicates
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with construction, equality, conformance, compatibility, Display, FromStr
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct with all Behaviour API fields
- `crates/conform7-semantics/src/familiar_kinds.rs` — All `K_*` and `CON_*` global constants
- `crates/conform7-semantics/src/lattice.rs` — `superkind`, `join`, `meet` functions
- `crates/conform7-semantics/src/kinds_behaviour.rs` — Full `Kinds::Behaviour` API (~40 functions)

### What's needed

1. **`InferenceSubject` struct** — the central data structure with broader_than, family, represents, inf_list, imp_list, permissions_list, assemblies, alias_variable, created_at, log_name.
2. **`InferenceSubjectFamily` struct** — family with method dispatch for name, certainty, permissions, model completion, model checking, compilation.
3. **`InferenceSubjectFamilyMethods`** — method table with function pointers for all inference subject family methods.
4. **Subject hierarchy operations** — `new`, `new_fundamental`, `is_within`, `falls_within`, `narrowest_broader_subject`, accessors.
5. **`Inference` struct** — lightweight structure with family, data, certainty, inferred_from, drawn_during_stage, drawn_from_metadata.
6. **Certainty levels** — `IMPOSSIBLE_CE`, `UNLIKELY_CE`, `UNKNOWN_CE`, `LIKELY_CE`, `INITIALLY_CE`, `CERTAIN_CE`.
7. **`InferenceFamily` struct** — family with method dispatch for logging, comparison, contradiction explanation.
8. **`InferenceFamilyMethods`** — method table with function pointers for all inference family methods.
9. **Inference operations** — `create_inference`, `join_inference`, `cmp`, accessors, `render_impossible`.
10. **`PropertyPermission` struct** — bridge between subjects and properties with owner, property, where_granted, storage_data.
11. **Property permission operations** — `find`, `grant`, accessors.
12. **Unit tests** — construct subjects, build hierarchy, draw inferences, compare inferences, grant permissions.

## Tasks

### 1. Create the knowledge module foundation

- [ ] Create `crates/conform7-semantics/src/knowledge/` directory with `mod.rs`:
  ```rust
  pub mod inference_subjects;
  pub mod inferences;
  pub mod property_permissions;
  ```

- [ ] Add `pub mod knowledge;` to `crates/conform7-semantics/src/lib.rs`.

### 2. Implement `InferenceSubject` and `InferenceSubjectFamily`

- [ ] Create `crates/conform7-semantics/src/knowledge/inference_subjects.rs` with:

  ```rust
  /// Maximum number of plugins that can attach data to a subject.
  pub const MAX_PLUGINS: usize = 8;

  /// An inference subject — anything about which an inference can be drawn.
  ///
  /// Corresponds to `inference_subject` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 111-128).
  ///
  /// Subjects form a hierarchy (a DAG) where narrower subjects inherit from
  /// broader ones. The top of the hierarchy is the `model_world` subject.
  #[derive(Clone, Debug)]
  pub struct InferenceSubject {
      /// The broader (more general) subject in the hierarchy, or None for the root.
      pub broader_than: Option<usize>,  // simplified: index into a subject registry
      /// The family this subject belongs to.
      pub infs_family: usize,  // simplified: index into a family registry
      /// Family-specific data (simplified: a string tag for now).
      pub represents: Option<&'static str>,
      /// List of inferences drawn about this subject (contingently true).
      pub inf_list: Vec<usize>,  // simplified: indices into an inference registry
      /// List of implications applying to this subject (necessarily true).
      pub imp_list: Vec<usize>,  // simplified: indices into an implication registry
      /// List of property permissions for this subject.
      pub permissions_list: Vec<usize>,  // simplified: indices into a permission registry
      /// Alias variable (for "player" aliasing "yourself").
      pub alias_variable: Option<&'static str>,
      /// Log name for debugging.
      pub log_name: Option<&'static str>,
  }
  ```

- [ ] Implement `InferenceSubjectFamily` (matching `inference_subject_family` in Inference Subjects.w lines 15-18):

  ```rust
  /// A family of related inference subjects.
  ///
  /// Corresponds to `inference_subject_family` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Inference Subjects.w`, lines 15-18).
  #[derive(Clone, Debug)]
  pub struct InferenceSubjectFamily {
      /// Name of this family (for debugging).
      pub name: &'static str,
      /// Method implementations for this family.
      pub methods: InferenceSubjectFamilyMethods,
  }

  /// Methods that can be implemented for an inference subject family.
  #[derive(Clone, Debug)]
  pub struct InferenceSubjectFamilyMethods {
      /// Get the name text of a subject.
      pub get_name_text: fn(&InferenceSubject) -> Option<&'static str>,
      /// Get the default certainty for a subject.
      pub get_default_certainty: fn(&InferenceSubject) -> i8,
      /// Called when a new property permission is granted.
      pub new_permission_granted: fn(&InferenceSubject, usize),
      /// Called to make a subject the domain of an adjectival constant.
      pub make_adj_const_domain: fn(&InferenceSubject, usize, usize),
      /// Called during model completion.
      pub complete_model: fn(&InferenceSubject),
      /// Called during model checking.
      pub check_model: fn(&InferenceSubject),
  }
  ```

- [ ] Implement subject creation functions (matching InferenceSubjects::new, new_fundamental, make_built_in in Inference Subjects.w lines 99-165):
  - `InferenceSubject::new(family: usize, broader_than: Option<usize>, represents: Option<&'static str>, log_name: Option<&'static str>) -> Self` — creates a new subject.
  - `InferenceSubject::new_fundamental(broader_than: Option<usize>, log_name: &'static str) -> Self` — creates a fundamental subject with the fundamentals family.

- [ ] Implement subject hierarchy operations (matching Inference Subjects.w lines 210-242):
  - `InferenceSubject::is_within(&self, larger: &InferenceSubject, registry: &[InferenceSubject]) -> bool` — tests if this subject is within (contained by) another.
  - `InferenceSubject::is_strictly_within(&self, larger: &InferenceSubject, registry: &[InferenceSubject]) -> bool` — tests if this subject is strictly within another.
  - `InferenceSubject::narrowest_broader_subject(&self) -> Option<usize>` — returns the immediate broader subject index.
  - `InferenceSubject::falls_within(&mut self, broad: usize)` — demotes this subject to a new broader subject.

- [ ] Implement subject accessors (matching Inference Subjects.w lines 247-267):
  - `InferenceSubject::get_inferences(&self) -> &[usize]` — returns the list of inference indices.
  - `InferenceSubject::get_implications(&self) -> &[usize]` — returns the list of implication indices.
  - `InferenceSubject::get_permissions(&self) -> &[usize]` — returns the list of permission indices.

- [ ] Implement subject method dispatch (matching Inference Subjects.w lines 365-463):
  - `InferenceSubject::get_name_text(&self, families: &[InferenceSubjectFamily]) -> Option<&'static str>` — dispatches to the family's get_name_text method.
  - `InferenceSubject::get_default_certainty(&self, families: &[InferenceSubjectFamily]) -> i8` — dispatches to the family's get_default_certainty method.
  - `InferenceSubject::new_permission_granted(&self, families: &[InferenceSubjectFamily], pp: usize)` — dispatches to the family's new_permission_granted method.

- [ ] Implement the fundamentals family (matching Inference Subjects.w lines 35-55):
  - `InferenceSubjectFamily::fundamentals() -> Self` — creates the fundamentals family with default method implementations.

- [ ] Add unit tests:
  - Test that `InferenceSubject::new` creates a subject with the correct family and broader_than.
  - Test that `InferenceSubject::new_fundamental` creates a fundamental subject.
  - Test that `is_within` returns true for subjects in the hierarchy.
  - Test that `is_strictly_within` returns false for the same subject.
  - Test that `falls_within` correctly demotes a subject.
  - Test that `narrowest_broader_subject` returns the correct broader subject.
  - Test that method dispatch works through the family.
  - Test the fundamentals family default implementations.

### 3. Implement `Inference` and `InferenceFamily`

- [ ] Create `crates/conform7-semantics/src/knowledge/inferences.rs` with:

  ```rust
  /// Certainty levels for inferences (matching the *_CE constants in Inferences.w).
  ///
  /// Corresponds to the certainty values in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Inferences.w`).
  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
  pub enum Certainty {
      /// Known to be false.
      Impossible = -2,
      /// Unlikely to be true.
      Unlikely = -1,
      /// No information.
      Unknown = 0,
      /// Likely to be true.
      Likely = 1,
      /// Initially true (for start-of-play state).
      Initially = 2,
      /// Certainly true.
      Certain = 3,
  }

  /// Comparison result values for inference comparison (matching CI_* constants).
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum InferenceComparison {
      /// One exists, the other doesn't.
      DifferInExistence = 1,
      /// Different families.
      DifferInFamily = 2,
      /// Different topics.
      DifferInTopic = 3,
      /// Different Boolean content.
      DifferInBooleanContent = 4,
      /// Different content.
      DifferInContent = 5,
      /// Different but duplicate inferences.
      DifferInCopyOnly = 6,
      /// Pointers to the same inference.
      Identical = 0,
  }
  ```

- [ ] Implement `Inference` struct (matching `inference` in Inferences.w lines 10-18):

  ```rust
  /// A single fact about the world model.
  ///
  /// Corresponds to `inference` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 10-18).
  #[derive(Clone, Debug)]
  pub struct Inference {
      /// The family this inference belongs to.
      pub family: usize,  // simplified: index into a family registry
      /// Family-specific data (simplified: a string tag for now).
      pub data: Option<&'static str>,
      /// The certainty of this inference.
      pub certainty: Certainty,
      /// Where this inference was drawn from (simplified: a string tag).
      pub inferred_from: Option<&'static str>,
      /// The building stage during which this was drawn.
      pub drawn_during_stage: i32,
      /// Whether this was drawn from the project's metadata file.
      pub drawn_from_metadata: bool,
  }
  ```

- [ ] Implement `InferenceFamily` (matching `inference_family` in Inferences.w lines 345-349):

  ```rust
  /// A family of related inferences.
  ///
  /// Corresponds to `inference_family` in the C reference
  /// (`inform7/knowledge-module/Chapter 5/Inferences.w`, lines 345-349).
  #[derive(Clone, Debug)]
  pub struct InferenceFamily {
      /// Name of this family (for debugging).
      pub name: &'static str,
      /// Method implementations for this family.
      pub methods: InferenceFamilyMethods,
  }

  /// Methods that can be implemented for an inference family.
  #[derive(Clone, Debug)]
  pub struct InferenceFamilyMethods {
      /// Log family-specific details.
      pub log_details: fn(&Inference) -> String,
      /// Compare two inferences from this family.
      pub compare: fn(&Inference, &Inference) -> i32,
      /// Explain a contradiction (returns true if handled).
      pub explain_contradiction: fn(&Inference, &Inference, i32, usize) -> bool,
  }
  ```

- [ ] Implement inference creation (matching Inferences::create_inference in Inferences.w lines 25-38):
  - `Inference::new(family: usize, data: Option<&'static str>, certainty: Certainty) -> Self` — creates a new inference. If certainty is Unknown, defaults to Certain.

- [ ] Implement inference accessors (matching Inferences.w lines 45-67):
  - `Inference::get_certainty(&self) -> Certainty`
  - `Inference::where_inferred(&self) -> Option<&'static str>`
  - `Inference::get_family(&self) -> usize`
  - `Inference::render_impossible(&mut self)` — sets certainty to Impossible.

- [ ] Implement inference comparison (matching Inferences::cmp in Inferences.w lines 112-127):
  - `Inference::cmp(&self, other: &Inference, families: &[InferenceFamily]) -> InferenceComparison` — compares two inferences, returning the appropriate CI_* value.

- [ ] Implement inference joining (matching Inferences::join_inference in Inferences.w lines 180-199):
  - `Inference::join(&self, subject: &mut InferenceSubject, families: &[InferenceFamily]) -> JoinResult` — attempts to join this inference to a subject's inference list, handling contradictions and duplicates.

  ```rust
  /// Result of joining an inference to a subject.
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum JoinResult {
      /// Inference was added to the list.
      Joined,
      /// Inference replaced an existing one.
      Replaced,
      /// Inference was discarded as redundant.
      DiscardedRedundant,
      /// Inference was discarded as a harmless contradiction.
      DiscardedContradiction,
      /// Inference was discarded because we already know better.
      DiscardedWeaker,
  }
  ```

- [ ] Add unit tests:
  - Test that `Inference::new` creates an inference with the correct family and certainty.
  - Test that `Inference::new` defaults Unknown to Certain.
  - Test that `render_impossible` sets certainty to Impossible.
  - Test that `cmp` returns Identical for the same inference.
  - Test that `cmp` returns DifferInFamily for different families.
  - Test that `cmp` returns DifferInExistence when one is None.
  - Test that `join` adds an inference to an empty list.
  - Test that `join` discards a weaker inference in favor of a stronger one.
  - Test that `join` replaces a weaker inference with a stronger one.
  - Test that `join` detects contradictions.
  - Test that `InferenceFamily` methods can be called on inferences.

### 4. Implement `PropertyPermission`

- [ ] Create `crates/conform7-semantics/src/knowledge/property_permissions.rs` with:

  ```rust
  /// A permission for a subject to have a property.
  ///
  /// Corresponds to `property_permission` in the C reference
  /// (`inform7/knowledge-module/Chapter 4/Property Permissions.w`, lines 24-35).
  #[derive(Clone, Debug)]
  pub struct PropertyPermission {
      /// The subject to whom permission is granted.
      pub property_owner: usize,  // simplified: index into a subject registry
      /// The property that is permitted (simplified: a string name for now).
      pub property_granted: &'static str,
      /// Where this permission was granted (simplified: a string tag).
      pub where_granted: Option<&'static str>,
      /// Storage data for compilation (simplified: a string tag).
      pub storage_data: Option<&'static str>,
  }
  ```

- [ ] Implement permission operations (matching PropertyPermissions::find, grant in Property Permissions.w lines 52-118):
  - `PropertyPermission::find(subject: &InferenceSubject, property: &str, subjects: &[InferenceSubject], allow_inheritance: bool) -> Option<usize>` — finds a permission for a subject/property pair, optionally following the subject hierarchy.
  - `PropertyPermission::grant(subject: &mut InferenceSubject, property: &'static str, where_granted: Option<&'static str>, subjects: &[InferenceSubject], permissions: &mut Vec<PropertyPermission>) -> usize` — grants a new permission, creating it if it doesn't exist.

- [ ] Implement permission accessors (matching Property Permissions.w lines 131-145):
  - `PropertyPermission::get_property(&self) -> &'static str`
  - `PropertyPermission::get_owner(&self) -> usize`
  - `PropertyPermission::get_storage_data(&self) -> Option<&'static str>`
  - `PropertyPermission::where_granted(&self) -> Option<&'static str>`

- [ ] Add unit tests:
  - Test that `grant` creates a new permission.
  - Test that `find` finds an existing permission.
  - Test that `find` follows the subject hierarchy when allow_inheritance is true.
  - Test that `find` returns None when no permission exists.
  - Test that `grant` returns the existing permission if one already exists.
  - Test accessor functions.

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `InferenceSubject` struct exists with broader_than, infs_family, represents, inf_list, imp_list, permissions_list, alias_variable, log_name fields.
- [ ] `InferenceSubjectFamily` struct exists with method dispatch for get_name_text, get_default_certainty, new_permission_granted, make_adj_const_domain, complete_model, check_model.
- [ ] `InferenceSubject::new` and `new_fundamental` create subjects correctly.
- [ ] `InferenceSubject::is_within` correctly tests subject containment in the hierarchy.
- [ ] `InferenceSubject::falls_within` correctly demotes a subject.
- [ ] `InferenceSubject::narrowest_broader_subject` returns the correct broader subject.
- [ ] `InferenceSubject::get_inferences`, `get_implications`, `get_permissions` return the correct lists.
- [ ] Method dispatch works through the family system.
- [ ] The fundamentals family exists with default implementations.
- [ ] `Certainty` enum exists with all six levels (Impossible, Unlikely, Unknown, Likely, Initially, Certain).
- [ ] `Inference` struct exists with family, data, certainty, inferred_from, drawn_during_stage, drawn_from_metadata fields.
- [ ] `InferenceFamily` struct exists with method dispatch for log_details, compare, explain_contradiction.
- [ ] `Inference::new` creates inferences correctly (defaults Unknown to Certain).
- [ ] `Inference::cmp` correctly compares inferences (Identical, DifferInFamily, DifferInExistence, etc.).
- [ ] `Inference::join` correctly adds inferences to a subject's list, handling contradictions and duplicates.
- [ ] `PropertyPermission` struct exists with property_owner, property_granted, where_granted, storage_data fields.
- [ ] `PropertyPermission::find` correctly finds permissions (with and without inheritance).
- [ ] `PropertyPermission::grant` correctly creates and reuses permissions.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Instance subjects**: `InstanceSubjects` (knowledge-module/Chapter 4/Instance Subjects.w) — the instances family of inference subjects — is deferred. We implement the generic subject infrastructure but not the instance-specific family.
- **Kind subjects**: `KindSubjects` (knowledge-module/Chapter 4/Kind Subjects.w) — the kinds family of inference subjects — is deferred. We implement the generic subject infrastructure but not the kind-specific family.
- **Variable subjects**: `VariableSubjects` (knowledge-module/Chapter 4/Variable Subjects.w) — the variables family of inference subjects — is deferred.
- **Relation subjects**: `RelationSubjects` (knowledge-module/Chapter 4/Relation Subjects.w) — the relations family of inference subjects — is deferred.
- **Conditions of subjects**: `Conditions of Subjects` (knowledge-module/Chapter 4/Conditions of Subjects.w) — conditions on subjects — is deferred.
- **Instances**: `Instances` (knowledge-module/Chapter 2/Instances.w) — the instance creation and management system — is deferred. This plan focuses on the inference subject infrastructure that instances will use.
- **Properties**: `Properties` (knowledge-module/Chapter 3/Properties.w) — the property creation and management system — is deferred. We implement the `PropertyPermission` bridge but not the full `property` struct.
- **Either-or properties**: `EitherOrProperties` (knowledge-module/Chapter 3/Either-Or Properties.w) — either/or property data — is deferred.
- **Valued properties**: `ValueProperties` (knowledge-module/Chapter 3/Valued Properties.w) — valued property data — is deferred.
- **Inferences families**: The specific inference families (property inferences, relation inferences, etc.) are deferred. We implement the generic `InferenceFamily` infrastructure but not the specific families.
- **Implications**: The `implication` struct and implication-specific logic are deferred. We include the `imp_list` field on subjects but don't implement the implication system.
- **Assemblies data**: The `assemblies_data` struct and generalisation system are deferred. We include the concept but don't implement the full system.
- **Plugin data**: The plugin attachment system (`additional_data_for_plugins`, `ATTACH_PLUGIN_DATA_TO_SUBJECT`) is deferred.
- **Model completion and checking**: `InferenceSubjects::complete_model` and `check_model` are deferred. We include the method slots but don't implement the model completion pipeline.
- **Compilation methods**: `EMIT_ELEMENT_INFS_MTID`, `EMIT_ALL_INFS_MTID`, `EMIT_ONE_INFS_MTID` — the compilation methods for subjects — are deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (knowledge-module/Chapter 1/Assert Propositions.w) — the assertion pipeline that converts propositions to inferences — is deferred.
- **The model world**: `The Model World` (knowledge-module/Chapter 5/The Model World.w) — the model world construction — is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
