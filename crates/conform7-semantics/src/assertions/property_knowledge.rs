//! Property knowledge — tracking known properties of kinds.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 3/Property Knowledge.w`. It tracks
//! which properties are known to belong to which kinds, and provides
//! property-related queries for the assertion engine.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 3/Property Knowledge.w`
//!
//! # Status
//!
//! **Stub** — deferred to a later plan.

use conform7_syntax::parse_node::ParseNode;

/// Property knowledge tracker.
pub struct PropertyKnowledge;

impl PropertyKnowledge {
    /// Assert that a property applies to a kind (stub).
    pub fn assert_property(_owner: &mut ParseNode, _property: &mut ParseNode) {
        // Deferred: property knowledge tracking
    }

    /// Check if a property is known for a kind (stub).
    pub fn is_property_known(_node: &ParseNode) -> bool {
        // Deferred: property knowledge lookup
        false
    }
}
