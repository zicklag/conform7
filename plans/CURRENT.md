# Plan 1: Textual Inter Compatibility

**Status**: Textual Inter complete; binary reader deferred
**Started**: 2026-06-27
**Target**: 1-2 weeks

## Goal

Implement the `conform7-inter` crate — a Rust library that can read and write
Inter files (textual `.intert` format) with 100% fidelity to the existing
`inter` tool.

This is the compatibility linchpin: the Inter format is the handoff between
our Rust compiler and the existing C toolchain. If we can't produce
byte-identical Inter, nothing else matters.

## Why Textual First

- **Testable in complete isolation.** No parser, no world model, no Salsa.
  Just construct Inter trees in memory and write them out.
- **The existing `inter` tool is our oracle.** We can verify correctness by
  round-tripping through it.
- **Unlocks all future testing.** Once this works, every subsequent piece
  (parser, world model, etc.) can be tested by comparing its Inter output to
  what `inform7` produces for the same input.

## Deliverables

### Crate: `conform7-inter`

```
crates/conform7-inter/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API, re-exports
│   ├── tree.rs          # InterTree, Package, SymbolsTable, Symbol
│   ├── instruction.rs   # Instruction constructors and types
│   ├── value.rs         # Inter value pairs (two-word values)
│   ├── types.rs         # Inter type system (int32, int16, text, enum, struct, ...)
│   └── textual.rs       # Textual .intert reader and writer
└── tests/
    ├── roundtrip_tests.rs    # Read .intert → write → read → assert match
    ├── inter_compat_tests.rs # Cross-validate with the `inter` tool
    └── fixtures/             # Test fixtures
```

### Capabilities

1. ✅ **In-memory Inter tree** — data structures for packages, symbols,
   instructions, values, and types
2. ✅ **Textual Inter reader** — parse `.intert` files into the in-memory tree
3. ✅ **Textual Inter writer** — write the in-memory tree as `.intert` text
4. ✅ **Round-trip fidelity** — textual read → write → read produces identical
   trees; cross-validated against the `inter` tool

## Test Strategy

### Unit tests
- Core data structures (tree, symbols, values, instructions, types).
- Text escaping round-trip.

### Integration tests

- `roundtrip_tests.rs` — read every bundled `.intert` fixture, write it
  back, and parse the result; assert the final tree matches the first.

- `inter_compat_tests.rs` — feed our re-serialized text to the official
  `inter` tool and verify it parses without errors. This is the strongest
  practical oracle we have without building the C toolchain.

## Success Criteria

- [x] All bundled `.intert` fixtures round-trip through our textual
      reader/writer with identical output.
- [x] All bundled `.intert` fixtures (except `misc.intert`) are accepted by
      the existing `inter` tool after re-serialization.
- [x] Programmatic construction of Hello World produces correct output.
- [x] All tests pass on CI.

## Out of Scope

- Binary Inter reader/writer (deferred; the core pipeline emits textual Inter
  and hands off to the existing `inter` tool)
- Salsa integration (comes in the compiler driver)
- I7 parsing (comes in `conform7-syntax`)
- World model (comes in `conform7-semantics`)
- LSP (comes in `conform7-lsp`)
- Inter pipeline/optimization (handled by existing `inter` tool)
- Code generation to I6/C (handled by existing `inter` tool)
