# Future Performance Optimizations

This file tracks potential performance improvements identified during
development. These are not urgent — correctness and compatibility come first —
but are worth revisiting once the compiler is working end-to-end.

## Token text allocation (`conform7-syntax`)

**Status**: Noted
**Severity**: Medium (allocates per token)

`Token.text` is currently a heap-allocated `String`. For a typical I7 source
file with thousands of tokens, this means thousands of small heap allocations
during lexing.

**Options**:

1. **Borrow from source** — Add a lifetime parameter `Token<'a>` that borrows
   `&'a str` from the source string. This eliminates allocations entirely but
   adds lifetime complexity to the public API and makes it harder to store
   tokens in a Salsa database (which requires `'static`).

2. **String interning** — Use an interner (e.g., `lasso` or a simple
   `Vec<String>` arena) to deduplicate repeated tokens. Words like "the",
   "a", "is" appear many times and would share a single allocation. The
   interner can be `'static`-friendly, making it compatible with Salsa.

3. **Keep `String`** — The allocation cost may be negligible for typical
   source sizes (a few thousand tokens). Profile before optimizing.

## Byte-based column tracking (`conform7-syntax`)

**Status**: Noted
**Severity**: Low (LSP concern only)

Columns are byte-based, not UTF-16 code-unit based. The LSP protocol uses
UTF-16 code units for positions. A translation layer will be needed when
building the LSP server.

**Options**:

1. **Translate at LSP boundary** — Keep byte-based columns internally and
   convert to UTF-16 offsets only when constructing LSP `Position` values.
   This is the standard approach used by rust-analyzer and others.

2. **Track both** — Maintain parallel byte and UTF-16 column counters during
   lexing. Slightly more work during lexing but avoids a post-processing
   pass.

## Inter tree memory layout (`conform7-inter`)

**Status**: Noted
**Severity**: Low (not yet profiled)

The `InterTree` uses `Vec<Package>` with `Rc<RefCell<SymbolsTable>>` and
`Vec<PackageItem>` for children. This is a pointer-heavy representation
that may have cache locality issues for large trees.

**Options**:

1. **Arena allocation** — Store all nodes in a flat arena with index-based
   references instead of `Rc<RefCell<...>>`. This improves cache locality and
   reduces reference-counting overhead.

2. **Wait for profiling** — The current representation is simple and correct.
   Optimize only if profiling shows it's a bottleneck.

## Textual Inter I/O (`conform7-inter`)

**Status**: Noted
**Severity**: Low (not the hot path)

The textual Inter reader/writer in `textual.rs` (2664 lines) uses string
matching and allocation-heavy patterns. It's only used during compilation
(startup) and for debugging, so it's unlikely to be a bottleneck.

**Options**:

1. **Nothing** — It's fast enough for its use case.
2. **Binary Inter** — If we ever need to read/write Inter files in a hot
   loop (e.g., incremental compilation), a binary format would be faster.
   This was removed in Plan 1 (see `plans/PAST-1.md`).
