# AGENTS.md — Conform7 Development Principles

## Project Goal

Build a 100% compatible, lightning-fast Inform 7 compiler and LSP in Rust,
reusing the existing Inter IR and pipeline for code generation.

## Core Principles

### 1. 100% Compatibility

If a game compiles with the official `inform7` compiler, it must compile
identically with Conform7. Same world model, same Inter output, same story
file. This is non-negotiable.

The reference implementation lives at `gitignore/inform/`. Every design
decision should be validated against it.

A successfull implementation should be able to correctly compile all of the examples from the inform book and cookbook.

### 2. Reuse Non-C Inform Source

All non-C Inform source that makes up the default world model must be reused
as-is. This includes:

- `inform7/Internal/Inter/` — precompiled kits (BasicInformKit,
  WorldModelKit, CommandParserKit, EnglishLanguageKit,
  BasicInformExtrasKit)
- `inform7/Internal/Extensions/` — Standard Rules extensions
- `inform7/Internal/Languages/` — Language definitions
- `inform7/Internal/HTML/`, `Templates/`, `Miscellany/` — Supporting files

We parse and compile these the same as user source. The only thing we
replace is the C compiler pipeline.

### 3. Comprehensive Conformance Testing

Test coverage must be extensive. The goal is not just "a bunch of tests" but
comprehensive conformance tests:

- **Test against the official compiler.** For every test case, compile with
  both `inform7` and Conform7 and verify byte-for-byte identical Inter output.
- **Use Inform's existing test suites.** The `inform7/Tests/` and
  `inter/Tests/` directories contain hundreds of test cases. We should pass
  all of them.
- **Use the Examples and Cookbook.** `resources/Documentation/Examples/`
  and the Recipe Book contain real-world I7 source. These become our
  integration tests.
- **Be conscious of non-determinism.** Random seeds, timestamps, and
  platform-specific details must be controlled for byte-identical output.

### 4. Test Every Piece Independently

Every individual piece of the codebase should be tested for expected
behavior whenever possible:

- **Lexer** — tokenize I7 source, verify token stream
- **Parser** — parse I7 source, verify AST structure
- **World model** — process assertions, verify kind/instance/property tables
- **Inter emission** — emit Inter for a world model, verify bytecode
- **Binary format** — round-trip Inter files, verify byte-identical output

### 5. Integration Tests for Composition

Higher-level pieces must have tests that verify they correctly combine
lower-level pieces:

- Parse + world model: parse I7 source, verify world model state
- World model + Inter: build world model, verify emitted Inter
- Full pipeline: I7 source → Inter → `inter` tool → story file

### 6. Clean Abstractions (Hiding)

Focus on independently testable sub-sections. Things that make sense in
isolation should be usable to build larger components. Each crate/module
should have a clear, minimal public API.

- The parser shouldn't know about the world model
- The world model shouldn't know about Inter emission
- Inter emission shouldn't know about the LSP
- The LSP queries the Salsa database; it doesn't know how things are computed

### 7. Abstraction Through Reduction

Actively seek the least amount of system that produces the same behavior.
Cross-cut across different parts of the stack to find common patterns:

- Can two similar queries be one parameterized query?
- Can three modules share a common data structure?
- Is there a general mechanism that replaces several special cases?

Prefer fewer, more general abstractions over many specific ones.

### 8. Reference-First Development

Before implementing any component, study the corresponding C implementation
in `gitignore/inform/`:

- `inform7/core-module/` — Compilation pipeline, world model construction
- `inform7/assertions-module/` — Assertion sentence processing
- `inform7/knowledge-module/` — Kinds, instances, properties
- `inform7/imperative-module/` — Phrases and rules
- `inform7/runtime-module/` — Inter emission from I7
- `inter/bytecode-module/` — Inter tree data structures, binary format
- `inter/building-module/` — Inter construction middleware
- `inter/pipeline-module/` — Linking, optimization passes
- `inter/final-module/` — Code generation to I6/C

Understand the existing design before reimagining it.

**Important: Read the `.w` files, not the Tangled C.** The Inform source is
organized as literate programs using the `inweb` system. Each module's
`Chapter N/` directories contain `.w` files — these are the authoritative
source, with extensive prose explanations interleaved with the code. The
`Tangled/` directory contains generated C files produced by the `inweb`
tangler. These are stripped of all commentary and are not meant for human
consumption. Always read the `.w` files to understand both *what* the code
does and *why*.

### 9. Literate Code Through Extensive Documentation

Inform's source uses the `inweb` literate programming system, where `.w`
files interleave prose explanation with code. We adopt the same spirit but
without a separate tangling tool: every module, type, function, and
non-obvious block of code should have clear explanatory comments.

- **Module-level docs** (`//!`): Explain the module's role, its place in the
  overall architecture, and reference the corresponding Inform `.w` chapter.
- **Type and function docs** (`///`): Describe what, why, and how. Include
  examples for public API surfaces.
- **Inline comments** (`//`): Clarify non-obvious logic, document invariants,
  and cross-reference the C implementation where relevant.

The goal is that a new contributor (or your future self) can read any file
from top to bottom and understand not just *what* it does but *why* it does
it that way — without needing to consult external documentation.

### 10. Start Small, Ship Early

The first milestone should be the smallest thing that proves the
architecture works end-to-end. Build outward from there.

### 11. Salsa-First Design

Model everything as Salsa queries. Inputs are source files. Derived queries
are parsing, name resolution, world model, Inter emission. The LSP is just a
consumer of the Salsa database. This gives us incrementality for free.

### 12. Plan-Driven Development

Work proceeds in small, focused plans. Each plan targets a single
well-defined milestone that can be completed, tested, and demonstrated.

- **`plans/PLAN-N.md`** — Each plan gets its own numbered file (e.g.,
  `PLAN-1.md`, `PLAN-2.md`, …). The file stays put forever — no renaming
  or archiving.
- **`plans/CURRENT.md`** — A one-liner pointing to the active plan, e.g.
  `Current plan: PLAN-6.md`. This is the only file that changes between
  plans.

A plan is done when all its success criteria are met and all tests pass.
When a plan is complete, update its status to "Complete", write the next
`PLAN-(N+1).md` with status "In progress", and update `CURRENT.md` to
point to the new plan.

### 13. Clean Git Commit History

Make commits with in-depth descriptions as code is written. Each commit
should capture a coherent change with a message that explains *what* was
done and *why*. This keeps the history readable and makes it easy to
understand the evolution of the codebase.

### 14. No `unsafe` Rust

We shouldn't need it. If we need specially optimized code using `unsafe`
somehow it should be independencies like `salsa` or we need to find
other dependencies to help with our specific need that abstract over
the unsafe code for us.
