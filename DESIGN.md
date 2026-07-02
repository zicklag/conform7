# Conform7: A Rust-based Inform 7 Compiler & LSP

## Vision

A lightning-fast, incrementally-compiled Inform 7 toolchain with a world-class
LSP that provides deep interactive insight into your interactive fiction
project. Reuses the existing Inform 7 **Inter** IR and pipeline for code
generation, while replacing the frontend and middle-end with a modern Rust
codebase.

## Guiding Principles

1. **Incremental everything.** Every keystroke should feel instant. Salsa
   ensures we only recompute what changed.
2. **Reuse, don't rebuild.** The existing `inter` tool and its pipeline
   (linking, optimization, codegen to I6/C) are battle-tested. We emit Inter
   bytecode and hand off.
3. **Deep insight.** The LSP isn't just autocomplete — it's a window into the
   world model. You should be able to explore kinds, instances, relations,
   rules, and phrases interactively.
4. **Start small, ship early.** Phase 1 is parsing + world model + LSP. Code
   generation comes after the interactive experience is solid.
5. **Reference-first development.** Before implementing any component, study
   the corresponding C implementation in `gitignore/inform/`. Read the `.w`
   files (literate programs), not the Tangled C. Understand the existing
   design before reimagining it.

---

## Technology Stack

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Incremental computation | **Salsa** | Powers rust-analyzer; memoizes queries, tracks dependencies, minimal recomputation |
| Grammar format | **Preform** | The same declarative pattern-matching grammar the C compiler uses; `Syntax.preform` is the authoritative grammar, loaded and matched at runtime |
| Matching engine | **Custom backtracking engine** | Implements the Preform matching algorithm: fixed words, `...` wildcards, sub-nonterminal recursion, internal NT dispatch |
| Syntax trees | **ParseNode** | Mirrors the C `parse_node` struct directly; linked-list children, sibling links, `next_alternative` for ambiguous parses; simple, faithful, proven |
| LSP server | **tower-lsp-server** | Active community fork; used by Biome, Oxc, Deno |
| Diagnostics | **Ariadne** | Beautiful, colorful error output with labeled spans |
| Async runtime | **Tokio** | De facto standard; required by tower-lsp-server |
| Inter emission | **Textual Inter** | We emit textual `.intert` files; the existing C `inter` tool handles linking, optimization, and codegen. Binary Inter is deferred. |

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────┐
│                    LSP Server                             │
│                 (tower-lsp-server)                        │
│  ┌────────────────────────────────────────────────────┐  │
│  │  hover │ goto-def │ completion │ semantic-tokens  │  │
│  │  references │ rename │ diagnostics │ code-lens    │  │
│  └──────────────────────┬─────────────────────────────┘  │
├─────────────────────────┼────────────────────────────────┤
│              Salsa Database (incremental)                 │
│                                                          │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────┐  │
│  │  Lexer   │   │   Preform   │   │   World Model   │  │
│  │ (tokens) │──→│   Matching  │──→│  (kinds, insts, │  │
│  │          │   │   Engine    │   │   props, verbs)  │  │
│  └──────────┘   └──────┬──────┘   └────────┬────────┘  │
│                        │                    │           │
│  Sentence Breaker      │ ParseNode trees    │           │
│  ┌─────────────────┐   │                    │           │
│  │ classified      │   │                    │           │
│  │ sentences       │   │                    │           │
│  └─────────────────┘   │                    │           │
│                        ▼                    ▼           │
│              ┌─────────────────────────────────────┐     │
│              │         Inter Emission             │     │
│              │     (textual .intert output)       │     │
│              └──────────────────┬──────────────────┘     │
└─────────────────────────────────┼────────────────────────┘
                                  │
              Textual Inter file  │
                                  ▼
┌──────────────────────────────────────────────────────────┐
│              Existing C toolchain                         │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────────┐  │
│  │  inter   │──→│   pipeline   │──→│  codegen → I6/C │  │
│  └──────────┘   └──────────────┘   └─────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Crate Structure

