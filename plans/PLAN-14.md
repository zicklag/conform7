# Plan 14: Kind System Foundation — Kind and KindConstructor Data Structures
**Status**: Complete
**Target**: 2-3 days

## Goal

Create the `conform7-semantics` crate with the core kind system data structures — `Kind`, `KindConstructor`, and the construction functions that build kind trees. This is the foundation of the world model: every assertion sentence, every property, every instance, every relation ultimately depends on the kind system.

This is the smallest next step after PLAN-13 because:

1. **The syntax/linguistics pipeline is complete.** PLANs 1-13 built the full pipeline: Inter compatibility, lexer, sentence breaker, Preform matching engine, internal NT dispatch, article system, verb data structures, verb conjugation, verb phrase seeking, and the `<sentence>` internal NT. The frontend can now parse sentences into `VERB_NT` diagrams.

2. **The world model needs kinds first.** Before we can process assertion sentences ("X is a kind of Y", "X has a property called P"), we need the kind system that defines what kinds are, how they relate, and how they're constructed. The kind system is the type system of Inform 7 — everything else (instances, properties, relations) builds on top of it.

3. **The kind system is independently testable.** The C reference has a standalone `kinds-test` REPL that exercises kind construction, conformance, and textual I/O in isolation. We can replicate this with Rust unit tests that construct kinds, test equality, test conformance, and round-trip through textual descriptions.

4. **The K-grammar is a Preform grammar.** The `<k-kind>` nonterminal (defined in `Describing Kinds.w`) parses textual kind descriptions like "number", "list of texts", "relation of numbers to texts". Once we have the kind data structures, we can implement the K-grammar as Preform internal NTs — the same pattern used for `<sentence>` in PLAN-13.

5. **Prerequisite for assertion processing.** Assertion sentences like "A weight is a kind of value" need to create new kinds. "The blue ball is a thing" needs to create instances of kinds. "A person has a number called the age" needs to add properties to kinds. All of these depend on the kind system.

## Background

### C reference architecture

The kind system is defined across several files in `services/kinds-module/`:

#### Core data structures

**`kind` struct** (`services/kinds-module/Chapter 2/Kinds.w`, lines 9-14):
```c
typedef struct kind {
    struct kind_constructor *construct;  /* which can never be NULL */
    int kind_variable_number;            /* only used if construct is CON_KIND_VARIABLE */
    struct unit_sequence *intermediate_result;  /* only used if construct is CON_INTERMEDIATE */
    struct kind *kc_args[MAX_KIND_CONSTRUCTION_ARITY];  /* used if arity positive, or for CON_KIND_VARIABLE */
} kind;
```

Kinds are tree structures. A base kind like `number` is a leaf node. A composite kind like `list of numbers` is a unary construction (arity 1). A binary construction like `relation of numbers to texts` has arity 2. The maximum arity is 2 (`MAX_KIND_CONSTRUCTION_ARITY`).

**`kind_constructor` struct** (`services/kinds-module/Chapter 4/Kind Constructors.w`, lines 24-97):
A large struct (~30 fields) with metadata about each kind constructor:
- **Group**: `PUNCTUATION_GRP` (1), `PROTOCOL_GRP` (2), `BASE_CONSTRUCTOR_GRP` (3), `PROPER_CONSTRUCTOR_GRP` (4)
- **Arity**: 0 for base, 1 for unary, 2 for binary
- **Variance**: `COVARIANT` or `CONTRAVARIANT` for each argument
- **Tupling**: `NO_TUPLING`, `ALLOW_NOTHING_TUPLING`, `ARBITRARY_TUPLING`
- **Cached kind**: pointer to the cached base construction
- **Casting rules**: linked list of `kind_constructor_casting_rule`
- **Instance rules**: linked list of `kind_constructor_instance`
- **Literal patterns**: how constant values of this kind are expressed
- **Inference subject**: `base_as_infs` for knowledge about values
- **Dimensional rules**: for arithmetic operations
- **Run-time identifiers**: Inter identifiers, printing routines, etc.

#### Construction functions

