# Plan 21: Binary Predicates — The Relation Infrastructure
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the binary predicate system — the core data structures and creation functions for binary predicates (relations) in the calculus module. This includes the `bp_family` struct with method dispatch, the `bp_term_details` struct for term metadata, and the `binary_predicate` struct with creation functions (`make_equality`, `make_pair`, `make_single`), reversal management, and task function storage.

This is the smallest next step after PLAN-20 because:

1. **It's the next piece of calculus infrastructure.** The architecture is: kinds → calculus → knowledge. PLAN-16 implemented terms, atoms, propositions, and unary predicates. PLAN-17-20 implemented the knowledge module foundation, kind subjects, property inferences, and relation inferences. What's missing is the binary predicate system — the data structure that represents relations in the calculus engine. Every relation in Inform (containment, wearing, equality, property-setting, provision) is a `binary_predicate`.

2. **It's a prerequisite for the relation-based startup items.** The `KnowledgeModule::start()` sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls:
   - `PropertyInferences::start()` — PLAN-19 (Complete)
   - `RelationInferences::start()` — PLAN-20 (Complete)
   - `InstanceAdjectives::start()` — depends on adjective meaning system (assertions-module)
   - `EitherOrPropertyAdjectives::start()` — depends on adjective meaning system
   - `MeasurementAdjectives::start()` — depends on adjective meaning system
   - `SameAsRelations::start()` — creates a `bp_family` (depends on binary predicate system)
   - `SettingPropertyRelations::start()` — creates a `bp_family`
   - `ComparativeRelations::start()` — creates a `bp_family`
   - `ProvisionRelation::start()` — creates a `bp_family`

   Items 6-9 all depend on the binary predicate system. Without `bp_family` and `binary_predicate`, we cannot create any of these relation families.

3. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For binary-predicate atoms, they need to look up the `binary_predicate` struct to determine which inference family to use. Without the binary predicate system, the assertion pipeline cannot process relational facts.

4. **It's a prerequisite for `RelationSubjects`.** `RelationSubjects::from_bp(bp)` (Chapter 4/Relation Subjects.w) creates an inference subject for a binary predicate. This is used by `RelationInferences::draw` (PLAN-20) to find the subject to join inferences to. Currently, PLAN-20 uses a simplified string-based lookup; with the binary predicate system, we can use the actual `binary_predicate` struct.

5. **Independently testable.** We can create `bp_family` instances, create `binary_predicate` structs with `make_equality` and `make_pair`, test reversal relationships, test term details, test task function storage, and verify the family method dispatch — all without needing the full relation system or the knowledge module.

6. **It bridges the calculus module to the knowledge module.** The `binary_predicate` struct has a `knowledge_about_bp` field (an inference subject) that connects the calculus representation of a relation to the knowledge module's world model. This is the bridge that the assertion pipeline will use.

## Background

### C reference architecture

#### Binary Predicate Families (`Chapter 3/Binary Predicate Families.w`, lines 1-162)

Binary predicates are grouped into families. Each family provides method dispatch for typechecking, asserting, and compiling:

```c
typedef struct bp_family {
    struct method_set *methods;
    CLASS_DEFINITION
} bp_family;

bp_family *BinaryPredicateFamilies::new(void) {
    bp_family *f = CREATE(bp_family);
    f->methods = Methods::new_set();
    return f;
}
```

Family methods (all optional):
- `STOCK_BPF_MTID` — stock up on relations (stage 1: built-in essentials; stage 2: one per value property)
- `TYPECHECK_BPF_MTID` — typecheck the terms of a relation
- `ASSERT_BPF_MTID` — assert a relation as a true fact about the model world
- `SCHEMA_BPF_MTID` — compile run-time code to test/make-true/make-false a relation
- `DESCRIBE_FOR_PROBLEMS_BPF_MTID` — describe the relation in problem messages
- `DESCRIBE_FOR_INDEX_BPF_MTID` — describe the relation in the Phrasebook index

Stocking happens in two stages:
- `BinaryPredicateFamilies::first_stock()` — called early, creates built-in essentials (equality)
- `BinaryPredicateFamilies::second_stock()` — called later, creates one relation per value property

