# PLAN-10 Review Context

## What was implemented
PLAN-10: Linguistics Module Foundation — Sentence Diagram Node Types and Noun Phrase Parsing.

## Files changed
- `crates/conform7-syntax/src/node_type.rs` — Added 13 linguistics node types (Verb, UnparsedNoun, Pronoun, DefectiveNoun, CommonNoun, ProperNoun, Relationship, Called, With, And, Kind, PropertyList, XOfY)
- `crates/conform7-syntax/src/parse_node.rs` — Added ArticleUsage annotation
- `crates/conform7-syntax/src/preform.rs` — Minor API changes for linguistics support
- `crates/conform7-syntax/src/preform_internal.rs` — Added 3 article internal NTs (article, definite-article, indefinite-article)
- `crates/conform7-syntax/src/linguistics.rs` — New module with Article, ArticleUsage, SmallWordSet, Diagrams, NounPhrases, parse_noun_phrase
- `crates/conform7-syntax/src/lib.rs` — Added linguistics module export

## Test results
- 287 tests pass (up from 244)
- 43 new linguistics tests
- 13 preform_internal tests
- `cargo clippy --all-targets` is clean

## Review criteria
1. Does the implementation match the plan's success criteria?
2. Are the C .w files properly referenced in comments?
3. Does the code follow existing patterns (heading.rs, structural.rs)?
4. Are there any missing edge cases or error handling?
5. Is the public API clean and minimal?
6. Are tests comprehensive and grounded in the C reference?
7. Any unnecessary abstractions or code that isn't pulling its weight?

## Files to review
Read these files to review the implementation:
- `crates/conform7-syntax/src/node_type.rs` — check new node types
- `crates/conform7-syntax/src/parse_node.rs` — check annotation additions
- `crates/conform7-syntax/src/preform_internal.rs` — check article NTs
- `crates/conform7-syntax/src/linguistics.rs` — check the new module
- `crates/conform7-syntax/src/lib.rs` — check exports
- `crates/conform7-syntax/src/preform.rs` — check any changes