- **`Kinds::base_construction(con)`** (Kinds.w, lines 77-95): Creates or returns cached base kind (arity 0). For arity 1 constructors, defaults the argument to `K_value`. For arity 2, defaults both to `K_value`.
- **`Kinds::unary_con(con, X)`** (Kinds.w, lines 143-150): Creates a unary construction like `list of X`.
- **`Kinds::binary_con(con, X, Y)`** (Kinds.w, lines 152-163): Creates a binary construction like `relation of X to Y`.
- **`Kinds::function_kind(no_args, args, return_K)`** (Kinds.w, lines 198-204): Creates function kinds using `CON_TUPLE_ENTRY` punctuation.
- **`Kinds::var_construction(N, declaration)`** (Kinds.w, lines 122-130): Creates kind variable placeholders A-Z.

#### Equality and conformance

- **`Kinds::eq(K1, K2)`** (Kinds.w, lines 579-594): Structural equality — compares constructors, intermediate results, variable numbers, and all child kinds recursively.
- **`Kinds::conforms_to(from, to)`** (Kinds.w, lines 608-612): Tests if `from` conforms to `to` (is-a relationship).
- **`Kinds::compatible(from, to)`** (Kinds.w, lines 614-625): Three-valued compatibility (always/sometimes/never).

#### The lattice of kinds

**`Latticework::super(K)`** (`The Lattice of Kinds.w`, lines 89-119): Returns the superkind of K. Hard-codes the built-in indefinite kind hierarchy:
```
value
  stored_value
    sayable_value
      understandable_value
        arithmetic_value
          real_arithmetic_value
        enumerated_value
      pointer_value
```

**`Latticework::join(K1, K2)`** / **`Latticework::meet(K1, K2)`** (lines 134-140): Lattice join and meet operations.

#### Familiar kinds and constructors

**Familiar Kinds** (`Familiar Kinds.w`): Global variables for commonly used kinds and constructors:
- Protocol kinds: `K_value`, `K_stored_value`, `K_pointer_value`, `K_sayable_value`, `K_understandable_value`, `K_arithmetic_value`, `K_real_arithmetic_value`, `K_enumerated_value`
- Punctuation: `CON_TUPLE_ENTRY`, `K_void`, `CON_VOID`, `K_nil`, `CON_NIL`, `K_unknown`, `CON_UNKNOWN`, `CON_INTERMEDIATE`, `CON_KIND_VARIABLE`
- Base kinds: `K_number`, `K_text`, `K_object`, `K_real_number`, `K_truth_state`, `K_table`, `K_unicode_character`, `K_verb`, etc.
- Proper constructors: `CON_list_of`, `CON_description`, `CON_relation`, `CON_rule`, `CON_rulebook`, `CON_activity`, `CON_phrase`, `CON_property`, `CON_table_column`, `CON_combination`, `CON_variable`

#### Kind subjects

**KindSubjects** (`inform7/knowledge-module/Chapter 4/Kind Subjects.w`): Bridges kinds to the inference subject system. Each base kind gets an inference subject, enabling property assertions on kinds. This is the connection between the kind system and the world model.

#### Kind predicates

**KindPredicates** (`services/calculus-module/Chapter 2/Kind Predicates.w`): Defines unary predicates `kind=K` for the calculus module. These are used in propositions like "X is a number" or "X is a kind of vehicle".

### Key C source files

- `services/kinds-module/Chapter 2/Kinds.w` — `kind` struct, construction functions, equality, conformance, substitution, weakening
- `services/kinds-module/Chapter 4/Kind Constructors.w` — `kind_constructor` struct, creation, arity, variance, tupling
- `services/kinds-module/Chapter 2/Familiar Kinds.w` — Global `K_*` and `CON_*` variables
- `services/kinds-module/Chapter 2/The Lattice of Kinds.w` — Superkind hierarchy, join, meet, order relation
- `services/kinds-module/Chapter 2/Using Kinds.w` — `Kinds::Behaviour` API: definiteness, object-ness, enumeration, quasinumerical, etc.
- `services/kinds-module/Chapter 2/Describing Kinds.w` — `<k-kind>` Preform grammar for parsing kind descriptions, textual pretty-printer
- `services/kinds-module/Chapter 1/Kinds Module.w` — Module setup, class declarations, problem handler
- `services/kinds-module/Preliminaries/What This Module Does.w` — Overview of the kind system
- `inform7/knowledge-module/Chapter 4/Kind Subjects.w` — Inference subjects for kinds
- `services/calculus-module/Chapter 2/Kind Predicates.w` — Kind predicates for the calculus

### Current Rust state