#### Binary Predicate Term Details (`Chapter 3/Binary Predicate Term Details.w`, lines 1-142)

Each term of a binary predicate has metadata about its domain:

```c
typedef struct bp_term_details {
    struct wording called_name;          /* "(called...)" name, if any exists */
    TERM_DOMAIN_CALCULUS_TYPE *implies_infs; /* the domain of values allowed */
    struct kind *implies_kind;            /* the kind of these values */
    struct i6_schema *function_of_other;  /* where one term can be deduced from the other */
    char *index_term_as;                  /* usually null, used in Phrasebook index */
} bp_term_details;
```

Key operations:
- `BPTerms::new(infs)` — creates term details with a domain inference subject
- `BPTerms::new_kind(K)` — creates term details with a domain kind
- `BPTerms::new_full(infs, K, CW, f)` — creates term details with all fields
- `BPTerms::set_domain(bptd, K)` — fills in the domain later (for BPs created before domains are known)
- `BPTerms::set_function(bptd, f)` — sets the function-of-other schema
- `BPTerms::kind(bptd)` — returns the kind of a term

#### Binary Predicates (`Chapter 3/Binary Predicates.w`, lines 1-371)

The central data structure:

```c
typedef struct binary_predicate {
    struct bp_family *relation_family;
    general_pointer family_specific;       /* details for particular kinds of BP */

    struct word_assemblage relation_name;
    struct parse_node *bp_created_at;
    struct text_stream *debugging_log_name;

    struct bp_term_details term_details[2]; /* 0 is the left term, 1 is the right */

    struct binary_predicate *reversal;     /* the partner BP (B(x,y) iff R(y,x)) */
    int right_way_round;                  /* was this BP created directly? */

    /* how to compile code which tests or forces this BP to be true or false: */
    struct i6_schema *task_functions[4];   /* I6 schema for tasks */

    char *loop_parent_optimisation_proviso;
    char *loop_parent_optimisation_ranger;

    /* somewhere to stash what we know about these relationships: */
    TERM_DOMAIN_CALCULUS_TYPE *knowledge_about_bp; /* inference subject */

    CLASS_DEFINITION
} binary_predicate;
```

Key creation functions:
- `BinaryPredicates::make_equality(family, WA)` — creates the equality relation (its own reversal)
- `BinaryPredicates::make_pair(family, left, right, name, namer, mtf, tf, source_name)` — creates a matched pair of BPs (each is the reversal of the other)
- `BinaryPredicates::make_single(family, left, right, name, mtf, tf, rn)` — creates a single BP (internal, called by the above two)

Key accessors:
- `BinaryPredicates::get_reversal(bp)` — returns the reversal
- `BinaryPredicates::is_the_wrong_way_round(bp)` — tests if this is the wrong way round
- `BinaryPredicates::get_test_function(bp)` — returns the test schema
- `BinaryPredicates::can_be_made_true_at_runtime(bp)` — tests if the BP can be made true at run-time
- `BinaryPredicates::kind(bp)` — returns the kind of the relation
- `BinaryPredicates::term_kind(bp, t)` — returns the kind of a term
- `BinaryPredicates::set_index_details(bp, left, right)` — sets index display names

### Key C source files

- `services/calculus-module/Chapter 3/Binary Predicate Families.w` — `bp_family` struct, method dispatch, stocking (162 lines)
- `services/calculus-module/Chapter 3/Binary Predicate Term Details.w` — `bp_term_details` struct, `BPTerms` functions (142 lines)
- `services/calculus-module/Chapter 3/Binary Predicates.w` — `binary_predicate` struct, creation functions, accessors, reversal management (371 lines)
- `services/calculus-module/Chapter 1/Calculus Module.w` — module setup, startup (references `BINARY_PREDICATE_CREATED_CALCULUS_CALLBACK`)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — startup sequence, calls `SameAsRelations::start()` etc. (line 42-45)
- `inform7/knowledge-module/Chapter 4/Relation Subjects.w` — `RelationSubjects::from_bp(bp)` uses `binary_predicate` (deferred)
- `inform7/knowledge-module/Chapter 5/Relation Inferences.w` — `RelationInferences::draw(bp, ...)` uses `binary_predicate` (PLAN-20)

