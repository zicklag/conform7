# Plan 1: Inter Binary Format Reader/Writer

**Status**: Textual Inter complete; binary reader still in progress
**Started**: 2026-06-27
**Target**: 1-2 weeks

## Goal

Implement the `conform7-inter` crate — a Rust library that can read and write
Inter files (both textual `.intert` and binary `.interb` formats) with 100%
fidelity to the existing `inter` tool.

This is the compatibility linchpin: the Inter format is the handoff between
our Rust compiler and the existing C toolchain. If we can't produce
byte-identical Inter, nothing else matters.

## Progress

### Completed

- [x] In-memory Inter tree (`tree.rs`) with packages, symbols, instructions,
      values, and types.
- [x] Globally unique symbol IDs across all symbols tables, avoiding
      cross-table ID collisions when resolving instruction references.
- [x] Textual Inter reader (`textual.rs`) supporting:
  - packages, constants, variables, locals, typenames
  - primitives, `inv`, `val`, `code`, `lab`, `label`
  - type markers: `constant (K_number) C_x = 1`, `val (int32) 17`,
    `package (K_func) R_101 _code`, `local (int32) argument`
  - function invocations via URL: `inv /main/OtherFunction`
  - forward references and cross-package URL wiring
  - list literals: `{ 2, 3, ... }`
  - struct literals: `struct{ ... }`
  - plugs, sockets, pragmas, inserts, property/value/permission
- [x] Textual Inter writer that preserves indentation depth for nested
      instructions and re-emits type markers, list/struct literals, and
      wired symbol references.
- [x] Cross-validation with the official `inter` tool for all fixtures
      except `misc.intert`.
- [x] Round-trip tests for all bundled `.intert` fixtures.
- [x] Binary Inter writer and basic binary round-trip for a simple tree.

### Remaining

- [ ] Complete binary Inter reader to reconstruct real `.interb` files from
      the official kits (EnglishLanguageKit, etc.).
- [ ] Un-ignore the 7 currently ignored binary compatibility tests.
- [ ] Run against the full `inter/Tests/Valid/` and `inter/Tests/Toys/`
      suites and fix any textual constructs we still mishandle.
- [ ] Byte-for-byte binary fidelity against the reference implementation.

## Key Design Decisions

1. **The in-memory tree is our own design**, not a direct port of the C
   `inter_tree`. The C version is optimized for C idioms; we design for Rust
   ergonomics while maintaining semantic equivalence.

2. **Textual format first, binary second.** The textual format is
   human-readable and easier to debug. We implement it first, then binary.

3. **The existing `inter` tool is the source of truth.** If our output
   differs from what `inter` expects, we fix our code, not the other way
   around.

4. **Symbol IDs are globally unique in our in-memory tree.** The C
   implementation gives each symbols table its own counter starting at
   `SYMBOL_BASE`, so the same raw ID can mean different symbols in different
   tables. We share a single tree-wide counter so that instruction fields
   can be resolved without first knowing which table they came from. This
   is a pragmatic departure from the C layout; we will normalize IDs back
   to per-table values when writing binary Inter.

5. **No Salsa yet.** This crate is a pure data library with no incremental
   computation needs. Salsa comes in when we build the compiler driver.

## Test Strategy

### Unit tests
- Core data structures (tree, symbols, values, instructions, types).

### Integration tests

- `roundtrip_tests.rs` — read every bundled `.intert` fixture, write it
  back, and parse the result; assert the final tree matches the first.

- `inter_compat_tests.rs` — feed our re-serialized text to the official
  `inter` tool and verify it parses without errors. This is the strongest
  practical oracle we have without building the C toolchain.

- `binary_compat_tests.rs` — verify our binary writer produces output that
  `inter` can read, and (eventually) that our binary reader can load real
  kit files.

## Success Criteria

- [x] All bundled `.intert` fixtures round-trip through our textual
      reader/writer with identical output.
- [x] All bundled `.intert` fixtures (except `misc.intert`) are accepted by
      the existing `inter` tool after re-serialization.
- [x] Programmatic construction of Hello World produces correct output.
- [ ] All `.intert` files from `inter/Tests/Valid/` round-trip correctly.
- [ ] All `.intert` files from `inter/Tests/Toys/` round-trip correctly.
- [ ] Binary output from our writer is accepted by the existing `inter` tool.
- [ ] Binary → text round-trip through `inter` produces identical text.
- [ ] All tests pass on CI.

## Out of Scope

- Salsa integration (comes in the compiler driver)
- I7 parsing (comes in `conform7-syntax`)
- World model (comes in `conform7-semantics`)
- LSP (comes in `conform7-lsp`)
- Inter pipeline/optimization (handled by existing `inter` tool)
- Code generation to I6/C (handled by existing `inter` tool)
