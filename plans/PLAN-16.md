# Plan 16: Calculus Module Foundation — Terms, Atoms, and Propositions
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the core data structures of the calculus module — the predicate calculus engine that represents logical propositions in Inform 7. This includes terms (variables, constants, functions), atoms (the building blocks of propositions), propositions (linked lists of atoms with conjunction, negation, and quantification), and unary predicates with their families.

This is the smallest next step after PLAN-15 because:

1. **The kind system is complete.** PLAN-14 implemented `Kind`, `KindConstructor`, construction functions, equality, conformance, the lattice, familiar kinds, and textual I/O. PLAN-15 implemented the full `Kinds::Behaviour` API with ~40 functions for querying and operating on kinds. The kind system is ready for downstream consumers.

2. **The calculus module is the next layer in the dependency chain.** The architecture is: kinds → calculus → knowledge. The calculus module (`services/calculus-module/`) is a service that provides the predicate calculus representation. The knowledge module (`inform7/knowledge-module/`) builds on top of it to represent facts about the game world. Before we can implement instances, properties, or inference subjects, we need the proposition representation.

3. **Independently testable.** The C reference has a `calculus-test` REPL that exercises terms, atoms, propositions, and unary predicates in isolation. We can replicate this with Rust unit tests that construct terms, build atoms, compose propositions with conjunction/negation/quantification, and test validity.

4. **Prerequisite for kind predicates.** `KindPredicates` (services/calculus-module/Chapter 2/Kind Predicates.w) defines unary predicates `kind=K` for the calculus module. These are used in propositions like "X is a number" or "X is a kind of vehicle". Kind predicates depend on both the kind system and the unary predicate system.

5. **Prerequisite for the knowledge module.** The knowledge module's inference subjects, instances, and properties all use propositions to represent facts. `InferenceSubjects` stores lists of `inference` and `implication` structures that contain propositions. Without the proposition data structures, we can't build the world model.

## Background

### C reference architecture

The calculus module is defined across several files in `services/calculus-module/`:

#### Terms (`Chapter 4/Terms.w`, lines 1-221)

A "term" can be a constant, a variable, or a function of another term:

```c
typedef struct pcalc_term {
    int variable;              /* 0 to 25, or -1 for "not a variable" */
    struct parse_node *constant; /* or NULL for "not a constant" */
    struct pcalc_func *function; /* or NULL for "not a function of another term" */
    int cinder;                /* scope tracking for I6 local variables */
    struct kind *term_checked_as_kind; /* or NULL if unchecked */
} pcalc_term;
```

Variables are represented by numbers 0 to 25 (letters x, y, z, a, b, c, ..., w). Constants are pointers to specification structures. Functions are chains of binary predicates applied to terms (e.g., `f_A(f_B(f_C(x)))`).

#### Atomic Propositions (`Chapter 4/Atomic Propositions.w`, lines 1-302)

Atoms are the syntactic pieces from which propositions are built:

```c
typedef struct pcalc_prop {
    int element;              /* one of the *_ATOM constants */
    int arity;                /* 1 for quantifiers and unary predicates; 2 for BPs; 0 otherwise */
    struct general_pointer predicate; /* indicates which predicate structure is meant */
    struct binary_predicate *saved_bp; /* for problem messages only */
    struct pcalc_term terms[MAX_ATOM_ARITY]; /* terms to which the predicate applies */
    struct quantifier *quant; /* QUANTIFIER_ATOM: which one */
    int quantification_parameter; /* QUANTIFIER_ATOM: e.g., the 3 in "all three" */
    struct pcalc_prop *next;  /* next atom in the list for this proposition */
} pcalc_prop;
```

Atom element types:
- `QUANTIFIER_ATOM` (1) — any generalised quantifier
- `PREDICATE_ATOM` — a regular predicate (unary or binary)
- `NEGATION_OPEN_ATOM` — logical negation applied to contents of group
- `NEGATION_CLOSE_ATOM` — end of logical negation
- `DOMAIN_OPEN_ATOM` — holds the domain of a quantifier
- `DOMAIN_CLOSE_ATOM` — end of quantifier domain