### Current Rust state

- `crates/conform7-semantics/src/calculus/terms.rs` — `PcalcTerm` struct, `PcalcFunc` struct, creation, accessors, Display.
- `crates/conform7-semantics/src/calculus/atoms.rs` — `AtomElement` enum, `PcalcProp` struct, `PredicateRef` enum, `QuantifierRef` enum, creation, validation, Display.
- `crates/conform7-semantics/src/calculus/propositions.rs` — `Propositions` struct, conjunction, validity, complexity, copy, concatenate.
- `crates/conform7-semantics/src/calculus/unary_predicates.rs` — `UnaryPredicate` struct, creation, copy, Display.
- `crates/conform7-semantics/src/calculus/unary_predicate_families.rs` — `UpFamily` struct, method dispatch, logging, kind inference.
- `crates/conform7-semantics/src/calculus/kind_predicates.rs` — `KindPredicates` module, `kind=K` unary predicates.
- `crates/conform7-semantics/src/calculus/mod.rs` — module declarations for all calculus submodules.
- `crates/conform7-semantics/src/knowledge/` — Complete knowledge module with inference subjects, inferences, property permissions, setup, kind subjects, property inferences, relation inferences.

### What's needed

1. **`BpFamily` struct** — a family of related binary predicates with method dispatch for stocking, typechecking, asserting, schema compilation, and description.
2. **`BpFamilyMethods` struct** — method table with function pointers for all bp_family methods (STOCK, TYPECHECK, ASSERT, SCHEMA, DESCRIBE_FOR_PROBLEMS, DESCRIBE_FOR_INDEX).
3. **`BpTermDetails` struct** — metadata for a binary predicate term: domain (inference subject or kind), called name, function-of-other schema, index term name.
4. **`BPTerms` functions** — `new`, `new_kind`, `new_full`, `set_domain`, `set_function`, `kind`.
5. **`BinaryPredicate` struct** — the central data structure with family, terms, reversal, task functions, knowledge_about_bp, and debugging info.
6. **`BinaryPredicates` creation functions** — `make_equality`, `make_pair`, `make_single`.
7. **Reversal management** — `get_reversal`, `is_the_wrong_way_round`.
8. **Task function management** — `get_test_function`, `can_be_made_true_at_runtime`.
9. **Accessors** — `kind`, `term_kind`, `set_index_details`, `get_log_name`.
10. **`BinaryPredicateFamilies` management** — `new`, `first_stock`, `second_stock`, method dispatch functions (`typecheck`, `assert`, `get_schema`, `describe_for_problems`, `describe_for_index`).
11. **Unit tests** — create families, create binary predicates with make_equality and make_pair, test reversal relationships, test term details, test task function storage, test family method dispatch.

## Tasks

### 1. Create the `BpFamily` struct and `BpFamilyMethods`

