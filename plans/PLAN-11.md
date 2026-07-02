# Plan 11: Verb System Data Structures and Creation

**Status**: In progress
**Target**: 3-5 days

## Goal

Implement the verb system data structures — the foundation for sentence-level parsing. This includes the `verb`, `verb_form`, `verb_sense`, `verb_meaning`, `verb_usage`, `verb_usage_tier`, `preposition`, and `special_meaning_holder` types, along with their creation functions and a simplified verb conjugation system for English.

This is the smallest next step after PLAN-10 because:

1. **Noun phrase parsing is complete.** PLAN-10 implemented `<np-unparsed>`, `<np-articled>`, the article system, and the diagram node types. The next piece of the linguistics module is the verb system, which is needed to turn sentences into full VERB_NT diagrams with subject and object children.

2. **The verb system is the prerequisite for sentence parsing.** The `<sentence>` internal NT calls `VerbPhrases::seek`, which searches for verb usages in a wording, identifies the primary verb, and splits the sentence into subject and object phrases. Before we can implement `VerbPhrases::seek`, we need the data structures that represent verbs, their forms, their usages, and their meanings.

3. **Independently testable.** We can construct verb conjugations, create verb forms and usages, build prepositions, and test the creation and lookup functions without needing the full `VerbPhrases::seek` algorithm. Each data structure can be unit-tested in isolation.

4. **Follows the same pattern as PLAN-10.** The article system in PLAN-10 established the pattern for linguistic data structures (small word sets, internal NTs, creation functions). The verb system follows the same pattern but is more complex, with multiple interconnected types.

## Background

### C reference architecture

The verb system in the C reference spans two modules:

**Linguistics module** (`services/linguistics-module/Chapter 3/`):

- **Verbs.w** — the `verb` struct (conjugation, forms, stock), `verb_form` struct (prepositions, form structures, senses), `verb_sense` struct (holder for verb meaning), and creation functions (`Verbs::new_verb`, `Verbs::add_form`, `Verbs::find_form`).
- **Verb Meanings.w** — the `verb_meaning` struct (regular meaning, special meaning, indirection), creation functions (`VerbMeanings::regular`, `VerbMeanings::special`, `VerbMeanings::indirected`), and the `VERB_MEANING_LINGUISTICS_TYPE` abstraction.
- **Verb Usages.w** — the `verb_usage` struct (grammatical usage, text, tier), `verb_usage_tier` struct (priority, contents), the search list and tier system, and creation functions (`VerbUsages::new`, `VerbUsages::parse_against_verb`).
- **Prepositions.w** — the `preposition` struct (text, lexical entry), creation functions (`Prepositions::make`), and parsing against preposition usages.
- **Special Meanings.w** — the `special_meaning_holder` struct (function pointer, name, metadata), creation functions (`SpecialMeanings::declare`), and the `ACCEPT_SMFT` task.
- **Adverbs of Certainty.w** — the certainty level constants (`CERTAIN_CE`, `LIKELY_CE`, `UNKNOWN_CE`, etc.) and the `<certainty>` internal NT.

**Inflections module** (`services/inflections-module/Chapter 3/`):

- **Verb Conjugation.w** — the `verb_conjugation` struct (infinitive, participles, tabulations), `verb_tabulation` struct (auxiliary, text for each tense/sense/person/number), and the conjugation creation functions (`Conjugation::conjugate`, `Conjugation::conjugate_with_overrides`).
- **Linguistic Constants.w** — the `lcon_ti` type and the constants for voice, tense, sense, person, number, case, and gender.

**Linguistics module** (`services/linguistics-module/Chapter 1/`):

- **Stock Control.w** — the `grammatical_category`, `linguistic_stock_item`, and `grammatical_usage` types that provide the inventory system for all linguistic items.

### Key C source files

