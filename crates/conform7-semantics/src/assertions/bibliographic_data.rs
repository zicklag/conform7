//! Bibliographic data — processing BIBLIOGRAPHIC_NT nodes during pass 2.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Bibliographic Data.w`. It handles
//! bibliographic data nodes that contain metadata about the source text
//! (title, author, etc.).
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Bibliographic Data.w`
//!
//! # Status
//!
//! **Stub** — deferred to a later plan.

use conform7_syntax::parse_node::ParseNode;

/// Bibliographic data processor.
pub struct BibliographicData;

impl BibliographicData {
    /// Process bibliographic data from a BIBLIOGRAPHIC_NT node (stub).
    ///
    /// In the C reference, this extracts bibliographic metadata (title,
    /// author, etc.) from the source text. Currently a no-op.
    ///
    /// # References
    ///
    /// - C reference: `BibliographicData::bibliographic_data` in
    ///   `inform7/assertions-module/Chapter 2/Bibliographic Data.w`
    pub fn bibliographic_data(_node: &mut ParseNode) {
        // Deferred: bibliographic data processing
    }
}
