# PLAN-11 Implementation Context

## Plan
Read `plans/PLAN-11.md` for the full plan with all tasks and success criteria.

## Current Codebase State

### Project structure
- Workspace root: `/home/zicklag/git/zicklag/conform7/`
- Two crates: `conform7-syntax` and `conform7-inter`
- All work happens in `conform7-syntax` for this plan

### Key files
- `crates/conform7-syntax/src/linguistics.rs` — Article system, Diagrams, NounPhrases, parse_noun_phrase
- `crates/conform7-syntax/src/node_type.rs` — NodeType enum with linguistics variants
- `crates/conform7-syntax/src/parse_node.rs` — ParseNode, Annotation enum
- `crates/conform7-syntax/src/preform.rs` — Preform matching engine, InternalRegistry, InternalNonterminal trait
- `crates/conform7-syntax/src/preform_internal.rs` — Basic and article internal NTs
- `crates/conform7-syntax/src/lib.rs` — Module exports

### Current patterns
- Each module is a single `.rs` file in `src/`
- Tests are `#[cfg(test)] mod tests { ... }` at the bottom of each module
- Public API is re-exported from `lib.rs`
- C .w files are referenced in doc comments

### Build & test
```bash
cd /home/zicklag/git/zicklag/conform7
cargo test  # 292 tests currently pass
cargo clippy --all-targets
```

## Implementation Order (suggested)

1. Add `WordAssemblage` to `linguistics.rs` (or a new module)
2. Add `linguistic_constants` module with `Lcon` type
3. Add `stock_control` module with `Stock`, `GrammaticalCategory`, `LinguisticStockItem`, `GrammaticalUsage`
4. Add certainty level constants and `<certainty>` internal NT
5. Add verb conjugation (simplified for English)
6. Add verb data structures (`Verb`, `VerbForm`, `VerbSense`) and `Verbs` registry
7. Add verb meaning types and `VerbMeanings` creation functions
8. Add verb usage types and `VerbUsages` creation functions
9. Add preposition type and `Prepositions` creation functions
10. Add special meaning holder type and `SpecialMeanings` creation functions
11. Wire up verb system internal NTs in `InternalRegistry::linguistics()`
12. Add integration tests
