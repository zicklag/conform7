//! AUGMENT_CSEQ bench — augment model world with low-level properties.
//!
//! This module implements the AUGMENT_CSEQ bench from the C pipeline:
//!
//! ```c
//! @<Augment model world with low-level properties@> =
//!     BENCH(World::stage_V)                              // add stage ordering check
//!     BENCH(MappingHints::traverse_for_map_parameters)   // stub (deferred)
//! ```
//!
//! # References
//!
//! - C reference: `inform7/core-module/Chapter 1/Pass 3 of 3.w` (lines 165-169)
//! - Plan: PLAN-59

use crate::assertions::mapping_hints::MappingHints;
use crate::knowledge::world::World;

/// Run the AUGMENT_CSEQ bench.
///
/// Calls `World::stage_V` to advance the world-building stage, then
/// `MappingHints::traverse_for_map_parameters` (currently a no-op stub).
pub fn run_augment_cseq() {
    World::stage_V();
    MappingHints::traverse_for_map_parameters();
}
