# Plan 28: The Adjective Meaning System — Core Infrastructure
**Status**: Complete
**Target**: 2-3 days

## Goal

Implement the core adjective meaning infrastructure — the `Adjective` struct, `AdjectiveMeaning` struct, `AdjectiveMeaningFamily` struct, and the basic management functions for creating and working with adjective meanings. This is the foundation for all adjective-related operations in the knowledge module.

This is the smallest next step after PLAN-27 because:

1. **It's the next item in the knowledge module startup that has no remaining dependencies.** The startup sequence (`inform7/knowledge-module/Chapter 1/Knowledge Module.w`, lines 36-45) calls `InstanceAdjectives::start()`, `EitherOrPropertyAdjectives::start()`, and `MeasurementAdjectives::start()` — all three depend on the adjective meaning system (`AdjectiveMeanings`, `AdjectiveMeaningFamily`, `Adjective`, `AdjectiveAmbiguity`, `AdjectiveMeaningDomains`). Without this infrastructure, none of them can be implemented.

2. **It's a prerequisite for `InstanceAdjectives`.** `InstanceAdjectives::start()` (Chapter 2/Instances as Adjectives.w) creates an `enumerative_amf` family and registers instance-based adjective meanings. It calls `AdjectiveMeanings::new_family(2)`, `AdjectiveMeanings::new()`, `Adjectives::declare()`, and `AdjectiveAmbiguity::add_meaning_to_adjective()`. All of these require the core adjective meaning infrastructure.

3. **It's a prerequisite for `EitherOrPropertyAdjectives`.** `EitherOrPropertyAdjectives::start()` (Chapter 3/Either-Or Property Adjectives.w) creates an `either_or_property_amf` family and registers either-or property adjective meanings. It calls `AdjectiveMeanings::new_family(1)`, `AdjectiveMeanings::new()`, `Adjectives::declare()`, and `AdjectiveMeaningDomains::set_from_kind()`. All of these require the core infrastructure.

4. **It's a prerequisite for `MeasurementAdjectives`.** `MeasurementAdjectives::start()` (Chapter 3/Measurement Adjectives.w) creates a `measurement_amf` family and registers measurement-based adjective meanings. It calls `AdjectiveMeanings::new_family(3)`, `AdjectiveMeanings::new()`, `Adjectives::declare()`, and `AdjectiveMeaningDomains::set_from_text()`. All of these require the core infrastructure.

5. **It's a prerequisite for `ComparativeRelations`.** `ComparativeRelations::start()` (Chapter 3/Comparative Relations.w) depends on measurement adjectives, which in turn depend on the adjective meaning system. The entire chain starts here.

6. **It's a prerequisite for the assertion pipeline.** `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) convert propositions into inferences. For adjective atoms, they need the adjective meaning system to resolve which meaning applies and to draw the appropriate inferences. Without the adjective meaning system, the assertion pipeline cannot process adjective facts.

7. **Independently testable.** We can create the `Adjective` struct, create `AdjectiveMeaning` instances, create `AdjectiveMeaningFamily` instances, test the family method dispatch, test the `AdjectiveMeanings::new` and `AdjectiveMeanings::new_family` functions, test `AdjectiveAmbiguity::add_meaning_to_adjective`, test `AdjectiveMeaningDomains::set_from_kind` and `set_from_instance`, test `AdjectiveMeaningDomains::get_kind`, test `AdjectiveMeaningDomains::weak_match`, and test the `Adjectives::declare` function — all without needing the full property system, instances, or run-time compilation.

## Background

### C reference architecture

#### Adjective (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 1-162)

The `Adjective` struct represents a single adjective (e.g., "empty", "open", "red") which may have multiple inflected forms and meanings:

```c
typedef struct adjective {
    struct lexical_cluster *adjective_names;
    struct linguistic_stock_item *in_stock;

    #ifdef ADJECTIVE_COMPILATION_LINGUISTICS_CALLBACK
    struct adjective_compilation_data adjective_compilation;
    #endif
    #ifdef ADJECTIVE_MEANING_LINGUISTICS_CALLBACK
    struct adjective_meaning_data adjective_meanings;
    #endif

    CLASS_DEFINITION
} adjective;
```

Key functions:

```c
adjective *Adjectives::declare(wording W, NATURAL_LANGUAGE_WORDS_TYPE *nl) {
    adjective *adj;
    LOOP_OVER(adj, adjective) {
        wording C = Clusters::get_form_in_language(adj->adjective_names, FALSE, nl);
        if (Wordings::match(C, W)) return adj;
    }
    adj = NULL;
    if (Wordings::nonempty(W)) adj = Adjectives::parse(W);
    if (adj) return adj;
    adj = CREATE(adjective);
    adj->adjective_names = Clusters::new();
    Clusters::add_with_agreements(adj->adjective_names, W, nl);
    #ifdef ADJECTIVE_MEANING_LINGUISTICS_CALLBACK
    ADJECTIVE_MEANING_LINGUISTICS_CALLBACK(adj);
    #endif
    #ifdef ADJECTIVE_COMPILATION_LINGUISTICS_CALLBACK
    ADJECTIVE_COMPILATION_LINGUISTICS_CALLBACK(adj, W);
    #endif
    @<Register the new adjective with the lexicon module@>;
    adj->in_stock = Stock::new(adjectives_category, STORE_POINTER_adjective(adj));
    return adj;
}
```

#### Adjective Meaning (`assertions-module/Chapter 8/Adjective Meanings.w`, lines 1-410)

The `AdjectiveMeaning` struct represents one individual meaning an adjective can have:

