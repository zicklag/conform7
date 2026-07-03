# Plan 34: Ordering Instances — The `OrderingInstances` Linked List
**Status**: Complete
**Target**: 1 day

## Goal

Implement the `OrderingInstances` module — a simple linked list system for placing instances in a defined order. This creates an ordered list of instances that is used not only for compilation, but also for instance counting (e.g., marking the black gate as the 8th instance of "door"), so it's needed earlier than the compilation phase.

This is the smallest next step after PLAN-33 because:

1. **It's the smallest remaining independent module in the knowledge module.** At only ~60 lines of C (`Ordering Instances.w`), this is the simplest piece of the knowledge module not yet implemented. It provides a linked list ordering of instances with `begin()`, `place_next()`, and `objects_in_definition_sequence()` — all straightforward operations on a `Vec<Option<usize>>` next-pointer array.

2. **It depends only on instances (PLAN-30, Complete).** The `OrderingInstances` module stores instance pointers in a linked list indexed by allocation ID. The `Instance` struct already exists with all necessary fields. No other dependencies are needed.

3. **It's independently testable without grammar parsing, the assertions module, or run-time compilation.** We can create instances, place them in order, and verify the ordering — all programmatically. No Preform grammar, no `Nouns`, no `Lexicon`, no `RTVariables` are needed.

4. **It's a prerequisite for downstream systems that need instance ordering.** The `The Model World` module (Chapter 5) uses `LOOP_THROUGH_INSTANCE_ORDERING` during world model completion. The `OrderingInstances` system is also used by the spatial plugin and the instance counting system. Building it now establishes the ordering infrastructure.

5. **It introduces the linked-list ordering pattern — a fundamental data structure.** The ordering is stored as a linked list with links in an array indexed by the allocation IDs of the instances. This is a simple but important pattern used throughout the compiler for ordering instances, and implementing it now establishes the pattern for future ordering needs.

6. **It's independently testable.** We can create instances, call `OrderingInstances::begin()` to initialise the list, call `OrderingInstances::place_next()` to add instances in order, and then iterate through the list to verify the ordering. We can also test `OrderingInstances::objects_in_definition_sequence()` which orders instances by their creation order.

## Background

### C reference architecture

#### Ordering Instances (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 1-58)

The Ordering Instances system provides a simple linked list for placing instances in order:

```c
instance *first_instance_in_list = NULL;
instance **next_instance_in_current_list = NULL;
instance *last_instance_in_list = NULL;
```

Key functions:

```c
void OrderingInstances::begin(void) {
    int i, nc = NUMBER_CREATED(instance);
    if (next_instance_in_current_list == NULL) {
        next_instance_in_current_list = (instance **)
            (Memory::calloc(nc, sizeof(instance *), OBJECT_COMPILATION_MREASON));
    }
    for (i=0; i<nc; i++) next_instance_in_current_list[i] = NULL;
    first_instance_in_list = NULL;
    last_instance_in_list = NULL;
}
```

```c
void OrderingInstances::place_next(instance *I) {
    if (last_instance_in_list == NULL)
        first_instance_in_list = I;
    else
        next_instance_in_current_list[last_instance_in_list->allocation_id] = I;
    last_instance_in_list = I;
}
```

```c
void OrderingInstances::objects_in_definition_sequence(void) {
    OrderingInstances::begin();
    instance *I;
    LOOP_OVER_INSTANCES(I, K_object)
        OrderingInstances::place_next(I);
}
```

Access macros:

```c
#define FIRST_IN_INSTANCE_ORDERING first_instance_in_list
#define NEXT_IN_INSTANCE_ORDERING(I) next_instance_in_current_list[I->allocation_id]
#define LOOP_THROUGH_INSTANCE_ORDERING(I)
    for (I=FIRST_IN_INSTANCE_ORDERING; I; I=NEXT_IN_INSTANCE_ORDERING(I))
```

### Key C source files

