# Plan 15: Kinds::Behaviour API — High-Level Kind Queries and Operations
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the `Kinds::Behaviour` API — the high-level query and operation interface for kinds. This is the bridge between the raw kind data structures (PLAN-14) and the rest of the world model. Every part of the compiler that works with kinds — instances, properties, assertions, the calculus module — uses this API to ask questions like "Is this kind an object?", "Does this kind have named constants?", "Is this kind definite?", "What is the superkind of this kind?", and "Create a new enumerated value for this kind."

This is the smallest next step after PLAN-14 because:

1. **The kind data structures are complete.** PLAN-14 implemented `Kind`, `KindConstructor`, construction functions, equality, conformance, the lattice, familiar kinds, and textual I/O. What's missing is the high-level API that the rest of the compiler uses to interact with kinds.

2. **The Behaviour API is the public face of the kind system.** In the C reference, `Kinds::Behaviour` is the module that everything else imports. It provides ~40 functions that answer questions about kinds. Without it, no other module can meaningfully work with kinds.

3. **Independently testable.** Each Behaviour function has clear inputs and outputs. We can test definiteness, object-ness, enumeration status, quasinumerical status, name access, and conversion operations with known kinds from the familiar kinds module.

4. **Prerequisite for everything else.** Before we can implement instances ("create an instance of kind K"), properties ("add a property to kind K"), or assertions ("X is a kind of Y"), we need the Behaviour API to answer questions about kinds. For example, `Instances::new` calls `Kinds::Behaviour::is_object` and `Kinds::Behaviour::has_named_constant_values` — both Behaviour functions.

5. **Adds fields to `KindConstructor`.** Several Behaviour functions access fields on `KindConstructor` that were deferred in PLAN-14: `is_incompletely_defined`, `next_free_value`, `explicit_identifier`, `where_defined_in_source_text`, `constant_compilation_method`, `dimensional_form`, `dimensional_form_fixed`, `specification_text`, `index_priority`, `indexed_grey_if_empty`, `index_default_value`, `index_minimum_value`, `index_maximum_value`, `documentation_reference`, `can_exchange`, `distinguishing_routine`, `uses_block_values`, `small_block_size`, `heap_size_estimate`, `class_number`, `superkind_set_at`, `uses_signed_comparisons`, `comparison_fn_identifier`, `dim_rules`. Adding these fields now is the right time — before any downstream code depends on the constructor layout.

## Background

### C reference architecture

The `Kinds::Behaviour` API is defined in `services/kinds-module/Chapter 2/Using Kinds.w`. It provides the following categories of functions:

#### Names of kinds (lines 9-33)

- `Kinds::Behaviour::get_name(K, plural_form)` — returns the wording of the kind's name.
- `Kinds::Behaviour::get_name_in_play(K, plural_form, nl)` — returns the name in the language of play.
- `Kinds::Behaviour::get_noun(K)` — returns the noun for the kind.
- `Kinds::Behaviour::get_range_number(K)` / `set_range_number(K, r)` — range number for indexing.

#### Being an object (lines 38-57)

- `Kinds::Behaviour::is_object(K)` — true if K conforms to K_object and is not nil/void.
- `Kinds::Behaviour::is_subkind_of_object(K)` — true if K is a proper subkind of object.
- `Kinds::Behaviour::is_object_of_kind(K, L)` — true if K is an object and conforms to L.

#### Definiteness (lines 64-114)

- `Kinds::Behaviour::is_kind_of_kind(K)` — true if K's constructor is a protocol.
- `Kinds::Behaviour::definite(K)` — true if K is definite (not a protocol, not a variable, and all children are definite).
- `Kinds::Behaviour::semidefinite(K)` — true if K is semidefinite (definite or involves a kind variable).
- `Kinds::Behaviour::involves_var(K, v)` — true if K involves kind variable v.

#### How this came into being (lines 125-190)

- `Kinds::Behaviour::is_built_in(K)` — true if K was created by a Neptune file (not in source text).
- `Kinds::Behaviour::get_creating_sentence(K)` — the parse node where K was defined.
- `Kinds::Behaviour::is_uncertainly_defined(K)` — true if K is incompletely defined.
- `Kinds::Behaviour::is_an_enumeration(K)` — true if K is an enumeration kind.
- `Kinds::Behaviour::convert_to_unit(K)` — convert K to a unit kind.
- `Kinds::Behaviour::convert_to_enumeration(K)` — convert K to an enumeration kind.
- `Kinds::Behaviour::convert_to_real(K)` — convert K to use real arithmetic.
- `Kinds::Behaviour::new_enumerated_value(K)` — create a new enumerated value (returns the next free value index).

#### Compatibility with other kinds (lines 198-206)

- `Kinds::Behaviour::set_superkind_set_at(K, S)` / `get_superkind_set_at(K)` — where the superkind was set.

#### How constant values are expressed (lines 214-230)