```
conform7/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── conform7-syntax/          # I7 frontend: lexer, sentence breaker, Preform engine, parse trees
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── syntax_kind.rs    # SyntaxKind enum for all token/node types
│   │   │   ├── token.rs          # Token type with source location
│   │   │   ├── lexer.rs          # I7 source lexer (tokenizer)
│   │   │   ├── sentence.rs       # Sentence breaker (token stream → classified sentences)
│   │   │   ├── wording.rs        # Word range references into source text
│   │   │   ├── node_type.rs      # NodeType enum with metadata (arity, category, flags)
│   │   │   ├── parse_node.rs     # ParseNode tree (children, siblings, alternatives)
│   │   │   ├── heading.rs        # Heading sentence → HEADING_NT parse node
│   │   │   ├── structural.rs     # Structural sentence → parse node
│   │   │   ├── preform.rs        # Preform grammar parser + matching engine
│   │   │   ├── preform_internal.rs # Internal NT registry and implementations
│   │   │   ├── linguistics.rs    # Linguistics module: articles, diagrams, noun phrases
│   │   │   ├── linguistic_constants.rs # Lcon type and grammatical constants
│   │   │   ├── stock_control.rs  # Linguistic stock registry
│   │   │   ├── word_assemblage.rs # Multi-word text type
│   │   │   ├── verb_conjugation.rs # Verb conjugation tables
│   │   │   └── verbs.rs          # Verb, VerbForm, VerbSense, VerbMeaning, VerbUsage, Preposition, SpecialMeaning
│   │   └── Cargo.toml
│   │
│   ├── conform7-semantics/       # World model, type checking, name resolution
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs             # Salsa database definition
│   │   │   ├── world/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── kinds.rs      # Kinds (number, text, thing, room, ...)
│   │   │   │   ├── instances.rs  # Instances (objects, rooms, ...)
│   │   │   │   ├── properties.rs # Properties (either/or, value, relation)
│   │   │   │   ├── relations.rs  # Binary predicates, verbs
│   │   │   │   ├── values.rs     # Values and type checking
│   │   │   │   └── actions.rs    # Actions and action patterns
│   │   │   ├── phrases/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── definitions.rs # Phrase definitions and signatures
│   │   │   │   └── compilation.rs # Compiling phrase bodies
│   │   │   ├── rules/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── rulebooks.rs  # Rulebooks and rule ordering
│   │   │   │   └── compilation.rs
│   │   │   ├── tables.rs         # Table definitions
│   │   │   ├── equations.rs      # Equation parsing and compilation
│   │   │   ├── scenes.rs         # Scene definitions
│   │   │   └── name_resolution.rs # Resolving names to kinds/instances/etc.
│   │   └── Cargo.toml
│   │
│   ├── conform7-inter/           # Inter IR read/write
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tree.rs           # InterTree, Package, SymbolsTable, Symbol
│   │   │   ├── instruction.rs    # Instruction constructors and types
│   │   │   ├── value.rs          # Inter value pairs
│   │   │   ├── types.rs          # Inter type system
│   │   │   └── textual.rs        # Textual .intert reader and writer
│   │   ├── tests/
│   │   │   ├── roundtrip_tests.rs
│   │   │   └── inter_compat_tests.rs
│   │   └── Cargo.toml
│   │
│   ├── conform7-compiler/        # Top-level compiler driver
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── driver.rs         # Compilation driver (orchestrates stages)
│   │   │   ├── stages.rs         # Compilation stage sequencing
│   │   │   ├── extensions.rs     # Extension loading and management
│   │   │   ├── build.rs          # Build management (like inbuild)
│   │   │   └── diagnostics.rs    # Error/warning reporting (Ariadne)
│   │   └── Cargo.toml
│   │
│   └── conform7-lsp/             # LSP server binary
│       ├── src/
│       │   ├── main.rs           # Server entry point
│       │   ├── server.rs         # LanguageServer trait implementation
│       │   ├── handlers/
│       │   │   ├── mod.rs
│       │   │   ├── hover.rs
│       │   │   ├── completion.rs
│       │   │   ├── goto_def.rs
│       │   │   ├── goto_ref.rs
│       │   │   ├── semantic_tokens.rs
│       │   │   ├── diagnostics.rs
│       │   │   ├── code_lens.rs
│       │   │   └── rename.rs
│       │   ├── capabilities.rs   # LSP capability declarations
│       │   └── world_explorer.rs # Interactive world model exploration
│       └── Cargo.toml
│
├── tests/                        # Integration tests
│   ├── fixtures/                 # Test I7 source files
│   └── integration.rs
│
├── plans/                        # Plan-driven development
│   ├── CURRENT.md                # Points to the active plan
│   ├── PLAN-1.md                 # Textual Inter compatibility
│   ├── PLAN-2.md                 # I7 Lexer Foundation
│   ├── ...
│   └── FUTURE-PERF.md            # Performance optimization notes
│
└── docs/
    └── inter-format.md           # Notes on Inter binary format
```

---

## Salsa Database Design

*Note: The Salsa database is a design target, not yet implemented. The current
codebase builds data structures directly without Salsa. Salsa integration will
be added when the compiler driver is built.*

The Salsa database is the heart of the system. Every compiler phase is a query.