- `services/linguistics-module/Chapter 3/Verbs.w` — verb, verb_form, verb_sense structs and creation
- `services/linguistics-module/Chapter 3/Verb Meanings.w` — verb_meaning struct and creation
- `services/linguistics-module/Chapter 3/Verb Usages.w` — verb_usage, verb_usage_tier structs and creation
- `services/linguistics-module/Chapter 3/Prepositions.w` — preposition struct and creation
- `services/linguistics-module/Chapter 3/Special Meanings.w` — special_meaning_holder struct and creation
- `services/linguistics-module/Chapter 3/Adverbs of Certainty.w` — certainty level constants
- `services/inflections-module/Chapter 3/Verb Conjugation.w` — verb_conjugation struct and conjugation
- `services/inflections-module/Chapter 3/Linguistic Constants.w` — lcon_ti type and constants
- `services/linguistics-module/Chapter 1/Stock Control.w` — grammatical_category, linguistic_stock_item, grammatical_usage
- `services/linguistics-module/Chapter 4/Verb Phrases.w` — VerbPhrases::seek (deferred to PLAN-12)

### Current Rust state

- `crates/conform7-syntax/src/linguistics.rs` — article system, diagram constructors, noun phrase parsing (from PLAN-10)
- `crates/conform7-syntax/src/node_type.rs` — NodeType enum with linguistics variants (VERB_NT, etc.)
- `crates/conform7-syntax/src/preform.rs` — Preform matching engine with internal NT dispatch
- `crates/conform7-syntax/src/preform_internal.rs` — basic internal NTs

### What's missing

1. **Word assemblage type**: No `word_assemblage` equivalent for representing multi-word verb/preposition texts.
2. **Verb conjugation**: No `verb_conjugation` or `verb_tabulation` types for conjugating verbs into their forms.
3. **Verb data structures**: No `verb`, `verb_form`, `verb_sense`, `verb_meaning`, `verb_usage`, or `verb_usage_tier` types.
4. **Preposition data structure**: No `preposition` type.
5. **Special meaning holder**: No `special_meaning_holder` type.
6. **Stock control**: No `grammatical_category`, `linguistic_stock_item`, or `grammatical_usage` types.
7. **Linguistic constants**: No `lcon_ti` type or constants for voice, tense, sense, person, number.
8. **Certainty levels**: No certainty level constants or `<certainty>` internal NT.

## Tasks

### 1. Add word assemblage type

The `word_assemblage` type represents a multi-word text (e.g., "carry out" for a phrasal verb, "in front of" for a preposition). It is used throughout the verb system for verb texts, preposition texts, and reference texts.

- [ ] Add a `WordAssemblage` struct to `conform7-syntax`:
  ```rust
  pub struct WordAssemblage {
      pub words: Vec<String>,
  }
  ```
- [ ] Implement constructors: `new(words: Vec<String>)`, `lit_0()` (empty), `lit_1(word: &str)`, `join(a, b)`.
- [ ] Implement accessors: `first_word()`, `length()`, `nonempty()`, `eq()`.
- [ ] Implement `Display` for logging.
- [ ] Add unit tests for construction, joining, equality, and length.

### 2. Add linguistic constants module

The `lcon_ti` type encodes a linguistic constant — a compact representation of a word form with grammatical attributes (voice, tense, sense, person, number, case, gender). For this plan we implement a simplified version that supports the attributes needed by the verb system.

- [ ] Add a `linguistic_constants` module to `conform7-syntax` with:
  - `Lcon` struct wrapping an ID and grammatical attributes.
  - Constants for voice: `ACTIVE_VOICE`, `PASSIVE_VOICE`.
  - Constants for tense: `IS_TENSE`, `WAS_TENSE`, `HAS_TENSE`, `HAD_TENSE`, `WILL_TENSE`, `WOULD_TENSE`.
  - Constants for sense: `POSITIVE_SENSE`, `NEGATIVE_SENSE`.
  - Constants for person: `FIRST_PERSON`, `SECOND_PERSON`, `THIRD_PERSON`.
  - Constants for number: `SINGULAR_NUMBER`, `PLURAL_NUMBER`.
  - Constants for case: `NOMINATIVE_CASE`, `ACCUSATIVE_CASE`, `GENITIVE_CASE`, `DATIVE_CASE`.
  - Constants for gender: `NEUTER_GENDER`, `MASCULINE_GENDER`, `FEMININE_GENDER`, `COMMON_GENDER`.
  - `Lcon::of_id(id)` and `Lcon::get_id(lcon)` for stock references.
  - Helper methods: `get_voice`, `get_tense`, `get_sense`, `get_person`, `get_number`, `get_case`, `get_gender`.