- `Kinds::Behaviour::has_named_constant_values(K)` — true if K has named constants (enumeration or uncertainly defined).
- `Kinds::Behaviour::get_constant_compilation_method(K)` — how constants of this kind are compiled.

#### Performing arithmetic (lines 238-295)

- `Kinds::Behaviour::uses_signed_comparisons(K)` — true if K uses signed comparisons.
- `Kinds::Behaviour::get_comparison_routine(K)` — the comparison routine identifier.
- `Kinds::Behaviour::is_quasinumerical(K)` — true if K supports arithmetic.
- `Kinds::Behaviour::get_dimensional_form(K)` — the dimensional form of K.
- `Kinds::Behaviour::test_if_derived(K)` / `now_derived(K)` — derived kind status.
- `Kinds::Behaviour::scale_factor(K)` — the scaling factor for K.
- `Kinds::Behaviour::get_dim_rules(K)` — the dimensional rules for K.

#### Identifier (lines 300-303)

- `Kinds::Behaviour::get_identifier(K)` — the explicit identifier for K.

#### Storing values at run-time (lines 313-352)

- `Kinds::Behaviour::uses_block_values(K)` — true if K uses block (pointer) values.
- `Kinds::Behaviour::get_small_block_size(K)` — the small block size.
- `Kinds::Behaviour::get_heap_size_estimate(K)` — the heap size estimate.
- `Kinds::Behaviour::get_distinguisher(K)` — the distinguishing routine.
- `Kinds::Behaviour::can_exchange(K)` — true if values of K can be serialized.

#### Indexing and documentation (lines 357-408)

- `Kinds::Behaviour::get_documentation_reference(K)` / `set_documentation_reference(K, dr)` — doc reference.
- `Kinds::Behaviour::get_index_default_value(K)` / `get_index_minimum_value(K)` / `get_index_maximum_value(K)` — index value bounds.
- `Kinds::Behaviour::get_index_priority(K)` — index priority.
- `Kinds::Behaviour::indexed_grey_if_empty(K)` — grey-out in index if empty.
- `Kinds::Behaviour::set_specification_text(K, desc)` / `get_specification_text(K)` — specification text.

### Key C source files

- `services/kinds-module/Chapter 2/Using Kinds.w` — The full `Kinds::Behaviour` API (~410 lines).
- `services/kinds-module/Chapter 4/Kind Constructors.w` — The `kind_constructor` struct fields accessed by Behaviour functions.
- `services/kinds-module/Chapter 2/Familiar Kinds.w` — Global `K_*` and `CON_*` variables used in tests.
- `services/kinds-module/Chapter 2/Kinds.w` — `Kinds::eq`, `Kinds::conforms_to`, `Kinds::compatible` used internally.
- `services/kinds-module/Chapter 2/The Lattice of Kinds.w` — `Latticework::super` used by `is_object` and friends.

### Current Rust state

- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with construction, equality, conformance, compatibility, Display, FromStr.
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct with group, arity, variance, tupling, definite, arithmetic, enumeration, object_kind flags.
- `crates/conform7-semantics/src/familiar_kinds.rs` — All `K_*` and `CON_*` global constants.
- `crates/conform7-semantics/src/lattice.rs` — `superkind`, `join`, `meet` functions.

### What's needed

1. **New fields on `KindConstructor`**: `is_incompletely_defined`, `next_free_value`, `explicit_identifier`, `where_defined_in_source_text`, `constant_compilation_method`, `dimensional_form`, `dimensional_form_fixed`, `specification_text`, `index_priority`, `indexed_grey_if_empty`, `index_default_value`, `index_minimum_value`, `index_maximum_value`, `documentation_reference`, `can_exchange`, `distinguishing_routine`, `uses_block_values`, `small_block_size`, `heap_size_estimate`, `class_number`, `superkind_set_at`, `uses_signed_comparisons`, `comparison_fn_identifier`, `dim_rules`.

2. **`KindsBehaviour` module** (or `kinds_behaviour.rs`): A module providing the Behaviour API as free functions and/or methods on `Kind` and `KindConstructor`.

3. **Definiteness queries**: `definite`, `semidefinite`, `involves_var`, `is_kind_of_kind`.

4. **Object queries**: `is_object`, `is_subkind_of_object`, `is_object_of_kind`.

5. **Definition status**: `is_built_in`, `is_uncertainly_defined`, `is_an_enumeration`, `convert_to_unit`, `convert_to_enumeration`, `convert_to_real`, `new_enumerated_value`.

6. **Constant value queries**: `has_named_constant_values`, `get_constant_compilation_method`.

7. **Arithmetic queries**: `is_quasinumerical`, `uses_signed_comparisons`, `get_comparison_routine`, `get_dimensional_form`, `test_if_derived`, `now_derived`, `scale_factor`, `get_dim_rules`.

8. **Name queries**: `get_name`, `get_name_in_play`, `get_noun`, `get_range_number`, `set_range_number`.

