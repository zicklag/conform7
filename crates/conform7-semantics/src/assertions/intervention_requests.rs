//! Intervention requests — processing INFORM6CODE_NT nodes during pass 2.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Intervention Requests.w`. It handles
//! Inform 6 code blocks that appear in the source text, converting them into
//! intervention requests for the runtime.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Intervention Requests.w`
//!
//! # Status
//!
//! **Stub** — deferred to a later plan.

use conform7_syntax::parse_node::ParseNode;

/// Intervention request processor.
pub struct InterventionRequests;

impl InterventionRequests {
    /// Make an intervention request from an INFORM6CODE_NT node (stub).
    ///
    /// In the C reference, this converts an Inform 6 code block into an
    /// intervention request that the runtime can execute. Currently a no-op.
    ///
    /// # References
    ///
    /// - C reference: `InterventionRequests::make` in
    ///   `inform7/assertions-module/Chapter 2/Intervention Requests.w`
    pub fn make(_node: &mut ParseNode) {
        // Deferred: intervention request processing
    }
}