```rust
#[salsa::jar(db = Db)]
struct Jar(
    // --- Inputs ---
    crate::source::source_text,
    crate::source::extension_text,
    crate::source::project_config,

    // --- Parsing ---
    crate::parser::parse_source,
    crate::parser::parse_headings,
    crate::parser::parse_sentences,
    crate::parser::ast,

    // --- Name Resolution ---
    crate::name_resolution::resolve_name,
    crate::name_resolution::resolve_kind,
    crate::name_resolution::resolve_instance,

    // --- World Model ---
    crate::world::kinds,
    crate::world::instances,
    crate::world::properties,
    crate::world::relations,
    crate::world::actions,
    crate::world::world_model,

    // --- Phrases & Rules ---
    crate::phrases::phrase_definitions,
    crate::phrases::compile_phrase,
    crate::rules::rulebooks,
    crate::rules::compile_rule,

    // --- Type Checking ---
    crate::types::type_of,
    crate::types::check_phrase_call,

    // --- Inter Emission ---
    crate::inter::emit_module,
    crate::inter::emit_tree,

    // --- Diagnostics ---
    crate::diagnostics::parse_errors,
    crate::diagnostics::semantic_errors,
    crate::diagnostics::all_diagnostics,
);

#[salsa::db(Jar)]
pub trait Db: salsa::Database {
    // Additional non-query methods can go here
}
```

### Key Query Design Patterns

**Inputs** are the base data that changes when the user edits:
```rust
#[salsa::input]
struct SourceFile {
    path: PathBuf,
    #[returns(ref)]
    contents: String,
}
```

**Interned** values for deduplication and fast comparison:
```rust
#[salsa::interned]
struct KindId {
    #[returns(ref)]
    name: String,
}

#[salsa::interned]
struct InstanceId {
    #[returns(ref)]
    name: String,
}
```

**Tracked** functions for derived computations:
```rust
#[salsa::tracked]
fn world_model(db: &dyn Db) -> WorldModel {
    // Builds kinds, instances, properties, relations from assertions
    // Salsa tracks which inputs this reads and memoizes the result
}

#[salsa::tracked]
fn type_of(db: &dyn Db, expr: ExprId) -> Type {
    // Type inference for an expression
}
```

**Accumulators** for diagnostics (errors/warnings collected during compilation):
```rust
#[salsa::accumulator]
struct Diagnostic(Conform7Diagnostic);

#[salsa::tracked]
fn all_diagnostics(db: &dyn Db) -> Vec<Conform7Diagnostic> {
    // Diagnostics are accumulated by other queries as they run
    parse_errors::accumulated::<Diagnostic>(db)
        .into_iter()
        .chain(semantic_errors::accumulated::<Diagnostic>(db))
        .collect()
}
```

### Salsa Integration Strategy

The C reference builds the world model through **sequential imperative passes**
over the sentence list. Each pass mutates global registries. This is the natural
approach in C, but it conflicts with Salsa's model where every tracked function
is a pure, deterministic computation.

The resolution is to **process all sentences at once** in each query, rather than
one sentence at a time. The C compiler already decouples declaration from
resolution across multiple passes — we make that explicit in the query graph.

#### Declaration-extraction pattern

Instead of a `WorldModel::process_sentence(&mut self, sentence)` that mutates
internal state, each query takes the full sentence list and returns a complete
table:

```rust
// Pass 1: extract every declaration from every sentence
#[salsa::tracked]
fn all_declarations(db: &dyn Db) -> Vec<Declaration> {
    let sentences = break_sentences(db);
    sentences.iter().flat_map(extract_declarations).collect()
    // Returns: [Instance("Ms. Mary", kind:"woman", loc:"Lab"),
    //           Kind("woman", parent:"person"),
    //           Instance("Lab", kind:"room"),
    //           Relation("loving", left:"person", right:"person"),
    //           Verb("adore", means:"loving relation"), ...]
}

// Pass 2: build the kind hierarchy (pure DAG, no forward references)
#[salsa::tracked]
fn kinds(db: &dyn Db) -> KindTable {
    let decls = all_declarations(db);
    KindTable::from_declarations(decls)
}

// Pass 3: build instances, resolving kind/location references
#[salsa::tracked]
fn instances(db: &dyn Db) -> InstanceTable {
    let decls = all_declarations(db);
    let kinds = kinds(db);
    InstanceTable::from_declarations(decls, &kinds)
}
```

The dependency graph is explicit:

```
source_text → all_declarations → kinds → instances → world_model
```

If the user edits the sentence that defines "Lab", Salsa invalidates
`all_declarations`, which propagates to `kinds`, `instances`, and
`world_model`. If the user edits an unrelated rule body, only the queries
that actually read that sentence's data are affected.

#### Handling cycles

I7 allows forward references that create cycles in the query graph:

```
The verb to adore means the loving relation.
Loving relates one person to one person.
```

The first sentence references "loving relation" before it's declared. In
Salsa, this is a cycle between `verbs` and `relations`:

```rust
#[salsa::tracked]
fn relations(db: &dyn Db) -> RelationTable {
    let decls = all_declarations(db);
    let verbs = verbs(db);  // may be a cycle
    RelationTable::from_declarations(decls, &verbs)
}

#[salsa::tracked]
fn verbs(db: &dyn Db) -> VerbTable {
    let decls = all_declarations(db);
    let relations = relations(db);  // may be a cycle
    VerbTable::from_declarations(decls, &relations)
}
```