#### Propositions (`Chapter 4/Propositions.w`, lines 1-775)

Propositions are represented as flat linked lists of atoms. Conjunction is implied by adjacency (no explicit AND atom). Negation is bracketed: `NOT< --> P --> NOT>`. Quantification is bracketed: `QUANTIFIER --> IN< --> P --> IN>`.

Key operations:
- `Propositions::implied_conjunction_between` — determines if two adjacent atoms form a conjunction
- `Propositions::is_syntactically_valid` — validates bracket matching, quantifier-domain pairing
- `Propositions::is_complex` — checks if a proposition contains quantifiers, negation, or non-equality binary predicates
- `Propositions::copy` — deep-copies a proposition
- `Propositions::concatenate` — concatenates two propositions (with variable renaming to avoid clashes)

#### Unary Predicates (`Chapter 2/Unary Predicates.w`, lines 1-47)

A lightweight structure representing a unary predicate:

```c
typedef struct unary_predicate {
    struct up_family *family;
    struct kind *assert_kind;
    int composited;           /* for kind UPs only: composite determiner/noun like "somewhere" */
    int unarticled;           /* for kind UPs only: unarticled usage like "vehicle" */
    struct wording calling_name; /* for calling UPs only */
    lcon_ti lcon;             /* for adjectival UPs only */
} unary_predicate;
```

#### Unary Predicate Families (`Chapter 2/Unary Predicate Families.w`)

Families group related unary predicates. Each family has methods for logging, kind inference, testability, and testing. The calculus module defines the family infrastructure; specific families (kind predicates, adjectival predicates, calling predicates) are defined elsewhere.

#### Kind Predicates (`Chapter 2/Kind Predicates.w`, lines 1-125)

A bridge between the kind system and the calculus module. For every kind K, the calculus module provides a unary predicate `kind=K`:

```c
pcalc_prop *KindPredicates::new_atom(kind *K, pcalc_term t) {
    unary_predicate *up = UnaryPredicates::new(kind_up_family);
    up->assert_kind = K;
    return Atoms::unary_PREDICATE_new(up, t);
}
```

### Key C source files

- `services/calculus-module/Chapter 4/Terms.w` — `pcalc_term` struct, creation, copy, variable/constant underlying, writing
- `services/calculus-module/Chapter 4/Atomic Propositions.w` — `pcalc_prop` struct, atom element types, creation, quantifier/predicate atoms, validation, writing
- `services/calculus-module/Chapter 4/Propositions.w` — proposition representation, conjunction, validity, complexity, copy, concatenate, simplify
- `services/calculus-module/Chapter 2/Unary Predicates.w` — `unary_predicate` struct, creation, copy, logging
- `services/calculus-module/Chapter 2/Unary Predicate Families.w` — `up_family` struct, method dispatch, logging, kind inference
- `services/calculus-module/Chapter 2/Kind Predicates.w` — `kind=K` unary predicates, composited/unarticled variants
- `services/calculus-module/Chapter 4/Binding and Substitution.w` — variable binding, substitution, well-formedness
- `services/calculus-module/Chapter 4/Type Check Propositions.w` — type checking propositions against the kind system
- `services/calculus-module/Chapter 1/Calculus Module.w` — module setup, startup

### Current Rust state

- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct with construction, equality, conformance, compatibility, Display, FromStr
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct with all Behaviour API fields
- `crates/conform7-semantics/src/familiar_kinds.rs` — All `K_*` and `CON_*` global constants
- `crates/conform7-semantics/src/lattice.rs` — `superkind`, `join`, `meet` functions
- `crates/conform7-semantics/src/kinds_behaviour.rs` — Full `Kinds::Behaviour` API (~40 functions)

### What's needed