- [ ] Add unit tests for Lcon construction and attribute access.

### 3. Add stock control types

The stock control system provides the inventory for all linguistic items (verbs, verb forms, prepositions, nouns, etc.). Each item belongs to a grammatical category and can be looked up by its allocation ID.

- [ ] Add a `stock_control` module to `conform7-syntax` with:
  - `GrammaticalCategory` struct (name, method set stub, item count).
  - `LinguisticStockItem` struct (category, data pointer as `Box<dyn Any>`).
  - `Stock` struct as a registry of categories and items.
  - `Stock::new_category(name)` — create a new grammatical category.
  - `Stock::new(category, data)` — create a new stock item and register it.
  - `Stock::to_lcon(item)` — convert a stock item to an Lcon reference.
  - `Stock::from_lcon(lcon)` — look up a stock item from an Lcon reference.
  - `GrammaticalUsage` struct (used item, language, possible forms as `Vec<Lcon>`).
  - `Stock::new_usage(item, language)` — create a new grammatical usage.
  - `Stock::add_form_to_usage(usage, form)` — add a possible form to a usage.
  - `Stock::first_form_in_usage(usage)` — get the first form.
  - `Stock::usage_might_be_singular(usage)` — check if usage could be singular.
  - `Stock::usage_might_be_third_person(usage)` — check if usage could be third person.
- [ ] Add unit tests for stock creation, item registration, and usage creation.

### 4. Add certainty level constants

- [ ] Add certainty level constants to the linguistics module:
  ```rust
  pub const IMPOSSIBLE_CE: i32 = -2;
  pub const UNLIKELY_CE: i32 = -1;
  pub const UNKNOWN_CE: i32 = 0;
  pub const LIKELY_CE: i32 = 1;
  pub const CERTAIN_CE: i32 = 2;
  pub const INITIALLY_CE: i32 = 3;
  ```
- [ ] Add a `Certainty::write` function for display.
- [ ] Add the `<certainty>` internal NT that matches certainty adverbs and returns the corresponding level (matching `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`):
  - `always`/`certainly` → `CERTAIN_CE`
  - `usually`/`normally` → `LIKELY_CE`
  - `rarely`/`seldom` → `UNLIKELY_CE`
  - `never` → `IMPOSSIBLE_CE`
  - `initially` → `INITIALLY_CE`
- [ ] Register `<certainty>` in `InternalRegistry::linguistics()`.
- [ ] Add unit tests for certainty matching.

### 5. Add verb conjugation (simplified for English)

The verb conjugation system turns a base verb form (infinitive) into all its conjugated variants. For this plan we implement a simplified version that handles the most common English verbs using hardcoded conjugation tables, with the full `Conjugation::conjugate` system deferred.

- [ ] Add a `verb_conjugation` module to `conform7-syntax` with:
  - `VerbConjugation` struct:
    ```rust
    pub struct VerbConjugation {
        pub infinitive: WordAssemblage,
        pub past_participle: WordAssemblage,
        pub present_participle: WordAssemblage,
        pub tabulations: [VerbTabulation; 2], // active, passive
        pub auxiliary_only: bool,
    }
    ```
  - `VerbTabulation` struct:
    ```rust
    pub struct VerbTabulation {
        pub to_be_auxiliary: WordAssemblage,
        pub vc_text: [[[[WordAssemblage; NO_KNOWN_NUMBERS]; NO_KNOWN_PERSONS]; NO_KNOWN_SENSES]; NO_KNOWN_TENSES],
    }
    ```
  - Constants: `NO_KNOWN_VOICES = 2`, `NO_KNOWN_TENSES = 6`, `NO_KNOWN_SENSES = 2`, `NO_KNOWN_PERSONS = 3`, `NO_KNOWN_NUMBERS = 2`.
