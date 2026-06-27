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

---

## Technology Stack

| Concern | Choice | Rationale |
|---------|--------|-----------|
| Incremental computation | **Salsa** | Powers rust-analyzer; memoizes queries, tracks dependencies, minimal recomputation |
| Parsing | **Chumsky** | Parser combinators with best-in-class error recovery; I7 syntax is simple enough that parser performance is not the bottleneck |
| Syntax trees | **Rowan** | Lossless red-green trees; full fidelity (whitespace, comments); proven in rust-analyzer |
| LSP server | **tower-lsp-server** | Active community fork; used by Biome, Oxc, Deno |
| Diagnostics | **Ariadne** | Beautiful, colorful error output with labeled spans |
| Async runtime | **Tokio** | De facto standard; required by tower-lsp-server |
| Inter emission | **Custom binary writer** | We emit Inter bytecode directly; the existing C `inter` tool handles linking, optimization, and codegen |

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
│  │  Parser  │   │   Semantic   │   │   Inter Emit    │  │
│  │ (Chumsky)│──→│   Analysis   │──→│  (binary/text)  │  │
│  │          │   │  (World Model)│   │                 │  │
│  └──────────┘   └──────────────┘   └────────┬────────┘  │
│                                              │           │
│  Rowan CST/AST                                │           │
│  ┌──────────────────────────────────────┐    │           │
│  │  Green tree (immutable, interned)    │    │           │
│  │  Red tree  (parent pointers, offset) │    │           │
│  │  AST layer (typed wrappers)          │    │           │
│  └──────────────────────────────────────┘    │           │
└──────────────────────────────────────────────┼───────────┘
                                               │
                    Inter bytecode file         │
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
│   ├── conform7-syntax/          # Rowan AST definitions + Chumsky parser
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ast.rs            # AST node types (generated or hand-written)
│   │   │   ├── syntax_kind.rs    # SyntaxKind enum for all node/token types
│   │   │   ├── parser/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── lexer.rs      # Tokenizer (I7 has unique lexing needs)
│   │   │   │   ├── source.rs     # Source file, headings, sentences
│   │   │   │   ├── assertions.rs # "Peter is a man", "The tally is a number"
│   │   │   │   ├── phrases.rs    # "To expose (X - a value): ..."
│   │   │   │   ├── rules.rs      # "Every turn: ...", "Instead of..."
│   │   │   │   ├── i6_schema.rs  # (- ... -) embedded I6 code
│   │   │   │   └── error.rs      # Parse error types + recovery strategies
│   │   │   └── ast_ext.rs        # Extension methods on AST nodes
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
│   ├── conform7-inter/           # Inter IR emission
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tree.rs           # Inter tree construction in memory
│   │   │   ├── package.rs        # Package hierarchy (main, modules, submodules)
│   │   │   ├── symbol.rs         # Symbol tables and wiring
│   │   │   ├── instruction.rs    # Inter instruction constructors
│   │   │   ├── value.rs          # Inter value pairs
│   │   │   ├── types.rs          # Inter type system
│   │   │   ├── emit.rs           # High-level emission API (like Produce)
│   │   │   ├── binary.rs         # Binary Inter file writer
│   │   │   └── textual.rs        # Textual Inter file writer (for debugging)
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
└── docs/
    └── inter-format.md           # Notes on Inter binary format
```

---

## Salsa Database Design

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

### Lexer

The I7 lexer is unusual because it's natural language. Key token types:

- **Headings**: `Chapter 1 - The Beginning` (volume, book, part, chapter, section)
- **Strings**: `"Hello, world!"`
- **Bracketed I6**: `(- ... -)` — embedded Inform 6 code
- **Text substitutions**: `[tally]`, `[the noun]`, `[if condition]...[end if]`
- **Comments**: `[ ... ]` in some contexts
- **Words**: Everything else is natural language words

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SyntaxKind {
    // Tokens
    Word,           // any natural language word
    QuotedString,   // "text"
    I6Bracketed,    // (- ... -)
    TextSubstitution, // [value], [if ...], etc.
    HeadingMarker,  // Volume, Book, Part, Chapter, Section
    Comment,        // [comment text]
    Newline,
    Whitespace,

    // Nodes
    Source,         // root
    Heading,
    Sentence,
    Assertion,
    PhraseDefinition,
    PhraseBody,
    RuleDefinition,
    RuleBody,
    I6Schema,
    Error,
}
```