1. **`PcalcTerm` struct** — representation of terms (variables 0-25, constants, functions of other terms).
2. **`PcalcFunc` struct** — function application of a binary predicate to a term (for `f_A(f_B(f_C(x)))` chains).
3. **Atom element types** — `QUANTIFIER_ATOM`, `PREDICATE_ATOM`, `NEGATION_OPEN_ATOM`, `NEGATION_CLOSE_ATOM`, `DOMAIN_OPEN_ATOM`, `DOMAIN_CLOSE_ATOM`.
4. **`PcalcProp` struct** — atomic proposition with element type, arity, predicate reference, terms, quantifier, and next pointer.
5. **Proposition operations** — creation, conjunction (implied by adjacency), negation (bracketed), quantification (bracketed), copy, concatenate, validity checking.
6. **`UnaryPredicate` struct** — lightweight predicate with family reference, assert_kind, composited/unarticled flags.
7. **`UpFamily` struct** — family of related unary predicates with method dispatch for logging, kind inference, testability, testing.
8. **Kind predicates** — `kind=K` unary predicates that bridge kinds to the calculus module.
9. **Unit tests** — construct terms, build atoms, compose propositions, test validity, test unary predicates and families.

## Tasks

### 1. Create the calculus module foundation

- [ ] Create `crates/conform7-semantics/src/calculus/` directory with `mod.rs`:
  ```rust
  pub mod terms;
  pub mod atoms;
  pub mod propositions;
  pub mod unary_predicates;
  pub mod unary_predicate_families;
  pub mod kind_predicates;
  ```

- [ ] Add `pub mod calculus;` to `crates/conform7-semantics/src/lib.rs`.

### 2. Implement `PcalcTerm` and `PcalcFunc`

- [ ] Create `crates/conform7-semantics/src/calculus/terms.rs` with:

  ```rust
  /// Maximum number of variables (0-25 for letters x, y, z, a, b, ..., w).
  pub const MAX_VARIABLES: usize = 26;

  /// Variable letters lookup: x=0, y=1, z=2, a=3, b=4, ..., w=25.
  pub const PCALC_VARS: [char; 26] = ['x', 'y', 'z', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
      'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w'];

  /// A term in predicate calculus — a variable, constant, or function of another term.
  ///
  /// Corresponds to `pcalc_term` in the C reference
  /// (`services/calculus-module/Chapter 4/Terms.w`, lines 27-33).
  ///
  /// At all times exactly one of `variable`, `constant`, or `function` is used.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct PcalcTerm {
      /// Variable number 0-25, or -1 for "not a variable".
      pub variable: i8,
      /// Constant value (simplified: a string name for now).
      pub constant: Option<&'static str>,
      /// Function of another term, or None.
      pub function: Option<Box<PcalcFunc>>,
  }
  ```

- [ ] Implement `PcalcFunc`:

  ```rust
  /// A function application inside a term.
  ///
  /// Corresponds to `pcalc_func` in the C reference
  /// (`services/calculus-module/Chapter 4/Terms.w`, lines 51-55).
  ///
  /// Terms such as f_A(f_B(f_C(x))) are chains of these structures.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct PcalcFunc {
      /// The binary predicate (simplified: a string name for now).
      pub bp_name: &'static str,
      /// Which term of the predicate this derives from (0 or 1).
      pub from_term: u8,
      /// The term to which we apply the function.
      pub fn_of: Box<PcalcTerm>,
  }
  ```

- [ ] Implement term creation functions (matching Terms.w lines 63-83):
  - `PcalcTerm::new_variable(v: u8) -> Self` — creates a variable term (0-25).
  - `PcalcTerm::new_constant(c: &'static str) -> Self` — creates a constant term.
  - `PcalcTerm::new_function(bp_name: &'static str, fn_of: PcalcTerm, from_term: u8) -> Self` — creates a function term.

