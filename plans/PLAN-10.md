# Plan 10: Linguistics Module Foundation — Sentence Diagram Node Types and Noun Phrase Parsing
**Status**: Complete
**Target**: 3-5 days

## Goal

Implement the foundation of the linguistics module: the parse node types for sentence diagrams (VERB_NT, UNPARSED_NOUN_NT, COMMON_NOUN_NT, PROPER_NOUN_NT, etc.), the article system (definite/indefinite articles), and the basic noun phrase parsing nonterminals (`<np-unparsed>`, `<np-articled>`). This is the bridge between the Preform matching engine and the world model — the first step that turns raw source text into structured parse trees with linguistic annotations.

This is the smallest next step after PLAN-9 because:

1. **The Preform matching engine is ready.** PLAN-9 added internal nonterminal dispatch. The engine can now match any nonterminal in `Syntax.preform`, including `<np-unparsed>` and `<np-articled>`. What's missing is the code that *interprets* those matches into parse nodes.

2. **Noun phrases are the building blocks of all sentence parsing.** Every sentence in Inform 7 has a subject and an object, both of which are noun phrases. Implementing noun phrase parsing first means verb phrase parsing and sentence parsing can be built on top.

3. **The article system is the simplest linguistic construct.** Articles ("the", "a/an") have a small, fixed vocabulary and simple grammar. Implementing them first validates the architecture for the more complex verb and noun systems.

4. **Independently testable.** We can test noun phrase parsing with synthetic grammars and with the real `Syntax.preform` grammar, without needing any world model, vocabulary tables, or verb definitions.

## Background

### C reference architecture

The linguistics module in the C reference (`services/linguistics-module/`) defines:

- **Sentence diagram node types** (`Chapter 1/Diagrams.w`): VERB_NT, UNPARSED_NOUN_NT, COMMON_NOUN_NT, PROPER_NOUN_NT, PRONOUN_NT, DEFECTIVE_NOUN_NT, RELATIONSHIP_NT, CALLED_NT, WITH_NT, AND_NT, KIND_NT, PROPERTY_LIST_NT, X_OF_Y_NT. These are the building blocks of the parse tree that represents a parsed sentence.

- **Articles** (`Chapter 2/Articles.w`): The `article` and `article_usage` types, the `<article>`, `<definite-article>`, `<indefinite-article>` internal nonterminals, and the article tables (`<definite-article-table>`, `<indefinite-article-table>`) that define the actual article words.

- **Noun phrases** (`Chapter 4/Noun Phrases.w`): Four levels of noun phrase parsing:
  - NP1 (`<np-unparsed>`): raw, unannotated text → `UNPARSED_NOUN_NT`
  - NP2 (`<np-articled>`): article + unparsed text → annotated `UNPARSED_NOUN_NT`
  - NP3 (`<np-articled-list>`): list-divided noun phrases → `AND_NT`
  - NP4 (`<np-nonrelative>`): full noun phrases with relative clauses, callings, etc.

- **Linguistic stock** (`Chapter 1/Stock Control.w`): The `linguistic_stock_item`, `grammatical_usage`, and `small_word_set` types that provide fast lookup of linguistic items by vocabulary entry.

### Key C source files

- `services/linguistics-module/Chapter 1/Diagrams.w` — sentence diagram node types and constructors
- `services/linguistics-module/Chapter 2/Articles.w` — article types, internal NTs, and tables
- `services/linguistics-module/Chapter 4/Noun Phrases.w` — noun phrase parsing at 4 levels
- `services/linguistics-module/Chapter 1/Stock Control.w` — linguistic stock and small word sets
- `services/words-module/Chapter 4/Basic Nonterminals.w` — basic internal NTs (already ported in PLAN-9)
- `inform7/Internal/Languages/English/Syntax.preform` — the real Preform grammar

### Current Rust state

