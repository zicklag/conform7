# NOW — Lossless Inter Tree Refactor

**Status**: Complete — all success criteria met
**Completed**: 2026-06-27

## Why we are here

`conform7-inter` currently passes its tests and produces output the official
`inter` tool accepts, but it does **not** preserve the true structure of an
Inter tree. The biggest symptom is that the textual writer emits all
instructions before all child packages, so it cannot round-trip files where
`intert` interleaves them (e.g. `nesting.intert`, `linkage.intert`,
`externing.intert`, `typedfunction.intert`, `misc.intert`).

A second symptom is that the reader has semantic parsing bugs that were hidden
because the existing structural tests are too shallow. For example,
`propertyvalue P_strength of I_citrus = 20` is parsed as owner `P_strength`,
property `of`, and a synthetic value reference.

Both symptoms share one root cause: we built the parser and writer before we
finished the in-memory model. The model was shaped by what was easy to parse,
not by what Inter actually is.

This guide restarts the **foundation** of `conform7-inter`, not the whole crate.
We keep the correct constants, value formats, type constructors, and tests, and
we rebuild the tree and the I/O layers on top of a lossless data model.

## Core principle

The C reference (`inter/bytecode-module/Chapter 2/Inter Trees.w`,
`Packages.w`, `Symbols Tables.w`, `The Warehouse.w`, plus
`Chapter 3/Inter in Text Files.w`) treats an Inter tree as:

1. A hierarchy of **packages**.
2. Each package has an ordered sequence of **contents**: instructions and
   child packages intermixed.
3. Each package has a **symbols table** giving IDs to named entities.
4. The tree has a **warehouse** of interned strings and other resources.
5. Textual I/O is a thin layer over this model.

Our new Rust implementation must mirror that exactly.

## What we keep

- `crates/conform7-inter/src/instruction.rs` — `ConstructId`, `Instruction`
  frame layout, and unit tests are correct and match the C reference.
- `crates/conform7-inter/src/value.rs` — `ValueFormat` codes, `InterValue`,
  and escaping are correct.
- `crates/conform7-inter/src/types.rs` — `TypeConstructor`, `Tid`, and the
  constructor table are correct.
- The fixture files in `crates/conform7-inter/tests/fixtures/`.
- The oracle test harness in `inter_compat_tests.rs`, though its assertions
  will be tightened.

## What we change

### 1. `tree.rs` — a lossless, ordered package model

Replace the split `instructions` / `children` / `child_order` design with a
single ordered contents list:

```rust
/// One entry in a package's body.
///
/// A package's body is an ordered sequence of instructions and child
/// packages. This matches the textual Inter format, where lines at the
/// same indentation level can be either instructions or `package` declarations.
pub enum PackageItem {
    Instruction(Instruction),
    Child(Package),
}

pub struct Package {
    pub resource_id: u32,
    pub name: String,
    pub package_type: PackageType,
    pub type_marker: Option<u32>,
    pub symbols: SymbolsTable,
    pub items: Vec<PackageItem>,
    pub flags: u32,
}
```

Convenience APIs must support both programmatic construction and traversal:

```rust
impl Package {
    pub fn push_instruction(&mut self, instr: Instruction);
    pub fn push_child(&mut self, child: Package) -> &mut Package;
    pub fn instructions(&self) -> impl Iterator<Item = &Instruction>;
    pub fn instructions_mut(&mut self) -> impl Iterator<Item = &mut Instruction>;
    pub fn children(&self) -> impl Iterator<Item = &Package>;
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Package>;
    pub fn child(&self, name: &str) -> Option<&Package>;
    pub fn child_mut(&mut self, name: &str) -> Option<&mut Package>;
    pub fn find_child_mut(&mut self, url: &str) -> Option<&mut Package>;
}
```

`SymbolsTable` and `InterTree` remain conceptually similar, but add helper
methods for building trees:

```rust
impl InterTree {
    pub fn ensure_main_package(&mut self) -> &mut Package;
    pub fn insert_primitive(&mut self, name: &str) -> u32;
    pub fn insert_packagetype(&mut self, name: &str);
    pub fn intern_string(&mut self, s: &str) -> u32;
    pub fn find_package(&self, url: &str) -> Option<&Package>;
    pub fn find_package_mut(&mut self, url: &str) -> Option<&mut Package>;
}
```

The global symbol ID counter simplification is retained for now (globally
unique symbol IDs), with the existing documented note that this is a
convenience for textual fidelity, not binary fidelity.

### 2. `textual.rs` — writing from the new model

