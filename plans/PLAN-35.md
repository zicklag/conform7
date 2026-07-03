# Plan 35: Creation Predicates — The `calling`, `is_a_var`, `is_a_const`, and `is_a_kind` Unary Predicate Families
**Status**: Complete
**Target**: 1 day

## Goal

Implement the Creation Predicates system — four unary predicate families that are used by the Assert Propositions module to create instances, variables, constants, and kinds during proposition assertion. This creates the `calling_up_family`, `is_a_var_up_family`, `is_a_const_up_family`, and `is_a_kind_up_family` families with their associated unary predicates and helper functions.

This is the smallest next step after PLAN-34 because:

1. **It's the first module in the assertions module startup sequence.** The assertions module startup (`inform7/assertions-module/Chapter 1/Assertions Module.w`, lines 25-37) calls `CreationPredicates::start()` at line 26 — right after `AdjectivalPredicates::start()` at line 25. But `AdjectivalPredicates` depends on the adjective meaning system (PLAN-28, Complete), adjective ambiguity, and the adjectival predicates infrastructure — all of which are in the assertions module's Chapter 8. `CreationPredicates` is simpler: it creates four unary predicate families with minimal methods, depending only on calculus infrastructure that already exists.

2. **It depends only on calculus infrastructure that's already built.** The `CreationPredicates` module uses `UnaryPredicateFamilies::new()` (PLAN-16, Complete), `UnaryPredicates::new()` (PLAN-15, Complete), `Atoms::unary_PREDICATE_new()` (PLAN-14, Complete), and `Terms::new_variable()` (PLAN-13, Complete) — all from the calculus module. No knowledge module dependencies, no Preform grammar, no run-time compilation.

3. **It's the smallest remaining independent module.** At only ~200 lines of C (`The Creation Predicates.w`), this is one of the simplest pieces of the assertions module not yet implemented. It creates four families with straightforward methods: `calling_up_family` (for naming bound variables), `is_a_var_up_family` (for variable declarations), `is_a_const_up_family` (for constant declarations), and `is_a_kind_up_family` (for kind declarations).

4. **It's a prerequisite for the Assert Propositions module.** The `Assert::inner_slated` function (`inform7/knowledge-module/Chapter 1/Assert Propositions.w`, lines 254-272) uses all four families when processing `QUANTIFIER_ATOM` atoms during proposition assertion. Without the creation predicates, the assertion pipeline cannot create instances, variables, constants, or kinds from propositions.

5. **It's independently testable without grammar parsing, the knowledge module, or run-time compilation.** We can create the four families via `CreationPredicates::start()`, create unary predicates for each family, test the helper functions (`is_calling_up_atom`, `what_kind_of_calling`, `calling_up`, `get_calling_name`, `is_a_var_up`, `is_a_const_up`, `is_a_kind_up`, `what_kind`, `contains_callings`), and test the stock methods — all programmatically. No Preform grammar, no knowledge module, no run-time compilation.

6. **It introduces the creation predicate pattern — a fundamental assertion mechanism.** The four families represent the four ways the assertion pipeline creates things from propositions: calling (naming a bound variable), is-a-var (declaring a variable), is-a-const (declaring a constant), and is-a-kind (declaring a kind). Implementing these now establishes the pattern for the Assert Propositions module.

## Background

### C reference architecture

#### Creation Predicates (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 1-201)

The Creation Predicates system provides four unary predicate families used during proposition assertion:

```c
up_family *calling_up_family = NULL;
up_family *is_a_var_up_family = NULL;
up_family *is_a_const_up_family = NULL;
up_family *is_a_kind_up_family = NULL;

unary_predicate *is_a_var_up = NULL;
unary_predicate *is_a_const_up = NULL;
```

The `start()` function creates the families and adds methods:

```c
void CreationPredicates::start(void) {
    calling_up_family = UnaryPredicateFamilies::new();
    METHOD_ADD(calling_up_family, LOG_UPF_MTID, CreationPredicates::log_calling);
    // #ifdef CORE_MODULE: TYPECHECK_UPF_MTID, SCHEMA_UPF_MTID

    is_a_var_up_family = UnaryPredicateFamilies::new();
    METHOD_ADD(is_a_var_up_family, STOCK_UPF_MTID, CreationPredicates::stock_is_a_var);
    METHOD_ADD(is_a_var_up_family, LOG_UPF_MTID, CreationPredicates::log_is_a_var);

    is_a_const_up_family = UnaryPredicateFamilies::new();
    METHOD_ADD(is_a_const_up_family, STOCK_UPF_MTID, CreationPredicates::stock_is_a_const);
    METHOD_ADD(is_a_const_up_family, LOG_UPF_MTID, CreationPredicates::log_is_a_const);

    is_a_kind_up_family = UnaryPredicateFamilies::new();
    METHOD_ADD(is_a_kind_up_family, LOG_UPF_MTID, CreationPredicates::log_is_a_kind);
    // #ifdef CORE_MODULE: TYPECHECK_UPF_MTID
}
```

Key helper functions:

```c
int CreationPredicates::is_calling_up_atom(pcalc_prop *prop) {
    if ((prop->element == PREDICATE_ATOM) && (prop->arity == 1)) {
        unary_predicate *up = RETRIEVE_POINTER_unary_predicate(prop->predicate);
        if (up->family == calling_up_family) return TRUE;
    }
    return FALSE;
}

kind *CreationPredicates::what_kind_of_calling(pcalc_prop *prop) {
    if ((prop) && (prop->element == PREDICATE_ATOM) && (prop->arity == 1)) {
        unary_predicate *up = RETRIEVE_POINTER_unary_predicate(prop->predicate);
        if (up->family == calling_up_family) return up->assert_kind;
    }
    return NULL;
}

pcalc_prop *CreationPredicates::calling_up(wording W, pcalc_term t, kind *K) {
    unary_predicate *up = UnaryPredicates::new(calling_up_family);
    up->calling_name = W;
    up->assert_kind = K;
    return Atoms::unary_PREDICATE_new(up, t);
}

wording CreationPredicates::get_calling_name(pcalc_prop *prop) {
    if ((prop->element == PREDICATE_ATOM) && (prop->arity == 1)) {
        unary_predicate *up = RETRIEVE_POINTER_unary_predicate(prop->predicate);
        if (up->family == calling_up_family) return up->calling_name;
    }
    return EMPTY_WORDING;
}
```

Stock methods create singleton unary predicates:

```c
void CreationPredicates::stock_is_a_var(up_family *self, int n) {
    if (n == 1) {
        is_a_var_up = UnaryPredicates::new(is_a_var_up_family);
    }
}

void CreationPredicates::stock_is_a_const(up_family *self, int n) {
    if (n == 1) {
        is_a_const_up = UnaryPredicates::new(is_a_const_up_family);
    }
}
```

Atom creation helpers:

```c
pcalc_prop *CreationPredicates::is_a_var_up(pcalc_term t) {
    return Atoms::unary_PREDICATE_new(is_a_var_up, t);
}

pcalc_prop *CreationPredicates::is_a_const_up(pcalc_term t) {
    return Atoms::unary_PREDICATE_new(is_a_const_up, t);
}

pcalc_prop *CreationPredicates::is_a_kind_up(pcalc_term t, kind *K) {
    unary_predicate *up = UnaryPredicates::new(is_a_kind_up_family);
    up->assert_kind = K;
    return Atoms::unary_PREDICATE_new(up, t);
}

kind *CreationPredicates::what_kind(pcalc_prop *prop) {
    if ((prop) && (prop->element == PREDICATE_ATOM) && (prop->arity == 1)) {
        unary_predicate *up = RETRIEVE_POINTER_unary_predicate(prop->predicate);
        if (up->family == is_a_kind_up_family) return up->assert_kind;
    }
    return NULL;
}
```