- `crates/conform7-syntax/src/preform.rs` — Preform grammar parser and matching engine with internal NT dispatch
- `crates/conform7-syntax/src/preform_internal.rs` — three basic internal NTs (`<if-start-of-paragraph>`, `<if-not-cap>`, `<preform-nonterminal>`)
- `crates/conform7-syntax/src/node_type.rs` — enumerated node types (currently only base syntax types and Inform-only structural types; no linguistics types)
- `crates/conform7-syntax/src/parse_node.rs` — `ParseNode` struct with children, alternatives, and annotations

### What's missing

1. **Linguistics node types**: VERB_NT, UNPARSED_NOUN_NT, COMMON_NOUN_NT, PROPER_NOUN_NT, PRONOUN_NT, DEFECTIVE_NOUN_NT, RELATIONSHIP_NT, CALLED_NT, WITH_NT, AND_NT, KIND_NT, PROPERTY_LIST_NT, X_OF_Y_NT are not yet in the `NodeType` enum.

2. **Article system**: No `article`/`article_usage` types, no `<article>`/`<definite-article>`/`<indefinite-article>` internal NTs, no article tables.

3. **Noun phrase parsing**: No function that calls the matching engine for `<np-unparsed>` or `<np-articled>` and creates parse nodes from the results.

4. **Linguistic stock**: No `linguistic_stock_item`, `grammatical_usage`, or `small_word_set` types.

## Tasks

### 1. Add linguistics node types to `NodeType`

- [ ] Add the sentence diagram node types from `services/linguistics-module/Chapter 1/Diagrams.w`:
  - `Verb` (VERB_NT)
  - `UnparsedNoun` (UNPARSED_NOUN_NT)
  - `Pronoun` (PRONOUN_NT)
  - `DefectiveNoun` (DEFECTIVE_NOUN_NT)
  - `CommonNoun` (COMMON_NOUN_NT)
  - `ProperNoun` (PROPER_NOUN_NT)
  - `Relationship` (RELATIONSHIP_NT)
  - `Called` (CALLED_NT)
  - `With` (WITH_NT)
  - `And` (AND_NT)
  - `Kind` (KIND_NT)
  - `PropertyList` (PROPERTY_LIST_NT)
  - `XOfY` (X_OF_Y_NT)
- [ ] Add metadata entries for each new type (min/max children, category, flags) matching the C definitions in `Diagrams.w`:
  - VERB_NT: arity 0, L3_NCAT
  - RELATIONSHIP_NT: arity 0-2, L3_NCAT, ASSERT_NFLAG
  - CALLED_NT: arity 2, L3_NCAT
  - WITH_NT: arity 2, L3_NCAT, ASSERT_NFLAG
  - AND_NT: arity 2, L3_NCAT, ASSERT_NFLAG
  - KIND_NT: arity 0-1, L3_NCAT, ASSERT_NFLAG
  - UNPARSED_NOUN_NT: arity 0, L3_NCAT, ASSERT_NFLAG
  - PRONOUN_NT: arity 0, L3_NCAT, ASSERT_NFLAG
  - DEFECTIVE_NOUN_NT: arity 0, L3_NCAT, ASSERT_NFLAG
  - PROPER_NOUN_NT: arity 0, L3_NCAT, ASSERT_NFLAG
  - COMMON_NOUN_NT: arity 0-INFTY, L3_NCAT, ASSERT_NFLAG
  - PROPERTY_LIST_NT: arity 0-INFTY, L3_NCAT, ASSERT_NFLAG
  - X_OF_Y_NT: arity 2, L3_NCAT, ASSERT_NFLAG
- [ ] Add the `linguistic_error_here_ANNOT` annotation type and the annotation permission entries matching `Diagrams.w` (verb_ANNOT, noun_ANNOT, article_ANNOT, etc.) — these can be stub annotations for now, with the full annotation system to be implemented when the verb/noun systems are ported.

### 2. Implement the article system

- [ ] Add `article` and `article_usage` types to a new `linguistics` module in `conform7-syntax`:
  ```rust
  pub struct Article {
      pub name: String,
  }

  pub struct ArticleUsage {
      pub article: Article,
      pub word: String,
  }
  ```