- `crates/conform7-syntax/` — Full syntax/linguistics pipeline: lexer, sentence breaker, Preform engine, parse nodes, linguistics module, verb system, verb phrase seeking, internal NTs including `<sentence>`
- `crates/conform7-inter/` — Inter IR read/write (textual `.intert` format)
- `crates/conform7-semantics/` — **Does not exist yet.** This is the target crate for this plan.

### What's needed

1. **New crate `conform7-semantics`** with the core kind data structures.
2. **`Kind` struct** — tree node with constructor reference and up to 2 child kinds.
3. **`KindConstructor` struct** — constructor metadata (name, arity, variance, group, etc.).
4. **`ConstructorId` / `KindId`** — interned identifiers for referencing kinds and constructors.
5. **Construction functions** — `base_construction`, `unary_con`, `binary_con`, `function_kind`, `var_construction`.
6. **Familiar kinds and constructors** — global constants for `K_number`, `K_text`, `K_object`, `CON_list_of`, `CON_relation`, etc.
7. **Equality** — structural `Kind::eq` that compares constructors and children recursively.
8. **Conformance** — `Kind::conforms_to` using the lattice hierarchy.
9. **Lattice basics** — `superkind`, `join`, `meet` for the built-in indefinite kind hierarchy.
10. **Textual I/O** — `Display` impl for kinds (pretty-printing), and a basic `from_str` for parsing simple kind names.
11. **Unit tests** — construction, equality, conformance, lattice operations, textual round-trip.

## Tasks

### 1. Create the `conform7-semantics` crate

- [ ] Create `crates/conform7-semantics/Cargo.toml`:
  ```toml
  [package]
  name = "conform7-semantics"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  ```

- [ ] Create `crates/conform7-semantics/src/lib.rs` with module declarations:
  ```rust
  pub mod kinds;
  pub mod kind_constructors;
  pub mod familiar_kinds;
  pub mod lattice;
  ```

- [ ] Add the crate to the workspace in `Cargo.toml` (it should already be covered by `members = ["crates/*"]`).

### 2. Implement `KindConstructor` struct

The `kind_constructor` in C is a large struct (~30 fields). For this plan, we implement the subset needed for kind construction and the lattice.

- [ ] Create `crates/conform7-semantics/src/kind_constructors.rs` with:

  ```rust
  /// Group classification for kind constructors.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub enum ConstructorGroup {
      /// Punctuation nodes used in kind construction only (CON_TUPLE_ENTRY, CON_VOID, CON_NIL).
      Punctuation,
      /// Protocol-like "kinds of kinds" (arithmetic value, sayable value, etc.).
      Protocol,
      /// Base constructors with arity 0 (number, text, thing, etc.).
      Base,
      /// Proper constructors with positive arity (list of ..., relation of ... to ..., etc.).
      Proper,
  }

  /// Variance of a constructor argument.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub enum Variance {
      Covariant,
      Contravariant,
  }

  /// Tupling permission for a constructor argument.
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub enum Tupling {
      /// A single kind.
      None,
      /// A single kind, or "nothing".
      AllowNothing,
      /// A list of kinds of any length.
      Arbitrary,
  }

  /// A kind constructor — the "type constructor" that builds kinds.
  ///
  /// Corresponds to `kind_constructor` in the C reference
  /// (`services/kinds-module/Chapter 4/Kind Constructors.w`, lines 24-97).
  ///
  /// This is a simplified version containing only the fields needed for
  /// kind construction, the lattice, and textual I/O. Additional fields
  /// (casting rules, literal patterns, dimensional rules, run-time
  /// identifiers, etc.) will be added in later plans.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct KindConstructor {
      /// The name of this constructor (e.g., "number", "list of K", "relation of K to L").
      pub name: &'static str,
      /// The group classification.
      pub group: ConstructorGroup,
      /// Arity: 0 for base, 1 for unary, 2 for binary.
      pub arity: u8,
      /// Variance of each argument (only meaningful for arity > 0).
      pub variance: [Variance; 2],
      /// Tupling of each argument.
      pub tupling: [Tupling; 2],
      /// Whether this is a definite kind (not a protocol/indefinite kind).
      pub definite: bool,
      /// Whether this is an arithmetic kind.
      pub arithmetic: bool,
      /// Whether this is an enumeration kind.
      pub enumeration: bool,
      /// Whether this is an object kind (subkind of object).
      pub object_kind: bool,
  }
  ```

