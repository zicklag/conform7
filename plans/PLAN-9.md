# Plan 9: Internal Nonterminal Dispatch for Preform

**Status**: Complete  
**Target**: 3–5 days

## Goal

Wire up the missing `internal` nonterminal dispatch layer in the Preform
matching engine so that `match_nonterminal` no longer silently returns `None`
for internal nonterminals. Implement the three simplest internal nonterminals
from the C reference (`<if-start-of-paragraph>`, `<if-not-cap>`,
`<preform-nonterminal>`) and exercise them against the real English
`Syntax.preform` grammar.

This is the smallest next step after PLAN-8. The obvious follow-on — parsing
a real assertion such as *"The Lab is a room."* with the internal `<sentence>`
nonterminal — is too large for one milestone: `<sentence>` lives in the
linguistics module and builds full verb/subject/object parse-node diagrams
(`VerbPhrases::seek` in `services/linguistics-module/Chapter 4/Verb Phrases.w`).
Dispatch plus the trivial position/name internals is independently testable
and unblocks every production in `Syntax.preform` that contains `internal`
markers, including `<dividing-sentence>` and `<structural-sentence>`.

## Background

- **How internal NTs are declared and bound.** In the C reference,
  `services/words-module/Chapter 4/Nonterminals.w` defines the
  `INTERNAL_NONTERMINAL(name, identifier, min, max)` macro. It sets
  `identifier->internal_definition = identifier##R` (the C function that
  implements the NT) and records length bounds for the optimizer. At run time,
  `Preform::parse_nt_against_word_range` in
  `services/words-module/Chapter 4/Preform.w` checks `nt->internal_definition`
  and, if it is set, calls that function with the wording `W` and pointers for
  the integer/pointer results `Q`/`QP`.

- **The three simple internals we will implement.**
  `services/words-module/Chapter 4/Basic Nonterminals.w` defines them:
  - `<preform-nonterminal> internal 1` matches a single source word that is the
    exact name of a declared nonterminal and returns the `nonterminal *`.
  - `<if-start-of-paragraph> internal 0` is a zero-width match when the first
    word of the wording is at the start of a paragraph (first source word or
    immediately after a paragraph break).
  - `<if-not-cap> internal 0` is a zero-width match when the first word is not
    unexpectedly upper-case.

- **Where they are used in the real grammar.**
  `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform` declares
  all three as `internal` and uses them in real productions, e.g.:
  ```text
  <dividing-sentence> ::=
      <if-start-of-paragraph> <heading> |
      <extension-end-marker-sentence>
  ```
  and
  ```text
  <np-articled> ::=
      ... |
      <if-not-cap> <indefinite-article> <np-unparsed> |
      <if-not-cap> <definite-article> <np-unparsed> |
      <np-unparsed>
  ```

- **Current Rust state.** `crates/conform7-syntax/src/preform.rs` exposes
  `match_nonterminal(grammar, name, word_text, wording) -> Option<Match>`.
  Today it explicitly returns `None` as soon as it sees `nt.internal`, because
  there is no registry of Rust implementations and no way to pass token/position
  context into a matcher.

## Tasks

### 1. Add the internal-NT machinery to `preform.rs`

- [x] Add `InternalPayload`, `InternalResult`, and the `InternalNonterminal` trait
- [x] Add `PreformContext<'a>` bundling the `Grammar`, the word-text slice,
      and the paragraph-start flag needed by the simple internals.
- [x] Add `InternalRegistry` mapping nonterminal name → `Box<dyn InternalNonterminal>`.
- [x] Change the public `match_nonterminal` entry point to:
      ```rust
      pub fn match_nonterminal(
          ctx: &PreformContext,
          registry: &InternalRegistry,
          name: &str,
          wording: Wording,
      ) -> Option<Match>
      ```
      and update `try_match_production` recursion to use the same shape.
- [x] Extend `Match` with an optional `internal: Option<InternalResult>` field
      so callers can retrieve the integer/pointer results produced by an
      internal NT.
