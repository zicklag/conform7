# Plan 13: Wire VerbPhrases::seek into the `<sentence>` Internal NT
**Status**: Complete
**Target**: 1-2 days

## Goal

Wire the fully implemented `VerbPhrases::seek` algorithm into the Preform matching engine by creating the `<sentence>` and `<sentence-without-occurrences>` internal nonterminals. This is the bridge that turns the verb-finding algorithm into a callable grammar nonterminal — the final step before full sentence parsing can produce structured `VERB_NT` diagrams from any assertion sentence.

This is the smallest next step after PLAN-12 because:

1. **VerbPhrases::seek is complete.** PLAN-12 implemented the full `VerbPhrases::seek` algorithm: viability map calculation, verb usage seek loop, form structure checking, preposition checking, diagram building, `VerbPhrases::accept`, `VerbPhrases::default_verb`, and corrective surgery. All of this is tested and working in `verb_phrases.rs`.

2. **The `<sentence>` internal NT is the consumer.** In the C reference, `<sentence>` is declared as an `internal` nonterminal that calls `VerbPhrases::seek(W, X, XP, 0, TRUE)` and then applies corrective surgery. The Rust equivalent needs to be a struct implementing `InternalNonterminal` that delegates to `VerbPhrases::seek`.

3. **The internal NT dispatch system is ready.** PLAN-9 added internal NT dispatch to the Preform matching engine. PLAN-10/11/12 added article, certainty, and verb-related internal NTs. The `<sentence>` internal NT follows the same pattern.

4. **Independently testable.** We can test the `<sentence>` internal NT with synthetic sentences and known verb registries, verifying that it produces correct `VERB_NT` diagrams. We can also test against the real `Syntax.preform` grammar to verify the dispatch works end-to-end.

5. **Prerequisite for the `<sentence>` nonterminal in the real grammar.** The real `Syntax.preform` declares `<sentence> internal` (line 2680) and `<sentence-without-occurrences> internal` (line 2682). Once the Rust implementation is registered, the real grammar's `<sentence>` nonterminal will dispatch to our code.

## Background

### C reference architecture

The `<sentence>` internal NT is defined in `services/linguistics-module/Chapter 4/Verb Phrases.w` (lines 40-53):

```c
<sentence> internal {
    int rv = VerbPhrases::seek(W, X, XP, 0, TRUE);
    VerbPhrases::corrective_surgery(*XP);
    @<Trace diagram@>;
    return rv;
}

<sentence-without-occurrences> internal {
    int rv = VerbPhrases::seek(W, X, XP, 0, FALSE);
    VerbPhrases::corrective_surgery(*XP);
    @<Trace diagram@>;
    return rv;
}
```

Key differences from the C signature:
- C: `VerbPhrases::seek(W, X, XP, existential_OP_edge, detect_occurrences)` — takes wording `W`, word range `X`, parse node pointer `XP`, existential edge, and occurrence flag.
- Rust: `VerbPhrases::seek(wording, ctx, registry, detect_occurrences) -> Option<ParseNode>` — takes wording, context, registry, and occurrence flag; returns `Option<ParseNode>` directly.

The Rust `VerbPhrases::seek` already calls `corrective_surgery` internally (line 201-203 of `verb_phrases.rs`), so the `<sentence>` internal NT just needs to call `seek` and return the result.

### Current Rust state

- `crates/conform7-syntax/src/verb_phrases.rs` — Full `VerbPhrases::seek` implementation with viability map, seek loop, accept, default_verb, and corrective surgery. All unit tests pass.
- `crates/conform7-syntax/src/preform_internal.rs` — Internal NT implementations for `<if-start-of-paragraph>`, `<if-not-cap>`, `<preform-nonterminal>`, `<article>`, `<definite-article>`, `<indefinite-article>`, `<certainty>`, `<nonimperative-verb>`, `<negated-noncopular-verb-present>`, `<pre-verb-rc-marker>`, `<pre-verb-certainty>`, `<post-verb-certainty>`.
- `crates/conform7-syntax/src/preform.rs` — `InternalPayload` enum (None, Integer, Nonterminal, Article), `InternalResult` struct, `InternalNonterminal` trait, `InternalRegistry`, `match_nonterminal_impl` function.
- `crates/conform7-syntax/src/parse_node.rs` — `Annotation` enum with `VerbalCertainty`, `SentenceIsExistential`, `LinguisticErrorHere`, `VerbUsage`, `PrepositionRef`, `SecondPrepositionRef` variants. `ParseNode` struct with verb-related methods.
- `crates/conform7-syntax/src/linguistics.rs` — `Diagrams` constructors, `NounPhrases` module, article system.

### What's missing

1. **`ParseNode` variant in `InternalPayload`**: The `<sentence>` internal NT needs to return a parse node. Currently `InternalPayload` only has `None`, `Integer`, `Nonterminal`, and `Article` variants. We need to add a `ParseNode(Box<ParseNode>)` variant so the matching engine can carry the sentence diagram.

