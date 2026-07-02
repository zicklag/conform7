# Conform7 Project State (Updated)

## Completed (Plans 1-11)

### conform7-inter crate
- Inter IR read/write with round-trip fidelity

### conform7-syntax crate
- **Lexer** — state machine tokenizer
- **Sentence breaker** — FSM that splits tokens into sentences
- **ParseNode** — tree data model
- **NodeType** — enumerated node types with metadata
- **Heading/Structural AST** — sentence-to-AST bridges
- **Preform grammar parser** — parses Syntax.preform format
- **Preform matching engine** — backtracking matcher with internal NT dispatch
- **Linguistics module** (PLAN-10):
  - 13 linguistics NodeType variants
  - Article system (Article, ArticleUsage, SmallWordSet)
  - Three article internal NTs
  - Diagram constructor functions
  - Noun phrase parsing at NP1/NP2 levels
  - Public `parse_noun_phrase` API
- **Verb system** (PLAN-11):
  - WordAssemblage type
  - Lcon/linguistic constants
  - Stock control (GrammaticalCategory, LinguisticStockItem, Stock, GrammaticalUsage)
  - Certainty levels and `<certainty>` internal NT
  - Verb conjugation (simplified for English: "to be", "to have", regular)
  - Verb data structures (Verb, VerbForm, VerbSense, VerbMeaning, VerbUsage, VerbUsageTier, Preposition, SpecialMeaningHolder)
  - Verbs registry with creation and lookup functions
  - Verb system internal NTs (6 new NTs)
  - 105 new tests

### Test status
- 397 tests pass, 0 failures
- `cargo clippy --all-targets` is clean

## What's Next

The next logical step is PLAN-12: VerbPhrases::seek — the verb-finding algorithm that searches for verb usages in a wording, builds the viability map, and produces VERB_NT sentence diagrams. This is the bridge between the verb system data structures and full sentence parsing.

After that:
- PLAN-13: Full sentence parsing (`<sentence>` internal NT)
- PLAN-14: World model kinds (conform7-semantics crate)
- PLAN-15: Assertion processing
- PLAN-16: Inter emission from world model
- PLAN-17+: Compiler driver, LSP, etc.
