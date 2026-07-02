# Conform7 Project State (Updated)

## Completed (Plans 1-10)

### conform7-inter crate
- InterTree, Package, SymbolsTable, Symbol data structures
- Instruction constructors (30+ instruction types)
- InterValue (two-word values, 11 formats)
- InterType system (int32, int16, text, enum, struct, etc.)
- Textual Inter reader/writer (.intert format)
- Round-trip fidelity tests against the `inter` tool

### conform7-syntax crate
- **SyntaxKind enum** — all I7 token/node types
- **Lexer** — state machine tokenizer
- **Sentence breaker** — FSM that splits tokens into sentences
- **ParseNode** — tree data model with child/sibling/alternative links
- **NodeType** — enumerated node types with metadata
- **Heading AST** — sentence-to-AST bridge for headings
- **Structural AST** — sentence-to-AST bridge for Include/Use/Table/Equation
- **Preform grammar parser** — parses Syntax.preform format
- **Preform matching engine** — backtracking matcher with internal NT dispatch
- **Internal nonterminal dispatch** — InternalRegistry, InternalNonterminal trait
- **Three basic internal NTs**: `<if-start-of-paragraph>`, `<if-not-cap>`, `<preform-nonterminal>`
- **Linguistics module** (NEW in PLAN-10):
  - 13 linguistics NodeType variants (Verb, UnparsedNoun, Pronoun, etc.)
  - Article system (Article, ArticleUsage, SmallWordSet)
  - Three article internal NTs (`<article>`, `<definite-article>`, `<indefinite-article>`)
  - Diagram constructor functions (Diagrams::new_*)
  - Noun phrase parsing at NP1 (`<np-unparsed>`) and NP2 (`<np-articled>`) levels
  - Public `parse_noun_phrase` API
  - 30+ tests

### Test status
- 292 tests pass, 0 failures
- `cargo clippy --all-targets` is clean

## What's Next

The next independently testable piece should be one of:
1. **World model kinds** — the kind hierarchy (value, object, room, thing, etc.) — this is the foundation of the world model and doesn't depend on the full assertion parser
2. **Verb phrase parsing** — building on the linguistics module to parse verb phrases
3. **Full sentence parsing** — combining noun and verb phrases to parse complete sentences

The kind system is the most foundational next step. It would require creating a new `conform7-semantics` crate (or adding to the existing syntax crate) with:
- Kind hierarchy (value, object, room, thing, container, supporter, etc.)
- Kind relationships (subkinding, conjunction kinds)
- Instance tracking
- Property definitions

This is independently testable: we can construct kinds programmatically and verify the hierarchy without needing to parse any I7 source.