- `inform7/knowledge-module/Chapter 2/Ordering Instances.w` — `OrderingInstances` module, `begin`, `place_next`, `objects_in_definition_sequence`, `first_instance_in_list`, `next_instance_in_current_list`, `last_instance_in_list` (58 lines)
- `inform7/knowledge-module/Chapter 2/Instances.w` — `instance` struct with `allocation_id` field (PLAN-30, Complete)
- `inform7/knowledge-module/Chapter 5/The Model World.w` — uses `LOOP_THROUGH_INSTANCE_ORDERING` during world model completion (deferred)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/instances.rs` — `Instance` struct with `name`, `as_adjective`, `as_subject`, `enumeration_index`, `kind_coincident` fields; `Instances` management functions (`new`, `to_kind`, `as_subject`, `make_kind_coincident`, `get_name_in_play`, `get_constant_instance`, `is_kind_of`, `index_of_instance`, `instance_with_enumeration_index`); unit tests (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/instance_subjects.rs` — `InstanceSubjects` family, `InstanceSubjects::family()`, `InstanceSubjects::from_instance()`, `InstanceSubjects::to_instance()`, `InstanceSubjects::to_object_instance()`, unit tests (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/instance_adjectives.rs` — `InstanceAdjectives` module, `enumerative_amf` family, `start()`, `is_enumerative()`, `make_adjectival()`, `assert()`, unit tests (PLAN-30, Complete).
- `crates/conform7-semantics/src/knowledge/value_properties.rs` — `ValueProperties` module, `kind`, `set_kind`, `make_coincide_with_kind`, `coincides_with_kind`, `make_setting_bp`, `get_setting_bp`, `set_stored_relation`, `get_stored_relation`, `obtain`, `obtain_within_kind`, `can_name_coincide_with_kind`, `assert`, unit tests (PLAN-33, Complete).
- `crates/conform7-semantics/src/knowledge/measurement_adjectives.rs` — `MeasurementAdjectives` module, `measurement_amf` family, `start()`, `is_measurement()`, `claim_definition()`, `assert()`, `prepare_schemas()`, unit tests (PLAN-32, Complete).
- `crates/conform7-semantics/src/knowledge/measurements.rs` — `MeasurementDefinition` struct, `Measurements` management functions, `MEASURE_T_OR_LESS`, `MEASURE_T_EXACTLY`, `MEASURE_T_OR_MORE` constants, unit tests (PLAN-31, Complete).
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
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules (includes `pub mod value_properties;` from PLAN-33).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).

### What's needed

1. **`OrderingInstances` module** — a new module `ordering_instances` in the knowledge crate with:
   - `OrderingInstances::begin(instances)` — initialises the ordering linked list:
     - Creates a `next_instance` array (a `Vec<Option<usize>>`) with one entry per instance, all set to `None`
     - Resets `first_instance` and `last_instance` to `None`
   - `OrderingInstances::place_next(instance_idx, instances)` — adds an instance to the end of the list:
     - If `last_instance` is `None`, sets `first_instance` to this instance
     - Otherwise, sets `next_instance[last_instance]` to this instance
     - Sets `last_instance` to this instance
   - `OrderingInstances::objects_in_definition_sequence(instances)` — orders all object instances by definition sequence:
     - Calls `begin()` to reset the list
     - Iterates over all instances and places each one via `place_next()`
     - Simplified: iterates over all instances (not just `K_object` instances, since we don't have a kind system that distinguishes object instances yet)
   - `OrderingInstances::first()` — returns the index of the first instance in the ordering, or `None` if the list is empty
   - `OrderingInstances::next(instance_idx)` — returns the index of the next instance after the given one in the ordering, or `None` if it's the last
   - `OrderingInstances::iter()` — returns an iterator over the ordered instance indices
   - Global state: `first_instance: Option<usize>`, `last_instance: Option<usize>`, `next_instance: Vec<Option<usize>>`

2. **Integration with the knowledge module** — add the `ordering_instances` module declaration to the knowledge module's `mod.rs`.

3. **Unit tests** — test `OrderingInstances::begin` (resets the list), test `OrderingInstances::place_next` (adds instances in order, maintains first/last pointers), test `OrderingInstances::objects_in_definition_sequence` (orders instances by creation order), test `OrderingInstances::first` (returns the first instance, returns None for empty list), test `OrderingInstances::next` (returns the next instance, returns None for last), test `OrderingInstances::iter` (iterates through all ordered instances).

## Tasks

### 1. Create the `OrderingInstances` module

- [ ] Create `crates/conform7-semantics/src/knowledge/ordering_instances.rs` with:

  ```rust
  /// The Ordering Instances system — a simple linked list for placing instances in order.
  ///
  /// Corresponds to `OrderingInstances` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`).
  ///
  /// Instances are stored as a linked list with links in an array indexed by
  /// the instance's position in the instances vector. This ordering is used
  /// not only for compilation, but also for instance counting (e.g., marking
  /// the black gate as the 8th instance of "door"), so it's needed earlier
  /// than the compilation phase.
  ///
  /// Simplified:
  /// - No `NUMBER_CREATED` macro (uses `instances.len()`)
  /// - No `Memory::calloc` (uses `vec![None; instances.len()]`)
  /// - No `OBJECT_COMPILATION_MREASON` memory reason
  /// - No `LOOP_OVER_INSTANCES` macro (uses `instances.iter().enumerate()`)
  /// - No `K_object` filtering (iterates all instances)
  /// - No `allocation_id` field (uses vector index)
  /// - No macros (uses methods and an iterator)
  use crate::knowledge::instances::Instance;

  /// The ordering instances module.
  ///
  /// Corresponds to `OrderingInstances` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`).
  pub struct OrderingInstances;
  ```

- [ ] Define global state:

  ```rust
  /// The first instance in the ordering, if any.
  /// Corresponds to `first_instance_in_list` in the C reference.
  static mut FIRST_INSTANCE: Option<usize> = None;

  /// The last instance in the ordering, if any.
  /// Corresponds to `last_instance_in_list` in the C reference.
  static mut LAST_INSTANCE: Option<usize> = None;

  /// The next-instance pointers, indexed by instance index.
  /// Corresponds to `next_instance_in_current_list` in the C reference.
  static mut NEXT_INSTANCE: Vec<Option<usize>> = Vec::new();
  ```

- [ ] Implement `OrderingInstances::begin`:

  ```rust
  /// Initialise the ordering linked list.
  ///
  /// Corresponds to `OrderingInstances::begin` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 23-32).
  ///
  /// Creates a next-instance array with one entry per instance, all set to None.
  /// Resets first and last instance pointers.
  pub fn begin(instances: &[Instance]) {
      unsafe {
          NEXT_INSTANCE = vec![None; instances.len()];
          FIRST_INSTANCE = None;
          LAST_INSTANCE = None;
      }
  }
  ```

- [ ] Implement `OrderingInstances::place_next`:

  ```rust
  /// Add an instance to the end of the ordering list.
  ///
  /// Corresponds to `OrderingInstances::place_next` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 34-40).
  ///
  /// If the list is empty, sets the first instance to this one.
  /// Otherwise, links the last instance to this one.
  /// Then sets the last instance to this one.
  pub fn place_next(instance_idx: usize) {
      unsafe {
          if LAST_INSTANCE.is_none() {
              FIRST_INSTANCE = Some(instance_idx);
          } else {
              NEXT_INSTANCE[LAST_INSTANCE.unwrap()] = Some(instance_idx);
          }
          LAST_INSTANCE = Some(instance_idx);
      }
  }
  ```

- [ ] Implement `OrderingInstances::objects_in_definition_sequence`:

  ```rust
  /// Order all instances by definition sequence.
  ///
  /// Corresponds to `OrderingInstances::objects_in_definition_sequence` in the C reference
  /// (`inform7/knowledge-module/Chapter 2/Ordering Instances.w`, lines 45-50).
  ///
  /// Simplified: iterates over all instances (not just K_object instances).
  pub fn objects_in_definition_sequence(instances: &[Instance]) {
      Self::begin(instances);
      for i in 0..instances.len() {
          Self::place_next(i);
      }
  }
  ```

- [ ] Implement `OrderingInstances::first`:

  ```rust
  /// Return the index of the first instance in the ordering.
  ///
  /// Corresponds to `FIRST_IN_INSTANCE_ORDERING` in the C reference.
  pub fn first() -> Option<usize> {
      unsafe { FIRST_INSTANCE }
  }
  ```

- [ ] Implement `OrderingInstances::next`:

  ```rust
  /// Return the index of the next instance after the given one in the ordering.
  ///
  /// Corresponds to `NEXT_IN_INSTANCE_ORDERING(I)` in the C reference.
  pub fn next(instance_idx: usize) -> Option<usize> {
      unsafe { NEXT_INSTANCE.get(instance_idx).copied().flatten() }
  }
  ```

- [ ] Implement `OrderingInstances::iter`:

  ```rust
  /// Return an iterator over the ordered instance indices.
  ///
  /// Corresponds to `LOOP_THROUGH_INSTANCE_ORDERING(I)` in the C reference.
  pub fn iter() -> OrderingIterator {
      OrderingIterator { current: Self::first() }
  }

  /// An iterator over ordered instance indices.
  pub struct OrderingIterator {
      current: Option<usize>,
  }

  impl Iterator for OrderingIterator {
      type Item = usize;

      fn next(&mut self) -> Option<Self::Item> {
          let result = self.current;
          if let Some(idx) = self.current {
              self.current = Self::next(idx);
          }
          result
      }
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod ordering_instances;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Write unit tests

- [ ] Test `OrderingInstances::begin`:
  - Resets the list (first and last are None)
  - Creates a next-instance array with the right size
- [ ] Test `OrderingInstances::place_next`:
  - Adds instances in order
  - First call sets first_instance
  - Subsequent calls link correctly
  - Last call sets last_instance
- [ ] Test `OrderingInstances::objects_in_definition_sequence`:
  - Orders instances by creation order (vector index order)
  - All instances are in the list
- [ ] Test `OrderingInstances::first`:
  - Returns the first instance after placing
  - Returns None for an empty list
- [ ] Test `OrderingInstances::next`:
  - Returns the next instance in the ordering
  - Returns None for the last instance
- [ ] Test `OrderingInstances::iter`:
  - Iterates through all ordered instances
  - Returns the correct sequence
  - Works with an empty list (no iterations)

## Success Criteria

- [ ] `crates/conform7-semantics/src/knowledge/ordering_instances.rs` exists with all the functions listed above
- [ ] `crates/conform7-semantics/src/knowledge/mod.rs` includes `pub mod ordering_instances;`
- [ ] All unit tests pass: `cargo test -p conform7-semantics --lib knowledge::ordering_instances`
- [ ] The existing test suite still passes: `cargo test -p conform7-semantics`
- [ ] `OrderingInstances::begin` correctly initialises the ordering list
- [ ] `OrderingInstances::place_next` correctly adds instances to the end of the list
- [ ] `OrderingInstances::objects_in_definition_sequence` correctly orders instances by creation order
- [ ] `OrderingInstances::first` correctly returns the first instance (or None for empty list)
- [ ] `OrderingInstances::next` correctly returns the next instance (or None for last)
- [ ] `OrderingInstances::iter` correctly iterates through all ordered instances

## Out of Scope

- **`K_object` filtering in `objects_in_definition_sequence`** — the C reference only places instances of kind `K_object`. Simplified: iterates over all instances. This can be refined when the kind system is more complete.
- **`allocation_id` field** — the C reference uses `allocation_id` as an index into the next-instance array. Simplified: uses the instance's vector index directly.
- **`NUMBER_CREATED` macro** — the C reference uses `NUMBER_CREATED(instance)` to count instances. Simplified: uses `instances.len()`.
- **`Memory::calloc` and `OBJECT_COMPILATION_MREASON`** — the C reference allocates memory with a specific reason. Simplified: uses `vec![None; instances.len()]`.
- **`LOOP_OVER_INSTANCES` macro** — the C reference uses a macro to iterate over instances. Simplified: uses `0..instances.len()`.
- **Macros (`FIRST_IN_INSTANCE_ORDERING`, `NEXT_IN_INSTANCE_ORDERING`, `LOOP_THROUGH_INSTANCE_ORDERING`)** — the C reference provides macros for accessing the ordering. Simplified: uses methods and an iterator.
- **`The Model World` module** — the module that uses `LOOP_THROUGH_INSTANCE_ORDERING` during world model completion. Deferred to a later plan.
- **Spatial plugin** — the spatial plugin uses instance ordering for room/region ordering. Deferred.
- **Instance counting** — the system that uses instance ordering for counting (e.g., "the 8th instance of door"). Deferred.
- **`NonlocalVariables`** — the module that creates variables with inference subjects. Deferred (depends on `VariableSubjects`, `Nouns`, `RTVariables`).
- **`VariableSubjects`** — the inference subject family for variables. Deferred (depends on `NonlocalVariables`).
- **`ConditionsOfSubjects`** — the module that uses `ValueProperties::make_coincide_with_kind` for parsing condition sentences. Deferred (depends on `Propositions::Abstract` and `Assert` from the assertions module).
- **`ComparativeRelations`** — the module that uses `ValueProperties::coincides_with_kind` for typechecking. Deferred (depends on `Grading`/linguistics).
- **`The Naming Thicket`** — the naming properties system. Deferred (depends on `ValueProperties::assert`, `EitherOrProperties::assert`, `PropertyInferences`).
- **`Indefinite Appearance`** — the indefinite appearance property system. Deferred (depends on `ValueProperties::assert`, `InferenceSubjects`, `KindSubjects`).
- **`The Model World`** — the world model completion stages. Deferred (depends on many systems).
- **`Assert Propositions`** — the assertion engine. Deferred (depends on the assertions module, `Propositions::Abstract`, `UnaryPredicateFamilies`, `BinaryPredicateFamilies`).
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