- [ ] Implement `Conjugation::conjugate(base_text, language)` that:
  1. Creates a `VerbConjugation` struct.
  2. Sets the infinitive, present_participle, and past_participle.
  3. Fills in the tabulation slots using a simplified conjugation algorithm.
- [ ] Implement hardcoded conjugation for the copular verb "to be":
  - Present: am, are, is / are
  - Past: was, were, was / were
  - Present participle: being
  - Past participle: been
- [ ] Implement hardcoded conjugation for "to have":
  - Present: have, have, has / have
  - Past: had, had, had / had
  - Present participle: having
  - Past participle: had
- [ ] Implement `Conjugation::find_by_infinitive(assemblage)` for looking up conjugations.
- [ ] Add unit tests for verb conjugation creation and lookup.

### 6. Add verb data structures and creation

- [ ] Add a `verbs` module to `conform7-syntax` with the core verb types:

  **Verb struct** (from `services/linguistics-module/Chapter 3/Verbs.w`):
  ```rust
  pub struct Verb {
      pub conjugation: Option<VerbConjugationRef>, // index into conjugation registry
      pub first_form: Option<VerbFormRef>, // index into form list
      pub base_form: Option<VerbFormRef>,
      pub in_stock: Option<LinguisticStockItemRef>,
  }
  ```

  **VerbForm struct** (from `Verbs.w`):
  ```rust
  pub struct VerbForm {
      pub underlying_verb: VerbRef,
      pub preposition: Option<PrepositionRef>,
      pub second_clause_preposition: Option<PrepositionRef>,
      pub form_structures: u8, // bitmap of SVO_FS_BIT, VO_FS_BIT, SVOO_FS_BIT, VOO_FS_BIT
      pub infinitive_reference_text: WordAssemblage,
      pub pos_reference_text: WordAssemblage,
      pub neg_reference_text: WordAssemblage,
      pub list_of_senses: Vec<VerbSenseRef>,
      pub next_form: Option<VerbFormRef>,
  }
  ```

  **VerbSense struct** (from `Verbs.w`):
  ```rust
  pub struct VerbSense {
      pub vm: VerbMeaning,
      pub next_sense: Option<VerbSenseRef>,
  }
  ```

  **Form structure constants**:
  ```rust
  pub const SVO_FS_BIT: u8 = 1;
  pub const VO_FS_BIT: u8 = 2;
  pub const SVOO_FS_BIT: u8 = 4;
  pub const VOO_FS_BIT: u8 = 8;
  ```

- [ ] Implement `Verbs` struct as a registry of all verbs, verb forms, and verb senses:
  - `Verbs::new_verb(conjugation, copular) -> VerbRef` — create a new verb (matching `Verbs::new_verb` in C).
  - `Verbs::new_operator_verb(meaning) -> VerbRef` — create an operator verb (matching `Verbs::new_operator_verb`).
  - `Verbs::add_form(verb, prep, second_prep, meaning, form_structs)` — add a form to a verb (matching `Verbs::add_form`).
  - `Verbs::find_form(verb, prep, second_prep) -> Option<VerbFormRef>` — find a form by prepositions.
  - `Verbs::base_form(verb) -> Option<VerbFormRef>` — get the base form (no prepositions).
  - `Verbs::from_lcon(lcon) -> Option<VerbRef>` — look up a verb from a stock reference.
  - `Verbs::to_lcon(verb) -> Lcon` — convert a verb to a stock reference.
  - Track `copular_verb: Option<VerbRef>` — the first copular verb registered.

- [ ] Add unit tests for verb creation, form creation, form lookup, and copular verb tracking.

### 7. Add verb meaning types and creation

- [ ] Add verb meaning types to the `verbs` module (from `services/linguistics-module/Chapter 3/Verb Meanings.w`):
  ```rust
  pub struct VerbMeaning {
      pub take_meaning_reversed: bool,
      pub regular_meaning: Option<Box<dyn Any>>, // in I7, a binary predicate
      pub special_meaning: Option<SpecialMeaningRef>,
      pub take_meaning_from: Option<VerbRef>,
      pub where_assigned: Option<usize>, // sentence index for problem messages
  }
  ```