2. **`<sentence>` internal NT implementation**: A `SentenceInternal` struct implementing `InternalNonterminal` that calls `VerbPhrases::seek` and returns the parse node.

3. **`<sentence-without-occurrences>` internal NT implementation**: Same as `<sentence>` but with `detect_occurrences = false`.

4. **Registration in `InternalRegistry::linguistics()`**: Both internal NTs need to be registered under their Preform names (`"sentence"` and `"sentence-without-occurrences"`).

5. **Integration with the real `Syntax.preform` grammar**: The real grammar declares `<sentence> internal` and `<sentence-without-occurrences> internal`. Once registered, these will dispatch to our Rust implementations.

### Key C source files

- `services/linguistics-module/Chapter 4/Verb Phrases.w` — `<sentence>` and `<sentence-without-occurrences>` internal NT definitions (lines 40-53), `VerbPhrases::seek` (lines 102-119), `VerbPhrases::corrective_surgery` (lines 582-601).
- `services/words-module/Chapter 4/Nonterminals.w` — `INTERNAL_NONTERMINAL` macro and `internal_definition` function pointer.
- `services/words-module/Chapter 4/Preform.w` — how `Preform::parse_nt_against_word_range` dispatches to internal NTs.
- `inform7/Internal/Languages/English/Syntax.preform` — the real Preform grammar declaring `<sentence> internal` (line 2680) and `<sentence-without-occurrences> internal` (line 2682).

## Tasks

### 1. Add `ParseNode` variant to `InternalPayload`

The `<sentence>` internal NT needs to return a parse node. Currently `InternalPayload` only supports simple payloads. We need to add a `ParseNode` variant.

- [ ] Add a `ParseNode(Box<ParseNode>)` variant to the `InternalPayload` enum in `crates/conform7-syntax/src/preform.rs`:
  ```rust
  /// A parse node (e.g., from the `<sentence>` internal NT).
  ParseNode(Box<ParseNode>),
  ```
- [ ] Update any pattern matches on `InternalPayload` that need to handle the new variant (e.g., in `Display` impls, test assertions, etc.).
- [ ] Verify that the `InternalPayload` enum's `Clone`, `Debug`, `PartialEq`, `Eq` derives still work (the `ParseNode` type already derives these).

### 2. Create `<sentence>` internal NT

- [ ] Add a `SentenceInternal` struct to `crates/conform7-syntax/src/preform_internal.rs`:
  ```rust
  /// Internal nonterminal that parses a full sentence.
  ///
  /// Calls `VerbPhrases::seek` to find the primary verb, identify subject
  /// and object phrases, and build a `VERB_NT` sentence diagram.
  ///
  /// # References
  ///
  /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
  ///   `<sentence> internal` (lines 40-46).
  #[derive(Clone, Debug)]
  pub struct SentenceInternal;
  ```
- [ ] Implement `InternalNonterminal` for `SentenceInternal`:
  ```rust
  impl InternalNonterminal for SentenceInternal {
      fn match_nonterminal(
          &self,
          ctx: &PreformContext,
          wording: Wording,
      ) -> Option<InternalResult> {
          // Call VerbPhrases::seek with detect_occurrences = true.
          let node = VerbPhrases::seek(wording, ctx, ctx.verbs_registry?, true)?;
          Some(InternalResult {
              payload: InternalPayload::ParseNode(Box::new(node)),
          })
      }
  }
  ```
  Note: The `VerbPhrases::seek` function already calls `corrective_surgery` internally, so we don't need to call it again here.

### 3. Create `<sentence-without-occurrences>` internal NT

- [ ] Add a `SentenceWithoutOccurrencesInternal` struct to `crates/conform7-syntax/src/preform_internal.rs`:
  ```rust
  /// Internal nonterminal that parses a full sentence without detecting
  /// occurrences (adverbs like "for the third time").
  ///
  /// # References
  ///
  /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
  ///   `<sentence-without-occurrences> internal` (lines 48-53).
  #[derive(Clone, Debug)]
  pub struct SentenceWithoutOccurrencesInternal;
  ```
- [ ] Implement `InternalNonterminal` for `SentenceWithoutOccurrencesInternal`:
  ```rust
  impl InternalNonterminal for SentenceWithoutOccurrencesInternal {
      fn match_nonterminal(
          &self,
          ctx: &PreformContext,
          wording: Wording,
      ) -> Option<InternalResult> {
          // Call VerbPhrases::seek with detect_occurrences = false.
          let node = VerbPhrases::seek(wording, ctx, ctx.verbs_registry?, false)?;
          Some(InternalResult {
              payload: InternalPayload::ParseNode(Box::new(node)),
          })
      }
  }
  ```

### 4. Register both internal NTs in `InternalRegistry::linguistics()`

- [ ] Add registration of `"sentence"` and `"sentence-without-occurrences"` in the `InternalRegistry::linguistics()` method in `crates/conform7-syntax/src/preform_internal.rs`:
  ```rust
  registry.register("sentence", Box::new(SentenceInternal));
  registry.register("sentence-without-occurrences", Box::new(SentenceWithoutOccurrencesInternal));
  ```

