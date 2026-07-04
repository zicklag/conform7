//! Plugin calls — notifying plugins of new assertions.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Plugin Calls.w`. It provides hooks
//! for plugins to be notified when new assertions are processed.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Plugin Calls.w`

use conform7_syntax::parse_node::ParseNode;

/// Plugin notification system.
pub struct PluginCalls;

impl PluginCalls {
    /// Notify plugins of a new assertion (stub).
    ///
    /// Called after a sentence has been classified and diagrammed during the
    /// pre-pass. Gives plugins an opportunity to inspect or modify the
    /// assertion.
    ///
    /// # References
    ///
    /// - C reference: `inform7/assertions-module/Chapter 2/Plugin Calls.w` —
    ///   `PluginCalls::new_assertion_notify`
    pub fn new_assertion_notify(_node: &ParseNode) {
        // Deferred: plugin system
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn new_assertion_notify_does_not_panic() {
        let node = ParseNode::new(NodeType::Sentence, Wording::EMPTY);
        PluginCalls::new_assertion_notify(&node);
        // Should not panic
    }
}