Salsa detects cycles at runtime. When a cycle is detected, it uses the
**previous iteration's value** for the cyclic dependency and re-executes
the query. This repeats until the result stabilizes — a fixpoint iteration
that mirrors the C compiler's multi-pass approach, but automated.

The cycle is resolved because the declaration-extraction pass already found
both the verb and the relation. The first iteration builds verbs without
knowing about relations, and relations without knowing about verbs. The
second iteration fills in the missing references. By the third iteration,
both tables are complete and the result stabilizes.

#### What this means for data structure design now

To keep the path to Salsa integration clean, we design world model data
structures with these properties:

1. **Arena allocation, not `Box` pointers.** Instead of `Box<ParseNode>`
   trees, use `Vec<Thing>` with index handles. Arenas are Salsa-friendly
   (they're just `&[T]` references) and avoid ownership headaches.

2. **Interned IDs for named entities.** Kinds, instances, properties, verbs
   are identified by interned IDs, not `String` or `Rc<String>`. This makes
   comparison O(1) and fits Salsa's `#[salsa::interned]` pattern.

3. **Pure construction functions.** Each table is built by a function that
   takes inputs and returns a new table. No mutation of global state, no
   hidden dependencies on processing order.

4. **All-sentence queries, not per-sentence.** Queries process the full
   sentence list, not individual sentences. This avoids ordering
   dependencies and makes the dependency graph coarse enough to be
   efficient.

The frontend (lexer, sentence breaker, Preform engine, linguistics) is
stateless computation and doesn't need Salsa. The Salsa boundary starts at
the world model, where incrementality matters for the LSP.

---

## Parsing Strategy

### Inform 7 Source Structure

An I7 source file is organized as:

```
"My Story" by Author

Chapter 1 - The Beginning

The Lab is a room. "A sterile white laboratory."

Peter is a man in the Lab.

The tally is a number that varies. The tally is 0.

Every turn:
    say "The tally is [tally].";
    increment the tally.

To expose (X - a value):
    say "You admire [X]."

Instead of taking the beaker:
    say "It's bolted to the table."
```

### Pipeline

I7 parsing follows the C reference's architecture, not a conventional recursive
descent parser. The pipeline is:

```
Source text
    │
    ▼
Lexer (state machine) → flat Vec<Token>
    │
    ▼
Sentence breaker (FSM) → classified sentences (headings, structural, regular)
    │
    ▼
Preform matching engine (backtracking) → ParseNode trees
    │
    ▼
Linguistics module (articles, noun phrases, verb phrases) → annotated diagrams
    │
    ▼
World model (kinds, instances, properties, relations)
```

### Why Preform instead of a hand-written parser

I7's syntax is natural language, not a conventional programming language grammar.
The C reference uses **Preform** — a declarative pattern-matching grammar format
defined in `Syntax.preform` files (~720 nonterminals for English). Key reasons
for following this approach:

1. **The grammar is the source of truth.** `Syntax.preform` is the authoritative
   grammar used by the C compiler. We load it at runtime and match against it.
   No duplication, no drift.

2. **Backtracking with alternatives.** I7 sentences are inherently ambiguous.
   The Preform engine tries productions in order and backtracks on failure. The
   parse tree preserves `next_alternative` links for the world model to resolve
   later.

3. **Internal nonterminals.** Many grammar rules delegate to Rust functions
   (article lookup, verb conjugation matching, paragraph detection). These are
   registered by name in an `InternalRegistry` and dispatched during matching.

4. **Proven.** This architecture has been compiling I7 for 20+ years.

### Lexer

The I7 lexer is a state machine (not a parser combinator) that produces a flat
`Vec<Token>`. Key token types:

- **Words**: natural language words, case-preserved
- **Strings**: `"Hello, world!"`
- **I6 blocks**: `(- ... -)` — embedded Inform 6 code
- **Text substitutions**: `[tally]`, `[if condition]...[end if]`
- **Comments**: `[...]` outside strings
- **Paragraph breaks**: blank lines (significant — they end sentences)
- **Punctuation**: `. , : ; ? ! ( ) { }`
- **Heading markers**: Volume, Book, Part, Chapter, Section

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub text: String,
    pub range: Range<usize>,
}
```

### Sentence Breaker

The sentence breaker is a finite state machine that takes the token stream and
splits it into classified sentences:

```rust
pub struct Sentence {
    pub token_range: Range<usize>,
    pub classification: SentenceClassification,
}

pub enum SentenceClassification {
    Heading { level: HeadingLevel },
    Structural(StructuralType),
    Regular,
    RulePreamble,
    RulePhrase,
}
```

Sentence-ending punctuation:
- `.` (full stop) — always ends a sentence
- `;` (semicolon) — ends a phrase within a rule
- `:` (colon) — ends a rule preamble
- Paragraph break — always ends a sentence

### Preform Grammar

The Preform grammar format is:

```
<nonterminal-name> internal

<nonterminal-name> ::=
    production1 |
    production2 |
    ...