### 5. Add unit tests

- [ ] Test that `<sentence>` matches a simple copular sentence:
  - Register a copular verb "to be" with usage "is".
  - Create wording "The cat is on the mat".
  - Call `match_nonterminal_impl` with the `"sentence"` internal NT.
  - Verify it returns a `Match` with an `InternalPayload::ParseNode` containing a `VERB_NT` node.
  - Verify the `VERB_NT` node has the correct verb wording ("is").
  - Verify the `VERB_NT` node has subject and object children.

- [ ] Test that `<sentence>` matches a simple non-copular sentence:
  - Register a non-copular verb "to carry" with usage "carries".
  - Create wording "Peter carries the flash cards".
  - Verify the result is a `VERB_NT` with verb wording "carries".

- [ ] Test that `<sentence>` matches an imperative sentence:
  - Register a VO verb "to carry" with usage "carry".
  - Create wording "Carry the box".
  - Verify the result is a `VERB_NT` with verb wording "carry".

- [ ] Test that `<sentence>` fails on a sentence with no verb:
  - Create wording "The cat".
  - Verify `match_nonterminal_impl` returns `None`.

- [ ] Test that `<sentence-without-occurrences>` works the same as `<sentence>`:
  - Same test as the copular sentence but using `"sentence-without-occurrences"`.
  - Verify it produces the same result.

- [ ] Test that the `InternalPayload::ParseNode` variant round-trips correctly:
  - Create a `ParseNode` and wrap it in `InternalPayload::ParseNode(Box::new(node))`.
  - Extract it and verify the node type and wording are preserved.

### 6. Integration test with real Syntax.preform grammar

- [ ] Load the real `Syntax.preform` grammar from `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform`.
- [ ] Verify it declares `<sentence> internal` and `<sentence-without-occurrences> internal`.
- [ ] Create a `PreformContext` with a test verb registry and a simple sentence wording.
- [ ] Call `match_nonterminal_impl` with the real grammar and the `"sentence"` nonterminal.
- [ ] Verify it dispatches to our Rust implementation and returns a `VERB_NT` node.

## Success criteria

- [ ] `InternalPayload` has a `ParseNode(Box<ParseNode>)` variant that carries a parse node.
- [ ] `<sentence>` internal NT correctly calls `VerbPhrases::seek` and returns the resulting `VERB_NT` parse node.
- [ ] `<sentence-without-occurrences>` internal NT correctly calls `VerbPhrases::seek` with `detect_occurrences = false`.
- [ ] Both internal NTs are registered in `InternalRegistry::linguistics()` under their Preform names.
- [ ] `<sentence>` matches simple copular sentences ("The cat is on the mat") and returns a `VERB_NT` node.
- [ ] `<sentence>` matches simple non-copular sentences ("Peter carries the flash cards") and returns a `VERB_NT` node.
- [ ] `<sentence>` matches imperative sentences ("Carry the box") and returns a `VERB_NT` node.
- [ ] `<sentence>` fails on sentences with no verb ("The cat").
- [ ] `<sentence-without-occurrences>` produces the same results as `<sentence>` for simple sentences.
- [ ] The real `Syntax.preform` grammar's `<sentence> internal` declaration dispatches to our Rust implementation.
- [ ] All existing unit tests still pass after the changes.
- [ ] `cargo clippy --all-targets` is clean.

## Out of scope

- **Full sentence parsing with NP3/NP4 noun phrases**: The `<sentence>` internal NT uses `VerbPhrases::default_verb` which parses subject and object as `<np-unparsed>` (NP1 level). Full NP3/NP4 parsing (list-divided noun phrases, relative clauses, callings, kind phrases) is deferred.
- **Existential sentence recursion**: The `existential_OP_edge` parameter in `VerbPhrases::seek_inner` is accepted but not used for recursion. Full existential sentence handling ("There is a man who is in the Dining Room") is deferred.
- **Time period / occurrence detection**: The `detect_occurrences` parameter is passed through but occurrence detection is not yet implemented. Full occurrence detection (e.g., "for the third time") is deferred.
- **Special meaning functions**: The `ACCEPT_SMFT` task and actual special meaning implementations are deferred. `VerbPhrases::accept` falls through to `default_verb` for all senses.
- **Corrective surgery implementation**: The three corrective surgery functions (`perform_of_surgery`, `perform_location_surgery`, `perform_called_surgery`) are stubs that always return `false`. Full implementation is deferred to a later plan.
- **Preform grammar optimization**: NT incidence bits, word range extremes, and other Preform optimizations are deferred.
- **The `==>` compositor function syntax**: Compositors are implemented as Rust functions, not parsed from the grammar.
- **Full verb conjugation system**: The simplified conjugation from PLAN-11 is sufficient for testing. The full `Conjugation::conjugate` system with arbitrary verb conjugation tables is deferred.
