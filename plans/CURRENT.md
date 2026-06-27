# Plan 1: Inter Binary Format Reader/Writer

**Status**: In Progress
**Started**: 2026-06-27
**Target**: 1-2 weeks

## Goal

Implement the `conform7-inter` crate — a Rust library that can read and write
Inter files (both textual `.intert` and binary `.interb` formats) with 100%
fidelity to the existing `inter` tool.

This is the compatibility linchpin: the Inter format is the handoff between
our Rust compiler and the existing C toolchain. If we can't produce
byte-identical Inter, nothing else matters.

## Why This First

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
│   ├── tree.rs          # InterTree, InterPackage, InterNode
│   ├── symbol.rs        # Symbol tables, interning
│   ├── instruction.rs   # Instruction constructors and types
│   │                    # (CONSTANT_IST, VAL_IST, INV_IST, PACKAGE_IST, etc.)
│   ├── value.rs         # Inter value pairs (two-word values)
│   ├── types.rs         # Inter type system (int32, int16, text, enum, struct, ...)
│   ├── binary.rs        # Binary .interb reader and writer
│   └── textual.rs       # Textual .intert reader and writer
└── tests/
    ├── roundtrip.rs     # Read .intert → write .interb → read back → assert match
    ├── hello.rs         # Construct Hello.intert programmatically, verify output
    └── fixtures/        # Test fixtures copied from inter/Tests/
        ├── hello.intert
        ├── packages.intert
        ├── misc.intert
        └── ...
```

### Capabilities

1. **In-memory Inter tree** — data structures for packages, symbols,
   instructions, values, and types
2. **Textual Inter reader** — parse `.intert` files into the in-memory tree
3. **Textual Inter writer** — write the in-memory tree as `.intert` text
4. **Binary Inter reader** — parse `.interb` files into the in-memory tree
5. **Binary Inter writer** — write the in-memory tree as `.interb` binary
6. **Round-trip fidelity** — read → write → read produces identical trees;
   textual → binary → textual produces identical text

## Test Strategy

### Test 1: Programmatic Construction

Construct the Hello World Inter tree in Rust and write it as textual Inter.
Verify the output matches the expected text (modulo auto-inserted
`packagetype`/`primitive` declarations).

```rust
#[test]
fn construct_hello_world() {
    let mut tree = InterTree::new();
    let main = tree.root().add_package("main", PackageType::Plain);
    let main_fn = main.add_package("Main", PackageType::Code);
    main_fn.add_code(|b| {
        b.inv_primitive("!enableprinting");
        b.inv_primitive("!print").val_text("Hello, world.\n");
    });

    let text = tree.to_text();
    // The inter tool auto-inserts packagetype/primitive declarations
    assert!(text.contains("package main _plain"));
    assert!(text.contains("package Main _code"));
    assert!(text.contains("inv !enableprinting"));
    assert!(text.contains(r#"val "Hello, world.\n""#));
}
```

### Test 2: Textual Round-Trip

Read every `.intert` file from `inter/Tests/Valid/` and `inter/Tests/Toys/`,
write it back as text, and assert the output matches the input.

### Test 3: Binary Round-Trip

Read a textual Inter file, write it as binary, use the existing `inter` tool
to convert the binary back to text, and assert the text matches the original.

```rust
#[test]
fn binary_roundtrip_hello() {
    let input = include_str!("fixtures/hello.intert");
    let tree = InterTree::from_text(input).unwrap();
    let binary = tree.to_binary().unwrap();

    // Write binary to temp file
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &binary).unwrap();

    // Use inter tool to convert back to text
    let output = std::process::Command::new("inter/Tangled/inter")
        .arg(tmp.path())
        .arg("-format=text")
        .output()
        .unwrap();

    let roundtripped = String::from_utf8(output.stdout).unwrap();
    // Compare (accounting for auto-inserted declarations)
    assert_inter_text_eq(input, &roundtripped);
}
```

### Test 4: Cross-Validation with inform7

For a simple I7 program, compile with `inform7` to get the expected Inter,
then construct the same Inter tree programmatically in Rust and assert
byte-identical binary output.

## Reference Material

- `gitignore/inform/inter/bytecode-module/` — Inter tree data structures,
  binary format, textual format
- `gitignore/inform/inter/building-module/` — Inter construction middleware
  (primitives, package types, conventions)
- `gitignore/inform/inter/Tests/Valid/` — Valid textual Inter test files
- `gitignore/inform/inter/Tests/Toys/` — Toy Inter programs with expected
  outputs
- [Inter Bytecode Module Docs](https://ganelson.github.io/inform/bytecode-module/P-wtmd.html)
- [Inter Building Module Docs](https://ganelson.github.io/inform/building-module/P-wtmd.html)

## Key Design Decisions

1. **The in-memory tree is our own design**, not a direct port of the C
   `inter_tree`. The C version is optimized for C idioms; we design for Rust
   ergonomics while maintaining semantic equivalence.

2. **Textual format first, binary second.** The textual format is
   human-readable and easier to debug. We implement it first, then binary.

3. **The existing `inter` tool is the source of truth.** If our output
   differs from what `inter` expects, we fix our code, not the other way
   around.

4. **No Salsa yet.** This crate is a pure data library with no incremental
   computation needs. Salsa comes in when we build the compiler driver.

## Success Criteria

- [ ] All `.intert` files from `inter/Tests/Valid/` round-trip through our
      textual reader/writer with identical output
- [ ] All `.intert` files from `inter/Tests/Toys/` round-trip through our
      textual reader/writer with identical output
- [ ] Binary output from our writer is accepted by the existing `inter` tool
- [ ] Binary → text round-trip through `inter` tool produces identical text
- [ ] Programmatic construction of Hello World produces correct output
- [ ] All tests pass on CI

## Out of Scope

- Salsa integration (comes in the compiler driver)
- I7 parsing (comes in `conform7-syntax`)
- World model (comes in `conform7-semantics`)
- LSP (comes in `conform7-lsp`)
- Inter pipeline/optimization (handled by existing `inter` tool)
- Code generation to I6/C (handled by existing `inter` tool)