9. **Storage queries**: `uses_block_values`, `get_small_block_size`, `get_heap_size_estimate`, `get_distinguisher`, `can_exchange`.

10. **Indexing and documentation**: `get_documentation_reference`, `set_documentation_reference`, `get_index_default_value`, `get_index_minimum_value`, `get_index_maximum_value`, `get_index_priority`, `indexed_grey_if_empty`, `set_specification_text`, `get_specification_text`.

11. **Identifier**: `get_identifier`.

12. **Superkind tracking**: `set_superkind_set_at`, `get_superkind_set_at`.

13. **Unit tests**: Test each Behaviour function with known kinds from the familiar kinds module.

## Tasks

### 1. Add new fields to `KindConstructor`

The `KindConstructor` struct needs additional fields to support the Behaviour API. These fields were deferred in PLAN-14.

- [ ] Add the following fields to `KindConstructor` in `crates/conform7-semantics/src/kind_constructors.rs`:

  ```rust
  /// Whether this kind is incompletely defined (uncertainly defined).
  pub is_incompletely_defined: bool,

  /// Next free value index for enumeration kinds.
  pub next_free_value: i32,

  /// Explicit identifier for this kind (e.g., "K_number").
  pub explicit_identifier: Option<&'static str>,

  /// Where this kind was defined in source text (None for built-in kinds).
  pub where_defined_in_source_text: Option<usize>,

  /// How constants of this kind are compiled.
  pub constant_compilation_method: i32,

  /// Dimensional form for quasinumerical kinds.
  pub dimensional_form: Option<Box<UnitSequence>>,

  /// Whether the dimensional form is fixed (derived kind).
  pub dimensional_form_fixed: bool,

  /// Specification text for the index.
  pub specification_text: Option<&'static str>,

  /// Index priority.
  pub index_priority: i32,

  /// Whether to grey-out in index if empty.
  pub indexed_grey_if_empty: bool,

  /// Default value for the index.
  pub index_default_value: Option<&'static str>,

  /// Minimum value for the index.
  pub index_minimum_value: Option<&'static str>,

  /// Maximum value for the index.
  pub index_maximum_value: Option<&'static str>,

  /// Documentation reference.
  pub documentation_reference: Option<&'static str>,

  /// Whether values of this kind can be serialized.
  pub can_exchange: bool,

  /// Distinguishing routine name (None means ~= is sufficient).
  pub distinguishing_routine: Option<&'static str>,

  /// Whether this kind uses block (pointer) values.
  pub uses_block_values: bool,

  /// Small block size for block values.
  pub small_block_size: i32,

  /// Heap size estimate for block values.
  pub heap_size_estimate: i32,

  /// Range number for indexing.
  pub class_number: i32,

  /// Where the superkind was set.
  pub superkind_set_at: Option<usize>,

  /// Whether this kind uses signed comparisons.
  pub uses_signed_comparisons: bool,

  /// Comparison routine identifier.
  pub comparison_fn_identifier: Option<&'static str>,

  /// Dimensional rules for arithmetic operations.
  pub dim_rules: DimensionalRules,
  ```

- [ ] Add a `UnitSequence` type (simplified) for dimensional form representation:
  ```rust
  /// A unit sequence representing the dimensional form of a quasinumerical kind.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct UnitSequence {
      pub units: Vec<UnitEntry>,
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct UnitEntry {
      pub unit_kind: usize,  // index into a unit registry (simplified)
      pub exponent: i32,
  }
  ```

- [ ] Add a `DimensionalRules` type (simplified) for dimensional rule storage:
  ```rust
  /// Dimensional rules for arithmetic operations on a kind.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct DimensionalRules {
      pub rules: Vec<DimensionalRule>,
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct DimensionalRule {
      pub operation: DimensionalOp,
      pub result_kind: Option<usize>,  // simplified: kind index
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum DimensionalOp {
      Add,
      Subtract,
      Multiply,
      Divide,
      Root,
      Power,
  }
  ```

- [ ] Update `KindConstructor::new` to initialize all new fields to their default values (false/0/None).