```c
typedef struct adjective_meaning {
    struct adjective *owning_adjective;      /* of which this is a meaning */
    struct adjective_domain_data domain;    /* to what can this meaning be applied? */
    struct adjective_meaning_family *family;
    general_pointer family_specific_data;   /* to the relevant structure */
    struct adjective_meaning *negated_from; /* if explicitly constructed as such */
    struct wording indexing_text;           /* text to use in the Phrasebook index */
    struct parse_node *defined_at;         /* from what sentence this came (if it did) */
    int schemas_prepared;                  /* have schemas been prepared yet? */
    struct adjective_task_data task_data[NO_ATOM_TASKS + 1];
    int has_been_compiled_in_support_function;
    CLASS_DEFINITION
} adjective_meaning;
```

Creation:

```c
adjective_meaning *AdjectiveMeanings::new(adjective_meaning_family *family,
    general_pointer details, wording W) {
    adjective_meaning *am = CREATE(adjective_meaning);
    am->defined_at = current_sentence;
    am->indexing_text = W;
    am->owning_adjective = NULL;
    am->domain = AdjectiveMeaningDomains::new_from_text(EMPTY_WORDING);
    am->family = family;
    am->family_specific_data = details;
    am->has_been_compiled_in_support_function = FALSE;
    am->schemas_prepared = FALSE;
    am->negated_from = NULL;
    AdjectiveMeanings::initialise_all_task_data(am);
    return am;
}
```

#### Adjective Meaning Family (`assertions-module/Chapter 8/Adjective Meanings.w`, lines 206-226)

```c
typedef struct adjective_meaning_family {
    struct method_set *methods;
    int definition_claim_priority; /* 0 to 9: lower is better */
    CLASS_DEFINITION
} adjective_meaning_family;

adjective_meaning_family *AdjectiveMeanings::new_family(int N) {
    adjective_meaning_family *f = CREATE(adjective_meaning_family);
    f->definition_claim_priority = N;
    f->methods = Methods::new_set();
    return f;
}
```

Family methods include:

- `CLAIM_DEFINITION_SENTENCE_ADJM_MTID` — opportunity to claim a definition from source text
- `ASSERT_ADJM_MTID` — assert the adjective meaning on an inference subject
- `PREPARE_SCHEMAS_ADJM_MTID` — prepare I6 schemas for the meaning

#### Adjective Ambiguity (`assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 1-270)

Manages multiple contextual meanings per adjective:

```c
typedef struct adjective_meaning_data {
    struct linked_list *in_defn_order;       /* of |adjective_meaning| */
    struct linked_list *in_precedence_order; /* of |adjective_meaning| */
} adjective_meaning_data;

void AdjectiveAmbiguity::new_set(adjective *adj) {
    adj->adjective_meanings.in_defn_order = NEW_LINKED_LIST(adjective_meaning);
    adj->adjective_meanings.in_precedence_order = NEW_LINKED_LIST(adjective_meaning);
}

adjective *AdjectiveAmbiguity::add_meaning_to_adjective(adjective_meaning *am,
    adjective *adj) {
    ADD_TO_LINKED_LIST(am, adjective_meaning, adj->adjective_meanings.in_defn_order);
    am->owning_adjective = adj;
    return adj;
}
```

#### Adjective Meaning Domains (`assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 1-315)

What a single sense of an adjective can apply to:

```c
typedef struct adjective_domain_data {
    struct wording domain_text;              /* text given by author about the domain */
    struct inference_subject *domain_infs;   /* what domain the defn applies to */
    struct kind *domain_kind;               /* what kind of values */
    int currently_determining;
    int problems_thrown;
} adjective_domain_data;

void AdjectiveMeaningDomains::set_from_kind(adjective_meaning *am, kind *K) {
    am->domain = AdjectiveMeaningDomains::new_from_kind(K);
}

void AdjectiveMeaningDomains::set_from_instance(adjective_meaning *am, instance *I) {
    am->domain = AdjectiveMeaningDomains::new_from_instance(I);
}

kind *AdjectiveMeaningDomains::get_kind(adjective_meaning *am) {
    if (am == NULL) return NULL;
    if (am->domain.domain_infs == NULL) return NULL;
    return am->domain.domain_kind;
}

int AdjectiveMeaningDomains::weak_match(kind *K1, adjective_meaning *am) {
    kind *K2 = AdjectiveMeaningDomains::get_kind(am);
    if (RTKindIDs::weak_iname(K1) == RTKindIDs::weak_iname(K2)) return TRUE;
    return FALSE;
}
```

### Key C source files

