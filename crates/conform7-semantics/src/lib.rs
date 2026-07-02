//! Semantics for Inform 7 — the kind system foundation.
//!
//! This crate defines the core kind system data structures — `Kind` and
//! `KindConstructor` — along with construction functions, the kind lattice,
//! familiar kinds and constructors, and textual I/O.
//!
//! The kind system is the type system of Inform 7. Every assertion sentence,
//! every property, every instance, every relation ultimately depends on the
//! kind system.
//!
//! # Module Map
//!
//! | Module | C Reference | Purpose |
//! |--------|-------------|---------|
//! | [`kind_constructors`] | `Chapter 4/Kind Constructors.w` | `KindConstructor` struct and metadata |
//! | [`kinds`] | `Chapter 2/Kinds.w` | `Kind` struct, construction functions, equality |
//! | [`familiar_kinds`] | `Chapter 2/Familiar Kinds.w` | Global `K_*` and `CON_*` constants |
//! | [`lattice`] | `Chapter 2/The Lattice of Kinds.w` | Superkind hierarchy, conformance, join, meet |
//!
//! # References
//!
//! - C reference: `services/kinds-module/Chapter 2/Kinds.w`
//! - C reference: `services/kinds-module/Chapter 4/Kind Constructors.w`
//! - C reference: `services/kinds-module/Chapter 2/Familiar Kinds.w`
//! - C reference: `services/kinds-module/Chapter 2/The Lattice of Kinds.w`
//! - C reference: `services/kinds-module/Chapter 2/Describing Kinds.w`

pub mod kind_constructors;
pub mod kinds;
pub mod familiar_kinds;
pub mod lattice;
pub mod kinds_behaviour;
pub mod calculus;
pub mod knowledge;
