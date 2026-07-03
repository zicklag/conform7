# Plan 40: Kind Predicates Revisited — Typecheck, Assert, and Schema for the Kind Predicate Family

**Status**: Complete
**Target**: 1 day

## Goal

Implement the Kind Predicates Revisited system — adding `typecheck`, `assert`, and `schema` method slots to the unary predicate family method dispatch and wiring simplified implementations into the `kind_up_family` global. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 32), immediately after `EqualityDetails::start()` (PLAN-39, Complete).

This requires:
1. Extending `UpFamilyMethods` with optional `typecheck`, `assert`, and `schema` slots.
2. Creating the `KindPredicatesRevisited` module that defines the three methods and a helper to install them on a family.
3. Wiring those methods into `KIND_UP_FAMILY` and providing a `start()` hook for the assertions-module startup sequence.

## Background

### C reference architecture

The Kind Predicates Revisited system adds methods to the `kind_up_family` created by the calculus module's kind predicates system:

- `inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w` — `KindPredicatesRevisited::start`, `typecheck`, `assert`, `get_schema` (~122 lines)
- `services/calculus-module/Chapter 2/Unary Predicate Families.w` — `up_family`, method dispatch
- `services/calculus-module/Chapter 2/Kind Predicates.w` — creates `kind_up_family`

In the C reference:

```c
void KindPredicatesRevisited::start(void) {
    METHOD_ADD(kind_up_family, TYPECHECK_UPF_MTID, KindPredicatesRevisited::typecheck);
    METHOD_ADD(kind_up_family, ASSERT_UPF_MTID, KindPredicatesRevisited::assert);
    METHOD_ADD(kind_up_family, SCHEMA_UPF_MTID, KindPredicatesRevisited::get_schema);
}
```

The `typecheck` method checks that the term's kind is compatible with the asserted kind:

```c
int KindPredicatesRevisited::typecheck(up_family *self, unary_predicate *up,
        pcalc_prop *prop, variable_type_assignment *vta, tc_problem_kit *tck) {
    kind *need_to_find = up->assert_kind;
    if (Kinds::Behaviour::is_object(need_to_find)) need_to_find = K_object;
    kind *actually_find = TypecheckPropositions::kind_of_term(&(prop->terms[0]), vta, tck);
    if (Kinds::compatible(actually_find, need_to_find) == NEVER_MATCH) {
        // issue problem message
        return NEVER_MATCH;
    }
    return ALWAYS_MATCH;
}
```

The `assert` method sets the kind of an instance or makes a subkind:

```c
void KindPredicatesRevisited::assert(up_family *self, unary_predicate *up,
        int now_negated, pcalc_prop *pl) {
    if (now_negated) {
        // issue problem message
        return;
    }
    inference_subject *subj = Assert::subject_of_term(pl->terms[0]);
    instance *ox = InstanceSubjects::to_object_instance(subj);
    if (ox) Instances::set_kind(ox, up->assert_kind);
    else {
        kind *K = KindSubjects::to_kind(subj);
        if (K) Kinds::make_subkind(K, up->assert_kind);
    }
}
```

The `get_schema` method compiles kind tests to I6 code:

```c
void KindPredicatesRevisited::get_schema(up_family *self, int task, unary_predicate *up,
        annotated_i6_schema *asch, kind *K) {
    switch(task) {
        case TEST_ATOM_TASK:
            // Compile to I6 ofclass or true
        case NOW_ATOM_TRUE_TASK:
        case NOW_ATOM_FALSE_TASK:
            // Issue problem message about fixed kinds
    }
}
```

### Current Rust state

- `crates/conform7-semantics/src/calculus/unary_predicate_families.rs` — `UpFamily` struct, `UpFamilyMethods` struct with the four required methods `log`, `infer_kind`, `testable`, and `test`. No optional `typecheck`, `assert`, or `schema` slots and no `Default` implementation (derive cannot be used because function-pointer fields do not implement `Default`).
- `crates/conform7-semantics/src/calculus/kind_predicates.rs` — defines `KIND_UP_FAMILY` as an immutable `LazyLock<UpFamily>`. Methods are set only inside the `LazyLock::new` closure, so there is no way to add methods after creation.
- `crates/conform7-semantics/src/calculus/unary_predicates.rs` — `UnaryPredicate` struct.
- `crates/conform7-semantics/src/calculus/equality_details.rs` — `EqualityDetails` (PLAN-39, Complete), pattern for assertions-module method wiring.