Rewrite `write_package` to iterate `items` in order, emitting each
`Instruction` or `Child` with the correct indentation. Once the model is
ordered, the writer becomes trivial and faithful.

Verify that `Hello.intert` and `packages.intert` still produce identical
normalized output, then extend identical-output assertions to any fixture
that does not depend on features we have not yet implemented.

### 3. `textual.rs` — reading into the new model

Rewrite `textual::read` so that every parsed line appends either:

- an `Instruction` to the current package's `items`, or
- a `Child` package when a `package` line is encountered.

Fix the semantic parsers that were wrong:

- **`propertyvalue`**: parse `propertyvalue <property> [of] <owner> = <value>`.
  Store owner symbol ID, property symbol ID, and the value pair.
- **`permission`**: parse `permission for <kind> to have <property>`.
  Store kind symbol ID and property symbol ID.
- **`pragma`**: parse `pragma <target> <value>`.
  Store both target string ID and value string ID.
- **`instance`**, **`property`**, **`variable`**, **`constant`**, **`local`**:
  parse optional multi-token type markers like `(list of int32)` or
  `(function int32 -> int32)`.

Keep the existing forward-reference resolution pass, but report unresolved
references instead of silently leaving `wired_to_name` set.

### 4. Tests — tighten assertions

- Keep the existing unit tests.
- Rewrite `roundtrip_tests.rs` to assert **structural** equivalence (same
  package/item order, same symbols, same instructions) and extend
  **identical-output** checks to every fixture that should support it.
- Add targeted tests for each fixed semantic parser:
  - `propertyvalue` with `of`
  - `permission` with `for ... to have`
  - `pragma` with value
  - multi-token type markers
  - I6 opcode `@glk`
- Move `misc.intert` into the `inter` oracle comparison once the semantic
  parsers are correct.

## Phase-by-phase execution

### Phase A — Lossless tree model

1. Rewrite `tree.rs` with `PackageItem`, ordered `items`, and builder APIs.
2. Add/keep unit tests for:
   - creating packages and pushing children/instructions
   - ordered iteration
   - symbol table behavior
   - string interning
3. Temporarily disable `textual.rs` tests that cannot compile until the
   parser/writer are updated.

### Phase B — intert output

4. Rewrite `textual.rs` writer on top of the new model.
5. Restore the simplest round-trip tests.
6. Verify `Hello.intert` and `packages.intert` byte-identical output.

### Phase C — intert input

7. Rewrite `textual.rs` reader to append to `items`.
8. Fix `propertyvalue`, `permission`, and `pragma` parsing.
9. Add multi-token type marker support.
10. Restore all structural round-trip tests.

### Phase D — Validation

11. Extend identical-output assertions to as many fixtures as possible.
12. Add `misc.intert` and any previously-excluded interleaved fixtures to
    the `inter` oracle comparison.
13. Run `cargo test` and `cargo clippy --all-targets` until clean.

## Success criteria

- [x] `tree.rs` stores package contents as a single ordered sequence that can
      represent instruction/child-package interleaving.
- [x] `textual::write` iterates that sequence in order.
- [x] `textual::read` populates that sequence in order.
- [x] `propertyvalue`, `permission`, and `pragma` are parsed with full
      arguments preserved.
- [x] Multi-token type markers are parsed correctly.
- [x] Every fixture round-trips structurally.
- [x] Every fixture that does not depend on unimplemented features passes
      byte-identical output comparison (modulo whitespace normalization).
- [x] `misc.intert` is accepted by the official `inter` tool after
      re-serialization.
- [x] `cargo test` and `cargo clippy --all-targets` are clean.

## Out of scope for this refactor

- Binary Inter reader/writer (still deferred).
- Full construct level/permission validation (`tree_lint` equivalent).
- Annotation **writer** — annotations are parsed and stored on `Instruction`
  but the writer does not emit them yet. This means annotations are preserved
  in memory but lost on re-serialization. Adding writer emission is the next
  step for full annotation round-trip.
- I6 numeric notation (`$7f`, `$$101`) and I6 real notation (`$+3.1415`).
- Semantic type checking or cross-referencing beyond what is needed for I/O.

These can be added in later plans once the lossless model is in place.

## Why this is the right next step

The current codebase has the right constants and the right test fixtures, but
the wrong shape for the data it is supposed to represent. By rebuilding the
foundation in the order the C reference uses — model first, then output, then
input — we get an API that is both correct for round-tripping and usable for
the future Inform 7 reimplementation. Every later feature (world model, Inter
emission, Salsa integration) will depend on this data model, so making it
right now pays off across the whole project.