- [ ] Create `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` with:

  ```rust
  /// A family of related binary predicates.
  ///
  /// Corresponds to `bp_family` in the C reference
  /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 18-21).
  ///
  /// Each family provides method dispatch for typechecking, asserting, and
  /// compiling binary predicates. Inform currently has a little over 10
  /// different families.
  #[derive(Clone, Debug)]
  pub struct BpFamily {
      /// Name of this family (for debugging).
      pub name: &'static str,
      /// Method implementations for this family.
      pub methods: BpFamilyMethods,
  }

  /// Methods that can be implemented for a binary predicate family.
  ///
  /// Corresponds to the method IDs in the C reference
  /// (`services/calculus-module/Chapter 3/Binary Predicate Families.w`, lines 29-161).
  ///
  /// All methods are optional — the default implementations return
  /// DECLINE_TO_MATCH for typecheck, FALSE for assert, etc.
  #[derive(Clone, Debug)]
  pub struct BpFamilyMethods {
      /// Stock up on relations (stage 1: built-in essentials; stage 2: one per value property).
      /// Corresponds to STOCK_BPF_MTID.
      pub stock: Option<fn(&BpFamily, u8)>,
      /// Typecheck the terms of a relation.
      /// Corresponds to TYPECHECK_BPF_MTID.
      pub typecheck: Option<fn(&BpFamily, &BinaryPredicate, &[Option<usize>], &[Option<usize>]) -> i8>,
      /// Assert a relation as a true fact about the model world.
      /// Corresponds to ASSERT_BPF_MTID.
      pub assert: Option<fn(&BpFamily, &BinaryPredicate, usize, Option<&'static str>, usize, Option<&'static str>) -> bool>,
      /// Compile run-time code for a task (test, make-true, make-false).
      /// Corresponds to SCHEMA_BPF_MTID.
      pub schema: Option<fn(&BpFamily, u8, &BinaryPredicate) -> bool>,
      /// Describe the relation in problem messages.
      /// Corresponds to DESCRIBE_FOR_PROBLEMS_BPF_MTID.
      pub describe_for_problems: Option<fn(&BpFamily, &BinaryPredicate) -> String>,
      /// Describe the relation in the Phrasebook index.
      /// Corresponds to DESCRIBE_FOR_INDEX_BPF_MTID.
      pub describe_for_index: Option<fn(&BpFamily, &BinaryPredicate) -> String>,
  }
  ```

  Note: The method signatures are simplified for now. `typecheck` uses kind indices instead of `kind**` pointers. `assert` uses subject indices and optional value strings instead of `inference_subject*` and `parse_node*` pointers. `schema` returns a boolean instead of modifying an `annotated_i6_schema`. These will be refined when the full typechecking and compilation systems are integrated.

- [ ] Implement `BpFamily::new(name: &'static str) -> Self` — creates a new family with default (no-op) methods.

- [ ] Implement `BinaryPredicateFamilies` module with:
  - `BinaryPredicateFamilies::new_family(name: &'static str) -> BpFamily` — creates a new family.
  - `BinaryPredicateFamilies::first_stock(families: &mut [BpFamily])` — calls stock(1) on all families.
  - `BinaryPredicateFamilies::second_stock(families: &mut [BpFamily])` — calls stock(2) on all families.
  - `BinaryPredicateFamilies::typecheck(bp: &BinaryPredicate, kinds_of_terms: &[Option<usize>], kinds_required: &[Option<usize>], families: &[BpFamily]) -> i8` — dispatches to the family's typecheck method.
  - `BinaryPredicateFamilies::assert(bp: &BinaryPredicate, subj0: usize, spec0: Option<&'static str>, subj1: usize, spec1: Option<&'static str>, families: &[BpFamily]) -> bool` — dispatches to the family's assert method.
  - `BinaryPredicateFamilies::get_schema(task: u8, bp: &BinaryPredicate, families: &[BpFamily]) -> bool` — dispatches to the family's schema method, falling back to the BP's task_functions.
  - `BinaryPredicateFamilies::describe_for_problems(bp: &BinaryPredicate, families: &[BpFamily]) -> String` — dispatches to the family's describe_for_problems method.
  - `BinaryPredicateFamilies::describe_for_index(bp: &BinaryPredicate, families: &[BpFamily]) -> String` — dispatches to the family's describe_for_index method.

### 2. Create the `BpTermDetails` struct and `BPTerms` functions

- [ ] Create `crates/conform7-semantics/src/calculus/bp_term_details.rs` with:

  ```rust
  /// Metadata for a binary predicate term.
  ///
  /// Corresponds to `bp_term_details` in the C reference
  /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 25-31).
  ///
  /// Records the domain of values allowed for this term, the kind of those
  /// values, and optional function-of-other schema.
  #[derive(Clone, Debug)]
  pub struct BpTermDetails {
      /// The domain of values allowed (inference subject index).
      /// Simplified: uses an index into a subject registry instead of
      /// `TERM_DOMAIN_CALCULUS_TYPE*` pointer.
      pub implies_infs: Option<usize>,
      /// The kind of values allowed (kind index).
      /// Simplified: uses a kind index instead of `kind*` pointer.
      pub implies_kind: Option<usize>,
      /// The "(called...)" name, if any exists.
      /// Simplified: a string instead of `wording`.
      pub called_name: Option<&'static str>,
      /// Where one term can be deduced from the other.
      /// Simplified: a string schema instead of `i6_schema*`.
      pub function_of_other: Option<&'static str>,
      /// Text to use in the Phrasebook index (usually null).
      pub index_term_as: Option<&'static str>,
  }
  ```