### What's needed

1. **Extend `UpFamilyMethods`.** Add optional `typecheck`, `assert`, and `schema` slots. Because `#[derive(Default)]` is impossible for function-pointer fields, implement `Default` manually with safe no-op required methods and `None` for the new optional methods. Update every existing `UpFamilyMethods { ... }` struct literal to use `..Default::default()` (or explicitly set the new optional fields to `None`).
2. **Create the `KindPredicatesRevisited` module.** A new module `kind_predicates_revisited` with:
   - `KindPredicatesRevisited::wire(family: &mut UpFamily)` — installs the three optional methods on the supplied family.
   - `KindPredicatesRevisited::start()` — the assertions-module startup hook. Because the global `KIND_UP_FAMILY` is immutable after creation, the wiring is performed inside the `LazyLock::new` closure that builds `KIND_UP_FAMILY`; `start()` forces/verifies that initialization and serves as the C-compatible entry point.
   - `KindPredicatesRevisited::typecheck()` — simplified: returns `1` (`ALWAYS_MATCH`).
   - `KindPredicatesRevisited::assert()` — simplified: returns `false`.
   - `KindPredicatesRevisited::get_schema()` — simplified: returns `false`.
   - Unit tests against both a local family and the global `kind_up_family`.
3. **Wire the methods into `KIND_UP_FAMILY`.** In `kind_predicates.rs`, after constructing the family with its four required methods, call `KindPredicatesRevisited::wire(&mut family)` inside the `LazyLock::new` closure.
4. **Integration** — declare `pub mod kind_predicates_revisited;` in `mod.rs` and update the module map and references.

### Method signatures

The new optional slots should be:

```rust
pub typecheck: Option<
    fn(&UpFamily, &UnaryPredicate, kinds_of_terms: &[Option<usize>], kinds_required: &[Option<usize>]) -> i8,
>,
pub assert: Option<
    fn(&UpFamily, &UnaryPredicate, now_negated: bool, prop: &PcalcProp) -> bool,
>,
pub schema: Option<
    fn(&UpFamily, task: u8, &UnaryPredicate) -> bool,
>,
```

The signatures mirror the binary-predicate family style used by `EqualityDetails` while keeping the unary-proposition information (`PcalcProp`) available for the future full implementation. The concrete simplified methods return the placeholder values above.

## Tasks

### 1. Extend `UpFamilyMethods`

In `crates/conform7-semantics/src/calculus/unary_predicate_families.rs`:

- [ ] Add the three optional method slots listed above.
- [ ] Implement `Default` manually for `UpFamilyMethods`, returning no-op closures for the four required methods and `None` for the three optional ones.
- [ ] Update `UpFamilyMethods` struct literals in:
  - `kind_predicates.rs`
  - `creation_predicates.rs`
  - `unary_predicate_families.rs` tests
  - `unary_predicates.rs` tests
  to include `..Default::default()` or explicit `None`s.

### 2. Create the `KindPredicatesRevisited` module

Create `crates/conform7-semantics/src/calculus/kind_predicates_revisited.rs`:

- [ ] Module-level doc comment referencing `Kind Predicates Revisited.w` and the simplifications.
- [ ] Import `PcalcProp`, `UpFamily`, `UpFamilyMethods`, `UnaryPredicate`, and `KIND_UP_FAMILY` from `kind_predicates` for use by `start()`.
- [ ] `pub struct KindPredicatesRevisited;`
- [ ] `pub fn wire(family: &mut UpFamily)` that sets:
  ```rust
  family.methods.typecheck = Some(KindPredicatesRevisited::typecheck);
  family.methods.assert = Some(KindPredicatesRevisited::assert);
  family.methods.schema = Some(KindPredicatesRevisited::get_schema);
  ```