```

Productions contain:
- **Fixed words**: literal text like `to`, `is`, `a`, `room`
- **Ellipsis wildcards**: `...` (any number of words) or `.....` (exactly N)
- **Sub-nonterminals**: `<quoted-text>`, `<if-start-of-paragraph>`, etc.
- **Internal**: matching is done by a Rust function, not grammar rules

The grammar is parsed into in-memory data structures:

```rust
pub struct Grammar {
    pub nonterminals: HashMap<String, Nonterminal>,
}

pub struct Nonterminal {
    pub name: String,
    pub productions: Vec<Production>,
    pub internal: bool,
}

pub struct Production {
    pub tokens: Vec<ProductionToken>,
}

pub enum ProductionToken {
    FixedWord(String),
    Wildcard(usize),       // number of dots
    SubNonterminal(String),
}
```

### Matching Engine

The matching engine takes a nonterminal and a wording and tries to match it
against all productions with backtracking:

```rust
pub fn match_nonterminal(
    ctx: &PreformContext,
    registry: &InternalRegistry,
    name: &str,
    wording: Wording,
) -> Option<Match>
```

Internal nonterminals are Rust functions registered by name:

```rust
pub trait InternalNonterminal: Debug {
    fn match_internal(
        &self,
        ctx: &PreformContext,
        wording: Wording,
    ) -> Option<InternalResult>;
}
```

### Parse Trees

The matching engine produces `ParseNode` trees that mirror the C `parse_node`
struct:

```rust
pub struct ParseNode {
    pub wording: Wording,
    pub node_type: NodeType,
    pub annotations: Vec<Annotation>,
    pub down: Option<Box<ParseNode>>,        // first child
    pub next: Option<Box<ParseNode>>,         // next sibling
    pub next_alternative: Option<Box<ParseNode>>, // alternative interpretation
}
```

Key design points:
- **`next_alternative`** preserves ambiguous parses for the world model to resolve
- **`Wording`** is a lightweight `(start, end)` range into the source word stream
- **`NodeType`** is an enum with metadata (arity, category, flags) matching the C reference

### Linguistics Module

The linguistics module bridges raw Preform matches to structured parse trees:

- **Articles**: definite ("the"), indefinite ("a", "an", "some") — matched by
  internal NTs and returned as `ArticleUsage` values
- **Noun phrases**: `<np-unparsed>` (raw text), `<np-articled>` (article + noun),
  with NP3/NP4 (list-divided, relative clauses) planned
- **Verb system**: `Verb`, `VerbForm`, `VerbSense`, `VerbMeaning`, `VerbUsage`,
  `Preposition`, `SpecialMeaningHolder` — the data structures that represent
  verbs and their grammatical properties
- **Verb conjugation**: Simplified conjugation tables for common English verbs
  ("to be", "to have"), with the full Preform-based system deferred
- **Sentence diagrams**: `VERB_NT`, `UNPARSED_NOUN_NT`, `COMMON_NOUN_NT`,
  `PROPER_NOUN_NT`, etc. — the node types that represent parsed sentences

---

## Semantic Analysis: The World Model

This is where the bulk of the work happens. The world model mirrors the
existing Inform 7 compiler's model:

### Kinds
```
kind → "number" | "text" | "thing" | "room" | "direction"
     | "rule" | "action" | "table" | ...
     | "list of" kind
     | kind "that varies"
```

### Instances
```
"The Lab is a room."
→ Instance { name: "Lab", kind: Room, ... }

"Peter is a man in the Lab."
→ Instance { name: "Peter", kind: Man, location: Lab }
```

### Properties
```
"A person has a number called age."
→ Property { name: "age", kind: Number, owner: Person }

"The Lab is dark."
→ EitherOrProperty { name: "dark", applies_to: Lab }
```

### Relations
```
"Loving relates one person to one person."
→ Relation { name: "loving", left: Person, right: Person }
```

### Verbs
```
"The verb to adore means the loving relation."
→ Verb { name: "adore", relation: Loving }
```

### Phrases
```
"To expose (X - a value): ..."
→ Phrase {
    name: "expose",
    parameters: [("X", Value)],
    body: ...
}
```

### Rules
```
"Every turn: ..."
→ Rule { book: "every turn", body: ... }

"Instead of taking the beaker: ..."
→ Rule { book: "instead", action: Taking(beaker), body: ... }
```

### Salsa Query Flow

```
source_text (input)
    │
    ▼
parse_source → Ast
    │
    ├──► resolve_names → NameBindings
    │
    ├──► kinds → KindTable
    │
    ├──► instances → InstanceTable
    │
    ├──► properties → PropertyTable
    │
    ├──► relations → RelationTable
    │
    ├──► phrase_definitions → PhraseTable
    │
    ├──► rulebooks → RulebookTable
    │
    └──► world_model → WorldModel (aggregates all above)
            │
            ▼
        emit_tree → InterTree