- [ ] Implement `VerbMeanings` creation functions:
  - `VerbMeanings::meaninglessness() -> VerbMeaning` — create a meaningless verb meaning.
  - `VerbMeanings::is_meaningless(vm) -> bool` — check if a meaning is meaningless.
  - `VerbMeanings::regular(rel) -> VerbMeaning` — create a regular meaning.
  - `VerbMeanings::special(sm) -> VerbMeaning` — create a special meaning.
  - `VerbMeanings::indirected(from, reversed) -> VerbMeaning` — create an indirected meaning.
  - `VerbMeanings::follow_indirection(vm) -> Option<&VerbMeaning>` — resolve indirection.
  - `VerbMeanings::reverse_vmt(recto) -> Option<Box<dyn Any>>` — reverse a meaning (stub for now).

- [ ] Add unit tests for verb meaning creation and indirection.

### 8. Add verb usage types and creation

- [ ] Add verb usage types to the `verbs` module (from `services/linguistics-module/Chapter 3/Verb Usages.w`):
  ```rust
  pub struct VerbUsage {
      pub usage: GrammaticalUsage,
      pub vu_text: WordAssemblage,
      pub vu_allow_unexpected_upper_case: bool,
      pub next_in_search_list: Option<VerbUsageRef>,
      pub next_within_tier: Option<VerbUsageRef>,
      pub where_vu_created: Option<usize>,
      pub vu_lex_entry: Option<VerbConjugationRef>,
  }

  pub struct VerbUsageTier {
      pub priority: i32,
      pub tier_contents: Vec<VerbUsageRef>,
      pub next_tier: Option<VerbUsageTierRef>,
  }
  ```

- [ ] Implement `VerbUsages` creation and management:
  - `VerbUsages::new(wa, unexpected_upper_casing, usage, where) -> Option<VerbUsageRef>` — create a new verb usage (matching `VerbUsages::new` in C).
  - `VerbUsages::get_verb(vu) -> Option<VerbRef>` — get the verb from a usage.
  - `VerbUsages::parse_against_verb(tw, vu) -> Option<usize>` — parse a wording against a verb usage (matching the C function of the same name). This checks if the verb usage text appears at the start of the wording and returns the word position after the match.
  - `VerbUsages::mark_as_verb(word)` — mark a word as being a verb (stub for now).
  - `VerbUsages::adaptive_person(conjugation)` — get the adaptive person for a conjugation.
  - `VerbUsages::adaptive_number(conjugation)` — get the adaptive number for a conjugation.
  - Search list management: maintain a linked list of all usages in length order.
  - Tier management: maintain a linked list of tiers in priority order, with usages assigned to tiers.

- [ ] Add unit tests for verb usage creation, search list ordering, and tier management.

### 9. Add preposition type and creation

- [ ] Add preposition types to the `verbs` module (from `services/linguistics-module/Chapter 3/Prepositions.w`):
  ```rust
  pub struct Preposition {
      pub prep_text: WordAssemblage,
      pub prep_lex_entry: Option<VerbConjugationRef>,
      pub where_prep_created: Option<usize>,
      pub allow_unexpected_upper_case: bool,
      pub in_stock: Option<LinguisticStockItemRef>,
  }
  ```

- [ ] Implement `Prepositions` creation and management:
  - `Prepositions::make(wa, unexpected_upper_casing, where) -> PrepositionRef` — create or find a preposition (matching `Prepositions::make` in C). Prepositions are deduplicated by text.
  - `Prepositions::length(prep) -> usize` — get the word count of a preposition.
  - `Prepositions::mark_as_preposition(word)` — mark a word as a preposition (stub for now).
  - `Prepositions::get_where_pu_created(prep) -> Option<usize>` — get the creation location.
  - `Prepositions::parse_against(wording, prep) -> Option<usize>` — parse a wording against a preposition (matching the C function). Checks if the preposition text appears at the start of the wording.

- [ ] Add unit tests for preposition creation, deduplication, and parsing.