- `services/linguistics-module/Chapter 2/Adjectives.w` — the `adjective` struct, `Adjectives::declare`, `Adjectives::parse` (162 lines)
- `inform7/assertions-module/Chapter 8/Adjective Meanings.w` — `adjective_meaning` struct, `adjective_meaning_family` struct, `AdjectiveMeanings` management functions (410 lines)
- `inform7/assertions-module/Chapter 8/Adjective Ambiguity.w` — `adjective_meaning_data` struct, `AdjectiveAmbiguity` management functions (270 lines)
- `inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w` — `adjective_domain_data` struct, domain management functions (315 lines)
- `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup, calls `InstanceAdjectives::start()` (line 39), `EitherOrPropertyAdjectives::start()` (line 40), `MeasurementAdjectives::start()` (line 41)
- `inform7/knowledge-module/Chapter 2/Instances as Adjectives.w` — `InstanceAdjectives` (97 lines, depends on adjective meaning system)
- `inform7/knowledge-module/Chapter 3/Either-Or Property Adjectives.w` — `EitherOrPropertyAdjectives` (74 lines, depends on adjective meaning system)
- `inform7/knowledge-module/Chapter 3/Measurement Adjectives.w` — `MeasurementAdjectives` (197 lines, depends on adjective meaning system)
- `inform7/knowledge-module/Chapter 4/Inference Subjects.w` — `InferenceSubject` struct (PLAN-17)
- `inform7/knowledge-module/Chapter 4/Kind Subjects.w` — `KindSubjects::from_kind`, `KindSubjects::to_kind` (used by domain management)
- `inform7/knowledge-module/Chapter 5/Property Inferences.w` — `PropertyInferences::draw` (used by adjective assert methods)

### Current Rust state

- `crates/conform7-semantics/src/knowledge/setting_property_relation.rs` — `SettingPropertyRelations` module, `SettingPropertyRelations::start()`, `SettingPropertyRelations::stock()`, `SettingPropertyRelations::typecheck()`, `SettingPropertyRelations::assert()`, `SettingPropertyRelations::schema()`, `PropertySettingBpData` struct, unit tests (PLAN-27, Complete).
- `crates/conform7-semantics/src/knowledge/same_property_relation.rs` — `SameAsRelations` module, `SameAsRelations::start()`, `SameAsRelations::stock()`, `SameAsRelations::typecheck()`, unit tests (PLAN-26, Complete).
- `crates/conform7-semantics/src/knowledge/properties.rs` — `Property` struct, `EitherOrPropertyData`, `ValuePropertyData`, `Properties::create`, `Properties::obtain`, `Properties::to_kind`, `Properties::kind_of_contents`, `EitherOrProperties`, `ValueProperties`, unit tests (PLAN-25, Complete).
- `crates/conform7-semantics/src/knowledge/provision_relation.rs` — `ProvisionRelation` module, `ProvisionRelation::start()`, `ProvisionRelation::stock()`, `ProvisionRelation::typecheck()`, `ProvisionRelation::assert()`, unit tests (PLAN-23, Complete).
- `crates/conform7-semantics/src/knowledge/relation_subjects.rs` — `RelationSubjects` module, `RelationSubjects::family()`, `RelationSubjects::from_bp()`, `RelationSubjects::new()`, `RelationSubjects::to_bp()`, unit tests (PLAN-24, Complete).
- `crates/conform7-semantics/src/knowledge/property_inferences.rs` — `PropertyInferences` module, `PropertyInferenceData` struct, `PropertyInferences::start()`, unit tests (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/relation_inferences.rs` — `RelationInferences` module, `RelationInferenceData` struct, `RelationInferences::start()`, unit tests (PLAN-20, Complete).
- `crates/conform7-semantics/src/knowledge/inference_subjects.rs` — `InferenceSubject` struct, `InferenceSubjectFamily` struct, `InferenceSubjectFamilyMethods` struct, `InferenceSubjects` management functions, unit tests (PLAN-17, Complete).
- `crates/conform7-semantics/src/knowledge/inferences.rs` — `Inference` struct, `InferenceFamily` struct, `InferenceFamilyMethods` struct, `Certainty` enum, unit tests (PLAN-18, Complete).
- `crates/conform7-semantics/src/knowledge/property_permissions.rs` — `PropertyPermission` struct with `find` and `grant` methods (PLAN-19, Complete).
- `crates/conform7-semantics/src/knowledge/kind_subjects.rs` — `KindSubjects` module, `KindSubjects::family()`, `KindSubjects::from_kind()`, `KindSubjects::to_kind()`, unit tests (Complete).
- `crates/conform7-semantics/src/knowledge/setup.rs` — `setup_knowledge_module()` creates model_world, global_constants, global_variables.
- `crates/conform7-semantics/src/knowledge/mod.rs` — module declarations for all knowledge submodules.
- `crates/conform7-semantics/src/calculus/binary_predicates.rs` — `BinaryPredicate` struct with `knowledge_about_bp` field, `BinaryPredicates` creation functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/binary_predicate_families.rs` — `BpFamily` struct, `BpFamilyMethods` struct, `BinaryPredicateFamilies` management functions, `DECLINE_TO_MATCH` constant (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/bp_term_details.rs` — `BpTermDetails` struct, `BPTerms` functions (PLAN-21, Complete).
- `crates/conform7-semantics/src/calculus/equality_relation.rs` — `EqualityRelation` module with three families, unit tests (PLAN-22, Complete).
- `crates/conform7-semantics/src/kinds.rs` — `Kind` struct, `Kind::unary_con()`, `Kind::binary_con()`, `Compatibility` enum, `FromStr` parser, unit tests (Complete).
- `crates/conform7-semantics/src/kind_constructors.rs` — `KindConstructor` struct, `ConstructorGroup` enum, `Variance` enum, unit tests (Complete).

### What's needed

1. **`Adjective` struct** — the core data structure for a single adjective with:
   - `name` — the adjective name (simplified: a string instead of `lexical_cluster`)
   - `meanings` — a list of `AdjectiveMeaning` indices (simplified: `Vec<usize>` instead of linked lists)
   - `compilation_data` — simplified: a string tag (full `adjective_compilation_data` deferred)
   - `Adjectives::declare(name)` — find or create an adjective by name
   - `Adjectives::find(name)` — find an existing adjective by name
   - `Adjectives::get_nominative_singular(adj)` — return the adjective's name

2. **`AdjectiveMeaning` struct** — one meaning an adjective can have with:
   - `owning_adjective` — optional adjective index
   - `domain` — `AdjectiveDomainData` (what this meaning applies to)
   - `family` — `AdjectiveMeaningFamily` index
   - `family_specific_data` — optional string for family-specific data
   - `negated_from` — optional meaning index (if this is a negation)
   - `indexing_text` — optional string for the Phrasebook index
   - `schemas_prepared` — whether schemas have been prepared
   - `task_data` — simplified: task mode flags (full `adjective_task_data` with I6 schemas deferred)