- [x] Update existing doctests and unit tests in `preform.rs` to construct a
      `PreformContext` and an empty/default `InternalRegistry`.

### 2. Implement the three simple internal NTs

- [x] `<if-start-of-paragraph>`: zero-width success when `ctx.is_paragraph_start`
      is true, otherwise fail. The consumed `word_range` is empty
      (`start..start`).
- [x] `<if-not-cap>`: zero-width success when the first word does not begin
      with an upper-case letter. (This is a faithful enough stand-in for
      `Word::unexpectedly_upper_case` for the tested examples; document the
      simplification.)
- [x] `<preform-nonterminal>`: one-word match when the word looks like
      `<name>` and `name` exists in `ctx.grammar.nonterminals`. Return
      `InternalPayload::Nonterminal(name)`.
- [x] Provide `InternalRegistry::basic()` that registers these three NTs under
      the exact names used in `Syntax.preform`:
      `if-start-of-paragraph`, `if-not-cap`, `preform-nonterminal`.

### 3. Synthetic unit and integration tests

- [x] `<if-start-of-paragraph>` succeeds at the start of input and after a
      paragraph break, fails after a single newline or in mid-paragraph.
- [x] `<if-not-cap>` matches `apple` and `the`, fails on `Apple` and `The`.
- [x] `<preform-nonterminal>` matches `<foo>` when `<foo>` is declared,
      fails on `<foo>` when it is not declared, fails on `foo`.
- [x] A synthetic grammar containing
      ```text
      <if-start-of-paragraph> internal
      <heading> ::= chapter ... | section ...
      <dividing-sentence> ::= <if-start-of-paragraph> <heading>
      ```
      matches `"chapter 1 - X"` at paragraph start and fails without the
      paragraph-start flag.

### 4. Test against the real grammar oracle

- [x] Add a test that loads
      `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform`
      and confirms it declares the three internal NTs as internal and
      contains the `<dividing-sentence>` production shown above.
- [x] If the current 10.1 `Syntax.preform` cannot yet be parsed by
      `parse_preform_grammar`, fall back to the already-working
      `gitignore/inform/retrospective/6M62/Internal/Languages/English/Syntax.preform`
      fixture and fix the parser issues for the current file as part of this
      plan. The real `Syntax.preform` is the grammar oracle; the tests must
      run against it.

## Success criteria

- [x] `match_nonterminal` returns a successful `Match` for each of the three
      registered internal NTs instead of `None`.
- [x] Zero-width internals (`<if-start-of-paragraph>`, `<if-not-cap>`) consume
      no words and behave exactly as specified in
      `services/words-module/Chapter 4/Basic Nonterminals.w` for the tested
      cases.
- [x] `<preform-nonterminal>` consumes exactly one source word and returns
      the matching nonterminal's name.
- [x] A production that mixes an internal NT with a regular NT
      (`<dividing-sentence>`) matches/fails as the C grammar predicts.
- [x] All existing `preform.rs` unit tests still pass after the API change.
- [x] New tests are explicitly grounded in
      `services/words-module/Chapter 4/Basic Nonterminals.w` and
      `gitignore/inform/inform7/Internal/Languages/English/Syntax.preform`;
      the oracle is documented in the test comments.

## Out of scope

- Implementing the linguistics-module internal NTs (`<sentence>`,
  `<sentence-without-occurrences>`, `<np-as-subject>`, `<np-as-object>`,
  `<article>`, verb-phrase/noun-phrase internals). These build parse-node
  diagrams and require vocabulary/verb/article tables; they belong in later
  plans.
- Voracious internal NTs, `nt_extremes` length optimization, and NT incidence
  bitmap checks.
- Compositor functions / `==>` result extraction for regular NTs. Regular
  matches still return only a `match_number`.
- Converting matched productions into `SENTENCE_NT` parse nodes or wiring the
  output into the assertion pipeline.