### 10. Add special meaning holder type and creation

- [ ] Add special meaning types to the `verbs` module (from `services/linguistics-module/Chapter 3/Special Meanings.w`):
  ```rust
  pub type SpecialMeaningFn = fn(task: i32, node: &mut ParseNode, nps: &[Wording; 3]) -> bool;

  pub struct SpecialMeaningHolder {
      pub sm_func: SpecialMeaningFn,
      pub sm_name: String,
      pub metadata_n: i32,
  }
  ```

- [ ] Implement `SpecialMeanings` creation and management:
  - `SpecialMeanings::declare(func, name, metadata) -> SpecialMeaningRef` — declare a new special meaning (matching `SpecialMeanings::declare` in C).
  - `SpecialMeanings::find_from_wording(wording) -> Option<SpecialMeaningRef>` — find a special meaning by name.
  - `SpecialMeanings::call(smh, task, node, nps) -> bool` — call a special meaning function.
  - `SpecialMeanings::get_metadata_n(smh) -> i32` — get metadata.
  - `SpecialMeanings::get_name(smh) -> &str` — get the name.
  - `SpecialMeanings::is(smh, func) -> bool` — check if a special meaning uses a given function.
  - `SpecialMeanings::generic_smf(task, node, nps) -> bool` — the generic special meaning function that accumulates non-empty SPs and OPs as unparsed noun phrases (matching `SpecialMeanings::generic_smf` in C).

- [ ] Add unit tests for special meaning declaration, lookup, and calling.

### 11. Wire up verb system internal NTs

- [ ] Register the `<certainty>` internal NT in `InternalRegistry::linguistics()`.
- [ ] Add a `<nonimperative-verb>` internal NT stub that matches known verb usages (to be expanded in PLAN-12).
- [ ] Add a `<negated-noncopular-verb-present>` internal NT stub.
- [ ] Add a `<pre-verb-rc-marker>` internal NT stub for relative clause markers ("who", "which", "that").
- [ ] Add a `<pre-verb-certainty>` internal NT that matches certainty adverbs before the verb.
- [ ] Add a `<post-verb-certainty>` internal NT that matches certainty adverbs after the verb.
- [ ] Register all new internal NTs in `InternalRegistry::linguistics()`.

### 12. Integration tests

- [ ] Test that a verb conjugation for "to be" produces the correct forms:
  - Infinitive: "be"
  - Present, third person, singular, positive: "is"
  - Present, third person, plural, positive: "are"
  - Past, third person, singular, positive: "was"
  - Past, third person, plural, positive: "were"
- [ ] Test that `Verbs::new_verb` creates a verb with a single meaningless base form.
- [ ] Test that `Verbs::add_form` adds a form and `Verbs::find_form` finds it by prepositions.
- [ ] Test that the copular verb is tracked correctly (the first verb created with `copular=true`).
- [ ] Test that `VerbUsages::new` creates a usage and adds it to the search list.
- [ ] Test that `VerbUsages::parse_against_verb` matches "is" at the start of "is in the room" and returns the position after "is".
- [ ] Test that `Prepositions::make` deduplicates prepositions by text.
- [ ] Test that `Prepositions::parse_against` matches "in" at the start of "in the room".
- [ ] Test that `SpecialMeanings::declare` creates a holder and `SpecialMeanings::find_from_wording` finds it.
- [ ] Test that `SpecialMeanings::generic_smf` accepts a sentence with subject and object.
- [ ] Test that the `<certainty>` internal NT matches "always" and returns `CERTAIN_CE`.
- [ ] Test that the `<certainty>` internal NT fails on "xyzzy".
- [ ] Test that `VerbMeanings::indirected` creates a meaning that follows indirection correctly.
- [ ] Test that `Stock::new_category` and `Stock::new` create and register items correctly.
- [ ] Test that `Stock::to_lcon` and `Stock::from_lcon` round-trip correctly.
- [ ] Test that `GrammaticalUsage` creation and form addition works.
- [ ] Test that `WordAssemblage` construction, joining, and equality work.
- [ ] Test that `Lcon` construction and attribute access work.