- [ ] Implement `BPTerms` functions:
  - `BPTerms::new(infs: Option<usize>) -> BpTermDetails` — creates term details with a domain inference subject.
  - `BPTerms::new_kind(kind: Option<usize>) -> BpTermDetails` — creates term details with a domain kind.
  - `BPTerms::new_full(infs: Option<usize>, kind: Option<usize>, called_name: Option<&'static str>, function_of_other: Option<&'static str>) -> BpTermDetails` — creates term details with all fields.
  - `BPTerms::set_domain(bptd: &mut BpTermDetails, kind: Option<usize>, infs: Option<usize>)` — fills in the domain later.
  - `BPTerms::set_function(bptd: &mut BpTermDetails, f: Option<&'static str>)` — sets the function-of-other schema.
  - `BPTerms::kind(bptd: &BpTermDetails) -> Option<usize>` — returns the kind of a term.

### 3. Create the `BinaryPredicate` struct and `BinaryPredicates` creation functions

- [ ] Create `crates/conform7-semantics/src/calculus/binary_predicates.rs` with:

  ```rust
  /// A binary predicate — the underlying data structure for relations.
  ///
  /// Corresponds to `binary_predicate` in the C reference
  /// (`services/calculus-module/Chapter 3/Binary Predicates.w`, lines 60-86).
  ///
  /// A binary predicate B is such that for any combination x and y, and at
  /// any given moment at run-time, B(x, y) is either true or false. x and y
  /// are called its "terms", and are numbered 0 and 1.
  ///
  /// Each BP has a partner called its "reversal". If B is the original and R
  /// is its reversal, then B(x, y) is true if and only if R(y, x) is true.
  #[derive(Clone, Debug)]
  pub struct BinaryPredicate {
      /// The family this BP belongs to (index into a family registry).
      pub relation_family: usize,
      /// Family-specific data (simplified: a string tag for now).
      pub family_specific: Option<&'static str>,
      /// The relation name (simplified: a string instead of `word_assemblage`).
      pub relation_name: Option<&'static str>,
      /// Debugging log name.
      pub debugging_log_name: Option<&'static str>,
      /// Term details for the left (0) and right (1) terms.
      pub term_details: [BpTermDetails; 2],
      /// The reversal BP (index into a BP registry).
      pub reversal: Option<usize>,
      /// Was this BP created directly? (as opposed to being a reversal of another)
      pub right_way_round: bool,
      /// Task functions for compiling code (simplified: string schemas).
      /// Indices: 0=unused, 1=TEST_ATOM_TASK, 2=NOW_ATOM_TRUE_TASK, 3=NOW_ATOM_FALSE_TASK.
      pub task_functions: [Option<&'static str>; 4],
      /// Loop parent optimisation proviso (simplified: a string).
      pub loop_parent_optimisation_proviso: Option<&'static str>,
      /// Loop parent optimisation ranger (simplified: a string).
      pub loop_parent_optimisation_ranger: Option<&'static str>,
      /// Knowledge about this BP (inference subject index).
      /// This is the bridge between the calculus module and the knowledge module.
      pub knowledge_about_bp: Option<usize>,
  }
  ```

- [ ] Implement `BinaryPredicates` creation functions:
  - `BinaryPredicates::make_equality(family_idx: usize, relation_name: &'static str, bp_registry: &mut Vec<BinaryPredicate>) -> usize` — creates the equality relation (its own reversal). Sets `reversal` to its own index and `right_way_round` to true.
  - `BinaryPredicates::make_single(family_idx: usize, left_term: BpTermDetails, right_term: BpTermDetails, name: &'static str, test_fn: Option<&'static str>, make_true_fn: Option<&'static str>, relation_name: Option<&'static str>, bp_registry: &mut Vec<BinaryPredicate>) -> usize` — creates a single BP (internal helper).
  - `BinaryPredicates::make_pair(family_idx: usize, left_term: BpTermDetails, right_term: BpTermDetails, name: &'static str, namer: &'static str, make_true_fn: Option<&'static str>, test_fn: Option<&'static str>, source_name: Option<&'static str>, bp_registry: &mut Vec<BinaryPredicate>) -> usize` — creates a matched pair of BPs (each is the reversal of the other). Returns the index of the right-way-round BP.

