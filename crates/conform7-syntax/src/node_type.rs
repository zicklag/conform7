//! Enumerated node types for the Inform 7 syntax tree.
//!
//! Every node in a parse tree has a type. Some types are enumerated constants
//! (the `*_NT` family); others will eventually be Preform meaning codes. This
//! module defines the enumerated node type system, their categories, and
//! metadata about how many children each type expects.
//!
//! The C implementation splits this across:
//!
//! - `services/syntax-module/Chapter 2/Node Types.w` — the base enumerated
//!   node types and metadata machinery.
//! - `inform7/core-module/Chapter 1/Inform-Only Nodes and Annotations.w` —
//!   additional node types used only by Inform 7.
//!
//! We deliberately merge the two sources here, because a Rust enum is easiest
//! to work with when all variants are visible in one place. The plan is to grow
//! this enum as we port more of the grammar.

use std::fmt;

/// Node category, used to enforce the parse tree hierarchy.
///
/// In C, categories decide which nodes may be children of which others (see
/// `NodeType::parentage_allowed`). For now we store the category metadata but
/// do not enforce parentage rules — that will come when we build the actual
/// grammar parser.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    /// No node should have this category.
    Invalid,
    /// Top-level structural nodes (headings, root, inclusions).
    L1,
    /// Top-level declarations and assertion sentences.
    L2,
    /// Clauses inside sentences.
    L3,
    /// Code inside imperative rules and phrase definitions.
    Code,
    /// Run-time values that can be read but not assigned to.
    Rvalue,
    /// Run-time storage locations that can be assigned to.
    Lvalue,
    /// Conditions and Boolean combinations.
    Condition,
    /// Unknown / fallback category.
    Unknown,
}

/// Flags describing special behavior of a node type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeFlags {
    /// Node is not visited during generic traversals.
    pub dont_visit: bool,
    /// Node contains tab-delimited lists.
    pub tabbed: bool,
    /// Node compiles to a function call.
    pub phrasal: bool,
    /// Node is allowed on either side of an assertion.
    pub assert: bool,
}

/// Metadata describing an enumerated node type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeTypeMetadata {
    /// Human-readable name such as `"HEADING_NT"`.
    pub name: &'static str,
    /// Minimum legal number of child nodes.
    pub min_children: u32,
    /// Maximum legal number of child nodes, or `u32::MAX` for "unlimited".
    pub max_children: u32,
    /// Category in the parse tree hierarchy.
    pub category: NodeCategory,
    /// Special behavior flags.
    pub flags: NodeFlags,
}

/// An enumerated node type in an Inform 7 syntax tree.
///
/// The first block of variants corresponds to the base syntax module node
/// types in `services/syntax-module/Chapter 2/Node Types.w`. The remaining
/// variants are Inform-only node types from
/// `inform7/core-module/Chapter 1/Inform-Only Nodes and Annotations.w`.
///
/// This list is intentionally incomplete: we add variants as we need them for
/// parsing. The metadata table always returns sensible defaults for unknown
/// variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NodeType {
    // Base syntax-module node types.
    Invalid,
    Root,
    Inclusion,
    Heading,
    Include,
    BeginHere,
    EndHere,
    Sentence,
    Ambiguity,
    Unknown,

    // Inform-only structural nodes (L2/L3).
    DefnCont,
    Allowed,
    Every,
    Adjective,
    PropertyCalled,
    Created,

    // Structural sentence nodes (from inbuild/supervisor-module/Chapter 6/Source Text.w).
    Table,
    Equation,
    Use,

    // Code nodes.
    InvocationList,
    CodeBlock,
    InvocationListSay,
    Invocation,
    VoidContext,
    RvalueContext,
    LvalueContext,
    LvalueTrContext,
    SpecificRvalueContext,
    MatchingRvalueContext,
    NewLocalContext,
    LvalueLocalContext,
    ConditionContext,

    // Rvalue nodes.
    Constant,
    PhraseToDecideValue,

    // Lvalue nodes.
    LocalVariable,
    NonlocalVariable,
    PropertyValue,
    TableEntry,
    ListEntry,

    // Condition nodes.
    LogicalNot,
    LogicalTense,
    LogicalAnd,
    LogicalOr,
    TestProposition,
    TestPhraseOption,
    TestValue,
}