- [ ] Implement term accessors (matching Terms.w lines 127-144):
  - `PcalcTerm::constant_underlying(&self) -> Option<&'static str>` — returns the constant at the bottom of a function chain.
  - `PcalcTerm::variable_underlying(&self) -> Option<u8>` — returns the variable at the bottom of a function chain.

- [ ] Implement `Display` for `PcalcTerm` (matching Terms::write in Terms.w lines 195-220):
  - Variables: print the letter (x, y, z, a, b, ...).
  - Constants: print the constant name.
  - Functions: print the function application.

- [ ] Add unit tests:
  - Test that `new_variable(0)` creates a term with variable 0.
  - Test that `new_constant("hello")` creates a term with constant "hello".
  - Test that `new_function` creates a function term.
  - Test that `constant_underlying` follows function chains to find the constant.
  - Test that `variable_underlying` follows function chains to find the variable.
  - Test `Display` output for variables, constants, and functions.

### 3. Implement atom element types and `PcalcProp`

- [ ] Create `crates/conform7-semantics/src/calculus/atoms.rs` with:

  ```rust
  /// Maximum arity for atoms (matching MAX_ATOM_ARITY = 2).
  pub const MAX_ATOM_ARITY: usize = 2;

  /// Atom element types (matching the *_ATOM constants in Atomic Propositions.w lines 30-35).
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  pub enum AtomElement {
      /// A generalised quantifier (binds a variable).
      Quantifier,
      /// A regular predicate (unary or binary).
      Predicate,
      /// Logical negation opening bracket.
      NegationOpen,
      /// Logical negation closing bracket.
      NegationClose,
      /// Quantifier domain opening bracket.
      DomainOpen,
      /// Quantifier domain closing bracket.
      DomainClose,
  }
  ```

- [ ] Implement helper functions (matching Atoms::is_opener, is_closer, counterpart in Atomic Propositions.w lines 40-60):
  - `AtomElement::is_opener(&self) -> bool` — true for NegationOpen, DomainOpen.
  - `AtomElement::is_closer(&self) -> bool` — true for NegationClose, DomainClose.
  - `AtomElement::counterpart(&self) -> Option<AtomElement>` — returns the matching bracket element.

- [ ] Implement `PcalcProp` struct (matching `pcalc_prop` in Atomic Propositions.w lines 14-23):

  ```rust
  /// An atomic proposition — the building block of propositions.
  ///
  /// Corresponds to `pcalc_prop` in the C reference
  /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 14-23).
  ///
  /// Propositions are linked lists of these atoms. Conjunction is implied
  /// by adjacency (no explicit AND atom). Negation is bracketed:
  /// NOT< --> P --> NOT>. Quantification is bracketed:
  /// QUANTIFIER --> IN< --> P --> IN>.
  #[derive(Clone, Debug)]
  pub struct PcalcProp {
      /// The element type of this atom.
      pub element: AtomElement,
      /// Arity: 1 for quantifiers and unary predicates; 2 for binary predicates; 0 otherwise.
      pub arity: u8,
      /// Predicate reference (for PREDICATE_ATOM): either a unary or binary predicate name.
      pub predicate: Option<PredicateRef>,
      /// Terms to which the predicate applies (up to MAX_ATOM_ARITY).
      pub terms: [Option<PcalcTerm>; MAX_ATOM_ARITY],
      /// Quantifier reference (for QUANTIFIER_ATOM).
      pub quantifier: Option<QuantifierRef>,
      /// Quantification parameter (e.g., the 3 in "all three").
      pub quantification_parameter: i32,
      /// Next atom in the proposition linked list.
      pub next: Option<Box<PcalcProp>>,
  }
  ```

- [ ] Implement predicate and quantifier reference types:

  ```rust
  /// Reference to a predicate (unary or binary).
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum PredicateRef {
      /// A unary predicate, identified by name.
      Unary(&'static str),
      /// A binary predicate, identified by name.
      Binary(&'static str),
  }

  /// Reference to a quantifier.
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum QuantifierRef {
      /// Existential quantifier (there exists).
      Exists,
      /// Universal quantifier (for all).
      ForAll,
      /// "Not exists" quantifier.
      NotExists,
      /// "Not for all" quantifier.
      NotForAll,
  }
  ```

