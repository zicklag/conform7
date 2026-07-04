//! The assertion matrix — dispatch and case handlers for assertion processing.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 3/Assertions.w`. It implements the
//! 12×12 assertion matrix that maps pairs of node types to case numbers,
//! and dispatches to the appropriate case handler.
//!
//! # The Assertion Matrix
//!
//! The matrix is a 12×12 grid mapping node type pairs to case numbers:
//!
//! ```text
//!                    AND  WITH  XoY  KIND  ALLOW  PLIST  ADJ  ACTN  REL  EVRY  CN   PN
//! AND_NT           {  1,   2,   1,   1,   1,   1,   1,   1,   1,  16,   1,   1 }
//! WITH_NT          {  3,   4,   3,   3,   3,   3,   3,   3,  14,  16,   3,   3 }
//! X_OF_Y_NT        {  5,   2,   6,   7,   9,   7,   7,   7,  20,  16,  23,   7 }
//! KIND_NT          {  5,   2,   8,   8,   9,   8,   8,   8,   8,  16,   8,   8 }
//! ALLOWED_NT       {  5,   2,  10,  10,  9,  10,  10,  10,  10,  25,  25,  25 }
//! PROPERTY_LIST_NT {  5,   2,  11,  12,  9,  18,  22,  19,  20,  16,  18,  18 }
//! ADJECTIVE_NT     {  5,   2,  13,  12,  9,  22,  22,  24,  20,  16,  29,  29 }
//! ACTION_NT        {  5,   2,  11,  19,  9,  19,  19,  27,  20,  16,  32,  32 }
//! RELATIONSHIP_NT  {  5,  15,  21,  20,  9,  20,  42,  20,  28,  31,  34,  36 }
//! EVERY_NT         { 17,  17,  17,  17, 17,  17,  17,  17,  17,  33,  17,  17 }
//! COMMON_NOUN_NT   {  5,   2,  11,  12,  9,  18,  30,  19,  35,  16,  38,  39 }
//! PROPER_NOUN_NT   {  5,   2,  26,  12,  9,  18,  30,  19,  37,  16,  40,  41 }
//! ```
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 3/Assertions.w`
//!
//! # Status
//!
//! Cases 1, 3, 5 are implemented (recursive splitting). All other cases are
//! stubs that will be implemented in later plans.

use conform7_syntax::node_type::NodeType;
use conform7_syntax::parse_node::ParseNode;

/// The assertion matrix — dispatches assertion processing.
pub struct Assertions;

impl Assertions {
    // ── Matrix lookup ──────────────────────────────────────────────────────

    /// The 12×12 assertion matrix.
    ///
    /// Maps (px_node_type, py_node_type) to a case number (1–42).
    /// The row/column order is:
    ///   0: AND_NT, 1: WITH_NT, 2: X_OF_Y_NT, 3: KIND_NT, 4: ALLOWED_NT,
    ///   5: PROPERTY_LIST_NT, 6: ADJECTIVE_NT, 7: ACTION_NT, 8: RELATIONSHIP_NT,
    ///   9: EVERY_NT, 10: COMMON_NOUN_NT, 11: PROPER_NOUN_NT
    const MATRIX: [[u8; 12]; 12] = [
        // AND    WITH   XoY    KIND   ALLOW  PLIST  ADJ    ACTN   REL    EVRY   CN     PN
        [1,     2,     1,     1,     1,     1,     1,     1,     1,     16,    1,     1],  // AND
        [3,     4,     3,     3,     3,     3,     3,     3,     14,    16,    3,     3],  // WITH
        [5,     2,     6,     7,     9,     7,     7,     7,     20,    16,    23,    7],  // X_OF_Y
        [5,     2,     8,     8,     9,     8,     8,     8,     8,     16,    8,     8],  // KIND
        [5,     2,     10,    10,    9,     10,    10,    10,    10,    25,    25,    25], // ALLOWED
        [5,     2,     11,    12,    9,     18,    22,    19,    20,    16,    18,    18], // PROPERTY_LIST
        [5,     2,     13,    12,    9,     22,    22,    24,    20,    16,    29,    29], // ADJECTIVE
        [5,     2,     11,    19,    9,     19,    19,    27,    20,    16,    32,    32], // ACTION
        [5,     15,    21,    20,    9,     20,    42,    20,    28,    31,    34,    36], // RELATIONSHIP
        [17,    17,    17,    17,    17,    17,    17,    17,    17,    33,    17,    17], // EVERY
        [5,     2,     11,    12,    9,     18,    30,    19,    35,    16,    38,    39], // COMMON_NOUN
        [5,     2,     26,    12,    9,     18,    30,    19,    37,    16,    40,    41], // PROPER_NOUN
    ];

    /// Map a node type to its matrix column/row index (0–11).
    ///
    /// Returns `None` if the node type is not one of the 12 assertion types.
    fn type_index(nt: NodeType) -> Option<usize> {
        match nt {
            NodeType::And => Some(0),
            NodeType::With => Some(1),
            NodeType::XOfY => Some(2),
            NodeType::Kind => Some(3),
            NodeType::Allowed => Some(4),
            NodeType::PropertyList => Some(5),
            NodeType::Adjective => Some(6),
            NodeType::Action => Some(7),
            NodeType::Relationship => Some(8),
            NodeType::Every => Some(9),
            NodeType::CommonNoun => Some(10),
            NodeType::ProperNoun => Some(11),
            _ => None,
        }
    }

    /// Look up the case number for a pair of node types.
    ///
    /// Returns the case number (1–42) for the given (px, py) pair.
    /// If either node type is not in the matrix, returns 0 (no case).
    pub fn which_assertion_case(px: &ParseNode, py: &ParseNode) -> u8 {
        let Some(ix) = Self::type_index(px.node_type()) else {
            return 0;
        };
        let Some(iy) = Self::type_index(py.node_type()) else {
            return 0;
        };
        Self::MATRIX[ix][iy]
    }

    // ── Entry point ───────────────────────────────────────────────────────

    /// Make a coupling assertion between px and py.
    ///
    /// This is the entry point for assertion processing. It looks up the
    /// case number in the matrix and dispatches to the appropriate handler.
    ///
    /// # References
    ///
    /// - C reference: `Assertions::make_coupling` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    pub fn make_coupling(px: &mut ParseNode, py: &mut ParseNode) {
        let case = Self::which_assertion_case(px, py);
        Self::make_assertion_recursive_inner(px, py, case);
    }

    /// Make an appearance assertion for a textual sentence (stub).
    ///
    /// This is called during pass 2 for textual sentences (sentences that
    /// contain only quoted text with no structural assertions).
    ///
    /// # References
    ///
    /// - C reference: `Assertions::make_appearance` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    pub fn make_appearance(_node: &mut ParseNode) {
        // Deferred: appearance assertion processing
    }

    /// Make a coupling assertion during pass 2.
    ///
    /// Like `make_coupling`, but with pass-2-specific guards:
    /// 1. Rejects three forms of assertion (stub problem messages)
    /// 2. Case 8 returns early
    ///
    /// # References
    ///
    /// - C reference: `@<Reject three forms of assertion@>` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    pub fn make_coupling_pass_2(px: &mut ParseNode, py: &mut ParseNode) {
        // @<Reject three forms of assertion@> — stub guards
        Self::reject_three_forms(px, py);

        let case = Self::which_assertion_case(px, py);

        // Case 8 returns early on pass 2
        if case == 8 {
            return;
        }

        Self::make_assertion_recursive_inner(px, py, case);
    }

    /// Reject three forms of assertion during pass 2 (stub).
    ///
    /// In the C reference, this emits problem messages for three assertion
    /// forms that are not allowed during pass 2. Currently a stub.
    ///
    /// # References
    ///
    /// - C reference: `@<Reject three forms of assertion@>` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    fn reject_three_forms(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: emit problem messages for disallowed assertion forms
    }

    // ── Dispatch ───────────────────────────────────────────────────────────

    /// Dispatch to the appropriate case handler.
    ///
    /// This is the recursive dispatch function. It handles the case number
    /// and calls the appropriate handler.
    ///
    /// # References
    ///
    /// - C reference: `Assertions::make_assertion_recursive_inner` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    fn make_assertion_recursive_inner(px: &mut ParseNode, py: &mut ParseNode, case: u8) {
        match case {
            1 => Self::case_1(px, py),
            2 => Self::case_2(px, py),
            3 => Self::case_3(px, py),
            4 => Self::case_4(px, py),
            5 => Self::case_5(px, py),
            6 => Self::case_6(px, py),
            7 => Self::case_7(px, py),
            8 => Self::case_8(px, py),
            9 => Self::case_9(px, py),
            10 => Self::case_10(px, py),
            11 => Self::case_11(px, py),
            12 => Self::case_12(px, py),
            13 => Self::case_13(px, py),
            14 => Self::case_14(px, py),
            15 => Self::case_15(px, py),
            16 => Self::case_16(px, py),
            17 => Self::case_17(px, py),
            18 => Self::case_18(px, py),
            19 => Self::case_19(px, py),
            20 => Self::case_20(px, py),
            21 => Self::case_21(px, py),
            22 => Self::case_22(px, py),
            23 => Self::case_23(px, py),
            24 => Self::case_24(px, py),
            25 => Self::case_25(px, py),
            26 => Self::case_26(px, py),
            27 => Self::case_27(px, py),
            28 => Self::case_28(px, py),
            29 => Self::case_29(px, py),
            30 => Self::case_30(px, py),
            31 => Self::case_31(px, py),
            32 => Self::case_32(px, py),
            33 => Self::case_33(px, py),
            34 => Self::case_34(px, py),
            35 => Self::case_35(px, py),
            36 => Self::case_36(px, py),
            37 => Self::case_37(px, py),
            38 => Self::case_38(px, py),
            39 => Self::case_39(px, py),
            40 => Self::case_40(px, py),
            41 => Self::case_41(px, py),
            42 => Self::case_42(px, py),
            _ => {
                // Unknown case — no-op
            }
        }
    }

    // ── Helper: allow_node_type ────────────────────────────────────────────

    /// Check if a node type is allowed on either side of an assertion.
    ///
    /// Returns true if the node type has the `ASSERT_NFLAG` flag set.
    ///
    /// # References
    ///
    /// - C reference: `Assertions::allow_node_type` in
    ///   `inform7/assertions-module/Chapter 3/Assertions.w`
    pub fn allow_node_type(p: &ParseNode) -> bool {
        p.node_type().is_assert()
    }

    // ── Case handlers ──────────────────────────────────────────────────────

    /// Case 1: px is AND — split px into sub-assertions.
    ///
    /// For each child of the AND node (px), call make_coupling with that
    /// child and py.
    fn case_1(px: &mut ParseNode, py: &mut ParseNode) {
        // Take children of px (the AND node) and make separate assertions
        let children = px.take_children();
        for mut child in children {
            Self::make_coupling(&mut child, py);
        }
    }

    /// Case 2: px is AND, py is WITH — stub.
    fn case_2(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: AND + WITH combination
    }

    /// Case 3: px is WITH — split px into sub-assertions.
    ///
    /// For each child of the WITH node (px), call make_coupling with that
    /// child and py.
    fn case_3(px: &mut ParseNode, py: &mut ParseNode) {
        // Take children of px (the WITH node) and make separate assertions
        let children = px.take_children();
        for mut child in children {
            Self::make_coupling(&mut child, py);
        }
    }

    /// Case 4: px is WITH, py is WITH — stub.
    fn case_4(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: WITH + WITH combination
    }

    /// Case 5: py is AND — split py into sub-assertions.
    ///
    /// For each child of the AND node (py), call make_coupling with px and
    /// that child.
    fn case_5(px: &mut ParseNode, py: &mut ParseNode) {
        // Take children of py (the AND node) and make separate assertions
        let children = py.take_children();
        for mut child in children {
            Self::make_coupling(px, &mut child);
        }
    }

    /// Case 6: X_OF_Y + X_OF_Y — stub.
    fn case_6(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: X_OF_Y + X_OF_Y
    }

    /// Case 7: X_OF_Y + (KIND|ALLOWED|PLIST|ADJ|ACTN|CN|PN) — stub.
    fn case_7(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: X_OF_Y + noun/kind
    }

    /// Case 8: KIND + (XoY|KIND|PLIST|ADJ|ACTN|CN|PN) — stub.
    fn case_8(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: KIND + noun/kind
    }

    /// Case 9: (XoY|KIND|ALLOW|PLIST|ADJ|ACTN|REL|CN|PN) + ALLOWED — stub.
    fn case_9(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ALLOWED assertions
    }

    /// Case 10: ALLOWED + (XoY|KIND|PLIST|ADJ|ACTN|REL) — stub.
    fn case_10(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ALLOWED + noun/kind
    }

    /// Case 11: (PLIST|ACTN|CN) + X_OF_Y — stub.
    fn case_11(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPERTY_LIST/ACTION/COMMON_NOUN + X_OF_Y
    }

    /// Case 12: (PLIST|ADJ|CN|PN) + KIND — stub.
    fn case_12(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPERTY_LIST/ADJECTIVE/COMMON_NOUN/PROPER_NOUN + KIND
    }

    /// Case 13: ADJECTIVE + X_OF_Y — stub.
    fn case_13(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ADJECTIVE + X_OF_Y
    }

    /// Case 14: WITH + RELATIONSHIP — stub.
    fn case_14(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: WITH + RELATIONSHIP
    }

    /// Case 15: RELATIONSHIP + WITH — stub.
    fn case_15(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + WITH
    }

    /// Case 16: (AND|WITH|XoY|KIND|PLIST|ADJ|ACTN|CN|PN) + EVERY — stub.
    fn case_16(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: EVERY assertions
    }

    /// Case 17: EVERY + (AND|WITH|XoY|KIND|ALLOW|PLIST|ADJ|ACTN|REL|CN|PN) — stub.
    fn case_17(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: EVERY + noun/kind
    }

    /// Case 18: (PLIST|CN|PN) + PROPERTY_LIST — stub.
    fn case_18(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPERTY_LIST assertions
    }

    /// Case 19: (PLIST|ACTN|CN|PN) + ACTION — stub.
    fn case_19(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ACTION assertions
    }

    /// Case 20: (XoY|PLIST|ADJ|ACTN|REL|CN|PN) + RELATIONSHIP — stub.
    fn case_20(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP assertions
    }

    /// Case 21: RELATIONSHIP + X_OF_Y — stub.
    fn case_21(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + X_OF_Y
    }

    /// Case 22: (PLIST|ADJ) + ADJECTIVE — stub.
    fn case_22(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ADJECTIVE assertions
    }

    /// Case 23: X_OF_Y + COMMON_NOUN — stub.
    fn case_23(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: X_OF_Y + COMMON_NOUN
    }

    /// Case 24: ADJECTIVE + ACTION — stub.
    fn case_24(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ADJECTIVE + ACTION
    }

    /// Case 25: ALLOWED + (EVERY|CN|PN) — stub.
    fn case_25(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ALLOWED + EVERY/COMMON_NOUN/PROPER_NOUN
    }

    /// Case 26: X_OF_Y + PROPER_NOUN — stub.
    fn case_26(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: X_OF_Y + PROPER_NOUN
    }

    /// Case 27: ACTION + ACTION — stub.
    fn case_27(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ACTION + ACTION
    }

    /// Case 28: RELATIONSHIP + RELATIONSHIP — stub.
    fn case_28(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + RELATIONSHIP
    }

    /// Case 29: ADJECTIVE + (CN|PN) — stub.
    fn case_29(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ADJECTIVE + COMMON_NOUN/PROPER_NOUN
    }

    /// Case 30: (CN|PN) + ADJECTIVE — stub.
    fn case_30(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: COMMON_NOUN/PROPER_NOUN + ADJECTIVE
    }

    /// Case 31: RELATIONSHIP + EVERY — stub.
    fn case_31(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + EVERY
    }

    /// Case 32: ACTION + (CN|PN) — stub.
    fn case_32(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: ACTION + COMMON_NOUN/PROPER_NOUN
    }

    /// Case 33: EVERY + EVERY — stub.
    fn case_33(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: EVERY + EVERY
    }

    /// Case 34: RELATIONSHIP + COMMON_NOUN — stub.
    fn case_34(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + COMMON_NOUN
    }

    /// Case 35: COMMON_NOUN + RELATIONSHIP — stub.
    fn case_35(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: COMMON_NOUN + RELATIONSHIP
    }

    /// Case 36: RELATIONSHIP + PROPER_NOUN — stub.
    fn case_36(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + PROPER_NOUN
    }

    /// Case 37: PROPER_NOUN + RELATIONSHIP — stub.
    fn case_37(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPER_NOUN + RELATIONSHIP
    }

    /// Case 38: COMMON_NOUN + COMMON_NOUN — stub.
    fn case_38(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: COMMON_NOUN + COMMON_NOUN
    }

    /// Case 39: COMMON_NOUN + PROPER_NOUN — stub.
    fn case_39(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: COMMON_NOUN + PROPER_NOUN
    }

    /// Case 40: PROPER_NOUN + COMMON_NOUN — stub.
    fn case_40(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPER_NOUN + COMMON_NOUN
    }

    /// Case 41: PROPER_NOUN + PROPER_NOUN — stub.
    fn case_41(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: PROPER_NOUN + PROPER_NOUN
    }

    /// Case 42: RELATIONSHIP + ADJECTIVE — stub.
    fn case_42(_px: &mut ParseNode, _py: &mut ParseNode) {
        // Deferred: RELATIONSHIP + ADJECTIVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::Wording;

    // ── Matrix lookup tests ────────────────────────────────────────────────

    /// Helper to create a node of a given type.
    fn node(nt: NodeType) -> ParseNode {
        ParseNode::new(nt, Wording::EMPTY)
    }

    /// Helper to assert a matrix entry.
    fn assert_case(px: NodeType, py: NodeType, expected: u8) {
        let mut px_node = node(px);
        let mut py_node = node(py);
        assert_eq!(
            Assertions::which_assertion_case(&px_node, &py_node),
            expected,
            "Mismatch for ({:?}, {:?})",
            px,
            py
        );
        // Also verify that make_coupling doesn't panic
        Assertions::make_coupling(&mut px_node, &mut py_node);
    }

    #[test]
    fn matrix_and_row() {
        // AND row
        assert_case(NodeType::And, NodeType::And, 1);
        assert_case(NodeType::And, NodeType::With, 2);
        assert_case(NodeType::And, NodeType::XOfY, 1);
        assert_case(NodeType::And, NodeType::Kind, 1);
        assert_case(NodeType::And, NodeType::Allowed, 1);
        assert_case(NodeType::And, NodeType::PropertyList, 1);
        assert_case(NodeType::And, NodeType::Adjective, 1);
        assert_case(NodeType::And, NodeType::Action, 1);
        assert_case(NodeType::And, NodeType::Relationship, 1);
        assert_case(NodeType::And, NodeType::Every, 16);
        assert_case(NodeType::And, NodeType::CommonNoun, 1);
        assert_case(NodeType::And, NodeType::ProperNoun, 1);
    }

    #[test]
    fn matrix_with_row() {
        assert_case(NodeType::With, NodeType::And, 3);
        assert_case(NodeType::With, NodeType::With, 4);
        assert_case(NodeType::With, NodeType::XOfY, 3);
        assert_case(NodeType::With, NodeType::Kind, 3);
        assert_case(NodeType::With, NodeType::Allowed, 3);
        assert_case(NodeType::With, NodeType::PropertyList, 3);
        assert_case(NodeType::With, NodeType::Adjective, 3);
        assert_case(NodeType::With, NodeType::Action, 3);
        assert_case(NodeType::With, NodeType::Relationship, 14);
        assert_case(NodeType::With, NodeType::Every, 16);
        assert_case(NodeType::With, NodeType::CommonNoun, 3);
        assert_case(NodeType::With, NodeType::ProperNoun, 3);
    }

    #[test]
    fn matrix_x_of_y_row() {
        assert_case(NodeType::XOfY, NodeType::And, 5);
        assert_case(NodeType::XOfY, NodeType::With, 2);
        assert_case(NodeType::XOfY, NodeType::XOfY, 6);
        assert_case(NodeType::XOfY, NodeType::Kind, 7);
        assert_case(NodeType::XOfY, NodeType::Allowed, 9);
        assert_case(NodeType::XOfY, NodeType::PropertyList, 7);
        assert_case(NodeType::XOfY, NodeType::Adjective, 7);
        assert_case(NodeType::XOfY, NodeType::Action, 7);
        assert_case(NodeType::XOfY, NodeType::Relationship, 20);
        assert_case(NodeType::XOfY, NodeType::Every, 16);
        assert_case(NodeType::XOfY, NodeType::CommonNoun, 23);
        assert_case(NodeType::XOfY, NodeType::ProperNoun, 7);
    }

    #[test]
    fn matrix_kind_row() {
        assert_case(NodeType::Kind, NodeType::And, 5);
        assert_case(NodeType::Kind, NodeType::With, 2);
        assert_case(NodeType::Kind, NodeType::XOfY, 8);
        assert_case(NodeType::Kind, NodeType::Kind, 8);
        assert_case(NodeType::Kind, NodeType::Allowed, 9);
        assert_case(NodeType::Kind, NodeType::PropertyList, 8);
        assert_case(NodeType::Kind, NodeType::Adjective, 8);
        assert_case(NodeType::Kind, NodeType::Action, 8);
        assert_case(NodeType::Kind, NodeType::Relationship, 8);
        assert_case(NodeType::Kind, NodeType::Every, 16);
        assert_case(NodeType::Kind, NodeType::CommonNoun, 8);
        assert_case(NodeType::Kind, NodeType::ProperNoun, 8);
    }

    #[test]
    fn matrix_allowed_row() {
        assert_case(NodeType::Allowed, NodeType::And, 5);
        assert_case(NodeType::Allowed, NodeType::With, 2);
        assert_case(NodeType::Allowed, NodeType::XOfY, 10);
        assert_case(NodeType::Allowed, NodeType::Kind, 10);
        assert_case(NodeType::Allowed, NodeType::Allowed, 9);
        assert_case(NodeType::Allowed, NodeType::PropertyList, 10);
        assert_case(NodeType::Allowed, NodeType::Adjective, 10);
        assert_case(NodeType::Allowed, NodeType::Action, 10);
        assert_case(NodeType::Allowed, NodeType::Relationship, 10);
        assert_case(NodeType::Allowed, NodeType::Every, 25);
        assert_case(NodeType::Allowed, NodeType::CommonNoun, 25);
        assert_case(NodeType::Allowed, NodeType::ProperNoun, 25);
    }

    #[test]
    fn matrix_property_list_row() {
        assert_case(NodeType::PropertyList, NodeType::And, 5);
        assert_case(NodeType::PropertyList, NodeType::With, 2);
        assert_case(NodeType::PropertyList, NodeType::XOfY, 11);
        assert_case(NodeType::PropertyList, NodeType::Kind, 12);
        assert_case(NodeType::PropertyList, NodeType::Allowed, 9);
        assert_case(NodeType::PropertyList, NodeType::PropertyList, 18);
        assert_case(NodeType::PropertyList, NodeType::Adjective, 22);
        assert_case(NodeType::PropertyList, NodeType::Action, 19);
        assert_case(NodeType::PropertyList, NodeType::Relationship, 20);
        assert_case(NodeType::PropertyList, NodeType::Every, 16);
        assert_case(NodeType::PropertyList, NodeType::CommonNoun, 18);
        assert_case(NodeType::PropertyList, NodeType::ProperNoun, 18);
    }

    #[test]
    fn matrix_adjective_row() {
        assert_case(NodeType::Adjective, NodeType::And, 5);
        assert_case(NodeType::Adjective, NodeType::With, 2);
        assert_case(NodeType::Adjective, NodeType::XOfY, 13);
        assert_case(NodeType::Adjective, NodeType::Kind, 12);
        assert_case(NodeType::Adjective, NodeType::Allowed, 9);
        assert_case(NodeType::Adjective, NodeType::PropertyList, 22);
        assert_case(NodeType::Adjective, NodeType::Adjective, 22);
        assert_case(NodeType::Adjective, NodeType::Action, 24);
        assert_case(NodeType::Adjective, NodeType::Relationship, 20);
        assert_case(NodeType::Adjective, NodeType::Every, 16);
        assert_case(NodeType::Adjective, NodeType::CommonNoun, 29);
        assert_case(NodeType::Adjective, NodeType::ProperNoun, 29);
    }

    #[test]
    fn matrix_action_row() {
        assert_case(NodeType::Action, NodeType::And, 5);
        assert_case(NodeType::Action, NodeType::With, 2);
        assert_case(NodeType::Action, NodeType::XOfY, 11);
        assert_case(NodeType::Action, NodeType::Kind, 19);
        assert_case(NodeType::Action, NodeType::Allowed, 9);
        assert_case(NodeType::Action, NodeType::PropertyList, 19);
        assert_case(NodeType::Action, NodeType::Adjective, 19);
        assert_case(NodeType::Action, NodeType::Action, 27);
        assert_case(NodeType::Action, NodeType::Relationship, 20);
        assert_case(NodeType::Action, NodeType::Every, 16);
        assert_case(NodeType::Action, NodeType::CommonNoun, 32);
        assert_case(NodeType::Action, NodeType::ProperNoun, 32);
    }

    #[test]
    fn matrix_relationship_row() {
        assert_case(NodeType::Relationship, NodeType::And, 5);
        assert_case(NodeType::Relationship, NodeType::With, 15);
        assert_case(NodeType::Relationship, NodeType::XOfY, 21);
        assert_case(NodeType::Relationship, NodeType::Kind, 20);
        assert_case(NodeType::Relationship, NodeType::Allowed, 9);
        assert_case(NodeType::Relationship, NodeType::PropertyList, 20);
        assert_case(NodeType::Relationship, NodeType::Adjective, 42);
        assert_case(NodeType::Relationship, NodeType::Action, 20);
        assert_case(NodeType::Relationship, NodeType::Relationship, 28);
        assert_case(NodeType::Relationship, NodeType::Every, 31);
        assert_case(NodeType::Relationship, NodeType::CommonNoun, 34);
        assert_case(NodeType::Relationship, NodeType::ProperNoun, 36);
    }

    #[test]
    fn matrix_every_row() {
        assert_case(NodeType::Every, NodeType::And, 17);
        assert_case(NodeType::Every, NodeType::With, 17);
        assert_case(NodeType::Every, NodeType::XOfY, 17);
        assert_case(NodeType::Every, NodeType::Kind, 17);
        assert_case(NodeType::Every, NodeType::Allowed, 17);
        assert_case(NodeType::Every, NodeType::PropertyList, 17);
        assert_case(NodeType::Every, NodeType::Adjective, 17);
        assert_case(NodeType::Every, NodeType::Action, 17);
        assert_case(NodeType::Every, NodeType::Relationship, 17);
        assert_case(NodeType::Every, NodeType::Every, 33);
        assert_case(NodeType::Every, NodeType::CommonNoun, 17);
        assert_case(NodeType::Every, NodeType::ProperNoun, 17);
    }

    #[test]
    fn matrix_common_noun_row() {
        assert_case(NodeType::CommonNoun, NodeType::And, 5);
        assert_case(NodeType::CommonNoun, NodeType::With, 2);
        assert_case(NodeType::CommonNoun, NodeType::XOfY, 11);
        assert_case(NodeType::CommonNoun, NodeType::Kind, 12);
        assert_case(NodeType::CommonNoun, NodeType::Allowed, 9);
        assert_case(NodeType::CommonNoun, NodeType::PropertyList, 18);
        assert_case(NodeType::CommonNoun, NodeType::Adjective, 30);
        assert_case(NodeType::CommonNoun, NodeType::Action, 19);
        assert_case(NodeType::CommonNoun, NodeType::Relationship, 35);
        assert_case(NodeType::CommonNoun, NodeType::Every, 16);
        assert_case(NodeType::CommonNoun, NodeType::CommonNoun, 38);
        assert_case(NodeType::CommonNoun, NodeType::ProperNoun, 39);
    }

    #[test]
    fn matrix_proper_noun_row() {
        assert_case(NodeType::ProperNoun, NodeType::And, 5);
        assert_case(NodeType::ProperNoun, NodeType::With, 2);
        assert_case(NodeType::ProperNoun, NodeType::XOfY, 26);
        assert_case(NodeType::ProperNoun, NodeType::Kind, 12);
        assert_case(NodeType::ProperNoun, NodeType::Allowed, 9);
        assert_case(NodeType::ProperNoun, NodeType::PropertyList, 18);
        assert_case(NodeType::ProperNoun, NodeType::Adjective, 30);
        assert_case(NodeType::ProperNoun, NodeType::Action, 19);
        assert_case(NodeType::ProperNoun, NodeType::Relationship, 37);
        assert_case(NodeType::ProperNoun, NodeType::Every, 16);
        assert_case(NodeType::ProperNoun, NodeType::CommonNoun, 40);
        assert_case(NodeType::ProperNoun, NodeType::ProperNoun, 41);
    }

    #[test]
    fn matrix_unknown_type_returns_zero() {
        let mut px = node(NodeType::Sentence);
        let mut py = node(NodeType::Verb);
        assert_eq!(Assertions::which_assertion_case(&px, &py), 0);
        Assertions::make_coupling(&mut px, &mut py);
    }

    // ── allow_node_type tests ─────────────────────────────────────────────

    #[test]
    fn allow_node_type_assert_types() {
        assert!(Assertions::allow_node_type(&node(NodeType::And)));
        assert!(Assertions::allow_node_type(&node(NodeType::With)));
        assert!(Assertions::allow_node_type(&node(NodeType::XOfY)));
        assert!(Assertions::allow_node_type(&node(NodeType::Kind)));
        assert!(Assertions::allow_node_type(&node(NodeType::Allowed)));
        assert!(Assertions::allow_node_type(&node(NodeType::PropertyList)));
        assert!(Assertions::allow_node_type(&node(NodeType::Adjective)));
        assert!(Assertions::allow_node_type(&node(NodeType::Action)));
        assert!(Assertions::allow_node_type(&node(NodeType::Relationship)));
        assert!(Assertions::allow_node_type(&node(NodeType::Every)));
        assert!(Assertions::allow_node_type(&node(NodeType::CommonNoun)));
        assert!(Assertions::allow_node_type(&node(NodeType::ProperNoun)));
    }

    #[test]
    fn allow_node_type_non_assert_types() {
        assert!(!Assertions::allow_node_type(&node(NodeType::Sentence)));
        assert!(!Assertions::allow_node_type(&node(NodeType::Verb)));
        assert!(!Assertions::allow_node_type(&node(NodeType::Root)));
        assert!(!Assertions::allow_node_type(&node(NodeType::Heading)));
    }

    // ── Case 1: AND split on px ────────────────────────────────────────────

    #[test]
    fn case_1_splits_and_on_px() {
        // px = AND(child1, child2), py = ProperNoun
        // After case 1, we should have made two assertions: (child1, py) and (child2, py)
        // Since all other cases are stubs, this just verifies no panic and that
        // the children are consumed.
        let mut px = ParseNode::new(NodeType::And, Wording::EMPTY);
        px.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        px.append_child(ParseNode::new(NodeType::CommonNoun, Wording::EMPTY));
        let mut py = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);

        Assertions::make_coupling(&mut px, &mut py);

        // Children should have been taken from px
        assert_eq!(px.child_count(), 0);
    }

    // ── Case 3: WITH split on px ───────────────────────────────────────────

    #[test]
    fn case_3_splits_with_on_px() {
        let mut px = ParseNode::new(NodeType::With, Wording::EMPTY);
        px.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        px.append_child(ParseNode::new(NodeType::CommonNoun, Wording::EMPTY));
        let mut py = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);

        Assertions::make_coupling(&mut px, &mut py);

        // Children should have been taken from px
        assert_eq!(px.child_count(), 0);
    }

    // ── Case 5: AND split on py ────────────────────────────────────────────

    #[test]
    fn case_5_splits_and_on_py() {
        let mut px = ParseNode::new(NodeType::ProperNoun, Wording::EMPTY);
        let mut py = ParseNode::new(NodeType::And, Wording::EMPTY);
        py.append_child(ParseNode::new(NodeType::ProperNoun, Wording::EMPTY));
        py.append_child(ParseNode::new(NodeType::CommonNoun, Wording::EMPTY));

        Assertions::make_coupling(&mut px, &mut py);

        // Children should have been taken from py
        assert_eq!(py.child_count(), 0);
    }

    // ── All stubs don't panic ──────────────────────────────────────────────

    /// Test that all 42 cases can be reached without panicking.
    /// We test a representative sample of matrix entries.
    #[test]
    fn all_matrix_entries_do_not_panic() {
        let types = [
            NodeType::And,
            NodeType::With,
            NodeType::XOfY,
            NodeType::Kind,
            NodeType::Allowed,
            NodeType::PropertyList,
            NodeType::Adjective,
            NodeType::Action,
            NodeType::Relationship,
            NodeType::Every,
            NodeType::CommonNoun,
            NodeType::ProperNoun,
        ];

        for &px_nt in &types {
            for &py_nt in &types {
                let mut px = ParseNode::new(px_nt, Wording::EMPTY);
                let mut py = ParseNode::new(py_nt, Wording::EMPTY);
                Assertions::make_coupling(&mut px, &mut py);
            }
        }
    }
}