- [ ] Implement accessor functions:
  - `BinaryPredicate::get_reversal(&self) -> Option<usize>` — returns the reversal index.
  - `BinaryPredicate::is_the_wrong_way_round(&self) -> bool` — tests if this is the wrong way round.
  - `BinaryPredicate::get_test_function(&self) -> Option<&'static str>` — returns the test schema.
  - `BinaryPredicate::can_be_made_true_at_runtime(&self, bp_registry: &[BinaryPredicate]) -> bool` — tests if the BP or its reversal can be made true at run-time.
  - `BinaryPredicate::kind(&self, kind_registry: &[crate::kinds::Kind]) -> Option<usize>` — returns the kind of the relation (simplified: returns the kind of term 0 if both terms have the same kind).
  - `BinaryPredicate::term_kind(&self, t: usize) -> Option<usize>` — returns the kind of a term.
  - `BinaryPredicate::set_index_details(&mut self, left: Option<&'static str>, right: Option<&'static str>)` — sets index display names for both terms (and the reversal's terms).

### 4. Add module declarations

- [ ] Add to `crates/conform7-semantics/src/calculus/mod.rs`:
  ```rust
  pub mod binary_predicate_families;
  pub mod bp_term_details;
  pub mod binary_predicates;
  ```

### 5. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/calculus/binary_predicate_families.rs`:
  - Test that `BpFamily::new` creates a family with the correct name.
  - Test that a family with no methods uses default (no-op) implementations.
  - Test that `first_stock` calls stock(1) on all families.
  - Test that `second_stock` calls stock(2) on all families.
  - Test that `typecheck` dispatches to the family's typecheck method.
  - Test that `typecheck` returns `DECLINE_TO_MATCH` for a family without a typecheck method.
  - Test that `assert` dispatches to the family's assert method.
  - Test that `assert` returns `false` for a family without an assert method.
  - Test that `describe_for_problems` dispatches to the family's method.
  - Test that `describe_for_index` dispatches to the family's method.

- [ ] Add unit tests in `crates/conform7-semantics/src/calculus/bp_term_details.rs`:
  - Test that `BPTerms::new` creates term details with the correct inference subject.
  - Test that `BPTerms::new_kind` creates term details with the correct kind.
  - Test that `BPTerms::new_full` creates term details with all fields.
  - Test that `BPTerms::set_domain` updates the domain.
  - Test that `BPTerms::set_function` updates the function-of-other schema.
  - Test that `BPTerms::kind` returns the implies_kind if set, or None otherwise.

- [ ] Add unit tests in `crates/conform7-semantics/src/calculus/binary_predicates.rs`:
  - Test that `make_equality` creates a BP with the correct family and name.
  - Test that `make_equality` creates a BP that is its own reversal.
  - Test that `make_equality` creates a BP with `right_way_round = true`.
  - Test that `make_pair` creates two BPs (original and reversal).
  - Test that `make_pair` sets the reversal correctly on both BPs.
  - Test that `make_pair` sets `right_way_round = true` on the original and `false` on the reversal.
  - Test that `make_pair` sets the term details correctly (swapped on reversal).
  - Test that `make_single` creates a BP with the correct fields.
  - Test that `get_reversal` returns the correct reversal index.
  - Test that `is_the_wrong_way_round` returns true for the reversal.
  - Test that `get_test_function` returns the test schema.
  - Test that `can_be_made_true_at_runtime` returns true if the BP has a make-true function.
  - Test that `can_be_made_true_at_runtime` returns true if the reversal has a make-true function.
  - Test that `set_index_details` updates both the BP and its reversal.
  - Test that `term_kind` returns the kind of the specified term.
  - Test that `kind` returns the kind of the relation.

### 6. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `BpFamily::new(name)` creates a family with the correct name and default methods.
- [ ] `BinaryPredicateFamilies::first_stock` calls stock(1) on all families.
- [ ] `BinaryPredicateFamilies::second_stock` calls stock(2) on all families.
- [ ] `BinaryPredicateFamilies::typecheck` dispatches to the family's typecheck method.
- [ ] `BinaryPredicateFamilies::typecheck` returns `DECLINE_TO_MATCH` for a family without a typecheck method.
- [ ] `BinaryPredicateFamilies::assert` dispatches to the family's assert method.
- [ ] `BinaryPredicateFamilies::assert` returns `false` for a family without an assert method.
- [ ] `BPTerms::new(infs)` creates term details with the correct inference subject.
- [ ] `BPTerms::new_kind(kind)` creates term details with the correct kind.
- [ ] `BPTerms::new_full` creates term details with all fields.
- [ ] `BPTerms::set_domain` updates the domain.
- [ ] `BPTerms::set_function` updates the function-of-other schema.
- [ ] `BinaryPredicates::make_equality` creates a BP that is its own reversal.
- [ ] `BinaryPredicates::make_pair` creates a matched pair with correct reversal links.
- [ ] `BinaryPredicates::make_pair` sets `right_way_round` correctly on both BPs.
- [ ] `BinaryPredicates::make_pair` swaps term details on the reversal.
- [ ] `BinaryPredicate::get_reversal` returns the correct reversal index.
- [ ] `BinaryPredicate::is_the_wrong_way_round` returns true for the reversal.
- [ ] `BinaryPredicate::get_test_function` returns the test schema.
- [ ] `BinaryPredicate::can_be_made_true_at_runtime` checks both the BP and its reversal.
- [ ] `BinaryPredicate::set_index_details` updates both the BP and its reversal.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **Full `i6_schema` struct**: The full I6 schema struct with schema modification functions is deferred. This plan uses simplified string schemas.
- **Full `annotated_i6_schema` struct**: The annotated schema struct used in schema compilation is deferred.
- **`word_assemblage` struct**: The full word assemblage struct with comparison functions is deferred. This plan uses simplified string names.
- **`wording` struct**: The full wording struct with word range operations is deferred. This plan uses simplified string names.
- **`parse_node` struct**: The full parse node struct with specification data is deferred. This plan uses simplified string values.
- **`TERM_DOMAIN_CALCULUS_TYPE`**: The full term domain calculus type (inference subject) is deferred. This plan uses simplified subject indices.
- **`BINARY_PREDICATE_CREATED_CALCULUS_CALLBACK`**: The callback for when a BP is created is deferred.
- **Loop optimisation**: The `write_optimised_loop_schema` function with loop parent/ranger optimisation is deferred.
- **`SameAsRelations`**: `SameAsRelations::start()` (Chapter 3/Same Property Relation.w) — the first concrete bp_family — is deferred.
- **`SettingPropertyRelations`**: `SettingPropertyRelations::start()` (Chapter 3/Setting Property Relation.w) is deferred.
- **`ComparativeRelations`**: `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) is deferred.
- **`ProvisionRelation`**: `ProvisionRelation::start()` (Chapter 3/The Provision Relation.w) is deferred.
- **`InstanceAdjectives`**: `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) — depends on the adjective meaning system (assertions-module) — is deferred.
- **`EitherOrPropertyAdjectives`**: `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) is deferred.
- **`MeasurementAdjectives`**: `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) is deferred.
- **`RelationSubjects` family**: The `RelationSubjects` inference subject family (Chapter 4/Relation Subjects.w) is deferred.
- **`ExplicitRelations`**: The explicit relations system (relation forms, storage, run-time) is deferred.
- **Instance Subjects**: `InstanceSubjects` (Chapter 4/Instance Subjects.w) is deferred.
- **Variable Subjects**: `VariableSubjects` (Chapter 4/Variable Subjects.w) is deferred.
- **Instances**: `Instances` (Chapter 2/Instances.w) is deferred.
- **Properties**: `Properties` (Chapter 3/Properties.w) is deferred.
- **Assert propositions**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) is deferred.
- **Run-time compilation**: All `RT*` functions (run-time compilation of relations, subjects, permissions) are deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