The `contains_callings` helper:

```c
int CreationPredicates::contains_callings(pcalc_prop *prop) {
    for (pcalc_prop *p = prop; p; p = p->next)
        if (CreationPredicates::is_calling_up_atom(p))
            return TRUE;
    return FALSE;
}
```

### Key C source files

- `inform7/assertions-module/Chapter 8/The Creation Predicates.w` — `CreationPredicates` module, `calling_up_family`, `is_a_var_up_family`, `is_a_const_up_family`, `is_a_kind_up_family`, `is_a_var_up`, `is_a_const_up`, `start`, `is_calling_up_atom`, `what_kind_of_calling`, `calling_up`, `get_calling_name`, `stock_is_a_var`, `stock_is_a_const`, `is_a_var_up` (atom), `is_a_const_up` (atom), `is_a_kind_up` (atom), `what_kind`, `contains_callings`, `log_calling`, `log_is_a_var`, `log_is_a_const`, `log_is_a_kind` (201 lines)
- `services/calculus-module/Chapter 2/Unary Predicate Families.w` — `UnaryPredicateFamilies::new`, `UpFamily` struct, method dispatch (PLAN-16, Complete)
- `services/calculus-module/Chapter 2/Unary Predicates.w` — `UnaryPredicate` struct, `UnaryPredicates::new` (PLAN-15, Complete)
- `services/calculus-module/Chapter 4/Atomic Propositions.w` — `Atoms::unary_PREDICATE_new`, `PcalcProp` struct (PLAN-14, Complete)
- `services/calculus-module/Chapter 4/Terms.w` — `PcalcTerm` struct, `Terms::new_variable` (PLAN-13, Complete)
- `inform7/knowledge-module/Chapter 1/Assert Propositions.w` — `Assert::inner_slated` uses creation predicates during proposition assertion (deferred)
- `inform7/assertions-module/Chapter 1/Assertions Module.w` — module startup, calls `CreationPredicates::start()` (line 26)

### Current Rust state