- [ ] Implement atom creation functions (matching Atoms::new, QUANTIFIER_new, unary_PREDICATE_new, binary_PREDICATE_new in Atomic Propositions.w lines 65-188):
  - `PcalcProp::new(element: AtomElement) -> Self` — creates a new atom with the given element type.
  - `PcalcProp::quantifier_new(quant: QuantifierRef, v: u8, parameter: i32) -> Self` — creates a quantifier atom.
  - `PcalcProp::unary_predicate_new(predicate_name: &'static str, term: PcalcTerm) -> Self` — creates a unary predicate atom.
  - `PcalcProp::binary_predicate_new(predicate_name: &'static str, term0: PcalcTerm, term1: PcalcTerm) -> Self` — creates a binary predicate atom.

- [ ] Implement atom accessors (matching Atomic Propositions.w lines 104-218):
  - `PcalcProp::is_quantifier(&self) -> bool`
  - `PcalcProp::get_quantifier(&self) -> Option<QuantifierRef>`
  - `PcalcProp::is_existence_quantifier(&self) -> bool`
  - `PcalcProp::is_forall_quantifier(&self) -> bool`
  - `PcalcProp::is_binary_predicate(&self) -> Option<&'static str>` — returns the binary predicate name if this is a binary predicate atom.
  - `PcalcProp::is_equality_predicate(&self) -> bool` — checks if this is the equality predicate.

- [ ] Implement atom validation (matching Atoms::validate in Atomic Propositions.w lines 223-239):
  - `PcalcProp::validate(&self) -> Result<(), String>` — validates arity, predicate presence, quantifier variable.

- [ ] Implement `Display` for `PcalcProp` (matching Atoms::write in Atomic Propositions.w lines 248-301):
  - Quantifier atoms: print the quantifier name and variable.
  - Predicate atoms: print the predicate name and terms.
  - Negation brackets: print `NOT<` and `NOT>`.
  - Domain brackets: print `IN<` and `IN>`.
  - Equality: special notation `(x == C)`.

- [ ] Add unit tests:
  - Test that `PcalcProp::new(NegationOpen)` creates an atom with the correct element.
  - Test that `quantifier_new(Exists, 0, 0)` creates a quantifier atom with variable 0.
  - Test that `unary_predicate_new("kind=number", term)` creates a unary predicate atom.
  - Test that `binary_predicate_new("equality", t0, t1)` creates a binary predicate atom.
  - Test `is_quantifier`, `is_existence_quantifier`, `is_forall_quantifier`.
  - Test `is_binary_predicate` returns the predicate name.
  - Test `is_equality_predicate` for equality atoms.
  - Test `validate` on valid and invalid atoms.
  - Test `Display` output for each atom type.

### 4. Implement propositions

- [ ] Create `crates/conform7-semantics/src/calculus/propositions.rs` with:

  ```rust
  /// Maximum proposition group nesting (matching MAX_PROPOSITION_GROUP_NESTING = 100).
  pub const MAX_PROPOSITION_GROUP_NESTING: usize = 100;
  ```

- [ ] Implement proposition creation and operations (matching Propositions.w lines 50-301):

  - `Propositions::new_single(atom: PcalcProp) -> PcalcProp` — creates a proposition from a single atom.
  - `Propositions::conjunction(p1: PcalcProp, p2: PcalcProp) -> PcalcProp` — concatenates two propositions (conjunction is implied by adjacency). Renames variables in p2 if they clash with p1.
  - `Propositions::negation(p: PcalcProp) -> PcalcProp` — wraps a proposition in NOT< ... NOT> brackets.
  - `Propositions::quantification(quant: PcalcProp, domain: PcalcProp) -> PcalcProp` — wraps a domain in QUANTIFIER ... IN< ... IN> brackets.
  - `Propositions::copy(prop: &PcalcProp) -> PcalcProp` — deep-copies a proposition (matching Propositions::copy in Propositions.w lines 287-301).