- [ ] Implement constructor creation and accessors:
  - `KindConstructor::new(name, group, arity) -> Self` — creates a new constructor with defaults (covariant, no tupling, not definite, not arithmetic, not enumeration, not object).
  - `KindConstructor::set_variance(&mut self, arg: usize, variance: Variance)` — set variance for argument 0 or 1.
  - `KindConstructor::set_tupling(&mut self, arg: usize, tupling: Tupling)` — set tupling for argument 0 or 1.
  - `KindConstructor::is_definite(&self) -> bool` — returns `self.definite`.
  - `KindConstructor::is_arithmetic(&self) -> bool` — returns `self.arithmetic`.
  - `KindConstructor::is_enumeration(&self) -> bool` — returns `self.enumeration`.
  - `KindConstructor::is_object_kind(&self) -> bool` — returns `self.object_kind`.

- [ ] Add unit tests:
  - Test that a base constructor has arity 0 and group `Base`.
  - Test that a proper constructor has arity > 0 and group `Proper`.
  - Test that a protocol constructor has group `Protocol` and is not definite.
  - Test variance and tupling setters.

### 3. Implement `Kind` struct

- [ ] Create `crates/conform7-semantics/src/kinds.rs` with:

  ```rust
  /// Maximum arity for kind constructions (matching MAX_KIND_CONSTRUCTION_ARITY = 2).
  pub const MAX_KIND_CONSTRUCTION_ARITY: usize = 2;

  /// A kind — a tree structure representing a type in Inform 7.
  ///
  /// Corresponds to `kind` in the C reference
  /// (`services/kinds-module/Chapter 2/Kinds.w`, lines 9-14).
  ///
  /// Kinds are trees built by applying kind constructors to other kinds.
  /// A base kind like `number` is a leaf. A composite kind like
  /// `list of numbers` is a unary construction. A binary construction
  /// like `relation of numbers to texts` has two children.
  #[derive(Clone, Debug)]
  pub struct Kind {
      /// The constructor used to build this kind (never None for valid kinds).
      pub construct: &'static KindConstructor,
      /// Kind variable number (1-26 for A-Z), only meaningful when construct is CON_KIND_VARIABLE.
      pub kind_variable_number: u8,
      /// Child kinds (up to MAX_KIND_CONSTRUCTION_ARITY).
      pub kc_args: [Option<Box<Kind>>; MAX_KIND_CONSTRUCTION_ARITY],
  }
  ```

- [ ] Implement construction functions:
  - `Kind::base_construction(con: &'static KindConstructor) -> Kind` — creates a base kind (arity 0). For arity 1 constructors, defaults the argument to `K_value`. For arity 2, defaults both to `K_value`. (Matching `Kinds::base_construction` in Kinds.w lines 77-95.)
  - `Kind::unary_con(con: &'static KindConstructor, x: Kind) -> Kind` — creates a unary construction. (Matching `Kinds::unary_con` in Kinds.w lines 143-150.)
  - `Kind::binary_con(con: &'static KindConstructor, x: Kind, y: Kind) -> Kind` — creates a binary construction. (Matching `Kinds::binary_con` in Kinds.w lines 152-163.)
  - `Kind::function_kind(args: &[Kind], return_k: Option<Kind>) -> Kind` — creates a function kind using `CON_TUPLE_ENTRY` punctuation. (Matching `Kinds::function_kind` in Kinds.w lines 198-204.)
  - `Kind::var_construction(n: u8, declaration: Option<Kind>) -> Kind` — creates a kind variable placeholder. (Matching `Kinds::var_construction` in Kinds.w lines 122-130.)

- [ ] Implement accessors:
  - `Kind::get_construct(&self) -> &'static KindConstructor` — returns the constructor.
  - `Kind::arity(&self) -> u8` — returns the arity of the constructor.
  - `Kind::is_proper(&self) -> bool` — returns true if arity > 0.
  - `Kind::is_intermediate(&self) -> bool` — checks if constructor is `CON_INTERMEDIATE`.
  - `Kind::get_variable_number(&self) -> u8` — returns variable number (0 if not a variable).
  - `Kind::unary_material(&self) -> Option<&Kind>` — returns the single child for unary constructions.
  - `Kind::binary_material(&self) -> (Option<&Kind>, Option<&Kind>)` — returns the two children for binary constructions.

