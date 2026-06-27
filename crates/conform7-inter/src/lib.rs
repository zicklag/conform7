//! Conform7 Inter — Read and write Inform 7 Intermediate Representation (Inter) files.
//!
//! This crate provides:
//! - An in-memory representation of Inter trees (packages, symbols, instructions)
//! - A textual Inter (`.intert`) reader and writer
//! - A binary Inter (`.interb`) reader and writer

pub mod binary;
pub mod instruction;
pub mod textual;
pub mod tree;
pub mod types;
pub mod value;

// Re-export key types
pub use instruction::{ConstructId, Instruction};
pub use tree::{InterTree, Package, PackageType, Symbol, SymbolType, SymbolsTable, WiringTarget};
pub use types::{InterType, Tid, TypeConstructor};
pub use value::{InterValue, ValueFormat};