- [ ] Implement proposition analysis (matching Propositions.w lines 50-273):

  - `Propositions::implied_conjunction_between(p1: &PcalcProp, p2: &PcalcProp) -> bool` — determines if two adjacent atoms form a conjunction (matching Propositions.w lines 50-57).
  - `Propositions::is_syntactically_valid(prop: &PcalcProp) -> Result<(), String>` — validates bracket matching, quantifier-domain pairing (matching Propositions::is_syntactically_valid in Propositions.w lines 209-247).
  - `Propositions::is_complex(prop: &PcalcProp) -> bool` — checks if a proposition contains quantifiers, negation, or non-equality binary predicates (matching Propositions::is_complex in Propositions.w lines 258-273).

- [ ] Implement proposition traversal:

  ```rust
  /// Traverse a proposition, visiting each atom in order.
  /// Returns the number of atoms visited.
  pub fn traverse<F>(prop: &PcalcProp, mut f: F) -> usize
  where
      F: FnMut(&PcalcProp, Option<&PcalcProp>) -> bool,
  {
      // f returns true to continue, false to stop
      // second argument is the previous atom (None for the first)
  }
  ```

- [ ] Implement `Display` for propositions (matching Propositions::write in Propositions.w lines 146-156):
  - Print `<< atom1 atom2 atom3 >>` with conjunction markers (`^`) between implied conjunctions.

- [ ] Add unit tests:
  - Test that a single-atom proposition is valid.
  - Test that `conjunction` creates a linked list of two atoms.
  - Test that `negation` wraps atoms in NOT< ... NOT> brackets.
  - Test that `quantification` wraps a domain in QUANTIFIER ... IN< ... IN> brackets.
  - Test `is_syntactically_valid` on valid propositions.
  - Test `is_syntactically_valid` on invalid propositions (unmatched brackets, quantifier without domain).
  - Test `is_complex` on simple and complex propositions.
  - Test `copy` produces a structurally identical but independent proposition.
  - Test `implied_conjunction_between` for various atom pairs.
  - Test `traverse` visits all atoms in order.
  - Test `Display` output for propositions.

### 5. Implement unary predicates and families

- [ ] Create `crates/conform7-semantics/src/calculus/unary_predicates.rs` with:

  ```rust
  /// A unary predicate — true or false when applied to a single term.
  ///
  /// Corresponds to `unary_predicate` in the C reference
  /// (`services/calculus-module/Chapter 2/Unary Predicates.w`, lines 11-18).
  #[derive(Clone, Debug)]
  pub struct UnaryPredicate {
      /// The family this predicate belongs to.
      pub family: &'static UpFamily,
      /// The kind asserted by this predicate (for kind=K predicates).
      pub assert_kind: Option<usize>,  // simplified: index into a kind registry
      /// Whether this is a composited predicate (composite determiner/noun like "somewhere").
      pub composited: bool,
      /// Whether this is an unarticled predicate (unarticled usage like "vehicle").
      pub unarticled: bool,
      /// Calling name (for calling predicates only).
      pub calling_name: Option<&'static str>,
  }
  ```

- [ ] Implement creation and copy (matching UnaryPredicates::new, copy in Unary Predicates.w lines 21-46):
  - `UnaryPredicate::new(family: &'static UpFamily) -> Self`
  - `UnaryPredicate::copy(&self) -> Self`

