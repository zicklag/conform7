//! Conform7 Inter — Read and write Inform 7 Intermediate Representation (Inter) files.
//!
//! This crate is the compatibility linchpin of the Conform7 compiler. It provides
//! everything needed to construct, read, and write Inter programs — the
//! intermediate representation that sits between the Inform 7 frontend (parsing,
//! world model) and the backend (linking, optimization, code generation).
//!
//! # What is Inter?
//!
//! Inter is a tree-structured intermediate representation used by the Inform
//! toolchain. It's the "IR" in the standard compiler pipeline:
//!
//! ```text
//! I7 source → AST → World Model → Inter tree → pipeline → I6/C code
//! ```
//!
//! An Inter tree consists of:
//! - **Packages** — nested boxes that organize code and data (like modules)
//! - **Symbols** — named references to constants, variables, types, functions, etc.
//! - **Instructions** — bytecode-like operations (invoke primitive, load value, etc.)
//!
//! Inter exists in three forms:
//! - **In memory** — the [`InterTree`] data structure, cross-referenced for fast access
//! - **Textual** (`.intert`) — human-readable, tab-indented format
//! - **Binary** (`.interb`) — compressed, fast-loading format
//!
//! # Architecture
//!
//! This crate mirrors the structure of the C `bytecode` module from the Inform
//! source (`inter/bytecode-module/`). The key design decision is that our
//! in-memory representation is a Rust-idiomatic design, not a direct port of
//! the C data structures. We maintain semantic equivalence while leveraging
//! Rust's type system and ownership model.
//!
//! ## Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`types`] | `Inter Data Types.w` | Type constructors (int32, text, list, etc.) and TID encoding |
//! | [`value`] | `Inter Value Pairs.w` | Two-word value representation with 11 formats |
//! | [`instruction`] | `Inter Constructs.w` | 30+ instruction types and their frame structure |
//! | [`tree`] | `Inter Trees.w`, `Packages.w`, `Symbols Tables.w` | The core tree, package hierarchy, and symbol management |
//! | [`textual`] | `Inter in Text Files.w` | Tab-indented human-readable format |
//!
//! # Usage
//!
//! The typical flow for the Conform7 compiler will be:
//!
//! 1. Build an [`InterTree`] programmatically using the types in [`tree`]
//! 2. Emit instructions using [`Instruction`] constructors
//! 3. Write the tree as textual Inter via [`textual::write`]
//! 4. Hand off the `.intert` file to the existing `inter` tool for linking and codegen
//!
//! For testing, we also read Inter files back via [`textual::read`] to verify
//! round-trip fidelity against the reference `inform7` compiler output.


pub mod instruction;
pub mod textual;
pub mod tree;
pub mod types;
pub mod value;

// Re-export key types so consumers only need `use conform7_inter::*`
pub use instruction::{ConstructId, Instruction};
pub use tree::{InterTree, Package, PackageType, Symbol, SymbolType, SymbolsTable, WiringTarget};
pub use types::{InterType, Tid, TypeConstructor};
pub use value::{InterValue, ValueFormat};