- [ ] Add setter methods for the new fields:
  - `KindConstructor::set_incompletely_defined(&mut self, val: bool)`
  - `KindConstructor::set_explicit_identifier(&mut self, id: &'static str)`
  - `KindConstructor::set_constant_compilation_method(&mut self, method: i32)`
  - `KindConstructor::set_uses_block_values(&mut self, val: bool)`
  - `KindConstructor::set_small_block_size(&mut self, size: i32)`
  - `KindConstructor::set_heap_size_estimate(&mut self, size: i32)`
  - `KindConstructor::set_can_exchange(&mut self, val: bool)`
  - `KindConstructor::set_uses_signed_comparisons(&mut self, val: bool)`
  - `KindConstructor::set_comparison_fn_identifier(&mut self, id: &'static str)`
  - `KindConstructor::set_distinguishing_routine(&mut self, routine: &'static str)`
  - `KindConstructor::set_documentation_reference(&mut self, dr: &'static str)`
  - `KindConstructor::set_index_priority(&mut self, priority: i32)`
  - `KindConstructor::set_indexed_grey_if_empty(&mut self, val: bool)`
  - `KindConstructor::set_index_default_value(&mut self, val: &'static str)`
  - `KindConstructor::set_index_minimum_value(&mut self, val: &'static str)`
  - `KindConstructor::set_index_maximum_value(&mut self, val: &'static str)`
  - `KindConstructor::set_specification_text(&mut self, text: &'static str)`
  - `KindConstructor::set_class_number(&mut self, n: i32)`
  - `KindConstructor::set_dimensional_form(&mut self, form: UnitSequence)`
  - `KindConstructor::set_dimensional_form_fixed(&mut self, val: bool)`
  - `KindConstructor::set_dim_rules(&mut self, rules: DimensionalRules)`

- [ ] Add unit tests:
  - Test that new fields default to false/0/None.
  - Test that setter methods update the fields correctly.
  - Test that `UnitSequence` and `DimensionalRules` can be constructed.

### 2. Create the `KindsBehaviour` module

- [ ] Create `crates/conform7-semantics/src/kinds_behaviour.rs` with the Behaviour API.

- [ ] Add `pub mod kinds_behaviour;` to `crates/conform7-semantics/src/lib.rs`.

- [ ] Implement name-related functions (matching Using Kinds.w lines 9-33):

  ```rust
  /// Get the name of a kind.
  ///
  /// Corresponds to `Kinds::Behaviour::get_name` in Using Kinds.w lines 9-12.
  pub fn get_name(k: &Kind) -> Option<&'static str> {
      Some(k.construct.name)
  }

  /// Get the range number for a kind.
  ///
  /// Corresponds to `Kinds::Behaviour::get_range_number` in Using Kinds.w lines 25-28.
  pub fn get_range_number(k: &Kind) -> i32 { ... }

  /// Set the range number for a kind.
  ///
  /// Corresponds to `Kinds::Behaviour::set_range_number` in Using Kinds.w lines 30-33.
  pub fn set_range_number(k: &mut Kind, r: i32) { ... }
  ```

- [ ] Implement object-related functions (matching Using Kinds.w lines 38-57):

  ```rust
  /// Returns true if K is an object kind (conforms to K_object, not nil/void).
  ///
  /// Corresponds to `Kinds::Behaviour::is_object` in Using Kinds.w lines 38-43.
  pub fn is_object(k: &Kind) -> bool { ... }

  /// Returns true if K is a proper subkind of object (not object itself, not nil/void).
  ///
  /// Corresponds to `Kinds::Behaviour::is_subkind_of_object` in Using Kinds.w lines 45-50.
  pub fn is_subkind_of_object(k: &Kind) -> bool { ... }

  /// Returns true if K is an object and conforms to L.
  ///
  /// Corresponds to `Kinds::Behaviour::is_object_of_kind` in Using Kinds.w lines 52-57.
  pub fn is_object_of_kind(k: &Kind, l: &Kind) -> bool { ... }
  ```

- [ ] Implement definiteness functions (matching Using Kinds.w lines 64-114):

  ```rust
  /// Returns true if K is a kind of kind (protocol kind).
  ///
  /// Corresponds to `Kinds::Behaviour::is_kind_of_kind` in Using Kinds.w lines 64-68.
  pub fn is_kind_of_kind(k: &Kind) -> bool { ... }

  /// Returns true if K is definite (not a protocol, not a variable,
  /// and all children are definite).
  ///
  /// Corresponds to `Kinds::Behaviour::definite` in Using Kinds.w lines 75-83.
  pub fn definite(k: &Kind) -> bool { ... }

  /// Returns true if K is semidefinite (definite or involves a kind variable).
  ///
  /// Corresponds to `Kinds::Behaviour::semidefinite` in Using Kinds.w lines 85-103.
  pub fn semidefinite(k: &Kind) -> bool { ... }

  /// Returns true if K involves kind variable v.
  ///
  /// Corresponds to `Kinds::Behaviour::involves_var` in Using Kinds.w lines 105-114.
  pub fn involves_var(k: &Kind, v: u8) -> bool { ... }
  ```