```

---

## Inter Emission

### Inter Format Overview

Inter is a hierarchical package structure containing instructions. The
top-level structure for an I7 compilation is:

```
root
  packagetype _plain
  packagetype _code
  packagetype _module
  packagetype _submodule
  packagetype _linkage
  ...primitives...

  package main _plain
    package architecture _linkage
      constant WORDSIZE = 4
      constant TARGET_GLULX = 1
      ...

    package connectors _linkage
      plug ...
      socket ...

    package generic _module
      package kinds _submodule
        ...
      package variables _submodule
        ...

    package source_text _module
      package kinds _submodule
        ...
      package variables _submodule
        ...
      package functions _submodule
        ...

    package BasicInformKit _module
      ... (transmigrated during linking)
```

### Emission API

We provide a high-level API inspired by the existing `Produce` module:

```rust
impl InterEmitter {
    /// Emit a numeric constant
    fn numeric_constant(&mut self, name: &InterName, kind: Kind, value: i32);

    /// Emit a function body
    fn function_body(&mut self, name: &InterName) -> FunctionBuilder;

    /// Emit an invocation of a primitive
    fn inv_primitive(&mut self, primitive: Primitive);

    /// Emit a value
    fn val(&mut self, kind: Kind, value: InterValue);

    /// Emit a property declaration
    fn property(&mut self, name: &InterName, owner: Kind, kind: Kind);

    /// Emit an instance declaration
    fn instance(&mut self, name: &InterName, kind: Kind);
}
```

### Output

We produce textual Inter files (`.intert`) that the existing `inter` tool can
read. The textual reader/writer is complete and cross-validated against the
official `inter` tool for round-trip fidelity. Binary Inter (`.interb`) is
deferred — the core pipeline emits textual Inter and hands off to the existing
C toolchain.

---

## LSP Features

### Phase 1: Essential Features

| Feature | Description | Priority |
|---------|-------------|----------|
| **Diagnostics** | Parse errors, semantic errors, type errors | P0 |
| **Hover** | Show kind/type/description of any identifier | P0 |
| **Go to Definition** | Jump to where a kind/instance/phrase/rule is defined | P0 |
| **Find References** | Find all uses of a kind/instance/phrase/rule | P0 |
| **Semantic Tokens** | Syntax highlighting based on semantic meaning | P0 |
| **Completion** | Context-aware autocomplete | P1 |
| **Rename** | Rename a kind/instance/phrase/rule across the project | P1 |

### Phase 2: World Model Insight

| Feature | Description | Priority |
|---------|-------------|----------|
| **World Explorer** | Interactive tree/table view of the world model | P1 |
| **Kind Hierarchy** | Visualize the kind tree | P1 |
| **Relation Graph** | Visualize relations between instances | P2 |
| **Rulebook Inspector** | See rule ordering, which rules fire when | P2 |
| **Phrase Signatures** | Show overloaded phrase variants | P2 |
| **Code Lens** | Show references count, rulebook membership | P2 |

### Phase 3: Advanced

| Feature | Description | Priority |
|---------|-------------|----------|
| **Map Preview** | Visualize room connections | P3 |
| **Action Trace** | Trace action processing through rules | P3 |
| **Extension Browser** | Browse installed extensions and their contents | P3 |
| **Inline Evaluation** | Evaluate simple expressions inline | P3 |

### LSP Implementation Pattern

Each LSP handler queries the Salsa database:

```rust
impl LanguageServer for Conform7Server {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let db = self.db.read().await;
        let file = file_from_url(&params.text_document_position_params.text_document.uri);
        let pos = offset_from_position(&params.text_document_position_params.position);

        // Query the Salsa database
        let ast = parse_source(&db, file);
        let node = ast.node_at_offset(pos);
        let name = resolve_name(&db, node);

        match name {
            Some(Name::Kind(kind_id)) => {
                let kind = kinds(&db).get(kind_id);
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Kind:** `{}`\n\n{}", kind.name(), kind.description()),
                    }),
                    range: Some(node.range()),
                }))
            }
            Some(Name::Instance(inst_id)) => {
                let inst = instances(&db).get(inst_id);
                Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**{}** ({})\n\nLocation: {}\n\n{}",
                            inst.name(),
                            inst.kind().name(),
                            inst.location().map_or("nowhere".into(), |l| l.name()),
                            inst.description(),
                        ),
                    }),
                    range: Some(node.range()),
                }))
            }
            _ => Ok(None),
        }
    }
}
```

---

## Compilation Pipeline

### Stage Sequence (mirrors existing I7 compiler)

```
1. Build Management
   ├── Load project configuration
   ├── Discover extensions
   ├── Read source files
   └── Create initial Inter hierarchy

2. Parsing
   ├── Lex source text → tokens
   ├── Break into sentences
   ├── Match sentences against Preform grammar → ParseNode trees
   └── Linguistics module: articles, noun phrases, verb phrases

