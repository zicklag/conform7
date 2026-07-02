# Plan 12: VerbPhrases::seek — Viability Map and Verb-Finding Algorithm
**Status**: Complete
**Target**: 3-5 days

## Goal

Implement the core `VerbPhrases::seek` algorithm — the verb-finding function that locates the primary verb in a sentence, identifies its subject and object phrases, and builds the `VERB_NT` sentence diagram. This is the heart of sentence-level parsing in Inform 7.

This is the smallest next step after PLAN-11 because:

1. **All verb data structures are complete.** PLAN-11 implemented `Verb`, `VerbForm`, `VerbSense`, `VerbMeaning`, `VerbUsage`, `VerbUsageTier`, `Preposition`, `SpecialMeaningHolder`, `WordAssemblage`, `Lcon`, `Stock`, `Conjugation`, and the certainty system. What's missing is the algorithm that *uses* these structures to find verbs in real sentences.

2. **The viability map is the entry point to sentence parsing.** Before we can parse a sentence, we need to know which words might be verbs. The viability map scores each word in a sentence for verb likelihood, using the `<nonimperative-verb>` and `<negated-noncopular-verb-present>` internal NTs (currently stubs). Implementing this is the first step of `VerbPhrases::seek`.

3. **The seek loop ties the verb system together.** The algorithm iterates verb usage tiers, checks form structures (SVO, VO, SVOO, VOO), verifies required prepositions, and builds the `VERB_NT` diagram with subject and object children. This is the integration point that validates the entire verb data structure design from PLAN-11.

4. **Independently testable.** We can test the viability map with synthetic sentences, test the seek loop with registered verb usages, and test the diagram construction with known verb forms. Each sub-component has clear inputs and outputs.

5. **Prerequisite for sentence parsing.** The `<sentence>` internal NT calls `VerbPhrases::seek` directly. Once seek is implemented, the `<sentence>` nonterminal can be wired up to produce full sentence diagrams.

## Background

### C reference architecture

The verb-finding algorithm is defined in `services/linguistics-module/Chapter 4/Verb Phrases.w`:

- **`VerbPhrases::seek`** (lines 102-119): Entry point that logs tracing info and delegates to `seek_inner`.
- **`VerbPhrases::seek_inner`** (lines 121-128): Calculates the viability map, then seeks verb usages.
- **Viability map calculation** (lines 158-184): Scores each word in the sentence (0 = not a verb, 1 = verb outside brackets, 2 = verb inside brackets, 3 = negated non-copular verb). Uses `<nonimperative-verb>` and `<negated-noncopular-verb-present>` internal NTs.
- **Seek verb usages** (lines 202-237): Two-pass loop over viability levels (1 then 2), then over tiers (excluding priority 0), then over positions, then over verb usages within each tier.
- **Verb usage matching** (lines 246-272): Tests whether a verb usage appears at the front of the tail wording, checks form structures, and builds subject/object wordings.
- **Form structure checking** (lines 285-291): Rejects matches where the verb is in the wrong position for its form structure (SVO verbs can't be at position 0, VO verbs must be at position 0).
- **Preposition checking** (lines 374-407): Verifies that required prepositions (first and second clause) are present in the object wording.
- **Diagram building** (lines 414-444): Creates a `VERB_NT` parse node with certainty annotations, verb usage reference, preposition references, and subject/object wordings.
- **`VerbPhrases::accept`** (lines 472-496): Tries each sense of the verb form, calling special meaning functions with `ACCEPT_SMFT`, falling back to the regular meaning.
- **`VerbPhrases::default_verb`** (lines 503-527): Builds the default sentence diagram for regular verb meanings, parsing subject and object as noun phrases.
- **Corrective surgery** (lines 582-696): Post-processing that performs "of surgery" (splitting `X of Y` noun phrases), "location surgery" (handling `X is on Y and under Z`), and "called surgery" (fixing `X called Y` ordering).

### Key C source files

- `services/linguistics-module/Chapter 4/Verb Phrases.w` — VerbPhrases::seek, accept, default_verb, corrective_surgery
- `services/linguistics-module/Chapter 3/Verbs.w` — verb, verb_form, verb_sense structs and creation
- `services/linguistics-module/Chapter 3/Verb Meanings.w` — verb_meaning struct and creation
- `services/linguistics-module/Chapter 3/Verb Usages.w` — verb_usage, verb_usage_tier structs and creation
- `services/linguistics-module/Chapter 3/Prepositions.w` — preposition struct and creation
- `services/linguistics-module/Chapter 3/Special Meanings.w` — special_meaning_holder struct and creation
- `services/linguistics-module/Chapter 1/Diagrams.w` — sentence diagram node types and constructors
- `services/linguistics-module/Chapter 4/Noun Phrases.w` — noun phrase parsing (for subject/object in default_verb)
- `services/words-module/Chapter 4/Nonterminals.w` — internal nonterminal dispatch

### Current Rust state

- `crates/conform7-syntax/src/verbs.rs` — Verb, VerbForm, VerbSense, VerbMeaning, VerbUsage, VerbUsageTier, Preposition, SpecialMeaningHolder, Verbs registry (all from PLAN-11)
- `crates/conform7-syntax/src/verb_conjugation.rs` — VerbConjugation, VerbTabulation, Conjugation (from PLAN-11)
- `crates/conform7-syntax/src/linguistic_constants.rs` — Lcon type and constants (from PLAN-11)
- `crates/conform7-syntax/src/stock_control.rs` — Stock, GrammaticalCategory, LinguisticStockItem, GrammaticalUsage (from PLAN-11)
- `crates/conform7-syntax/src/word_assemblage.rs` — WordAssemblage (from PLAN-11)
- `crates/conform7-syntax/src/linguistics.rs` — Article system, Diagrams constructors, NounPhrases, certainty constants (from PLAN-10/11)
- `crates/conform7-syntax/src/preform_internal.rs` — Internal NTs including `<nonimperative-verb>` (stub), `<negated-noncopular-verb-present>` (stub), `<pre-verb-rc-marker>`, `<pre-verb-certainty>`, `<post-verb-certainty>`, `<certainty>` (from PLAN-10/11)
- `crates/conform7-syntax/src/node_type.rs` — NodeType enum with VERB_NT, RELATIONSHIP_NT, etc. (from PLAN-10)
- `crates/conform7-syntax/src/parse_node.rs` — ParseNode with Annotation enum (HeadingLevel, ArticleUsage only so far)

### What's missing

1. **`<nonimperative-verb>` implementation**: Currently a stub that always fails. Needs to look up verb usages in the Verbs registry and check if a word matches any known verb usage text.
2. **`<negated-noncopular-verb-present>` implementation**: Currently a stub that always fails. Needs to match patterns like "does not VERB", "do not VERB", "did not VERB" for non-copular verbs.
3. **Viability map calculation**: The algorithm that scores each word in a sentence for verb likelihood.
4. **Verb usage seek loop**: The nested iteration over viability levels, tiers, positions, and verb usages.
5. **Form structure checking**: Rejecting verb matches in positions incompatible with their form structure (SVO/VO/SVOO/VOO).
6. **Preposition checking**: Verifying required prepositions are present in the object wording.
7. **Diagram building**: Creating VERB_NT nodes with certainty, verb usage, and preposition annotations.
8. **Verb-related annotations**: `verbal_certainty_ANNOT`, `sentence_is_existential_ANNOT`, `linguistic_error_here_ANNOT` in the Annotation enum.
9. **ParseNode verb methods**: `set_verb`/`get_verb`, `set_preposition`/`get_preposition`, `set_second_preposition`/`get_second_preposition`, `set_special_meaning`/`get_special_meaning`, `set_occurrence`/`get_occurrence`.
10. **`VerbPhrases::accept`**: The function that tries each sense of a verb form, calling special meaning functions and falling back to regular meanings.
11. **`VerbPhrases::default_verb`**: The function that builds the default sentence diagram for regular verb meanings.
12. **Corrective surgery**: Post-processing that performs "of surgery", "location surgery", and "called surgery" on the diagram tree.

## Tasks

### 1. Implement `<nonimperative-verb>` internal NT

The `<nonimperative-verb>` internal NT checks whether a word (or multi-word phrase) is a known non-imperative verb usage. In the C reference, this is used by the viability map to score words.

- [ ] Replace the `NonimperativeVerb` stub in `preform_internal.rs` with a real implementation that:
  1. Takes a wording (single word or multi-word verb phrase).
  2. Looks up the wording in the verb usage search list (from the `Verbs` registry).
  3. Returns a match if the wording matches any known verb usage text.
  4. The match payload should include the verb usage reference.
- [ ] The implementation needs access to the `Verbs` registry. Add a `verbs_registry: Option<&'static Verbs>` field to `PreformContext` (or use a thread-local/global registry reference).
- [ ] Add unit tests that:
  - Register a verb usage for "is" and verify `<nonimperative-verb>` matches "is".
  - Register verb usages for "carry", "carries", "carried" and verify matching.
  - Verify that non-verb words like "cat" do not match.
  - Verify that multi-word verb usages (e.g., "carry out") match correctly.

### 2. Implement `<negated-noncopular-verb-present>` internal NT

The `<negated-noncopular-verb-present>` internal NT matches negated present-tense verb forms for non-copular verbs (e.g., "does not carry", "do not carry", "doesn't carry"). This is used by the viability map to assign score 3 to negated verb words.

- [ ] Replace the `NegatedNoncopularVerbPresent` stub in `preform_internal.rs` with a real implementation that:
  1. Takes a wording starting at a potential verb position.
  2. Checks for patterns: `does not <verb>`, `do not <verb>`, `did not <verb>`, `doesn't <verb>`, `don't <verb>`, `didn't <verb>`.
  3. Verifies the `<verb>` part is a known non-copular verb usage (not "to be").
  4. Returns the word position after the full negated verb phrase.
- [ ] Add unit tests that:
  - Match "does not carry" against a registered non-copular verb.
  - Match "doesn't carry" against a registered non-copular verb.
  - Fail on "is not" (copular verb is exempt from score 3).
  - Fail on non-verb words.

### 3. Add verb-related annotations and ParseNode methods

The seek algorithm needs to annotate `VERB_NT` nodes with certainty levels, verb usage references, preposition references, and existential sentence flags.

- [ ] Add new variants to the `Annotation` enum in `parse_node.rs`:
  ```rust
  /// Verbal certainty level (from certainty adverbs).
  VerbalCertainty(i32),
  /// Whether the sentence is existential ("There is ...").
  SentenceIsExistential(bool),
  /// Linguistic error annotation.
  LinguisticErrorHere(i32),
  ```
- [ ] Add methods to `ParseNode` for verb-related data:
  - `set_verb_usage(&mut self, vu: VerbUsageRef)` / `get_verb_usage(&self) -> Option<VerbUsageRef>`
  - `set_preposition(&mut self, prep: Option<PrepositionRef>)` / `get_preposition(&self) -> Option<PrepositionRef>`
  - `set_second_preposition(&mut self, prep: Option<PrepositionRef>)` / `get_second_preposition(&self) -> Option<PrepositionRef>`
  - `set_special_meaning(&mut self, sm: SpecialMeaningRef)` / `get_special_meaning(&self) -> Option<SpecialMeaningRef>`
  - `set_occurrence(&mut self, _tp: ())` / `get_occurrence(&self)` — stub for now (time_period not yet implemented)
- [ ] Add unit tests for the new annotation variants and methods.

### 4. Implement viability map calculation

The viability map assigns a score (0-3) to each word in a sentence, indicating how likely it is to be part of a verb.

- [ ] Add a `VerbPhrases` module (or struct) in `crates/conform7-syntax/src/verb_phrases.rs`:
  ```rust
  pub struct VerbPhrases;
  ```
- [ ] Add the `VIABILITY_MAP_SIZE` constant (100, matching the C reference).
- [ ] Implement the viability map calculation (matching `@<Calculate the viability map@>` in Verb Phrases.w lines 158-184):
  1. Initialize all scores to 0.
  2. Track bracket depth (`bl`): increment on `(` or `{`, decrement on `)` or `}`.
  3. For each word in the wording:
     a. If the word matches `<nonimperative-verb>`:
        - Score 1 if outside brackets, 2 if inside brackets.
        - Check for negated non-copular verb present: if the word starts a negated verb phrase, score all words in that phrase as 3.
        - If in existential OP edge mode, verify `<pre-verb-rc-marker>` on the preceding text.
     b. Otherwise, score 0.
  4. Return the viability map as `[i32; VIABILITY_MAP_SIZE]`.
- [ ] Add a `ViabilityMap` struct to hold the scores and the wording reference.
- [ ] Add unit tests that:
  - A simple sentence "The cat is on the mat" scores "is" as 1, other words as 0.
  - A sentence with brackets "(The cat) is on the mat" scores "is" as 1 (brackets don't affect verb words).
  - A negated sentence "The cat does not carry the box" scores "does", "not", "carry" as 3.
  - A copular negated sentence "The cat is not a dog" scores "is" as 1 (copular exempt from score 3).

### 5. Implement the core seek loop

The seek loop iterates over viability levels, tiers, positions, and verb usages to find the best verb match.

- [ ] Implement `VerbPhrases::seek_inner(wording, registry, existential_op_edge, detect_occurrences) -> Option<ParseNode>`:
  1. Calculate the viability map.
  2. For viability levels 1 and 2:
     a. For each verb usage tier (excluding priority 0):
        - For each word position with matching viability:
          - Try each verb usage in the tier at that position.
  3. Return the first successful match as a `VERB_NT` parse node.
- [ ] Implement `VerbPhrases::seek(wording, registry, detect_occurrences) -> Option<ParseNode>`:
  1. Call `seek_inner` with `existential_op_edge = 0`.
  2. Apply corrective surgery on success.
  3. Return the result.
- [ ] Implement the verb usage matching at a position (matching `@<Seek verb usage at position pos@>` and `@<Consider whether this usage is being made at this position@>`):
  1. Get the tail wording from the current position.
  2. For each verb usage in the tier:
     a. Get the verb from the usage.
     b. For each verb form of the verb:
        - Skip if the verb meaning is meaningless.
        - Call `VerbUsages::parse_against_verb` to check if the usage text appears at the front of the tail wording.
        - If matched, check form structure compatibility.
        - If compatible, check required prepositions.
        - If prepositions are satisfied, build the diagram.
- [ ] Implement form structure checking (matching `@<Reject a match with verb in the wrong position@>`):
  1. If the verb form has no VO or VOO bits, the verb must not be at position 0 (it needs a subject).
  2. If the verb form has no SVO or SVOO bits, the verb must be at position 0 (it's imperative).
- [ ] Implement preposition checking (matching `@<Check whether we do indeed have these required prepositions in place@>`):
  1. If the verb form has a first preposition (`req1`), check it appears at the start of the object wording.
  2. If the verb form has a second preposition (`req2`), search for it later in the object wording.
  3. Trim the matched prepositions from the object wording.
- [ ] Implement diagram building (matching `@<Check whether any sense of this verb form will accept this usage and succeed if so@>`):
  1. Create a `VERB_NT` parse node.
  2. Set certainty annotations if present.
  3. Set verb usage, preposition references.
  4. Set the verb wording.
  5. Call `VerbPhrases::accept` to try each sense.
- [ ] Add unit tests that:
  - Seek finds "is" in "The cat is on the mat" with a registered copular verb.
  - Seek finds "carries" in "Peter carries the flash cards" with an SVO verb.
  - Seek finds "carry" in "Carry the box" with a VO verb (imperative).
  - Seek fails on a sentence with no verb.
  - Seek prefers longer verb usages over shorter ones at the same position.

### 6. Implement VerbPhrases::accept and VerbPhrases::default_verb

The accept function tries each sense of a verb form, calling special meaning functions and falling back to regular meanings.

- [ ] Implement `VerbPhrases::accept(vf, vp_pn, nps) -> Option<ParseNode>` (matching lines 472-496):
  1. Iterate through the verb senses of the verb form.
  2. For each sense, check if it has a special meaning.
  3. If it has a special meaning, call it with `ACCEPT_SMFT`.
  4. If the special meaning accepts, return the node.
  5. If no special meaning accepts, fall back to the regular meaning.
  6. Call `VerbPhrases::default_verb` for the regular meaning.
- [ ] Implement `VerbPhrases::default_verb(vm, vp_pn, nps) -> Option<ParseNode>` (matching lines 503-527):
  1. Parse the subject wording as `<np-as-subject>`.
  2. Parse the object wording as `<np-as-object>`.
  3. Build the sentence diagram: `VERB_NT -> SUBJECT_NT -> OBJECT_NT`.
  4. For non-copular verbs, wrap the object in a `RELATIONSHIP_NT` with the reversed verb meaning.
- [ ] Add unit tests that:
  - `default_verb` builds a correct diagram for a copular verb ("X is Y").
  - `default_verb` builds a correct diagram for a non-copular verb ("X carries Y").

### 7. Implement corrective surgery

Corrective surgery post-processes the diagram tree to handle special cases.

- [ ] Implement `VerbPhrases::corrective_surgery(p: &mut ParseNode)` (matching lines 582-601):
  1. Iterate until no more surgeries are possible.
  2. Try "of surgery", "location surgery", and "called surgery" on each node.
- [ ] Implement `VerbPhrases::perform_of_surgery(p: &mut ParseNode) -> bool` (matching lines 610-629):
  1. Check if the node is an `UNPARSED_NOUN_NT`.
  2. Search for "of" in the wording.
  3. If found, split into `X_OF_Y_NT` with left and right children.
- [ ] Implement `VerbPhrases::perform_location_surgery(p: &mut ParseNode) -> bool` (matching lines 651-673):
  1. Check if the node is a `RELATIONSHIP_NT` with an `AND_NT` child.
  2. Restructure the tree to handle "X is on Y and under Z".
- [ ] Implement `VerbPhrases::perform_called_surgery(p: &mut ParseNode) -> bool` (matching lines 682-696):
  1. Check if the node is a `CALLED_NT` with a `RELATIONSHIP_NT` child.
  2. Swap the node types to fix the ordering.
- [ ] Add unit tests that:
  - "of surgery" splits "north of the house" into `X_OF_Y_NT`.
  - "location surgery" restructures "X is on Y and under Z".
  - "called surgery" fixes "north of a room called the Hot and Cold Room".

### 8. Integration tests

- [ ] Test the full seek pipeline end-to-end:
  - Register a copular verb "to be" with forms for "is", "are", "was", "were".
  - Register a transitive verb "to carry" with forms for "carries", "carried", "carrying".
  - Test that `VerbPhrases::seek` correctly parses "The cat is on the mat".
  - Test that `VerbPhrases::seek` correctly parses "Peter carries the flash cards".
  - Test that `VerbPhrases::seek` correctly parses "Carry the box" (imperative).
  - Test that `VerbPhrases::seek` correctly handles negated sentences.
  - Test that `VerbPhrases::seek` correctly handles bracketed text.
- [ ] Test that the seek function integrates with the existing Preform matching engine and internal NT dispatch.

## Success criteria

- [ ] `<nonimperative-verb>` internal NT correctly matches known verb usages and returns the verb usage reference.
- [ ] `<negated-noncopular-verb-present>` internal NT correctly matches negated verb phrases and returns the end position.
- [ ] The `Annotation` enum has `VerbalCertainty`, `SentenceIsExistential`, and `LinguisticErrorHere` variants.
- [ ] `ParseNode` has methods for setting/getting verb usage, prepositions, and special meaning references.
- [ ] The viability map calculation correctly scores words (0 for non-verbs, 1 for verbs outside brackets, 2 for verbs inside brackets, 3 for negated non-copular verbs).
- [ ] The seek loop correctly finds the primary verb in simple sentences ("The cat is on the mat", "Peter carries the flash cards").
- [ ] The seek loop correctly handles imperative sentences ("Carry the box").
- [ ] The seek loop correctly handles negated sentences ("The cat does not carry the box").
- [ ] Form structure checking correctly rejects SVO verbs at position 0 and VO verbs not at position 0.
- [ ] Preposition checking correctly verifies required prepositions are present.
- [ ] `VerbPhrases::accept` correctly iterates verb senses and falls back to the regular meaning.
- [ ] `VerbPhrases::default_verb` builds correct sentence diagrams with subject and object children.
- [ ] Corrective surgery correctly performs "of surgery", "location surgery", and "called surgery".
- [ ] All existing unit tests still pass after the changes.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- **Full `<sentence>` internal NT**: The `<sentence>` nonterminal calls `VerbPhrases::seek` and then `VerbPhrases::corrective_surgery`. We implement seek and corrective_surgery as Rust functions; wiring them into the Preform `<sentence>` internal NT is deferred to a later plan.
- **Special meaning functions**: The `ACCEPT_SMFT` task and the actual special meaning implementations (e.g., "to mean", "to be called") are deferred. For now, `VerbPhrases::accept` will fall through to `default_verb` for all senses.
- **Time period / occurrence detection**: The `detect_occurrences` parameter and `Occurrence::parse` are stubbed. Full occurrence detection (e.g., "for the third time") is deferred.
- **Existential sentence recursion**: The recursive call to `VerbPhrases::seek` for existential sentences ("There is a man who is in the Dining Room") is deferred. The `existential_OP_edge` parameter is accepted but not used for recursion.
- **Full verb conjugation system**: The simplified conjugation from PLAN-11 is sufficient for testing. The full `Conjugation::conjugate` system with arbitrary verb conjugation tables is deferred.
- **Noun phrase parsing at NP3/NP4 levels**: `default_verb` uses `<np-as-subject>` and `<np-as-object>` which may need NP3/NP4 parsing for complex cases. For now, NP1/NP2 parsing is sufficient.
- **Preform grammar optimization**: NT incidence bits, word range extremes, and other Preform optimizations are deferred.
- **The `==>` compositor function syntax**: Compositors are implemented as Rust functions, not parsed from the grammar.