### Parser Design

The parser is a Chumsky recursive descent parser. Because I7 is mostly
natural language, the parser's job is relatively light:

1. **Split into headings and sentences** — this is mostly a line-by-line
   structural parse
2. **Classify sentences** — assertion, phrase definition, rule, or I6 schema
3. **Parse I6 schemas** — `(- ... -)` blocks get a sub-parser for I6 syntax
4. **Parse text substitutions** — `[name]`, `[if ...]`, `[otherwise]`, etc.

The heavy lifting happens in semantic analysis, not parsing.

```rust
fn i7_parser() -> impl Parser<Token, Ast> {
    // Top level: headings and sentences
    let heading = ...;
    let sentence = choice((
        assertion(),
        phrase_definition(),
        rule_definition(),
        i6_schema(),
        fallback_sentence(), // unclassified natural language
    ));

    heading.repeated().then(sentence.repeated()).map(|(h, s)| Ast { ... })
}
```

### Error Recovery

Chumsky's `recover_with(via_parser(...))` handles malformed input gracefully.
For I7, recovery is straightforward because most "errors" are just sentences
the parser doesn't understand yet — they get classified as fallback sentences
and diagnosed later in semantic analysis.

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

We produce binary Inter files (`.interb`) that the existing `inter` tool can
read. For debugging, we also support textual Inter output.

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
   ├── Lex source text
   ├── Parse into AST
   └── Extract headings, sentences, I6 schemas

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
   └── Write binary Inter file

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

### Phase 0: Foundation (Weeks 1-2)
- [ ] Set up Cargo workspace with all crates
- [ ] Define `SyntaxKind` enum for all I7 token/node types
- [ ] Implement Rowan green/red tree integration
- [ ] Set up Salsa database skeleton
- [ ] Implement basic lexer (words, strings, headings, I6 brackets)
- [ ] Write a small test suite with example I7 source files

### Phase 1: Parsing (Weeks 3-5)
- [ ] Implement Chumsky parser for I7 source structure
- [ ] Parse headings (volume, book, part, chapter, section)
- [ ] Parse sentences (assertions, phrases, rules)
- [ ] Parse I6 schemas `(- ... -)`
- [ ] Parse text substitutions `[...]`
- [ ] Error recovery for malformed input
- [ ] Generate Rowan CST from parser output
- [ ] Build typed AST layer over Rowan nodes

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
- [ ] Binary Inter file writer
- [ ] Textual Inter file writer (for debugging)
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

### 1. Why Rowan instead of a hand-rolled AST?

Rowan gives us lossless syntax trees for free. Every token, every whitespace,
every comment is preserved. This is essential for:
- Accurate error spans
- Code formatting / refactoring
- Semantic token highlighting
- Mapping LSP positions back to source

The red-green tree design means we can share subtrees (interning) and navigate
efficiently (parent pointers, offsets).

### 2. Why emit Inter instead of generating I6 directly?

The existing `inter` tool handles:
- Linking with kits (BasicInformKit, WorldModelKit, CommandParserKit)
- Pipeline optimizations (dead code elimination, constant folding)
- Code generation to I6 or C
- Index generation

Reimplementing all of that is a massive undertaking. By emitting Inter, we
get all of that for free. We can always reimplement the pipeline later if
needed.

### 3. Why Salsa instead of hand-rolled incremental computation?

Salsa is battle-tested (rust-analyzer compiles Rust on every keystroke). It
handles:
- Automatic dependency tracking
- Memoization with invalidation
- Cycle detection and fixpoint iteration
- Parallel computation
- Persistent caching

Building this ourselves would be a project in itself.

### 4. Why Chumsky instead of hand-written parser?

I7's syntax is simple enough that parser performance is not the bottleneck.
Chumsky gives us:
- Expressive combinators for rapid development
- Built-in error recovery
- Rich error types
- Pratt parsing for I6 expressions

If performance becomes an issue, we can hand-optimize the hot paths later.

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
- [Rowan: Red-Green Trees](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/syntax.md)
- [Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html)
- [Chumsky Guide](https://docs.rs/chumsky/latest/chumsky/guide/)
- [tower-lsp-server](https://github.com/tower-lsp-community/tower-lsp-server)
- [Ariadne Diagnostics](https://github.com/zesterer/ariadne)