- [ ] Create `crates/conform7-semantics/src/calculus/unary_predicate_families.rs` with:

  ```rust
  /// A family of related unary predicates.
  ///
  /// Corresponds to `up_family` in the C reference
  /// (`services/calculus-module/Chapter 2/Unary Predicate Families.w`).
  #[derive(Clone, Debug)]
  pub struct UpFamily {
      /// Name of this family (for debugging).
      pub name: &'static str,
      /// Method implementations for this family.
      pub methods: UpFamilyMethods,
  }

  /// Methods that can be implemented for a unary predicate family.
  #[derive(Clone, Debug)]
  pub struct UpFamilyMethods {
      /// Log a unary predicate to the debug log.
      pub log: fn(&UpFamily, &UnaryPredicate) -> String,
      /// Infer the kind from a unary predicate.
      pub infer_kind: fn(&UpFamily, &UnaryPredicate) -> Option<usize>,
      /// Whether predicates in this family are testable at compile-time.
      pub testable: fn(&UpFamily, &UnaryPredicate) -> bool,
      /// Test a predicate at compile-time (only called if testable returns true).
      pub test: fn(&UpFamily, &UnaryPredicate) -> bool,
  }
  ```

- [ ] Implement family creation:
  - `UpFamily::new(name: &'static str, methods: UpFamilyMethods) -> Self`

- [ ] Add unit tests:
  - Test that `UnaryPredicate::new` creates a predicate with the correct family.
  - Test that `UnaryPredicate::copy` produces an independent copy.
  - Test that `UpFamily` methods can be called on predicates.
  - Test a simple family with custom log/infer_kind/testable/test methods.

### 6. Implement kind predicates

- [ ] Create `crates/conform7-semantics/src/calculus/kind_predicates.rs` with:

  ```rust
  /// The family for kind=K unary predicates.
  ///
  /// Corresponds to `kind_up_family` in the C reference
  /// (`services/calculus-module/Chapter 2/Kind Predicates.w`, line 9).
  pub static KIND_UP_FAMILY: Lazy<UpFamily> = Lazy::new(|| {
      UpFamily::new("kind", UpFamilyMethods {
          log: kind_log,
          infer_kind: kind_infer_kind,
          testable: kind_testable,
          test: kind_test,
      })
  });
  ```

- [ ] Implement kind predicate functions (matching Kind Predicates.w lines 14-124):
  - `KindPredicates::new_atom(kind_name: &'static str, term: PcalcTerm) -> PcalcProp` — creates a `kind=K` atom (matching KindPredicates::new_atom in Kind Predicates.w lines 23-27).
  - `KindPredicates::is_kind_atom(prop: &PcalcProp) -> bool` — checks if an atom is a kind predicate (matching Kind Predicates.w lines 29-35).
  - `KindPredicates::get_kind(prop: &PcalcProp) -> Option<&'static str>` — extracts the kind name from a kind predicate atom (matching Kind Predicates.w lines 37-43).
  - `KindPredicates::new_composited_atom(kind_name: &'static str, term: PcalcTerm) -> PcalcProp` — creates a composited kind predicate (matching Kind Predicates.w lines 50-55).
  - `KindPredicates::is_composited_atom(prop: &PcalcProp) -> bool` — checks if a kind predicate is composited (matching Kind Predicates.w lines 57-65).

- [ ] Implement family method implementations:
  - `kind_log(family, up) -> String` — logs the kind predicate (matching KindPredicates::log_kind in Kind Predicates.w lines 119-124).
  - `kind_infer_kind(family, up) -> Option<usize>` — returns the assert_kind (matching KindPredicates::infer_kind in Kind Predicates.w lines 99-101).
  - `kind_testable(family, up) -> bool` — always returns true (matching KindPredicates::testable in Kind Predicates.w lines 109-111).
  - `kind_test(family, up) -> bool` — always returns true (matching KindPredicates::test in Kind Predicates.w lines 113-116).

- [ ] Add unit tests:
  - Test that `new_atom("number", term)` creates a PREDICATE_ATOM with arity 1.
  - Test that `is_kind_atom` returns true for kind predicate atoms.
  - Test that `get_kind` returns the correct kind name.
  - Test that `new_composited_atom` creates a composited kind predicate.
  - Test that `is_composited_atom` returns true for composited atoms.
  - Test the family method implementations (log, infer_kind, testable, test).

