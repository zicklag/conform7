//! Special meanings — processing special-meaning assertions.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 3/Special Meanings.w`. It handles
//! assertions that have special meanings (plugin-defined assertions).
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 3/Special Meanings.w`
//!
//! # Status
//!
//! **Stub** — deferred to a later plan.

use conform7_syntax::parse_node::ParseNode;

/// Special meaning assertion processor.
pub struct SpecialMeanings;

impl SpecialMeanings {
    /// Try to process a special meaning assertion (stub).
    ///
    /// Returns true if the assertion was handled as a special meaning.
    pub fn try_special_meaning(_px: &mut ParseNode, _py: &mut ParseNode) -> bool {
        // Deferred: special meaning processing
        false
    }
}