## Success criteria

- [ ] All verb system data structures (`Verb`, `VerbForm`, `VerbSense`, `VerbMeaning`, `VerbUsage`, `VerbUsageTier`, `Preposition`, `SpecialMeaningHolder`, `VerbConjugation`, `VerbTabulation`) are defined with correct fields matching the C reference.
- [ ] `Verbs::new_verb` creates a verb with a single meaningless base form, matching `Verbs::new_verb` in `services/linguistics-module/Chapter 3/Verbs.w`.
- [ ] `Verbs::add_form` adds a form to a verb and `Verbs::find_form` finds it by verb and prepositions.
- [ ] The copular verb is correctly identified (first verb created with `copular=true`).
- [ ] `VerbUsages::new` creates a usage and adds it to the search list in length order.
- [ ] `VerbUsages::parse_against_verb` correctly matches verb usage text at the start of a wording.
- [ ] `Prepositions::make` deduplicates prepositions by text.
- [ ] `Prepositions::parse_against` correctly matches preposition text at the start of a wording.
- [ ] `SpecialMeanings::declare` creates a holder and `SpecialMeanings::call` invokes the function.
- [ ] `SpecialMeanings::generic_smf` accepts a sentence with subject and object, creating unparsed noun nodes.
- [ ] The `<certainty>` internal NT correctly matches certainty adverbs and returns the corresponding level.
- [ ] Verb conjugation for "to be" produces correct forms for present and past tenses.
- [ ] `Stock::new_category` and `Stock::new` create and register items correctly.
- [ ] `Stock::to_lcon` and `Stock::from_lcon` round-trip correctly.
- [ ] `WordAssemblage` construction, joining, and equality work correctly.
- [ ] `Lcon` construction and attribute access work correctly.
- [ ] All new internal NTs are registered in `InternalRegistry::linguistics()` and dispatch correctly through the matching engine.
- [ ] All existing tests still pass after the API changes.
- [ ] New tests are explicitly grounded in the C reference files listed in the Background section.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- **VerbPhrases::seek** (the verb-finding algorithm). This is the main function that searches for verb usages in a wording, builds the viability map, and produces VERB_NT sentence diagrams. It depends on the verb system data structures implemented here and belongs in PLAN-12.
- **Full verb conjugation system** (`Conjugation::conjugate` with Preform-based tabulations). The simplified conjugation uses hardcoded tables for common English verbs. The full system with Preform-based tabulation instructions is deferred.
- **The `<sentence>` and `<sentence-without-occurrences>` internal NTs**. These call `VerbPhrases::seek` and produce full sentence diagrams. They depend on verb phrase parsing and belong in PLAN-12.
- **The full verb usage tier system** with priority-based search. The tier data structures are implemented, but the full priority-based search algorithm is part of `VerbPhrases::seek`.
- **Verb compilation data** (`VERB_COMPILATION_LINGUISTICS_CALLBACK`, `VERB_FORM_COMPILATION_LINGUISTICS_CALLBACK`). These are Inform-specific callbacks for code generation and are not needed for the linguistics module.
- **The `==>` compositor syntax** in the Preform grammar parser. Compositors are implemented as Rust functions, not parsed from the grammar.
- **Noun recognition** (`Nouns::recognise`) — turning `UNPARSED_NOUN_NT` into `COMMON_NOUN_NT`/`PROPER_NOUN_NT` by looking up known nouns. This requires the noun lexicon and belongs in a later plan.
- **The full `small_word_set` type** from Stock Control.w. The simplified `HashMap<String, T>` from PLAN-10 is sufficient for now.
- **The `linguistic_stock_item` flat array** for O(1) lookup by allocation ID. The simplified registry using `Vec<Box<dyn Any>>` is sufficient for now.
- **The `grammatical_category` method system** (`METHOD_ADD`, `VOID_METHOD_CALL`). Categories are implicit in the simplified implementation.
- **The `lcon_ti` full system** with bit-packed attributes. The simplified `Lcon` struct with explicit fields is sufficient for now.
- **Preform grammar optimization** (NT incidence bits, word range extremes).