- [ ] Implement the `<article>` internal nonterminal: look up a single word in the article small word set and return the matching `article_usage`.
- [ ] Implement the `<definite-article>` internal nonterminal: look up a single word in the definite-article small word set.
- [ ] Implement the `<indefinite-article>` internal nonterminal: look up a single word in the indefinite-article small word set.
- [ ] Implement a simplified `SmallWordSet` type (a `HashMap<String, T>` wrapper) that provides the fast word-lookup mechanism used by articles, verbs, and nouns in the C reference.
- [ ] Populate the article small word sets from the English article tables:
  - Definite: "the"
  - Indefinite: "a", "an", "some"
- [ ] Register the three article internal NTs in `InternalRegistry::linguistics()` (a new constructor alongside the existing `basic()`).

### 3. Implement basic noun phrase parsing (NP1 and NP2)

- [ ] Create a `linguistics` module in `conform7-syntax` with:
  - `mod linguistics;` in `lib.rs`
  - `src/linguistics.rs` (or `src/linguistics/mod.rs` with sub-modules)
- [ ] Implement the `Diagrams` constructor functions from `services/linguistics-module/Chapter 1/Diagrams.w`:
  - `Diagrams::new_unparsed_noun(wording) -> ParseNode` — creates an `UNPARSED_NOUN_NT` node
  - `Diagrams::new_proper_noun(wording) -> ParseNode` — creates a `PROPER_NOUN_NT` node
  - `Diagrams::new_common_noun(wording) -> ParseNode` — creates a `COMMON_NOUN_NT` node
  - `Diagrams::new_pronoun(wording) -> ParseNode` — creates a `PRONOUN_NT` node
  - `Diagrams::new_defective(wording) -> ParseNode` — creates a `DEFECTIVE_NOUN_NT` node
  - `Diagrams::new_kind(wording, child) -> ParseNode` — creates a `KIND_NT` node
  - `Diagrams::new_relationship(wording, child) -> ParseNode` — creates a `RELATIONSHIP_NT` node
  - `Diagrams::new_called(wording, child1, child2) -> ParseNode` — creates a `CALLED_NT` node
  - `Diagrams::new_with(wording, child1, child2) -> ParseNode` — creates a `WITH_NT` node
  - `Diagrams::new_and(wording, child1, child2) -> ParseNode` — creates an `AND_NT` node
  - `Diagrams::new_property_list(wording) -> ParseNode` — creates a `PROPERTY_LIST_NT` node
  - `Diagrams::new_x_of_y(wording, child1, child2) -> ParseNode` — creates an `X_OF_Y_NT` node
- [ ] Implement the `NounPhrases` module from `services/linguistics-module/Chapter 4/Noun Phrases.w`:
  - `NounPhrases::parse_np_unparsed(wording, grammar, registry) -> Option<ParseNode>` — matches `<np-unparsed>` against the wording and creates an `UNPARSED_NOUN_NT` node
  - `NounPhrases::parse_np_articled(wording, grammar, registry) -> Option<ParseNode>` — matches `<np-articled>` against the wording, extracts the article annotation, and creates an annotated `UNPARSED_NOUN_NT` node
  - `NounPhrases::add_article(node, article_usage) -> ParseNode` — adds an article annotation to a parse node (matching `NounPhrases::add_art` in the C reference)
- [ ] Implement the `parse_noun_phrase` public API function that:
  1. Tokenizes the input text using the existing lexer
  2. Creates a `PreformContext` from the token stream
  3. Calls the matching engine for `<np-unparsed>` or `<np-articled>`
  4. Creates the appropriate `ParseNode` from the match
  5. Returns the parse node

### 4. Synthetic unit and integration tests