- [ ] Implement definition status functions (matching Using Kinds.w lines 125-190):

  ```rust
  /// Returns true if K is built-in (not defined in source text).
  ///
  /// Corresponds to `Kinds::Behaviour::is_built_in` in Using Kinds.w lines 125-129.
  pub fn is_built_in(k: &Kind) -> bool { ... }

  /// Returns the creating sentence for K (None for built-in kinds).
  ///
  /// Corresponds to `Kinds::Behaviour::get_creating_sentence` in Using Kinds.w lines 131-134.
  pub fn get_creating_sentence(k: &Kind) -> Option<usize> { ... }

  /// Returns true if K is uncertainly defined (incompletely defined).
  ///
  /// Corresponds to `Kinds::Behaviour::is_uncertainly_defined` in Using Kinds.w lines 144-147.
  pub fn is_uncertainly_defined(k: &Kind) -> bool { ... }

  /// Returns true if K is an enumeration kind.
  ///
  /// Corresponds to `Kinds::Behaviour::is_an_enumeration` in Using Kinds.w lines 152-155.
  pub fn is_an_enumeration(k: &Kind) -> bool { ... }

  /// Convert K to a unit kind. Returns true if successful or already a unit.
  ///
  /// Corresponds to `Kinds::Behaviour::convert_to_unit` in Using Kinds.w lines 162-165.
  pub fn convert_to_unit(k: &mut Kind) -> bool { ... }

  /// Convert K to an enumeration kind.
  ///
  /// Corresponds to `Kinds::Behaviour::convert_to_enumeration` in Using Kinds.w lines 170-172.
  pub fn convert_to_enumeration(k: &mut Kind) { ... }

  /// Convert K to use real arithmetic.
  ///
  /// Corresponds to `Kinds::Behaviour::convert_to_real` in Using Kinds.w lines 177-179.
  pub fn convert_to_real(k: &mut Kind) { ... }

  /// Create a new enumerated value for K. Returns the next free value index.
  ///
  /// Corresponds to `Kinds::Behaviour::new_enumerated_value` in Using Kinds.w lines 186-190.
  pub fn new_enumerated_value(k: &mut Kind) -> i32 { ... }
  ```

- [ ] Implement superkind tracking (matching Using Kinds.w lines 198-206):

  ```rust
  /// Set where the superkind was set.
  ///
  /// Corresponds to `Kinds::Behaviour::set_superkind_set_at` in Using Kinds.w lines 198-201.
  pub fn set_superkind_set_at(k: &mut Kind, s: Option<usize>) { ... }

  /// Get where the superkind was set.
  ///
  /// Corresponds to `Kinds::Behaviour::get_superkind_set_at` in Using Kinds.w lines 203-206.
  pub fn get_superkind_set_at(k: &Kind) -> Option<usize> { ... }
  ```