3. **`AdjectiveMeaningFamily` struct** — a family of related adjective meanings with:
   - `definition_claim_priority` — priority for claiming definitions (0-9, lower is better)
   - `methods` — `AdjectiveMeaningFamilyMethods` struct with optional method pointers:
     - `assert` — assert the meaning on an inference subject
     - `claim_definition` — claim a definition from source text
     - `prepare_schemas` — prepare I6 schemas
     - `index` — produce index text
   - `AdjectiveMeanings::new_family(priority)` — create a new family

4. **`AdjectiveMeanings` management functions**:
   - `AdjectiveMeanings::new(family_idx, details, name)` — create a new meaning
   - `AdjectiveMeanings::negate(other_idx)` — create a negated meaning
   - `AdjectiveMeanings::assert(am_idx, subj_idx, parity)` — assert a meaning on a subject
   - `AdjectiveMeanings::claim_definition(...)` — claim a definition (simplified: no-op)

5. **`AdjectiveAmbiguity` management functions**:
   - `AdjectiveAmbiguity::new_set(adj_idx)` — initialise the meaning lists for an adjective
   - `AdjectiveAmbiguity::add_meaning_to_adjective(am_idx, adj_idx)` — add a meaning to an adjective
   - `AdjectiveAmbiguity::can_be_applied_to(adj_idx, kind_idx)` — check if an adjective can apply to a kind
   - `AdjectiveAmbiguity::first_meaning(adj_idx)` — get the first meaning of an adjective
   - `AdjectiveAmbiguity::sort(adj_idx)` — sort meanings into precedence order (simplified: no-op)

6. **`AdjectiveMeaningDomains` management functions**:
   - `AdjectiveDomainData` struct — stores domain text, inference subject, and kind
   - `AdjectiveMeaningDomains::new_from_text(wording)` — create domain from text
   - `AdjectiveMeaningDomains::new_from_kind(kind_idx)` — create domain from a kind
   - `AdjectiveMeaningDomains::new_from_instance(instance_idx)` — create domain from an instance
   - `AdjectiveMeaningDomains::set_from_kind(am_idx, kind_idx)` — set domain from a kind
   - `AdjectiveMeaningDomains::set_from_instance(am_idx, instance_idx)` — set domain from an instance
   - `AdjectiveMeaningDomains::set_from_text(am_idx, wording)` — set domain from text
   - `AdjectiveMeaningDomains::get_kind(am_idx)` — get the kind of a meaning's domain
   - `AdjectiveMeaningDomains::get_subject(am_idx)` — get the inference subject of a meaning's domain
   - `AdjectiveMeaningDomains::weak_match(kind_idx, am_idx)` — weak domain matching
   - `AdjectiveMeaningDomains::determine(am_idx)` — determine the domain (simplified: no-op for text-based domains)

7. **Integration with the knowledge module** — add the `adjectives` module declaration to the knowledge module's `mod.rs`.

8. **Unit tests** — create adjectives, create meanings, create families, test the declare/find round-trip, test adding meanings to adjectives, test domain management, test weak matching, test the assert method dispatch.

## Tasks

### 1. Create the `Adjective` struct and `AdjectiveMeaning` structs

- [ ] Create `crates/conform7-semantics/src/knowledge/adjectives.rs` with:

  ```rust
  /// An adjective — a word that can be applied to subjects to describe them.
  ///
  /// Corresponds to `adjective` in the C reference
  /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 18-30).
  ///
  /// Adjectives can have multiple meanings. For example, "empty" can mean
  /// "a container with nothing in it" or "a rulebook with no rules".
  /// Each meaning is represented by an `AdjectiveMeaning` struct.
  ///
  /// Simplified: uses string names instead of `lexical_cluster`, and a
  /// `Vec<usize>` for meanings instead of linked lists.
  #[derive(Clone, Debug)]
  pub struct Adjective {
      /// Name of the adjective (simplified: a string instead of `lexical_cluster`).
      pub name: &'static str,
      /// Meanings of this adjective, in definition order.
      /// Corresponds to `in_defn_order` in the C reference.
      pub meanings: Vec<usize>,
      /// Meanings sorted into precedence order.
      /// Corresponds to `in_precedence_order` in the C reference.
      pub sorted_meanings: Vec<usize>,
      /// Compilation data (simplified: a string tag).
      /// Full `adjective_compilation_data` is deferred.
      pub compilation_data: Option<&'static str>,
  }
  ```

- [ ] Define the `AdjectiveMeaning` struct:

  ```rust
  /// One individual meaning which an adjective can have.
  ///
  /// Corresponds to `adjective_meaning` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 10-28).
  ///
  /// For example, "odd" in the sense of numbers is a single meaning.
  /// Each meaning belongs to a family and has a domain (what it can apply to).
  #[derive(Clone, Debug)]
  pub struct AdjectiveMeaning {
      /// The adjective this meaning belongs to (index into the adjective registry).
      pub owning_adjective: Option<usize>,
      /// The domain of this meaning — what kinds/instances it can apply to.
      pub domain: AdjectiveDomainData,
      /// The family this meaning belongs to (index into the family registry).
      pub family: usize,
      /// Family-specific data (simplified: a string instead of `general_pointer`).
      pub family_specific_data: Option<&'static str>,
      /// If this meaning is a negation of another, the index of the original.
      pub negated_from: Option<usize>,
      /// Text to use in the Phrasebook index (simplified: a string).
      pub indexing_text: Option<&'static str>,
      /// Have schemas been prepared yet?
      pub schemas_prepared: bool,
      /// Task mode flags (simplified: no I6 schemas).
      /// Full `adjective_task_data` with I6 schemas is deferred.
      pub task_modes: [i8; 4], // 0=TEST, 1=NOW_TRUE, 2=NOW_FALSE, 3=unused
  }
  ```

