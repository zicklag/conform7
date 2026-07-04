//! Mapping hints module — stub for deferred implementation.
//!
//! This module corresponds to the `MappingHints` struct in the C reference
//! (`inform7/assertions-module/Chapter 2/Mapping Hints.w`). The real
//! implementation is deferred — it requires syntax tree traversal,
//! special meanings, Preform grammar, and a map plugin.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Mapping Hints.w`

/// Stub for the MappingHints struct.
///
/// The real implementation will traverse the syntax tree for map parameters
/// and build mapping hints from assertion sentences.
pub struct MappingHints;

#[allow(non_snake_case)]
impl MappingHints {
    /// Traverse the syntax tree for map parameters (stub).
    ///
    /// This is a no-op until the real implementation is wired up.
    pub fn traverse_for_map_parameters() {
        // Deferred: syntax tree traversal, special meanings, Preform grammar, map plugin
    }
}