- [ ] Implement constant value functions (matching Using Kinds.w lines 214-230):

  ```rust
  /// Returns true if K has named constant values.
  ///
  /// Corresponds to `Kinds::Behaviour::has_named_constant_values` in Using Kinds.w lines 214-219.
  pub fn has_named_constant_values(k: &Kind) -> bool { ... }

  /// Get the constant compilation method for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_constant_compilation_method` in Using Kinds.w lines 227-230.
  pub fn get_constant_compilation_method(k: &Kind) -> i32 { ... }
  ```

- [ ] Implement arithmetic functions (matching Using Kinds.w lines 238-295):

  ```rust
  /// Returns true if K uses signed comparisons.
  ///
  /// Corresponds to `Kinds::Behaviour::uses_signed_comparisons` in Using Kinds.w lines 238-241.
  pub fn uses_signed_comparisons(k: &Kind) -> bool { ... }

  /// Get the comparison routine identifier for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_comparison_routine` in Using Kinds.w lines 243-246.
  pub fn get_comparison_routine(k: &Kind) -> Option<&'static str> { ... }

  /// Returns true if K is quasinumerical (supports arithmetic).
  ///
  /// Corresponds to `Kinds::Behaviour::is_quasinumerical` in Using Kinds.w lines 255-258.
  pub fn is_quasinumerical(k: &Kind) -> bool { ... }

  /// Get the dimensional form of K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_dimensional_form` in Using Kinds.w lines 260-265.
  pub fn get_dimensional_form(k: &Kind) -> Option<&UnitSequence> { ... }

  /// Test if K is a derived kind.
  ///
  /// Corresponds to `Kinds::Behaviour::test_if_derived` in Using Kinds.w lines 267-270.
  pub fn test_if_derived(k: &Kind) -> bool { ... }

  /// Mark K as derived.
  ///
  /// Corresponds to `Kinds::Behaviour::now_derived` in Using Kinds.w lines 272-275.
  pub fn now_derived(k: &mut Kind) { ... }

  /// Get the scale factor for K.
  ///
  /// Corresponds to `Kinds::Behaviour::scale_factor` in Using Kinds.w lines 277-286.
  pub fn scale_factor(k: &Kind) -> i32 { ... }

  /// Get the dimensional rules for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_dim_rules` in Using Kinds.w lines 292-295.
  pub fn get_dim_rules(k: &Kind) -> &DimensionalRules { ... }
  ```

- [ ] Implement identifier function (matching Using Kinds.w lines 300-303):

  ```rust
  /// Get the explicit identifier for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_identifier` in Using Kinds.w lines 300-303.
  pub fn get_identifier(k: &Kind) -> Option<&'static str> { ... }
  ```

- [ ] Implement storage functions (matching Using Kinds.w lines 313-352):

  ```rust
  /// Returns true if K uses block (pointer) values.
  ///
  /// Corresponds to `Kinds::Behaviour::uses_block_values` in Using Kinds.w lines 313-316.
  pub fn uses_block_values(k: &Kind) -> bool { ... }

  /// Get the small block size for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_small_block_size` in Using Kinds.w lines 321-324.
  pub fn get_small_block_size(k: &Kind) -> i32 { ... }

  /// Get the heap size estimate for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_heap_size_estimate` in Using Kinds.w lines 330-333.
  pub fn get_heap_size_estimate(k: &Kind) -> i32 { ... }

  /// Get the distinguishing routine for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_distinguisher` in Using Kinds.w lines 340-343.
  pub fn get_distinguisher(k: &Kind) -> Option<&'static str> { ... }

  /// Returns true if values of K can be serialized.
  ///
  /// Corresponds to `Kinds::Behaviour::can_exchange` in Using Kinds.w lines 349-352.
  pub fn can_exchange(k: &Kind) -> bool { ... }
  ```

- [ ] Implement indexing and documentation functions (matching Using Kinds.w lines 357-408):

  ```rust
  /// Get the documentation reference for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_documentation_reference` in Using Kinds.w lines 357-360.
  pub fn get_documentation_reference(k: &Kind) -> Option<&'static str> { ... }

  /// Set the documentation reference for K.
  ///
  /// Corresponds to `Kinds::Behaviour::set_documentation_reference` in Using Kinds.w lines 362-365.
  pub fn set_documentation_reference(k: &mut Kind, dr: &'static str) { ... }

  /// Get the index default value for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_index_default_value` in Using Kinds.w lines 371-374.
  pub fn get_index_default_value(k: &Kind) -> Option<&'static str> { ... }

  /// Get the index minimum value for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_index_minimum_value` in Using Kinds.w lines 376-379.
  pub fn get_index_minimum_value(k: &Kind) -> Option<&'static str> { ... }

  /// Get the index maximum value for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_index_maximum_value` in Using Kinds.w lines 381-384.
  pub fn get_index_maximum_value(k: &Kind) -> Option<&'static str> { ... }

  /// Get the index priority for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_index_priority` in Using Kinds.w lines 386-389.
  pub fn get_index_priority(k: &Kind) -> i32 { ... }

  /// Returns true if K should be greyed out in the index when empty.
  ///
  /// Corresponds to `Kinds::Behaviour::indexed_grey_if_empty` in Using Kinds.w lines 391-394.
  pub fn indexed_grey_if_empty(k: &Kind) -> bool { ... }

  /// Set the specification text for K.
  ///
  /// Corresponds to `Kinds::Behaviour::set_specification_text` in Using Kinds.w lines 400-403.
  pub fn set_specification_text(k: &mut Kind, desc: &'static str) { ... }

  /// Get the specification text for K.
  ///
  /// Corresponds to `Kinds::Behaviour::get_specification_text` in Using Kinds.w lines 405-408.
  pub fn get_specification_text(k: &Kind) -> Option<&'static str> { ... }
  ```

### 3. Update familiar kinds with Behaviour metadata

The familiar kinds and constructors need to be updated with the new fields. For example:
- `K_number` should have `explicit_identifier = "K_number"`, `is_quasinumerical = true`, `uses_signed_comparisons = true`, `constant_compilation_method = NUMBER_CCM`, `uses_block_values = false`.
- `K_text` should have `explicit_identifier = "K_text"`, `uses_block_values = true`, `small_block_size = 64`, `heap_size_estimate = 128`, `distinguishing_routine = "TextDistinguisher"`.
- `K_object` should have `explicit_identifier = "K_object"`, `uses_block_values = true`, `small_block_size = 32`, `heap_size_estimate = 64`.

- [ ] Update `CON_NUMBER` in `familiar_kinds.rs` with:
  - `explicit_identifier = "K_number"`
  - `constant_compilation_method = 1` (NUMBER_CCM)
  - `uses_block_values = false`
  - `uses_signed_comparisons = true`
  - `dimensional_form = UnitSequence::scalar()` (dimensionless)

- [ ] Update `CON_TEXT` in `familiar_kinds.rs` with:
  - `explicit_identifier = "K_text"`
  - `uses_block_values = true`
  - `small_block_size = 64`
  - `heap_size_estimate = 128`
  - `distinguishing_routine = "TextDistinguisher"`

- [ ] Update `CON_OBJECT` in `familiar_kinds.rs` with:
  - `explicit_identifier = "K_object"`
  - `uses_block_values = true`
  - `small_block_size = 32`
  - `heap_size_estimate = 64`

- [ ] Update `CON_REAL_NUMBER` with:
  - `explicit_identifier = "K_real_number"`
  - `constant_compilation_method = 2` (REAL_CCM)
  - `uses_block_values = false`
  - `uses_signed_comparisons = true`

- [ ] Update `CON_TRUTH_STATE` with:
  - `explicit_identifier = "K_truth_state"`
  - `uses_block_values = false`

- [ ] Update `CON_TABLE` with:
  - `explicit_identifier = "K_table"`
  - `uses_block_values = true`

- [ ] Update `CON_UNICODE_CHARACTER` with:
  - `explicit_identifier = "K_unicode_character"`
  - `uses_block_values = false`

- [ ] Update `CON_VERB` with:
  - `explicit_identifier = "K_verb"`
  - `uses_block_values = false`

- [ ] Update protocol constructors with `explicit_identifier`:
  - `CON_VALUE` → `"K_value"`
  - `CON_STORED_VALUE` → `"K_stored_value"`
  - `CON_SAYABLE_VALUE` → `"K_sayable_value"`
  - `CON_UNDERSTANDABLE_VALUE` → `"K_understandable_value"`
  - `CON_ARITHMETIC_VALUE` → `"K_arithmetic_value"`
  - `CON_REAL_ARITHMETIC_VALUE` → `"K_real_arithmetic_value"`
  - `CON_ENUMERATED_VALUE` → `"K_enumerated_value"`
  - `CON_POINTER_VALUE` → `"K_pointer_value"`

- [ ] Update punctuation constructors with `explicit_identifier`:
  - `CON_TUPLE_ENTRY` → `"CON_TUPLE_ENTRY"`
  - `CON_VOID` → `"CON_VOID"`
  - `CON_NIL` → `"CON_NIL"`
  - `CON_UNKNOWN` → `"CON_UNKNOWN"`
  - `CON_INTERMEDIATE` → `"CON_INTERMEDIATE"`
  - `CON_KIND_VARIABLE` → `"CON_KIND_VARIABLE"`

- [ ] Update proper constructors with `explicit_identifier`:
  - `CON_list_of` → `"CON_list_of"`
  - `CON_description` → `"CON_description"`
  - `CON_relation` → `"CON_relation"`
  - `CON_rule` → `"CON_rule"`
  - `CON_rulebook` → `"CON_rulebook"`
  - `CON_activity` → `"CON_activity"`
  - `CON_phrase` → `"CON_phrase"`
  - `CON_property` → `"CON_property"`
  - `CON_table_column` → `"CON_table_column"`
  - `CON_combination` → `"CON_combination"`
  - `CON_variable` → `"CON_variable"`

### 4. Add unit tests

- [ ] Test name functions:
  - `get_name(K_number)` returns `"number"`.
  - `get_name(K_text)` returns `"text"`.
  - `get_name(K_object)` returns `"object"`.
  - `get_range_number` defaults to 0.
  - `set_range_number` updates the value.

- [ ] Test object functions:
  - `is_object(K_object)` returns `true`.
  - `is_object(K_number)` returns `false`.
  - `is_subkind_of_object(K_object)` returns `false` (object itself is not a subkind).
  - `is_subkind_of_object` on a subkind of object (once created) returns `true`.
  - `is_object_of_kind(K_object, K_value)` returns `true`.

- [ ] Test definiteness functions:
  - `definite(K_number)` returns `true`.
  - `definite(K_text)` returns `true`.
  - `definite(K_value)` returns `false` (protocol).
  - `definite(K_arithmetic_value)` returns `false` (protocol).
  - `definite(list of numbers)` returns `true` (all children definite).
  - `definite(list of values)` returns `false` (child is indefinite).
  - `is_kind_of_kind(K_value)` returns `true`.
  - `is_kind_of_kind(K_number)` returns `false`.
  - `semidefinite(K_number)` returns `true`.
  - `semidefinite(K_value)` returns `false`.
  - `involves_var` on a kind variable returns `true` for the matching number.
  - `involves_var` on a base kind returns `false`.

- [ ] Test definition status functions:
  - `is_built_in(K_number)` returns `true` (no creating sentence).
  - `is_uncertainly_defined(K_number)` returns `false`.
  - `is_an_enumeration(K_number)` returns `false`.
  - `is_an_enumeration` on an enumeration kind returns `true`.
  - `convert_to_enumeration` marks a kind as an enumeration.
  - `new_enumerated_value` returns incrementing values.
  - `has_named_constant_values` on an enumeration returns `true`.
  - `has_named_constant_values` on `K_number` returns `false`.

- [ ] Test arithmetic functions:
  - `is_quasinumerical(K_number)` returns `true`.
  - `is_quasinumerical(K_text)` returns `false`.
  - `is_quasinumerical(K_real_number)` returns `true`.
  - `uses_signed_comparisons(K_number)` returns `true`.
  - `get_identifier(K_number)` returns `Some("K_number")`.
  - `get_identifier(K_value)` returns `Some("K_value")`.

- [ ] Test storage functions:
  - `uses_block_values(K_number)` returns `false`.
  - `uses_block_values(K_text)` returns `true`.
  - `get_small_block_size(K_text)` returns `64`.
  - `get_heap_size_estimate(K_text)` returns `128`.

- [ ] Test indexing functions:
  - `get_index_priority` defaults to 0.
  - `set_specification_text` / `get_specification_text` round-trip.
  - `get_documentation_reference` defaults to None.
  - `set_documentation_reference` updates the value.

- [ ] Test that all existing tests still pass:
  - `cargo test -p conform7-semantics` passes.
  - `cargo clippy -p conform7-semantics` is clean.

### 5. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `KindConstructor` has all new fields: `is_incompletely_defined`, `next_free_value`, `explicit_identifier`, `where_defined_in_source_text`, `constant_compilation_method`, `dimensional_form`, `dimensional_form_fixed`, `specification_text`, `index_priority`, `indexed_grey_if_empty`, `index_default_value`, `index_minimum_value`, `index_maximum_value`, `documentation_reference`, `can_exchange`, `distinguishing_routine`, `uses_block_values`, `small_block_size`, `heap_size_estimate`, `class_number`, `superkind_set_at`, `uses_signed_comparisons`, `comparison_fn_identifier`, `dim_rules`.
- [ ] `UnitSequence` and `DimensionalRules` types exist for dimensional form representation.
- [ ] `kinds_behaviour` module exists with all Behaviour API functions.
- [ ] `get_name` returns the constructor name for any kind.
- [ ] `is_object` correctly identifies object kinds.
- [ ] `is_subkind_of_object` correctly identifies proper subkinds of object.
- [ ] `is_object_of_kind` correctly checks object conformance to a specific kind.
- [ ] `is_kind_of_kind` correctly identifies protocol kinds.
- [ ] `definite` correctly checks recursive definiteness (protocol kinds are not definite, composite kinds with indefinite children are not definite).
- [ ] `semidefinite` correctly checks semi-definiteness.
- [ ] `involves_var` correctly checks kind variable involvement.
- [ ] `is_built_in` correctly identifies built-in kinds.
- [ ] `is_uncertainly_defined` correctly checks incomplete definition status.
- [ ] `is_an_enumeration` correctly identifies enumeration kinds.
- [ ] `convert_to_enumeration` marks a kind as an enumeration.
- [ ] `new_enumerated_value` returns incrementing values.
- [ ] `has_named_constant_values` correctly identifies kinds with named constants.
- [ ] `is_quasinumerical` correctly identifies arithmetic kinds.
- [ ] `uses_signed_comparisons` correctly identifies kinds using signed comparisons.
- [ ] `get_identifier` returns the explicit identifier for each familiar kind.
- [ ] `uses_block_values` correctly identifies block value kinds.
- [ ] `get_small_block_size` and `get_heap_size_estimate` return correct values.
- [ ] All familiar kinds and constructors have `explicit_identifier` set.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Inference subjects**: The `InferenceSubjects` system (knowledge-module/Chapter 4/Inference Subjects.w) is deferred. This plan focuses on the kind-level Behaviour API only.
- **Kind subjects**: `KindSubjects` (knowledge-module/Chapter 4/Kind Subjects.w) — the bridge from kinds to inference subjects — is deferred.
- **Instance system**: `Instances` (knowledge-module/Chapter 2/Instances.w) and `InstanceSubjects` (knowledge-module/Chapter 4/Instance Subjects.w) are deferred.
- **Property system**: `Properties` (knowledge-module/Chapter 3/Properties.w), `PropertyPermissions` (knowledge-module/Chapter 4/Property Permissions.w), either/or and valued properties are deferred.
- **Relation system**: Relations between kinds are deferred.
- **Assertion processing**: Processing assertion sentences ("X is a kind of Y", "X has a property called P") is deferred.
- **Full `<k-kind>` Preform grammar**: The complete K-grammar from Describing Kinds.w is deferred. The simplified `FromStr` from PLAN-14 is sufficient.
- **Kind variable substitution**: `Kinds::substitute` and `Kinds::weaken` are deferred.
- **Dimensional analysis**: Full dimensional analysis with unit sequences, dimensional rules, and arithmetic on kinds is deferred. We add the data structures but not the analysis engine.
- **Casting rules and instance rules**: The linked lists of `kind_constructor_casting_rule` and `kind_constructor_instance` are deferred.
- **Literal patterns**: `literal_pattern` and how constant values of a kind are expressed are deferred.
- **Neptune files**: The Neptune file parser for defining built-in kinds from template files is deferred.
- **Run-time identifiers**: Inter identifiers, printing routines, recognition routines, and other run-time support fields on `KindConstructor` are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
- **Full `Kinds::Dimensions` API**: The dimensions system for unit analysis is deferred. We add the `UnitSequence` and `DimensionalRules` data structures but not the full analysis engine.