3. Semantic Analysis (Pass 1)
   ├── Pre-pass: identify major nodes
   ├── Build built-in kinds (number, text, thing, ...)
   ├── Build built-in verbs (to be, to have, ...)
   └── Build built-in relations (meaning, ...)

4. Semantic Analysis (Pass 2)
   ├── Process assertion sentences
   ├── Build kinds, instances, properties
   ├── Build relations
   └── Type checking

5. Semantic Analysis (Pass 3)
   ├── Process phrase definitions
   ├── Process rule definitions
   ├── Compile phrase bodies
   └── Compile rule bodies

6. Inter Emission
   ├── Emit kind declarations
   ├── Emit instance declarations
   ├── Emit property declarations
   ├── Emit relation declarations
   ├── Emit phrase/rule functions
   ├── Emit metadata
   └── Write textual Inter file

7. Hand off to `inter` tool
   ├── Link with kits (BasicInformKit, WorldModelKit, ...)
   ├── Pipeline optimizations
   ├── Code generation (I6 or C)
   └── Final story file
```

### Incremental Compilation Flow

```
User edits file
    │
    ▼
Salsa: set source_text input to new value
    │
    ▼
Salsa: invalidate dependent queries
    │
    ├── parse_source(file) → re-parse if changed
    │   ├── headings(file) → re-parse if changed
    │   └── sentences(file) → re-parse if changed
    │
    ├── kinds() → re-compute if assertions changed
    ├── instances() → re-compute if assertions changed
    ├── properties() → re-compute if assertions changed
    │
    ├── phrase_definitions() → re-compute if changed
    ├── rulebooks() → re-compute if changed
    │
    ├── world_model() → re-compute if any above changed
    │
    └── emit_tree() → re-emit if world model changed
            │
            ▼
        Write Inter file → run inter tool