- [ ] Define the `AdjectiveDomainData` struct:

  ```rust
  /// The domain of an adjective meaning — what it can validly apply to.
  ///
  /// Corresponds to `adjective_domain_data` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 28-34).
  ///
  /// For example, the meaning of "odd" for numbers has the set of all numbers
  /// as its domain, whereas a meaning from "Mrs Elspeth Spong can be odd"
  /// has only a single instance as domain.
  #[derive(Clone, Debug)]
  pub struct AdjectiveDomainData {
      /// Text given by author about the domain (simplified: a string).
      pub domain_text: Option<&'static str>,
      /// What domain the definition applies to (inference subject index).
      pub domain_infs: Option<usize>,
      /// What kind of values (kind index).
      pub domain_kind: Option<usize>,
      /// Are we currently working this out? (for circularity detection)
      pub currently_determining: bool,
  }
  ```

- [ ] Define the `AdjectiveMeaningFamily` struct:

  ```rust
  /// A family of related adjective meanings.
  ///
  /// Corresponds to `adjective_meaning_family` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 215-219).
  ///
  /// Each family represents a distinct purpose for creating adjectives:
  /// - Enumerative (instance-based) adjectives
  /// - Either-or property adjectives
  /// - Measurement adjectives
  /// - Condition-based adjectives
  /// - etc.
  #[derive(Clone, Debug)]
  pub struct AdjectiveMeaningFamily {
      /// Name of the family (for debugging).
      pub name: &'static str,
      /// Priority for claiming definitions (0 to 9: lower is better).
      pub definition_claim_priority: u8,
      /// Methods for this family.
      pub methods: AdjectiveMeaningFamilyMethods,
  }

  /// Methods that an adjective meaning family can provide.
  ///
  /// All methods are optional. Corresponds to the method set in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 228-301).
  #[derive(Clone, Debug)]
  pub struct AdjectiveMeaningFamilyMethods {
      /// Assert the meaning on an inference subject.
      /// Returns true if the assertion was handled.
      pub assert: Option<fn(usize, usize, bool, &mut [AdjectiveMeaning],
          &mut [crate::knowledge::inference_subjects::InferenceSubject]) -> bool>,
      /// Claim a definition from source text (simplified: no-op).
      pub claim_definition: Option<fn() -> bool>,
      /// Prepare I6 schemas (simplified: no-op).
      pub prepare_schemas: Option<fn(usize, i32)>,
      /// Produce index text (simplified: no-op).
      pub index: Option<fn(usize) -> Option<&'static str>>,
  }
  ```