- [ ] Test that the new `NodeType` variants have correct metadata (min/max children, category, flags).
- [ ] Test that `Diagrams::new_unparsed_noun` creates a node with the correct type and wording.
- [ ] Test that `Diagrams::new_and` creates a node with two children.
- [ ] Test that the `<article>` internal NT matches "the", "a", "an", "some" and fails on "xyzzy".
- [ ] Test that the `<definite-article>` internal NT matches "the" and fails on "a".
- [ ] Test that the `<indefinite-article>` internal NT matches "a", "an", "some" and fails on "the".
- [ ] Test that `parse_noun_phrase` with `<np-unparsed>` matches any text and returns an `UNPARSED_NOUN_NT` node.
- [ ] Test that `parse_noun_phrase` with `<np-articled>` matches "the room" and returns an `UNPARSED_NOUN_NT` node with an article annotation.
- [ ] Test that `parse_noun_phrase` with `<np-articled>` matches "a container" and returns an `UNPARSED_NOUN_NT` node with an indefinite article annotation.
- [ ] Test that `parse_noun_phrase` with `<np-articled>` fails on "xyzzy" (no article) but falls through to `<np-unparsed>`.

### 5. Test against the real grammar oracle

- [ ] Load `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform` and verify it declares `<np-unparsed>`, `<np-articled>`, `<article>`, `<definite-article>`, `<indefinite-article>`, and the article table nonterminals.
- [ ] Match `<np-unparsed>` against a real wording using the real `Syntax.preform` grammar and verify it returns a match.
- [ ] Match `<np-articled>` against "the room" using the real `Syntax.preform` grammar and verify it returns a match with the article consumed.

## Success criteria

- [ ] All new `NodeType` variants (VERB_NT, UNPARSED_NOUN_NT, COMMON_NOUN_NT, PROPER_NOUN_NT, PRONOUN_NT, DEFECTIVE_NOUN_NT, RELATIONSHIP_NT, CALLED_NT, WITH_NT, AND_NT, KIND_NT, PROPERTY_LIST_NT, X_OF_Y_NT) are defined with correct metadata.
- [ ] The `<article>`, `<definite-article>`, and `<indefinite-article>` internal NTs correctly match English articles and return `article_usage` values.
- [ ] `parse_noun_phrase` with `<np-unparsed>` matches any non-empty text and returns an `UNPARSED_NOUN_NT` node with the correct wording.
- [ ] `parse_noun_phrase` with `<np-articled>` matches "the room" and "a container" and returns annotated nodes.
- [ ] All article internal NTs are registered in `InternalRegistry::linguistics()` and dispatch correctly through the matching engine.
- [ ] All existing `preform.rs` and `preform_internal.rs` unit tests still pass after the API changes.
- [ ] New tests are explicitly grounded in `services/linguistics-module/Chapter 1/Diagrams.w`, `services/linguistics-module/Chapter 2/Articles.w`, and `services/linguistics-module/Chapter 4/Noun Phrases.w`.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- Verb phrase parsing (`<verb-phrase>`, `VerbPhrases::seek`). This requires the full verb system (verb types, verb forms, verb usages, verb meanings) and is a larger piece of work that belongs in a later plan.
- Full sentence parsing (`<sentence>` internal NT). This depends on both noun phrase and verb phrase parsing.
- NP3 and NP4 noun phrase levels (`<np-articled-list>`, `<np-nonrelative>`, `<np-relative-phrase-*>`, `<np-kind-phrase>`). These add list-divided noun phrases, relative clauses, callings, and kind phrases — all of which depend on the NP1/NP2 foundation.
- The full linguistic stock system (`linguistic_stock_item`, `grammatical_usage` with gender/number/case). The article system uses a simplified `SmallWordSet` for now.
- The `grammatical_category` and method system. Categories are implicit in the simplified implementation.
- The `lcon_ti` (linguistic constant) system. Article usages are simple structs without the full inflection system.
- Noun recognition (`Nouns::recognise`) — turning `UNPARSED_NOUN_NT` into `COMMON_NOUN_NT`/`PROPER_NOUN_NT` by looking up known nouns. This requires the noun lexicon.
- Preform grammar optimization (NT incidence bits, word range extremes).
- The `==>` compositor function syntax in the Preform grammar parser. Compositors are implemented as Rust functions, not parsed from the grammar.