- [ ] Implement `Kind::eq` (structural equality, matching `Kinds::eq` in Kinds.w lines 579-594):
  - Compare constructors (pointer identity).
  - Compare variable numbers.
  - Compare all child kinds recursively.
  - Two different `Kind` values can represent the same kind, so pointer comparison is insufficient.

- [ ] Implement `Display` for `Kind` (basic pretty-printer, matching `Kinds::Textual::write_inner` in Describing Kinds.w lines 551-560):
  - Base kinds: print the constructor name.
  - Unary constructions: `"<name> of <child>"` (e.g., "list of numbers").
  - Binary constructions: `"<name> of <child1> to <child2>"` (e.g., "relation of numbers to texts").
  - Kind variables: print the letter (A-Z).
  - For now, use a simplified format. Full multi-form constructor names (with strokes) are deferred.

- [ ] Add unit tests:
  - Test that `Kind::base_construction` creates a kind with the correct constructor and no children.
  - Test that `Kind::unary_con` creates a kind with one child.
  - Test that `Kind::binary_con` creates a kind with two children.
  - Test that `Kind::function_kind` creates a proper function kind with tuple entry structure.
  - Test that `Kind::var_construction` creates a kind with the correct variable number.
  - Test `Kind::eq` for equal kinds (same constructor, same children).
  - Test `Kind::eq` for different kinds (different constructor, different children).
  - Test `Kind::eq` for structurally different but semantically same kinds (e.g., two different `list of numbers` constructions should be equal).
  - Test `Display` output for base kinds, unary constructions, binary constructions, and kind variables.

### 4. Define familiar kinds and constructors

- [ ] Create `crates/conform7-semantics/src/familiar_kinds.rs` with global constants for all built-in kinds and constructors.

  Protocol kinds (matching Familiar Kinds.w lines 29-37):
  ```rust
  pub static CON_VALUE: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static K_value: Lazy<Kind> = Lazy::new(|| Kind::base_construction(&CON_VALUE));
  // ... K_stored_value, K_pointer_value, K_sayable_value, K_understandable_value,
  //     K_arithmetic_value, K_real_arithmetic_value, K_enumerated_value
  ```

  Punctuation constructors (matching Familiar Kinds.w lines 43-89):
  ```rust
  pub static CON_TUPLE_ENTRY: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_VOID: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_NIL: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_UNKNOWN: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_INTERMEDIATE: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_KIND_VARIABLE: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static K_void: Lazy<Kind> = Lazy::new(|| Kind::base_construction(&CON_VOID));
  pub static K_nil: Lazy<Kind> = Lazy::new(|| Kind::base_construction(&CON_NIL));
  pub static K_unknown: Lazy<Kind> = Lazy::new(|| Kind::base_construction(&CON_UNKNOWN));
  ```

  Base kinds (matching Familiar Kinds.w lines 95-109):
  ```rust
  pub static K_number: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_text: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_object: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_real_number: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_truth_state: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_table: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_unicode_character: Lazy<Kind> = Lazy::new(|| { ... });
  pub static K_verb: Lazy<Kind> = Lazy::new(|| { ... });
  // ... K_equation, K_grammatical_gender, K_natural_language, K_response,
  //     K_snippet, K_use_option, K_rulebook_outcome, K_understanding
  ```

  Proper constructors (matching Familiar Kinds.w lines 131-142):
  ```rust
  pub static CON_list_of: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_description: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_relation: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_rule: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_rulebook: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_activity: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_phrase: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_property: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_table_column: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_combination: Lazy<KindConstructor> = Lazy::new(|| { ... });
  pub static CON_variable: Lazy<KindConstructor> = Lazy::new(|| { ... });
  ```

  Each constructor should be configured with the correct:
  - Group (Protocol, Base, Proper, or Punctuation)
  - Arity (0, 1, or 2)
  - Variance (covariant/contravariant for each argument)
  - Definiteness (protocol kinds are not definite)
  - Arithmetic flag (arithmetic value, real arithmetic value, number, real number)
  - Enumeration flag (enumerated value and its subkinds)
  - Object kind flag (object and its subkinds)