```

---

## Development Phases

### Phase 0: Foundation (Weeks 1-2) — Complete
- [x] Set up Cargo workspace with all crates
- [x] Define `SyntaxKind` enum for all I7 token/node types
- [x] Implement lexer (words, strings, headings, I6 brackets, comments)
- [x] Implement sentence breaker (FSM for splitting token stream)
- [x] Implement `ParseNode` tree data model
- [x] Implement `NodeType` enum with metadata
- [x] Implement Preform grammar parser (loads `Syntax.preform`)
- [x] Implement Preform matching engine (backtracking, wildcards, sub-NTs)
- [x] Implement internal NT dispatch (registry of Rust functions)
- [x] Implement `conform7-inter` crate (textual Inter read/write, round-trip verified)

### Phase 1: Linguistics & Parsing (Weeks 3-5) — In Progress
- [x] Linguistics node types (VERB_NT, UNPARSED_NOUN_NT, etc.)
- [x] Article system (definite/indefinite articles, internal NTs)
- [x] Noun phrase parsing (NP1/NP2: unparsed, articled)
- [x] Word assemblage type
- [x] Linguistic constants (Lcon type, grammatical attributes)
- [x] Stock control (linguistic registry)
- [x] Verb conjugation (simplified for English)
- [ ] Verb data structures and creation (Verb, VerbForm, VerbSense, VerbMeaning, VerbUsage, Preposition, SpecialMeaning)
- [ ] Verb phrase parsing (VerbPhrases::seek)
- [ ] Full sentence parsing (<sentence> internal NT)
- [ ] NP3/NP4 noun phrase levels (list-divided, relative clauses)

### Phase 2: World Model (Weeks 6-10)
- [ ] Implement built-in kinds (number, text, thing, room, ...)
- [ ] Implement built-in verbs and relations
- [ ] Parse assertion sentences into world model
- [ ] Kind system (including list of, that varies, etc.)
- [ ] Instance creation and property assignment
- [ ] Either/or properties and value properties
- [ ] Relations and verbs
- [ ] Name resolution (resolving words to kinds/instances/properties)
- [ ] Type checking for values and expressions

### Phase 3: Phrases & Rules (Weeks 11-14)
- [ ] Phrase definition parsing and signature resolution
- [ ] Phrase body compilation (I6 schemas with substitutions)
- [ ] Rule definition parsing
- [ ] Rulebook organization and ordering
- [ ] Rule body compilation
- [ ] Action definitions and action patterns
- [ ] Table definitions

### Phase 4: Inter Emission (Weeks 15-17)
- [ ] Implement Inter tree data structure in memory
- [ ] Package hierarchy construction
- [ ] Symbol table management
- [ ] Instruction emission API
- [ ] Textual Inter file writer
- [ ] Integration test: compile simple I7 → Inter → run through `inter` tool

### Phase 5: LSP Server (Weeks 18-22)
- [ ] Set up tower-lsp-server with basic initialize/shutdown
- [ ] File watching and incremental re-parsing
- [ ] Diagnostics (parse errors + semantic errors)
- [ ] Hover (kind/instance/phrase/rule info)
- [ ] Go to definition
- [ ] Find references
- [ ] Semantic tokens (syntax highlighting)
- [ ] Completion (context-aware autocomplete)
- [ ] Rename

### Phase 6: World Model Insight (Weeks 23-26)
- [ ] World explorer panel (tree/table view)
- [ ] Kind hierarchy visualization
- [ ] Relation graph
- [ ] Rulebook inspector
- [ ] Phrase signature browser
- [ ] Code lens (reference counts, rulebook membership)

### Phase 7: Polish & Performance (Weeks 27-30)
- [ ] Performance profiling and optimization
- [ ] Large project testing (Counterfeit Monkey, etc.)
- [ ] Extension compatibility testing
- [ ] Error message quality improvements
- [ ] Documentation
- [ ] VS Code extension packaging

---

## Key Design Decisions

### 1. Why Preform instead of a hand-written parser?

I7's syntax is natural language, not a conventional programming language grammar.
The C reference uses **Preform** — a declarative pattern-matching grammar format
defined in `Syntax.preform` files (~720 nonterminals for English). We follow the
same approach because:

- **The grammar is the source of truth.** `Syntax.preform` is the authoritative
  grammar used by the C compiler. We load it at runtime and match against it.
  No duplication, no drift.
- **Backtracking with alternatives.** I7 sentences are inherently ambiguous.
  The Preform engine tries productions in order and backtracks on failure. The
  parse tree preserves `next_alternative` links for the world model to resolve
  later.
- **Internal nonterminals.** Many grammar rules delegate to Rust functions
  (article lookup, verb conjugation matching, paragraph detection). These are
  registered by name in an `InternalRegistry` and dispatched during matching.
- **Proven.** This architecture has been compiling I7 for 20+ years.

A conventional parser combinator library (e.g., Chumsky) would require either
duplicating the grammar in Rust code or writing a Preform-to-combinator compiler.
Neither is worth the cost.

### 2. Why ParseNode instead of Rowan (red-green trees)?

The `ParseNode` tree mirrors the C `parse_node` struct directly. This is simpler
and more faithful to the reference than Rowan's green/red tree architecture:

- **`next_alternative`** is a first-class concept — ambiguous parses are preserved
  as linked alternatives. Rowan has no equivalent.
- **Linked-list children** match the C reference's `down`/`next` traversal.
- **`Wording`** (word range into source) provides source mapping without a
  separate tree layer.
- **~400 lines of straightforward Rust** vs. Rowan's multi-layer abstraction
  (green tree, red tree, AST traits).

Rowan's strengths (lossless whitespace/comments, incremental reparsing) are
less relevant here: the lexer already preserves comments and whitespace as
tokens, and Salsa provides query-level incrementality.

### 3. Why emit Inter instead of generating I6 directly?

The existing `inter` tool handles:
- Linking with kits (BasicInformKit, WorldModelKit, CommandParserKit)
- Pipeline optimizations (dead code elimination, constant folding)
- Code generation to I6 or C
- Index generation

Reimplementing all of that is a massive undertaking. By emitting Inter, we
get all of that for free. We can always reimplement the pipeline later if
needed.

### 4. Why Salsa instead of hand-rolled incremental computation?

Salsa is battle-tested (rust-analyzer compiles Rust on every keystroke). It
handles:
- Automatic dependency tracking
- Memoization with invalidation
- Cycle detection and fixpoint iteration
- Parallel computation
- Persistent caching

Building this ourselves would be a project in itself.

### 5. How do we handle the Standard Rules and extensions?

The Standard Rules and extensions are just I7 source text. We parse them the
same way as user source. The challenge is that they're large (Standard Rules
is ~100K words). Salsa's incrementality means we only re-parse them when they
change (which is never during normal editing).

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Inter binary format is undocumented | Study the `bytecode` module source; write binary Inter files and verify with existing `inter` tool |
| I7 natural language parsing is harder than expected | Start with structural parsing only; classify sentences broadly; refine in semantic analysis |
| Standard Rules are too large for initial parsing | Use Salsa's incrementality; parse once, memoize forever |
| Extension compatibility issues | Test with popular extensions early; the I7 extension format is well-defined |
| Salsa learning curve | Start with simple queries; the Salsa book and rust-analyzer are excellent references |
| Inter pipeline changes break our output | Pin to a specific Inter version; add version compatibility tests |

---

## References

- [Inform 7 Compiler Structure](https://ganelson.github.io/inform/structure.html)
- [Inform 7 Core Module: How To Compile](https://ganelson.github.io/inform/core-module/1-htc.html)
- [Inter Bytecode Module](https://ganelson.github.io/inform/bytecode-module/P-wtmd.html)
- [Inter Building Module](https://ganelson.github.io/inform/building-module/P-wtmd.html)
- [Salsa Book](https://salsa-rs.github.io/salsa/)
- [tower-lsp-server](https://github.com/tower-lsp-community/tower-lsp-server)
- [Ariadne Diagnostics](https://github.com/zesterer/ariadne)
