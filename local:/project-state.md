# Conform7 Project State (Updated)

## Completed (Plans 1-13)

### conform7-inter crate
- Inter IR read/write with round-trip fidelity

### conform7-syntax crate
- **Lexer** — state machine tokenizer
- **Sentence breaker** — FSM that splits tokens into sentences
- **ParseNode** — tree data model with annotations
- **NodeType** — enumerated node types with metadata
- **Heading/Structural AST** — sentence-to-AST bridges
- **Preform grammar parser** — parses Syntax.preform format
- **Preform matching engine** — backtracking matcher with internal NT dispatch
- **Linguistics module** — articles, diagrams, noun phrases, verb system, verb phrases
- **VerbPhrases::seek** — verb-finding algorithm with viability map
- **<sentence> internal NT** — full sentence parsing via VerbPhrases::seek

### Test status
- 417 tests pass, 0 failures
- `cargo clippy --all-targets` is clean

## What's Next

The next logical step is to start building the world model. The smallest independently testable piece is the kind system — the foundation of the world model that defines the type hierarchy (value, object, room, thing, container, supporter, etc.).

This would require creating a new `conform7-semantics` crate with:
- Kind hierarchy (value, object, room, thing, container, supporter, etc.)
- Kind relationships (subkinding, conjunction kinds)
- Instance tracking
- Property definitions

After that:
- PLAN-15: Assertion processing
- PLAN-16: Inter emission from world model
- PLAN-17+: Compiler driver, LSP, etc.