- [ ] Add unit tests:
  - Test that all familiar kinds are constructable.
  - Test that `K_number` is definite, arithmetic, not an enumeration, not an object kind.
  - Test that `K_text` is definite, not arithmetic, not an enumeration, not an object kind.
  - Test that `K_object` is definite, not arithmetic, not an enumeration, is an object kind.
  - Test that `K_arithmetic_value` is not definite (it's a protocol).
  - Test that `CON_list_of` has arity 1 and is covariant.
  - Test that `CON_relation` has arity 2 and is covariant in both arguments.
  - Test that `CON_phrase` has arity 2, is contravariant in argument 0, covariant in argument 1.
  - Test that `list of numbers` can be constructed from `CON_list_of` and `K_number`.

### 5. Implement the kind lattice

- [ ] Create `crates/conform7-semantics/src/lattice.rs` with:

  ```rust
  /// The superkind of a given kind in the conformance hierarchy.
  ///
  /// Returns None for kinds that have no superkind (value, nil, void).
  ///
  /// Corresponds to `Latticework::super` in
  /// `services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119.
  pub fn superkind(k: &Kind) -> Option<&'static Kind> { ... }
  ```

  The superkind hierarchy for built-in indefinite kinds:
  ```
  value (None)
    stored_value (-> value)
      sayable_value (-> stored_value)
        understandable_value (-> sayable_value)
          arithmetic_value (-> understandable_value)
            real_arithmetic_value (-> arithmetic_value)
          enumerated_value (-> understandable_value)
        pointer_value (-> sayable_value)
  ```

  For base kinds that are subkinds of these protocols, the superkind is determined by the constructor's group and flags. For now, we hard-code the protocol hierarchy and use constructor flags for base kinds.

- [ ] Implement `Kind::conforms_to(&self, other: &Kind) -> bool`:
  - Returns true if `self` conforms to `other` (is-a relationship).
  - Walks the superkind chain of `self` looking for `other`.
  - For proper constructors (arity > 0), checks variance of each argument.
  - Corresponds to `Kinds::conforms_to` in Kinds.w lines 608-612.

- [ ] Implement `Kind::compatible(&self, other: &Kind) -> Compatibility`:
  - Three-valued result: `Always`, `Sometimes`, `Never`.
  - Corresponds to `Kinds::compatible` in Kinds.w lines 614-625.

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub enum Compatibility {
      Always,
      Sometimes,
      Never,
  }
  ```

- [ ] Implement `join(k1: &Kind, k2: &Kind) -> Kind` and `meet(k1: &Kind, k2: &Kind) -> Kind`:
  - Lattice join and meet operations.
  - Corresponds to `Latticework::join` and `Latticework::meet` in The Lattice of Kinds.w lines 134-140.
  - For proper constructors with the same constructor, recursively join/meet children with variance awareness.
  - For base kinds, walk the superkind chain.

- [ ] Add unit tests:
  - Test that `K_number` conforms to `K_arithmetic_value`.
  - Test that `K_number` conforms to `K_value`.
  - Test that `K_number` does NOT conform to `K_text`.
  - Test that `K_text` conforms to `K_sayable_value`.
  - Test that `K_object` conforms to `K_value`.
  - Test that `list of numbers` conforms to `list of arithmetic values` (covariance).
  - Test that `list of numbers` does NOT conform to `list of texts`.
  - Test that `K_nil` conforms to everything.
  - Test that `K_value` does not conform to `K_number`.
  - Test join: `join(K_number, K_text)` = `K_sayable_value`.
  - Test join: `join(K_number, K_real_number)` = `K_real_number` (floating-point promotion).
  - Test meet: `meet(K_number, K_arithmetic_value)` = `K_number`.
  - Test compatibility: `K_number` is always compatible with `K_real_number`.
  - Test compatibility: `K_object` is always compatible with `K_thing` (if thing is a subkind of object).

### 6. Add textual I/O

- [ ] Implement `FromStr` for `Kind` (basic parsing of kind names):
  - Parse single-word base kinds: "number", "text", "thing", "value", etc.
  - Parse unary constructions: "list of <kind>", "description of <kind>", etc.
  - Parse binary constructions: "relation of <kind> to <kind>", "phrase <kind> -> <kind>", etc.
  - For now, only parse the canonical forms (not the stroke-separated alternatives).
  - This is a simplified version of the `<k-kind>` Preform grammar from Describing Kinds.w.

- [ ] Improve `Display` for `Kind`:
  - Base kinds: print the constructor name.
  - Unary constructions: `"<name> of <child>"`.
  - Binary constructions: `"<name> of <child1> to <child2>"`.
  - Kind variables: print the letter.
  - Special cases: `"either/or property"` for `CON_property` of `K_truth_state`.
  - Use the constructor's name field, substituting `K` and `L` placeholders with actual child kinds.

- [ ] Add unit tests:
  - Test round-trip: parse a kind description, display it, and verify the output matches.
  - Test that `K_number` displays as "number".
  - Test that `list of numbers` displays as "list of numbers".
  - Test that `relation of numbers to texts` displays as "relation of numbers to texts".
  - Test that `phrase (number, text) -> truth state` displays correctly.
  - Test that `either/or property` displays correctly.

### 7. Integration with the workspace

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `crates/conform7-semantics/` exists with `Cargo.toml` and `src/lib.rs`.
- [ ] `KindConstructor` struct with fields: name, group, arity, variance, tupling, definite, arithmetic, enumeration, object_kind.
- [ ] `Kind` struct with fields: construct, kind_variable_number, kc_args.
- [ ] `Kind::base_construction` creates base kinds with the correct constructor.
- [ ] `Kind::unary_con` creates unary constructions (e.g., `list of numbers`).
- [ ] `Kind::binary_con` creates binary constructions (e.g., `relation of numbers to texts`).
- [ ] `Kind::function_kind` creates function kinds with tuple entry structure.
- [ ] `Kind::var_construction` creates kind variable placeholders (A-Z).
- [ ] `Kind::eq` correctly compares kinds structurally (not by pointer).
- [ ] All familiar kinds (`K_number`, `K_text`, `K_object`, `K_value`, etc.) are defined as global constants.
- [ ] All familiar constructors (`CON_list_of`, `CON_relation`, `CON_phrase`, etc.) are defined as global constants.
- [ ] `superkind` returns the correct superkind for each built-in indefinite kind.
- [ ] `Kind::conforms_to` correctly tests the is-a relationship (number conforms to arithmetic value, etc.).
- [ ] `Kind::compatible` returns the correct three-valued result.
- [ ] `join` and `meet` produce correct lattice operations.
- [ ] `Display` for `Kind` produces human-readable output ("number", "list of numbers", etc.).
- [ ] `FromStr` for `Kind` parses basic kind descriptions.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the new crate.

## Out of scope

- **Full `<k-kind>` Preform grammar**: The complete K-grammar from Describing Kinds.w (with stroke-separated constructor name alternatives, phrase-token mode, irregular constructions, etc.) is deferred. We implement a simplified `FromStr` that handles the canonical forms.
- **Kind variable substitution**: `Kinds::substitute` and `Kinds::weaken` are deferred. These depend on the full kind variable system being operational.
- **Kind variable context**: The `KIND_VARIABLE_FROM_CONTEXT` callback and the 26-variable array are deferred. Kind variables are created as placeholders but not yet resolved from context.
- **Dimensional analysis**: `Kinds::Dimensions`, `unit_sequence`, `dimensional_rules`, and arithmetic on kinds are deferred.
- **Casting rules and instance rules**: The linked lists of `kind_constructor_casting_rule` and `kind_constructor_instance` are deferred.
- **Literal patterns**: `literal_pattern`, `constant_compilation_method`, and how constant values of a kind are expressed are deferred.
- **Inference subjects**: `KindSubjects` (the bridge from kinds to the world model's inference subject system) is deferred. This plan focuses on the kind system in isolation.
- **Kind predicates**: `KindPredicates` (unary predicates for the calculus module) are deferred. These will be added when the calculus module is built.
- **Neptune files**: The Neptune file parser for defining built-in kinds from template files is deferred. All kinds are defined programmatically in Rust for now.
- **Run-time identifiers**: Inter identifiers, printing routines, recognition routines, and other run-time support fields on `KindConstructor` are deferred.
- **Indexing and documentation**: Specification text, index priority, documentation references on `KindConstructor` are deferred.
- **Full `Kinds::Behaviour` API**: Many behaviour methods (is_built_in, is_uncertainly_defined, get_constant_compilation_method, etc.) are deferred. We implement only the subset needed for kind construction and the lattice.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated. The kind system is built as plain Rust data structures.
- **Assertion processing**: Processing assertion sentences ("X is a kind of Y", "X has a property called P") is deferred. This plan only builds the kind data structures.
- **Instance system**: Instances of kinds (objects, rooms, etc.) are deferred to a later plan.
- **Property system**: Properties on kinds are deferred to a later plan.
- **Relation system**: Relations between kinds are deferred to a later plan.