- [ ] `pub fn start()` that forces the global `KIND_UP_FAMILY` to be initialized (so the assertions-module call has the same side effect as the C version):
  ```rust
  pub fn start() {
      LazyLock::<UpFamily>::force(&KIND_UP_FAMILY);
  }
  ```
- [ ] `pub fn typecheck(...)` returning `1` (`ALWAYS_MATCH`).
- [ ] `pub fn assert(...)` returning `false`.
- [ ] `pub fn get_schema(...)` returning `false`.
- [ ] `#[cfg(test)]` tests:
  - `wire` installs the three optional methods on a local family.
  - `typecheck` returns `1`.
  - `assert` returns `false`.
  - `get_schema` returns `false`.
  - `start` runs without panic and the global `kind_up_family` has the three methods installed.

### 3. Wire the methods into `KIND_UP_FAMILY`

In `crates/conform7-semantics/src/calculus/kind_predicates.rs`:

- [ ] Import `KindPredicatesRevisited::wire` from the new module.
- [ ] In the `LazyLock::new` closure, build the family with the four required methods and `..Default::default()`, then call `wire(&mut family)` before returning it.
  ```rust
  pub static KIND_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
      let mut family = UpFamily::new(
          "kind",
          UpFamilyMethods {
              log: kind_log,
              infer_kind: kind_infer_kind,
              testable: kind_testable,
              test: kind_test,
              ..Default::default()
          },
      );
      KindPredicatesRevisited::wire(&mut family);
      family
  });
  ```
- [ ] Keep `KIND_UP_FAMILY` public and immutable; no `static mut` or safe accessor is needed.

### 4. Integrate with the calculus module

- [ ] Add `pub mod kind_predicates_revisited;` to `crates/conform7-semantics/src/calculus/mod.rs` alphabetically after `kind_predicates`.
- [ ] Add a module-map row for `kind_predicates_revisited`.
- [ ] Add the C reference line `inform7/assertions-module/Chapter 8/Kind Predicates Revisited.w` to the references list.

### 5. Verify

- [ ] Run `cargo build` to ensure compilation.
- [ ] Run `cargo test -- calculus::kind_predicates_revisited` to verify all unit tests pass.
- [ ] Run `cargo clippy --all-targets` to confirm the crate remains clean.

## Success Criteria

- [ ] `UpFamilyMethods` has optional `typecheck`, `assert`, and `schema` slots.
- [ ] Every existing `UpFamilyMethods` struct literal compiles after the change.
- [ ] `KindPredicatesRevisited::wire()` installs `typecheck`, `assert`, and `schema` on a given family.
- [ ] `KindPredicatesRevisited::start()` runs without panic and forces the global `kind_up_family` to be initialized.
- [ ] The global `kind_up_family` carries the three optional methods.
- [ ] `KindPredicatesRevisited::typecheck()` returns `1` (`ALWAYS_MATCH`).
- [ ] `KindPredicatesRevisited::assert()` returns `false`.
- [ ] `KindPredicatesRevisited::get_schema()` returns `false`.
- [ ] The module compiles without errors.
- [ ] All unit tests pass.
- [ ] No new clippy warnings.

## Out of Scope

- **Kind compatibility checking**: `Kinds::compatible` is not implemented. Typecheck always returns `ALWAYS_MATCH`.
- **Instance/kind setting**: `Instances::set_kind` and `Kinds::make_subkind` are not implemented. Assert returns `false`.
- **I6 schema compilation**: `Calculus::Schemas::modify` is not implemented. Schema returns `false`.
- **Problem messages**: `StandardProblems::sentence_problem` is not implemented. No problem messages.
- **TypecheckPropositions integration**: `TypecheckPropositions::kind_of_term` is not implemented.
- **Unary predicate method dispatch helpers**: `UnaryPredicateFamilies::typecheck`, `assert`, and `get_schema` are not added in this plan; only the family method slots and their wiring.
- **Preform grammar / Salsa integration**.
