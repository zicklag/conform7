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

### 9. Start Small, Ship Early

The first milestone should be the smallest thing that proves the
architecture works end-to-end. Build outward from there.

### 10. Salsa-First Design

Model everything as Salsa queries. Inputs are source files. Derived queries
are parsing, name resolution, world model, Inter emission. The LSP is just a
consumer of the Salsa database. This gives us incrementality for free.

### 11. Plan-Driven Development

Work proceeds in small, focused plans. Each plan targets a single
well-defined milestone that can be completed, tested, and demonstrated.

- **`plans/CURRENT.md`** — The active plan. This is what we're working on
  right now.
- **`plans/COMPLETE-1.md`, `COMPLETE-2.md`, ...** — Completed plans,
  archived in order of completion.

A plan is done when all its success criteria are met and all tests pass.
When a plan is complete, move `CURRENT.md` to the next `COMPLETE-N.md`
and write a new `CURRENT.md` for the next milestone.
