# PLAN-12 Implementation Context

## Plan
Read `plans/PLAN-12.md` for the full plan with all 8 task groups and success criteria.

## Current Codebase State

### Project structure
- Workspace root: `/home/zicklag/git/zicklag/conform7/`
- Two crates: `conform7-syntax` and `conform7-inter`
- All work happens in `conform7-syntax` for this plan

### Key files
- `crates/conform7-syntax/src/verbs.rs` — Verb, VerbForm, VerbSense, VerbMeaning, VerbUsage, VerbUsageTier, Preposition, SpecialMeaningHolder, Verbs registry
- `crates/conform7-syntax/src/verb_conjugation.rs` — VerbConjugation, VerbTabulation, Conjugation
- `crates/conform7-syntax/src/linguistic_constants.rs` — Lcon type and constants
- `crates/conform7-syntax/src/stock_control.rs` — Stock, GrammaticalCategory, LinguisticStockItem, GrammaticalUsage
- `crates/conform7-syntax/src/word_assemblage.rs` — WordAssemblage
- `crates/conform7-syntax/src/linguistics.rs` — Article system, Diagrams, NounPhrases, certainty
- `crates/conform7-syntax/src/preform_internal.rs` — Internal NTs (basic, article, verb stubs)
- `crates/conform7-syntax/src/preform.rs` — PreformContext, InternalRegistry, match_nonterminal_impl
- `crates/conform7-syntax/src/parse_node.rs` — ParseNode, Annotation enum
- `crates/conform7-syntax/src/node_type.rs` — NodeType enum
- `crates/conform7-syntax/src/lib.rs` — Module exports

### Current patterns
- Each module is a single `.rs` file in `src/`
- Tests are `#[cfg(test)] mod tests { ... }` at the bottom of each module
- Public API is re-exported from `lib.rs`
- C .w files are referenced in doc comments

### Build & test
```bash
cd /home/zicklag/git/zicklag/conform7
cargo test  # 397 tests currently pass
cargo clippy --all-targets
```

## Implementation Order (suggested)

1. Implement `<nonimperative-verb>` internal NT (replace stub)
2. Implement `<negated-noncopular-verb-present>` internal NT (replace stub)
3. Add verb-related annotations and ParseNode methods
4. Implement viability map calculation
5. Implement the core seek loop
6. Implement VerbPhrases::accept and VerbPhrases::default_verb
7. Implement corrective surgery
8. Integration tests