### 7. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `PcalcTerm` struct exists with variable, constant, and function variants.
- [ ] `PcalcFunc` struct exists for function chains inside terms.
- [ ] `PcalcTerm::new_variable`, `new_constant`, `new_function` create terms correctly.
- [ ] `PcalcTerm::constant_underlying` and `variable_underlying` follow function chains.
- [ ] `AtomElement` enum exists with all six element types (Quantifier, Predicate, NegationOpen, NegationClose, DomainOpen, DomainClose).
- [ ] `AtomElement::is_opener`, `is_closer`, `counterpart` work correctly.
- [ ] `PcalcProp` struct exists with element, arity, predicate, terms, quantifier, and next fields.
- [ ] `PcalcProp::quantifier_new`, `unary_predicate_new`, `binary_predicate_new` create atoms correctly.
- [ ] `PcalcProp::validate` correctly validates atoms.
- [ ] `Propositions::conjunction` creates linked lists with implied conjunction.
- [ ] `Propositions::negation` wraps atoms in NOT< ... NOT> brackets.
- [ ] `Propositions::quantification` wraps domains in QUANTIFIER ... IN< ... IN> brackets.
- [ ] `Propositions::is_syntactically_valid` correctly validates bracket matching and quantifier-domain pairing.
- [ ] `Propositions::is_complex` correctly identifies complex propositions.
- [ ] `Propositions::copy` produces structurally identical independent copies.
- [ ] `UnaryPredicate` struct exists with family, assert_kind, composited, unarticled fields.
- [ ] `UpFamily` struct exists with method dispatch for log, infer_kind, testable, test.
- [ ] `KindPredicates::new_atom` creates `kind=K` unary predicate atoms.
- [ ] `KindPredicates::is_kind_atom` and `get_kind` correctly identify and extract kind predicates.
- [ ] `KindPredicates::new_composited_atom` creates composited kind predicates.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Binary predicates and families**: The full `binary_predicate` struct, `bp_family`, term details, reversal, and task functions are deferred. We implement only the atom-level binary predicate reference (by name) for proposition construction.
- **The equality relation**: `R_equality` and the equality binary predicate implementation are deferred. We detect equality by name in `is_equality_predicate`.
- **Binding and substitution**: `Propositions::substitute`, `Propositions::binding`, and well-formedness checking for variable scoping are deferred. We implement basic validity (bracket matching) but not full variable scoping checks.
- **Type checking**: `Propositions::type_check` and the full type-checking system are deferred. Kind predicates are created but not type-checked against the kind system.
- **Quantifier implementations**: The full `quantifier` struct with `Quantifiers::is_now_assertable` and quantifier-specific logic is deferred. We implement a simplified `QuantifierRef` enum.
- **Adjectival predicates**: `AdjectivalPredicates` and the adjectival unary predicate family are deferred.
- **Calling predicates**: Calling unary predicates (for named values) are deferred.
- **Cinders and deferrals**: The cinder system for I6 local variable scope tracking is deferred.
- **Compilation schemas**: `i6_schema` and the compilation of propositions to I6 code are deferred.
- **Sentence conversions**: `SentenceConversions` and `Simplifications` (Chapter 5) are deferred.
- **Inference subjects**: The `InferenceSubjects` system (knowledge-module/Chapter 4) is deferred. This plan focuses on the calculus module only.
- **Kind subjects**: `KindSubjects` (knowledge-module/Chapter 4/Kind Subjects.w) — the bridge from kinds to inference subjects — is deferred.
- **Instance system**: `Instances` (knowledge-module/Chapter 2/Instances.w) and `InstanceSubjects` are deferred.
- **Property system**: `Properties` (knowledge-module/Chapter 3/Properties.w) is deferred.
- **Full `Kinds::Dimensions` API**: The dimensions system for unit analysis is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
