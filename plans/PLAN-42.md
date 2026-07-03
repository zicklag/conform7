# Plan 42: Adjectives by Phrase — Foundation

**Status**: In progress
**Target**: 1 day

## Goal

Implement the foundation of the Adjectives by Phrase system — the `adjective_meaning_family` data structure and the `AdjectivesByPhrase` module with `start()` and `claim_definition()`. This is the next module in the assertions-module startup sequence (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 34), immediately after `ImperativeDefinitionFamilies::create()` (PLAN-41, Complete).

## Background

### C reference

- `inform7/assertions-module/Chapter 8/Adjectives by Phrase.w` — `AdjectivesByPhrase` module
- `inform7/assertions-module/Chapter 5/Adjectival Definition Family.w` — `AdjectivalDefinitionFamily`

### Current Rust state

- Assertions module with `ImperativeDefinitionFamilies` foundation
- Knowledge module with adjectives, adjective meanings, adjective domains
- 1374 tests pass, clippy clean

## Out of Scope

- **Full adjective meaning system**: The complete `AdjectiveMeanings` system is deferred.
- **Run-time adjective compilation**: `RTAdjectives::set_schemas_for_I7_phrase` is deferred.
- **Preform grammar / Salsa integration**.