impl NodeType {
    /// Return metadata for this node type.
    ///
    /// The values here are ported from the `NodeType::new` calls in the C
    /// implementation. `u32::MAX` is used in place of the C `INFTY` constant.
    pub fn metadata(self) -> NodeTypeMetadata {
        use NodeCategory as Cat;
        use NodeType::*;
        match self {
            Invalid => metadata("INVALID_NT", 0, u32::MAX, Cat::Invalid, NodeFlags::default()),
            Root => metadata("ROOT_NT", 0, u32::MAX, Cat::L1, NodeFlags { dont_visit: true, ..NodeFlags::default() }),
            Inclusion => metadata("INCLUSION_NT", 0, u32::MAX, Cat::L1, NodeFlags { dont_visit: true, ..NodeFlags::default() }),
            Heading => metadata("HEADING_NT", 0, u32::MAX, Cat::L1, NodeFlags::default()),
            Include => metadata("INCLUDE_NT", 0, 0, Cat::L2, NodeFlags::default()),
            BeginHere => metadata("BEGINHERE_NT", 0, 0, Cat::L2, NodeFlags::default()),
            EndHere => metadata("ENDHERE_NT", 0, 0, Cat::L2, NodeFlags::default()),
            Sentence => metadata("SENTENCE_NT", 0, u32::MAX, Cat::L2, NodeFlags::default()),
            Ambiguity => metadata("AMBIGUITY_NT", 0, u32::MAX, Cat::L1, NodeFlags::default()),
            Unknown => metadata("UNKNOWN_NT", 0, u32::MAX, Cat::Unknown, NodeFlags::default()),

            DefnCont => metadata("DEFN_CONT_NT", 0, u32::MAX, Cat::L2, NodeFlags { assert: true, ..NodeFlags::default() }),
            Allowed => metadata("ALLOWED_NT", 1, 1, Cat::L3, NodeFlags { assert: true, ..NodeFlags::default() }),
            Every => metadata("EVERY_NT", 0, u32::MAX, Cat::L3, NodeFlags { assert: true, ..NodeFlags::default() }),
            Adjective => metadata("ADJECTIVE_NT", 0, u32::MAX, Cat::L3, NodeFlags { assert: true, ..NodeFlags::default() }),
            PropertyCalled => metadata("PROPERTYCALLED_NT", 2, 2, Cat::L3, NodeFlags::default()),
            Created => metadata("CREATED_NT", 0, 0, Cat::L3, NodeFlags::default()),

            Table => metadata("TABLE_NT", 0, 0, Cat::L2, NodeFlags { tabbed: true, ..NodeFlags::default() }),
            Equation => metadata("EQUATION_NT", 0, 0, Cat::L2, NodeFlags::default()),
            Use => metadata("USE_NT", 0, 0, Cat::L2, NodeFlags::default()),

            InvocationList => metadata("INVOCATION_LIST_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            CodeBlock => metadata("CODE_BLOCK_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            InvocationListSay => metadata("INVOCATION_LIST_SAY_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            Invocation => metadata("INVOCATION_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            VoidContext => metadata("VOID_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            RvalueContext => metadata("RVALUE_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            LvalueContext => metadata("LVALUE_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            LvalueTrContext => metadata("LVALUE_TR_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            SpecificRvalueContext => metadata("SPECIFIC_RVALUE_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            MatchingRvalueContext => metadata("MATCHING_RVALUE_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            NewLocalContext => metadata("NEW_LOCAL_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            LvalueLocalContext => metadata("LVALUE_LOCAL_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),
            ConditionContext => metadata("CONDITION_CONTEXT_NT", 0, u32::MAX, Cat::Code, NodeFlags::default()),

            Constant => metadata("CONSTANT_NT", 0, 0, Cat::Rvalue, NodeFlags::default()),
            PhraseToDecideValue => metadata("PHRASE_TO_DECIDE_VALUE_NT", 1, 1, Cat::Rvalue, NodeFlags { phrasal: true, ..NodeFlags::default() }),

            LocalVariable => metadata("LOCAL_VARIABLE_NT", 0, 0, Cat::Lvalue, NodeFlags::default()),
            NonlocalVariable => metadata("NONLOCAL_VARIABLE_NT", 0, 0, Cat::Lvalue, NodeFlags::default()),
            PropertyValue => metadata("PROPERTY_VALUE_NT", 2, 2, Cat::Lvalue, NodeFlags::default()),
            TableEntry => metadata("TABLE_ENTRY_NT", 1, 4, Cat::Lvalue, NodeFlags::default()),
            ListEntry => metadata("LIST_ENTRY_NT", 2, 2, Cat::Lvalue, NodeFlags::default()),

            LogicalNot => metadata("LOGICAL_NOT_NT", 1, 1, Cat::Condition, NodeFlags::default()),
            LogicalTense => metadata("LOGICAL_TENSE_NT", 1, 1, Cat::Condition, NodeFlags::default()),
            LogicalAnd => metadata("LOGICAL_AND_NT", 2, 2, Cat::Condition, NodeFlags::default()),
            LogicalOr => metadata("LOGICAL_OR_NT", 2, 2, Cat::Condition, NodeFlags::default()),
            TestProposition => metadata("TEST_PROPOSITION_NT", 0, 0, Cat::Condition, NodeFlags::default()),
            TestPhraseOption => metadata("TEST_PHRASE_OPTION_NT", 0, 0, Cat::Condition, NodeFlags::default()),
            TestValue => metadata("TEST_VALUE_NT", 1, 1, Cat::Condition, NodeFlags::default()),
        }
    }

    /// Human-readable name such as `"HEADING_NT"`.
    pub fn name(self) -> &'static str {
        self.metadata().name
    }

    /// Category of this node type in the parse tree hierarchy.
    pub fn category(self) -> NodeCategory {
        self.metadata().category
    }

    /// True if this node type is allowed as the root of an assertion.
    pub fn is_assert(self) -> bool {
        self.metadata().flags.assert
    }

    /// True if this node type is marked `DONT_VISIT_NFLAG`.
    pub fn is_dont_visit(self) -> bool {
        self.metadata().flags.dont_visit
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

const fn metadata(
    name: &'static str,
    min_children: u32,
    max_children: u32,
    category: NodeCategory,
    flags: NodeFlags,
) -> NodeTypeMetadata {
    NodeTypeMetadata {
        name,
        min_children,
        max_children,
        category,
        flags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_metadata() {
        let m = NodeType::Heading.metadata();
        assert_eq!(m.name, "HEADING_NT");
        assert_eq!(m.category, NodeCategory::L1);
        assert_eq!(m.min_children, 0);
        assert_eq!(m.max_children, u32::MAX);
    }

    #[test]
    fn test_sentence_metadata() {
        let m = NodeType::Sentence.metadata();
        assert_eq!(m.name, "SENTENCE_NT");
        assert_eq!(m.category, NodeCategory::L2);
        assert_eq!(m.max_children, u32::MAX);
    }

    #[test]
    fn test_logical_and_metadata() {
        let m = NodeType::LogicalAnd.metadata();
        assert_eq!(m.name, "LOGICAL_AND_NT");
        assert_eq!(m.category, NodeCategory::Condition);
        assert_eq!(m.min_children, 2);
        assert_eq!(m.max_children, 2);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", NodeType::Root), "ROOT_NT");
        assert_eq!(format!("{}", NodeType::Constant), "CONSTANT_NT");
    }

    #[test]
    fn test_table_metadata() {
        let m = NodeType::Table.metadata();
        assert_eq!(m.name, "TABLE_NT");
        assert_eq!(m.category, NodeCategory::L2);
        assert!(m.flags.tabbed);
    }

    #[test]
    fn test_equation_metadata() {
        let m = NodeType::Equation.metadata();
        assert_eq!(m.name, "EQUATION_NT");
        assert_eq!(m.category, NodeCategory::L2);
    }

    #[test]
    fn test_use_metadata() {
        let m = NodeType::Use.metadata();
        assert_eq!(m.name, "USE_NT");
        assert_eq!(m.category, NodeCategory::L2);
    }
}