- [ ] Define global constants and the `AdjectiveMeanings` module struct:

  ```rust
  /// The adjective meanings module.
  ///
  /// Corresponds to `AdjectiveMeanings` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`).
  pub struct AdjectiveMeanings;
  ```

- [ ] Implement `Adjectives::declare()`:

  ```rust
  /// Find or create an adjective by name.
  ///
  /// Corresponds to `Adjectives::declare` in the C reference
  /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 57-78).
  ///
  /// Simplified:
  /// - No lexical cluster (uses string comparison)
  /// - No linguistic stock
  /// - No lexicon registration
  /// - No compilation data initialisation
  ///
  /// Returns the index of the adjective in the registry.
  pub fn declare(
      name: &'static str,
      registry: &mut Vec<Adjective>,
  ) -> usize {
      // Check if the adjective already exists.
      for (i, adj) in registry.iter().enumerate() {
          if adj.name == name {
              return i;
          }
      }

      // Create a new adjective.
      let adj = Adjective {
          name,
          meanings: Vec::new(),
          sorted_meanings: Vec::new(),
          compilation_data: None,
      };
      let idx = registry.len();
      registry.push(adj);
      idx
  }
  ```

- [ ] Implement `Adjectives::find()`:

  ```rust
  /// Find an existing adjective by name.
  ///
  /// Corresponds to `Adjectives::parse` in the C reference
  /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 122-125).
  ///
  /// Returns the adjective index, or None if not found.
  pub fn find(name: &str, registry: &[Adjective]) -> Option<usize> {
      registry.iter().position(|adj| adj.name == name)
  }
  ```

- [ ] Implement `Adjectives::get_nominative_singular()`:

  ```rust
  /// Get the nominative singular form of an adjective.
  ///
  /// Corresponds to `Adjectives::get_nominative_singular` in the C reference
  /// (`services/linguistics-module/Chapter 2/Adjectives.w`, lines 99-101).
  ///
  /// Simplified: returns the adjective's name directly (no inflection).
  pub fn get_nominative_singular(adj: &Adjective) -> &str {
      adj.name
  }
  ```

- [ ] Implement `AdjectiveMeanings::new_family()`:

  ```rust
  /// Create a new adjective meaning family.
  ///
  /// Corresponds to `AdjectiveMeanings::new_family` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 221-226).
  ///
  /// The priority determines the order in which families are offered the
  /// chance to claim definitions (0 to 9: lower is better).
  pub fn new_family(
      name: &'static str,
      priority: u8,
      methods: AdjectiveMeaningFamilyMethods,
      families: &mut Vec<AdjectiveMeaningFamily>,
  ) -> usize {
      let family = AdjectiveMeaningFamily {
          name,
          definition_claim_priority: priority,
          methods,
      };
      let idx = families.len();
      families.push(family);
      idx
  }
  ```

- [ ] Implement `AdjectiveMeanings::new()`:

  ```rust
  /// Create a new adjective meaning.
  ///
  /// Corresponds to `AdjectiveMeanings::new` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 33-47).
  ///
  /// Simplified:
  /// - No current_sentence tracking
  /// - No task data initialisation (task_modes default to 0 = NO_TASKMODE)
  ///
  /// Returns the index of the new meaning in the registry.
  pub fn new(
      family_idx: usize,
      details: Option<&'static str>,
      name: Option<&'static str>,
      registry: &mut Vec<AdjectiveMeaning>,
  ) -> usize {
      let am = AdjectiveMeaning {
          owning_adjective: None,
          domain: AdjectiveDomainData {
              domain_text: None,
              domain_infs: None,
              domain_kind: None,
              currently_determining: false,
          },
          family: family_idx,
          family_specific_data: details,
          negated_from: None,
          indexing_text: name,
          schemas_prepared: false,
          task_modes: [0, 0, 0, 0], // NO_TASKMODE for all tasks
      };
      let idx = registry.len();
      registry.push(am);
      idx
  }
  ```

- [ ] Implement `AdjectiveMeanings::negate()`:

  ```rust
  /// Create a negated copy of an existing adjective meaning.
  ///
  /// Corresponds to `AdjectiveMeanings::negate` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 53-74).
  ///
  /// Simplified:
  /// - No task data negation (task_modes are copied directly)
  /// - No schema modification
  pub fn negate(
      other_idx: usize,
      registry: &mut Vec<AdjectiveMeaning>,
  ) -> usize {
      let other = &registry[other_idx];
      let am = AdjectiveMeaning {
          owning_adjective: None,
          domain: other.domain.clone(),
          family: other.family,
          family_specific_data: other.family_specific_data,
          negated_from: Some(other_idx),
          indexing_text: other.indexing_text,
          schemas_prepared: false,
          task_modes: other.task_modes, // simplified: copy directly
      };
      let idx = registry.len();
      registry.push(am);
      idx
  }
  ```

- [ ] Implement `AdjectiveMeanings::assert()`:

  ```rust
  /// Assert an adjective meaning on an inference subject.
  ///
  /// Corresponds to `AdjectiveMeanings::assert` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meanings.w`, lines 293-301).
  ///
  /// Dispatches to the family's assert method if available.
  /// If the meaning is a negation, it follows the negation chain and
  /// flips the parity.
  ///
  /// Returns true if the assertion was handled, false otherwise.
  pub fn assert(
      am_idx: usize,
      subj_idx: usize,
      parity: bool,
      meanings: &mut [AdjectiveMeaning],
      subjects: &mut [crate::knowledge::inference_subjects::InferenceSubject],
      families: &[AdjectiveMeaningFamily],
  ) -> bool {
      let am = &meanings[am_idx];

      // Follow negation chain.
      let (actual_am_idx, actual_parity) = if let Some(negated_from) = am.negated_from {
          (negated_from, !parity)
      } else {
          (am_idx, parity)
      };

      let actual_am = &meanings[actual_am_idx];
      let family = &families[actual_am.family];

      if let Some(assert_fn) = family.methods.assert {
          assert_fn(actual_am_idx, subj_idx, actual_parity, meanings, subjects)
      } else {
          false // no assert method — decline
      }
  }
  ```

- [ ] Implement `AdjectiveAmbiguity::add_meaning_to_adjective()`:

  ```rust
  /// Add a meaning to an adjective.
  ///
  /// Corresponds to `AdjectiveAmbiguity::add_meaning_to_adjective` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 45-50).
  ///
  /// Adds the meaning to the adjective's definition-order list and sets
  /// the meaning's owning_adjective field.
  pub fn add_meaning_to_adjective(
      am_idx: usize,
      adj_idx: usize,
      adjectives: &mut [Adjective],
      meanings: &mut [AdjectiveMeaning],
  ) {
      adjectives[adj_idx].meanings.push(am_idx);
      meanings[am_idx].owning_adjective = Some(adj_idx);
  }
  ```

- [ ] Implement `AdjectiveAmbiguity::can_be_applied_to()`:

  ```rust
  /// Check if an adjective can be applied to a given kind.
  ///
  /// Corresponds to `AdjectiveAmbiguity::can_be_applied_to` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 95-113).
  ///
  /// Simplified: checks if any meaning of the adjective has a domain kind
  /// that is compatible with the given kind. Uses kind compatibility
  /// instead of the C reference's object/value distinction.
  ///
  /// Returns true if the adjective can be applied, false otherwise.
  pub fn can_be_applied_to(
      adj_idx: usize,
      kind_idx: Option<usize>,
      adjectives: &[Adjective],
      meanings: &[AdjectiveMeaning],
  ) -> bool {
      let adj = &adjectives[adj_idx];
      for &am_idx in &adj.meanings {
          let am = &meanings[am_idx];
          if let Some(am_kind) = am.domain.domain_kind {
              if let Some(target_kind) = kind_idx {
                  if am_kind == target_kind {
                      return true;
                  }
              } else {
                  return true; // null kind matches anything (simplified)
              }
          } else {
              return true; // undetermined domain matches anything (simplified)
          }
      }
      false
  }
  ```

- [ ] Implement `AdjectiveAmbiguity::first_meaning()`:

  ```rust
  /// Get the first meaning of an adjective.
  ///
  /// Corresponds to `AdjectiveAmbiguity::first_meaning` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Ambiguity.w`, lines 144-148).
  ///
  /// Returns the index of the first meaning, or None if the adjective has no meanings.
  pub fn first_meaning(adj_idx: usize, adjectives: &[Adjective]) -> Option<usize> {
      adjectives.get(adj_idx).and_then(|adj| adj.meanings.first().copied())
  }
  ```

- [ ] Implement `AdjectiveMeaningDomains::new_from_kind()`:

  ```rust
  /// Create domain data from a kind.
  ///
  /// Corresponds to `AdjectiveMeaningDomains::new_from_kind` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 70-76).
  ///
  /// Simplified: no KindSubjects::from_kind (uses kind index directly).
  pub fn new_from_kind(kind_idx: usize) -> AdjectiveDomainData {
      AdjectiveDomainData {
          domain_text: None,
          domain_infs: None, // simplified: no inference subject lookup
          domain_kind: Some(kind_idx),
          currently_determining: false,
      }
  }
  ```

- [ ] Implement `AdjectiveMeaningDomains::set_from_kind()`:

  ```rust
  /// Set the domain of an adjective meaning from a kind.
  ///
  /// Corresponds to `AdjectiveMeaningDomains::set_from_kind` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 67-69).
  pub fn set_from_kind(am_idx: usize, kind_idx: usize, meanings: &mut [AdjectiveMeaning]) {
      meanings[am_idx].domain = AdjectiveMeaningDomains::new_from_kind(kind_idx);
  }
  ```

- [ ] Implement `AdjectiveMeaningDomains::get_kind()`:

  ```rust
  /// Get the kind of a meaning's domain.
  ///
  /// Corresponds to `AdjectiveMeaningDomains::get_kind` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 238-242).
  ///
  /// Returns the kind index, or None if the domain is undetermined.
  pub fn get_kind(am_idx: usize, meanings: &[AdjectiveMeaning]) -> Option<usize> {
      meanings.get(am_idx).and_then(|am| am.domain.domain_kind)
  }
  ```

- [ ] Implement `AdjectiveMeaningDomains::weak_match()`:

  ```rust
  /// Weak domain matching — check if a kind is close enough for run-time checking.
  ///
  /// Corresponds to `AdjectiveMeaningDomains::weak_match` in the C reference
  /// (`inform7/assertions-module/Chapter 8/Adjective Meaning Domains.w`, lines 265-269).
  ///
  /// Simplified: exact kind comparison (no weak_iname weakening).
  /// In the full implementation, this would use RTKindIDs::weak_iname to
  /// treat all object kinds as just "object".
  pub fn weak_match(kind_idx: usize, am_idx: usize, meanings: &[AdjectiveMeaning]) -> bool {
      if let Some(am_kind) = AdjectiveMeaningDomains::get_kind(am_idx, meanings) {
          kind_idx == am_kind
      } else {
          false // undetermined domain never matches
      }
  }
  ```

### 2. Add module declaration

- [ ] Add `pub mod adjectives;` to `crates/conform7-semantics/src/knowledge/mod.rs`.

### 3. Add unit tests

- [ ] Add unit tests in `crates/conform7-semantics/src/knowledge/adjectives.rs`:

  - Test that `Adjectives::declare` creates a new adjective.
  - Test that `Adjectives::declare` returns the existing adjective if already declared.
  - Test that `Adjectives::find` finds an existing adjective.
  - Test that `Adjectives::find` returns None for a non-existent adjective.
  - Test that `Adjectives::get_nominative_singular` returns the adjective's name.
  - Test that `AdjectiveMeanings::new_family` creates a family with the correct priority.
  - Test that `AdjectiveMeanings::new_family` creates a family with the correct methods.
  - Test that `AdjectiveMeanings::new` creates a meaning with the correct family.
  - Test that `AdjectiveMeanings::new` creates a meaning with the correct family-specific data.
  - Test that `AdjectiveMeanings::new` creates a meaning with the correct indexing text.
  - Test that `AdjectiveMeanings::negate` creates a negated meaning.
  - Test that `AdjectiveMeanings::negate` sets the negated_from field correctly.
  - Test that `AdjectiveMeanings::negate` copies the domain from the original.
  - Test that `AdjectiveMeanings::assert` dispatches to the family's assert method.
  - Test that `AdjectiveMeanings::assert` returns false when the family has no assert method.
  - Test that `AdjectiveMeanings::assert` follows the negation chain and flips parity.
  - Test that `AdjectiveAmbiguity::add_meaning_to_adjective` adds the meaning to the adjective.
  - Test that `AdjectiveAmbiguity::add_meaning_to_adjective` sets the owning_adjective field.
  - Test that `AdjectiveAmbiguity::can_be_applied_to` returns true for matching kinds.
  - Test that `AdjectiveAmbiguity::can_be_applied_to` returns false for non-matching kinds.
  - Test that `AdjectiveAmbiguity::first_meaning` returns the first meaning.
  - Test that `AdjectiveAmbiguity::first_meaning` returns None for an adjective with no meanings.
  - Test that `AdjectiveMeaningDomains::new_from_kind` creates domain data with the correct kind.
  - Test that `AdjectiveMeaningDomains::set_from_kind` updates the meaning's domain.
  - Test that `AdjectiveMeaningDomains::get_kind` returns the domain kind.
  - Test that `AdjectiveMeaningDomains::get_kind` returns None for an undetermined domain.
  - Test that `AdjectiveMeaningDomains::weak_match` returns true for matching kinds.
  - Test that `AdjectiveMeaningDomains::weak_match` returns false for non-matching kinds.

### 4. Integration verification

- [ ] Verify the crate compiles: `cargo build -p conform7-semantics`.
- [ ] Verify all unit tests pass: `cargo test -p conform7-semantics`.
- [ ] Verify `cargo clippy -p conform7-semantics` is clean.
- [ ] Verify the full workspace still compiles: `cargo build --workspace`.

## Success criteria

- [ ] `Adjectives::declare()` creates a new adjective and returns its index.
- [ ] `Adjectives::declare()` returns the existing index for a duplicate declaration.
- [ ] `Adjectives::find()` finds an existing adjective by name.
- [ ] `Adjectives::find()` returns None for a non-existent adjective.
- [ ] `AdjectiveMeanings::new_family()` creates a family with the correct priority and methods.
- [ ] `AdjectiveMeanings::new()` creates a meaning with the correct family, details, and name.
- [ ] `AdjectiveMeanings::negate()` creates a negated meaning with the correct negated_from field.
- [ ] `AdjectiveMeanings::assert()` dispatches to the family's assert method.
- [ ] `AdjectiveMeanings::assert()` returns false when the family has no assert method.
- [ ] `AdjectiveMeanings::assert()` follows the negation chain and flips parity.
- [ ] `AdjectiveAmbiguity::add_meaning_to_adjective()` adds the meaning to the adjective's list.
- [ ] `AdjectiveAmbiguity::add_meaning_to_adjective()` sets the meaning's owning_adjective field.
- [ ] `AdjectiveAmbiguity::can_be_applied_to()` returns true for matching kinds.
- [ ] `AdjectiveAmbiguity::can_be_applied_to()` returns false for non-matching kinds.
- [ ] `AdjectiveAmbiguity::first_meaning()` returns the first meaning index.
- [ ] `AdjectiveAmbiguity::first_meaning()` returns None for an adjective with no meanings.
- [ ] `AdjectiveMeaningDomains::new_from_kind()` creates domain data with the correct kind.
- [ ] `AdjectiveMeaningDomains::set_from_kind()` updates the meaning's domain.
- [ ] `AdjectiveMeaningDomains::get_kind()` returns the domain kind.
- [ ] `AdjectiveMeaningDomains::get_kind()` returns None for an undetermined domain.
- [ ] `AdjectiveMeaningDomains::weak_match()` returns true for matching kinds.
- [ ] `AdjectiveMeaningDomains::weak_match()` returns false for non-matching kinds.
- [ ] All unit tests pass.
- [ ] `cargo clippy --all-targets` is clean for the semantics crate.

## Out of scope

- **`InstanceAdjectives`**: The instance adjectives system (`InstanceAdjectives::start()`, Chapter 2/Instances as Adjectives.w) is deferred. This plan builds the infrastructure it depends on, but does not implement the `enumerative_amf` family or `InstanceAdjectives::make_adjectival`.
- **`EitherOrPropertyAdjectives`**: The either-or property adjectives system (`EitherOrPropertyAdjectives::start()`, Chapter 3/Either-Or Property Adjectives.w) is deferred. This plan builds the infrastructure it depends on, but does not implement the `either_or_property_amf` family or `EitherOrPropertyAdjectives::create_for_property`.
- **`MeasurementAdjectives`**: The measurement adjectives system (`MeasurementAdjectives::start()`, Chapter 3/Measurement Adjectives.w) is deferred. This plan builds the infrastructure it depends on, but does not implement the `measurement_amf` family or `MeasurementAdjectives::claim_definition`.
- **`ComparativeRelations`**: The comparative relations family (`ComparativeRelations::start()`, Chapter 3/Comparative Relations.w) is deferred. This depends on measurement adjectives.
- **`AdjectiveMeaningDomains::determine`**: The full domain determination logic (parsing text-based domains, circularity detection, problem messages) is deferred. This plan implements only `set_from_kind` and `new_from_kind` — text-based domain resolution is deferred.
- **`AdjectiveAmbiguity::sort`**: The full meaning sorting into precedence order is deferred. This plan stores meanings in definition order only.
- **`AdjectiveAmbiguity::assert`**: The full adjective assertion logic (sorting, iterating in precedence order, strong matching) is deferred. This plan implements only `AdjectiveMeanings::assert` (single-meaning dispatch).
- **`AdjectiveAmbiguity::schema_for_task`**: The schema generation for adjectives is deferred. This depends on the full I6 schema system.
- **`AdjectiveMeanings::claim_definition`**: The definition claiming system (iterating families by priority, parsing definition sentences) is deferred.
- **`AdjectiveMeanings::make_schema`**: The schema creation for adjective meanings is deferred. This depends on `Calculus::Schemas`.
- **`AdjectiveMeanings::perform_task_via_function`**: The support function task mode is deferred.
- **`AdjectiveMeanings::get_schema`**: The schema retrieval for adjective meanings is deferred.
- **`AdjectiveCompilationData`**: The full compilation data for adjectives (`adjective_compilation_data`, `RTAdjectives`) is deferred.
- **`LexicalCluster`**: The full lexical cluster system for inflected adjective forms is deferred. This plan uses simple string names.
- **`LinguisticStock`**: The linguistic stock system for grammatical categories is deferred.
- **`Lexicon` registration**: Registering adjectives with the lexicon module for parsing is deferred.
- **`Preform` grammar**: The `<adjective-name>` Preform grammar for parsing adjective names is deferred.
- **`RTProperties`**: The run-time compilation system for properties is deferred.
- **`Calculus::Schemas`**: The full schema system for run-time code generation is deferred.
- **The model world**: `The Model World` (Chapter 5/The Model World.w) — the five-stage model completion process, depends on all inference subject families, is deferred.
- **`Assert Propositions`**: `Assert::true` and `Assert::true_about` (Chapter 1/Assert Propositions.w) — the assertion pipeline, depends on the full property system, instances, and typechecking, is deferred.
- **Salsa database integration**: The Salsa incremental computation framework is not yet integrated.