- `crates/conform7-semantics/src/calculus/unary_predicate_families.rs` — `UpFamily` struct, `UpFamilyMethods` struct, `UnaryPredicateFamilies::new()`, `UnaryPredicateFamilies::stock()`, `UnaryPredicateFamilies::typecheck()`, `UnaryPredicateFamilies::assert()`, `UnaryPredicateFamilies::log()`, unit tests (PLAN-16, Complete).
- `crates/conform7-semantics/src/calculus/unary_predicates.rs` — `UnaryPredicate` struct with `family`, `lcon`, `calling_name`, `assert_kind` fields, `UnaryPredicates::new()`, unit tests (PLAN-15, Complete).
- `crates/conform7-semantics/src/calculus/atoms.rs` — `PcalcProp` struct, `AtomElement` enum, `Atoms::unary_PREDICATE_new()`, `Atoms::PREDICATE_new()`, unit tests (PLAN-14, Complete).
- `crates/conform7-semantics/src/calculus/terms.rs` — `PcalcTerm` struct, `Terms::new_variable()`, `Terms::new_constant()`, unit tests (PLAN-13, Complete).
- `crates/conform7-semantics/src/calculus/propositions.rs` — `Propositions` module, proposition operations (conjunction, negation, quantification, validity), unit tests (PLAN-12, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/calculus/kind_predicates.rs` — `KindPredicates` module, `kind_up_family`, `KindPredicates::start()`, `KindPredicates::new_kind_up()`, unit tests (PLAN-15, Complete).
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules.
- `crates/conform7-semantics/src/knowledge/` — all knowledge module submodules (PLANs 17-34, Complete).

### What's needed

1. **`CreationPredicates` module** — a new module `creation_predicates` in the assertions crate (or calculus crate) with:
   - `CreationPredicates::start()` — creates the four unary predicate families:
     - `calling_up_family` — for naming bound variables (called atoms)
     - `is_a_var_up_family` — for variable declarations
     - `is_a_const_up_family` — for constant declarations
     - `is_a_kind_up_family` — for kind declarations
   - `CreationPredicates::is_calling_up_atom(prop)` — checks if a proposition atom is a calling atom
   - `CreationPredicates::what_kind_of_calling(prop)` — returns the kind from a calling atom
   - `CreationPredicates::calling_up(name, term, kind_name, families)` — creates a calling atom with a name and optional kind
   - `CreationPredicates::get_calling_name(prop)` — returns the calling name from a calling atom
   - `CreationPredicates::is_a_var_up(term, families)` — creates an is-a-var atom
   - `CreationPredicates::is_a_const_up(term, families)` — creates an is-a-const atom
   - `CreationPredicates::is_a_kind_up(term, kind_name, families)` — creates an is-a-kind atom with a specific kind
   - `CreationPredicates::what_kind(prop)` — returns the kind from an is-a-kind atom
   - `CreationPredicates::contains_callings(prop)` — checks if a proposition contains any calling atoms
   - Stock methods for `is_a_var_up_family` and `is_a_const_up_family` that create singleton unary predicates at stock stage 1
   - Log methods for all four families
   - Global constants for the four family indices
   - Global constants for the singleton `is_a_var_up` and `is_a_const_up` predicate indices
   - Simplified: no `#ifdef CORE_MODULE` typecheck/schema methods (deferred)
   - Simplified: no `PreformUtilities::wording` (uses string names)
   - Simplified: no `wording` type (uses `&str` for names)

2. **Integration with the calculus module** — add the `creation_predicates` module declaration to the calculus module's `mod.rs`.

3. **Unit tests** — test `CreationPredicates::start()` (creates four families with correct methods), test `CreationPredicates::is_calling_up_atom` (returns true for calling atoms, false otherwise), test `CreationPredicates::calling_up` (creates a calling atom with the right name and kind), test `CreationPredicates::get_calling_name` (returns the name from a calling atom), test `CreationPredicates::what_kind_of_calling` (returns the kind from a calling atom), test `CreationPredicates::is_a_var_up` (creates an is-a-var atom), test `CreationPredicates::is_a_const_up` (creates an is-a-const atom), test `CreationPredicates::is_a_kind_up` (creates an is-a-kind atom with the right kind), test `CreationPredicates::what_kind` (returns the kind from an is-a-kind atom), test `CreationPredicates::contains_callings` (returns true if any calling atoms exist in a proposition), test stock methods (create singleton predicates at stage 1), test log methods (produce correct output).

## Tasks

### 1. Create the `CreationPredicates` module

- [ ] Create `crates/conform7-semantics/src/calculus/creation_predicates.rs` with:

  ```rust
  /// The Creation Predicates system — unary predicate families for creating
  /// instances, variables, constants, and kinds during proposition assertion.
  ///
  /// Corresponds to `CreationPredicates` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`).
  ///
  /// Creates four unary predicate families:
  /// - calling_up_family — for naming bound variables (called atoms)
  /// - is_a_var_up_family — for variable declarations
  /// - is_a_const_up_family — for constant declarations
  /// - is_a_kind_up_family — for kind declarations
  ///
  /// These families are used by the Assert Propositions module when processing
  /// QUANTIFIER_ATOM atoms during proposition assertion. The calling family
  /// provides a way to name bound variables (e.g., "called the den"), while
  /// the is-a-var, is-a-const, and is-a-kind families mark what kind of thing
  /// is being created by an existential quantifier.
  ///
  /// Simplified:
  /// - No #ifdef CORE_MODULE typecheck/schema methods (deferred)
  /// - No PreformUtilities::wording (uses string names)
  /// - No wording type (uses &str for names)
  /// - No LocalVariables::ensure_calling (deferred)
  /// - No RTAdjectives::task_fn_iname (deferred)
  use crate::calculus::unary_predicate_families::{
      UpFamily, UpFamilyMethods, UnaryPredicateFamilies,
  };
  use crate::calculus::unary_predicates::{UnaryPredicate, UnaryPredicates};
  use crate::calculus::atoms::{AtomElement, PcalcProp, Atoms};
  use crate::calculus::terms::PcalcTerm;
  ```

- [ ] Define global constants and the `CreationPredicates` struct:

  ```rust
  /// Index of the calling family in the family registry.
  pub const CALLING_FAMILY: usize = 0;

  /// Index of the is-a-var family in the family registry.
  pub const IS_A_VAR_FAMILY: usize = 1;

  /// Index of the is-a-const family in the family registry.
  pub const IS_A_CONST_FAMILY: usize = 2;

  /// Index of the is-a-kind family in the family registry.
  pub const IS_A_KIND_FAMILY: usize = 3;

  /// The creation predicates module.
  ///
  /// Corresponds to `CreationPredicates` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`).
  pub struct CreationPredicates;
  ```

- [ ] Implement `CreationPredicates::start()`:

  ```rust
  /// Create the four creation predicate families with their methods.
  ///
  /// Corresponds to `CreationPredicates::start` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 22-42).
  ///
  /// Returns (families, predicates) where:
  /// - families[CALLING_FAMILY] = calling_up_family
  /// - families[IS_A_VAR_FAMILY] = is_a_var_up_family
  /// - families[IS_A_CONST_FAMILY] = is_a_const_up_family
  /// - families[IS_A_KIND_FAMILY] = is_a_kind_up_family
  /// - predicates is empty (stock methods fill it)
  pub fn start() -> (Vec<UpFamily>, Vec<UnaryPredicate>) {
      let calling_family = UpFamily {
          name: "calling",
          methods: UpFamilyMethods {
              log: Some(CreationPredicates::log_calling),
              ..UpFamilyMethods::default()
          },
      };

      let is_a_var_family = UpFamily {
          name: "is-a-var",
          methods: UpFamilyMethods {
              stock: Some(CreationPredicates::stock_is_a_var),
              log: Some(CreationPredicates::log_is_a_var),
              ..UpFamilyMethods::default()
          },
      };

      let is_a_const_family = UpFamily {
          name: "is-a-const",
          methods: UpFamilyMethods {
              stock: Some(CreationPredicates::stock_is_a_const),
              log: Some(CreationPredicates::log_is_a_const),
              ..UpFamilyMethods::default()
          },
      };

      let is_a_kind_family = UpFamily {
          name: "is-a-kind",
          methods: UpFamilyMethods {
              log: Some(CreationPredicates::log_is_a_kind),
              ..UpFamilyMethods::default()
          },
      };

      (
          vec![calling_family, is_a_var_family, is_a_const_family, is_a_kind_family],
          Vec::new(),
      )
  }
  ```

- [ ] Implement `CreationPredicates::is_calling_up_atom`:

  ```rust
  /// Check if a proposition atom is a calling atom.
  ///
  /// Corresponds to `CreationPredicates::is_calling_up_atom` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 59-65).
  pub fn is_calling_up_atom(prop: &PcalcProp) -> bool {
      if prop.element == AtomElement::Predicate && prop.arity == 1 {
          if let Some(up) = &prop.predicate {
              return up.family == CALLING_FAMILY;
          }
      }
      false
  }
  ```

- [ ] Implement `CreationPredicates::what_kind_of_calling`:

  ```rust
  /// Return the kind from a calling atom.
  ///
  /// Corresponds to `CreationPredicates::what_kind_of_calling` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 67-73).
  pub fn what_kind_of_calling(prop: &PcalcProp) -> Option<&'static str> {
      if prop.element == AtomElement::Predicate && prop.arity == 1 {
          if let Some(up) = &prop.predicate {
              if up.family == CALLING_FAMILY {
                  return up.assert_kind;
              }
          }
      }
      None
  }
  ```

- [ ] Implement `CreationPredicates::calling_up`:

  ```rust
  /// Create a calling atom with a name and optional kind.
  ///
  /// Corresponds to `CreationPredicates::calling_up` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 75-80).
  ///
  /// Simplified: uses &str for the name instead of wording.
  pub fn calling_up(name: &str, term: PcalcTerm, kind_name: Option<&'static str>) -> PcalcProp {
      let up = UnaryPredicate {
          family: CALLING_FAMILY,
          calling_name: Some(name.to_string()),
          assert_kind: kind_name,
          ..UnaryPredicate::default()
      };
      Atoms::unary_PREDICATE_new(up, term)
  }
  ```

- [ ] Implement `CreationPredicates::get_calling_name`:

  ```rust
  /// Return the calling name from a calling atom.
  ///
  /// Corresponds to `CreationPredicates::get_calling_name` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 82-88).
  pub fn get_calling_name(prop: &PcalcProp) -> Option<&str> {
      if prop.element == AtomElement::Predicate && prop.arity == 1 {
          if let Some(up) = &prop.predicate {
              if up.family == CALLING_FAMILY {
                  return up.calling_name.as_deref();
              }
          }
      }
      None
  }
  ```

- [ ] Implement stock methods:

  ```rust
  /// Stock method for the is-a-var family.
  ///
  /// Corresponds to `CreationPredicates::stock_is_a_var` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 95-99).
  ///
  /// At stock stage 1, creates the singleton is_a_var_up predicate.
  pub fn stock_is_a_var(families: &mut [UpFamily], predicates: &mut Vec<UnaryPredicate>, n: usize) {
      if n == 1 {
          let up = UnaryPredicates::new(IS_A_VAR_FAMILY);
          predicates.push(up);
      }
  }

  /// Stock method for the is-a-const family.
  ///
  /// Corresponds to `CreationPredicates::stock_is_a_const` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 101-105).
  ///
  /// At stock stage 1, creates the singleton is_a_const_up predicate.
  pub fn stock_is_a_const(families: &mut [UpFamily], predicates: &mut Vec<UnaryPredicate>, n: usize) {
      if n == 1 {
          let up = UnaryPredicates::new(IS_A_CONST_FAMILY);
          predicates.push(up);
      }
  }
  ```

- [ ] Implement atom creation helpers:

  ```rust
  /// Create an is-a-var atom.
  ///
  /// Corresponds to `CreationPredicates::is_a_var_up` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 107-109).
  ///
  /// Uses the singleton is_a_var_up predicate (assumes stock has been called).
  /// Simplified: takes the predicate index from the caller.
  pub fn is_a_var_up(term: PcalcTerm, is_a_var_idx: usize, predicates: &[UnaryPredicate]) -> PcalcProp {
      let up = predicates[is_a_var_idx].clone();
      Atoms::unary_PREDICATE_new(up, term)
  }

  /// Create an is-a-const atom.
  ///
  /// Corresponds to `CreationPredicates::is_a_const_up` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 111-113).
  ///
  /// Uses the singleton is_a_const_up predicate (assumes stock has been called).
  /// Simplified: takes the predicate index from the caller.
  pub fn is_a_const_up(term: PcalcTerm, is_a_const_idx: usize, predicates: &[UnaryPredicate]) -> PcalcProp {
      let up = predicates[is_a_const_idx].clone();
      Atoms::unary_PREDICATE_new(up, term)
  }

  /// Create an is-a-kind atom with a specific kind.
  ///
  /// Corresponds to `CreationPredicates::is_a_kind_up` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 115-119).
  pub fn is_a_kind_up(term: PcalcTerm, kind_name: &'static str) -> PcalcProp {
      let up = UnaryPredicate {
          family: IS_A_KIND_FAMILY,
          assert_kind: Some(kind_name),
          ..UnaryPredicate::default()
      };
      Atoms::unary_PREDICATE_new(up, term)
  }
  ```

- [ ] Implement `CreationPredicates::what_kind`:

  ```rust
  /// Return the kind from an is-a-kind atom.
  ///
  /// Corresponds to `CreationPredicates::what_kind` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 121-127).
  pub fn what_kind(prop: &PcalcProp) -> Option<&'static str> {
      if prop.element == AtomElement::Predicate && prop.arity == 1 {
          if let Some(up) = &prop.predicate {
              if up.family == IS_A_KIND_FAMILY {
                  return up.assert_kind;
              }
          }
      }
      None
  }
  ```

- [ ] Implement `CreationPredicates::contains_callings`:

  ```rust
  /// Check if a proposition contains any calling atoms.
  ///
  /// Corresponds to `CreationPredicates::contains_callings` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 195-200).
  ///
  /// Simplified: takes a slice of PcalcProp atoms instead of a linked list.
  pub fn contains_callings(prop: &[PcalcProp]) -> bool {
      prop.iter().any(|p| Self::is_calling_up_atom(p))
  }
  ```

- [ ] Implement log methods:

  ```rust
  /// Log method for the calling family.
  ///
  /// Corresponds to `CreationPredicates::log_calling` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 177-180).
  pub fn log_calling(up: &UnaryPredicate) -> String {
      let mut result = format!("called='{}'", up.calling_name.as_deref().unwrap_or(""));
      if let Some(kind) = up.assert_kind {
          result.push_str(&format!(":{}", kind));
      }
      result
  }

  /// Log method for the is-a-var family.
  ///
  /// Corresponds to `CreationPredicates::log_is_a_var` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 182-184).
  pub fn log_is_a_var(_up: &UnaryPredicate) -> String {
      "is-a-var".to_string()
  }

  /// Log method for the is-a-const family.
  ///
  /// Corresponds to `CreationPredicates::log_is_a_const` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 186-188).
  pub fn log_is_a_const(_up: &UnaryPredicate) -> String {
      "is-a-const".to_string()
  }

  /// Log method for the is-a-kind family.
  ///
  /// Corresponds to `CreationPredicates::log_is_a_kind` in the C reference
  /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 190-193).
  pub fn log_is_a_kind(up: &UnaryPredicate) -> String {
      let mut result = "is-a-kind".to_string();
      if let Some(kind) = up.assert_kind {
          result.push_str(&format!("={}", kind));
      }
      result
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod creation_predicates;` to `crates/conform7-semantics/src/calculus/mod.rs`.

### 3. Unit tests

- [ ] Test `CreationPredicates::start()` creates four families with correct names and methods:
  - `CALLING_FAMILY` (index 0) has `log` method
  - `IS_A_VAR_FAMILY` (index 1) has `stock` and `log` methods
  - `IS_A_CONST_FAMILY` (index 2) has `stock` and `log` methods
  - `IS_A_KIND_FAMILY` (index 3) has `log` method

- [ ] Test `CreationPredicates::is_calling_up_atom`:
  - Returns `true` for a calling atom created via `calling_up`
  - Returns `false` for a non-calling atom (e.g., an is-a-var atom)

- [ ] Test `CreationPredicates::calling_up`:
  - Creates a calling atom with the right name
  - Creates a calling atom with the right kind
  - Creates a calling atom with no kind

- [ ] Test `CreationPredicates::get_calling_name`:
  - Returns the name from a calling atom
  - Returns `None` for a non-calling atom

- [ ] Test `CreationPredicates::what_kind_of_calling`:
  - Returns the kind from a calling atom
  - Returns `None` for a calling atom with no kind
  - Returns `None` for a non-calling atom

- [ ] Test stock methods:
  - `stock_is_a_var` at stage 1 creates a singleton predicate
  - `stock_is_a_var` at other stages does nothing
  - `stock_is_a_const` at stage 1 creates a singleton predicate
  - `stock_is_a_const` at other stages does nothing

- [ ] Test `CreationPredicates::is_a_var_up`:
  - Creates an is-a-var atom with the right family
  - Uses the singleton predicate

- [ ] Test `CreationPredicates::is_a_const_up`:
  - Creates an is-a-const atom with the right family
  - Uses the singleton predicate

- [ ] Test `CreationPredicates::is_a_kind_up`:
  - Creates an is-a-kind atom with the right family
  - Creates an is-a-kind atom with the right kind

- [ ] Test `CreationPredicates::what_kind`:
  - Returns the kind from an is-a-kind atom
  - Returns `None` for a non-is-a-kind atom

- [ ] Test `CreationPredicates::contains_callings`:
  - Returns `true` if any atom in the slice is a calling atom
  - Returns `false` if no atoms are calling atoms
  - Returns `false` for an empty slice

- [ ] Test log methods:
  - `log_calling` produces `"called='name'"` or `"called='name':kind"`
  - `log_is_a_var` produces `"is-a-var"`
  - `log_is_a_const` produces `"is-a-const"`
  - `log_is_a_kind` produces `"is-a-kind"` or `"is-a-kind=kind"`

## Success Criteria

- [ ] `crates/conform7-semantics/src/calculus/creation_predicates.rs` exists with the `CreationPredicates` module
- [ ] `pub mod creation_predicates;` added to `crates/conform7-semantics/src/calculus/mod.rs`
- [ ] `CreationPredicates::start()` creates four families with correct methods
- [ ] `CreationPredicates::is_calling_up_atom` correctly identifies calling atoms
- [ ] `CreationPredicates::calling_up` creates calling atoms with name and kind
- [ ] `CreationPredicates::get_calling_name` retrieves the name from calling atoms
- [ ] `CreationPredicates::what_kind_of_calling` retrieves the kind from calling atoms
- [ ] Stock methods create singleton predicates at stage 1
- [ ] `CreationPredicates::is_a_var_up` and `is_a_const_up` create atoms using singleton predicates
- [ ] `CreationPredicates::is_a_kind_up` creates atoms with the specified kind
- [ ] `CreationPredicates::what_kind` retrieves the kind from is-a-kind atoms
- [ ] `CreationPredicates::contains_callings` correctly detects calling atoms in a proposition
- [ ] All log methods produce correct output
- [ ] All unit tests pass with `cargo test`
- [ ] The crate compiles with `cargo build` without warnings

## Out of Scope

- **Typecheck methods for creation predicates**: The C reference has `#ifdef CORE_MODULE` typecheck methods for the calling, is-a-var, is-a-const, and is-a-kind families. These are deferred because they depend on `TypecheckPropositions` and `Kinds::compatible` which are not yet implemented.
- **Schema methods for creation predicates**: The C reference has `#ifdef CORE_MODULE` schema methods for the calling family. These are deferred because they depend on `Calculus::Schemas`, `LocalVariables::ensure_calling`, and I6 schema compilation.
- **Assert Propositions module**: The `Assert::inner_slated` function that uses these creation predicates is not yet implemented. This plan only implements the creation predicate families and their helper functions.
- **Adjectival Predicates module**: The `AdjectivalPredicates` module (the next item in the assertions module startup) is not yet implemented. It depends on the adjective meaning system and will be addressed in a future plan.
- **Preform grammar**: No Preform grammar parsing is implemented. The creation predicates use simple string names instead of `wording` types.
- **Run-time compilation**: No I6 schema compilation or run-time code generation is implemented.
- **Knowledge module integration**: The creation predicates are purely calculus/assertions infrastructure. They don't depend on or integrate with the knowledge module.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
