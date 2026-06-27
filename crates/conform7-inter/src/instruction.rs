//! Inter instruction constructs — the ~25 instruction types in Inter bytecode.
//!
//! Based on `Chapter 3/Inter Constructs.w` from the bytecode module.
//!
//! # Overview
//!
//! Every node in an Inter tree is an **instruction** — a frame of `u32` words
//! (the "bytecode") identified by a **construct ID** in the first word. There
//! are about 25 different constructs, divided into three groups:
//!
//! 1. **Top-level constructs** (Chapter 4): `constant`, `variable`, `package`,
//!    `typename`, `primitive`, `property`, `instance`, etc. These define the
//!    structure and resources of the program.
//!
//! 2. **Code-level constructs** (Chapter 5): `inv`, `val`, `code`, `lab`,
//!    `label`, `cast`, `assembly`, etc. These appear inside function bodies
//!    and represent executable operations.
//!
//! 3. **Wiring constructs** (Chapter 6): `plug`, `socket`, `version`. These
//!    manage cross-package and cross-tree symbol connections.
//!
//! # Instruction Frames
//!
//! Each instruction occupies a contiguous sequence of words called its
//! **frame**. The first word is always the construct ID. The meaning of
//! subsequent words depends on the construct:
//!
//! ```text
//! CONSTANT_IST:  [ID, SYMBOL_ID, VALUE_FORMAT, VALUE_CONTENT]
//! INV_IST:       [ID, PRIMITIVE_SYMBOL_ID]
//! VAL_IST:       [ID, VALUE_FORMAT, VALUE_CONTENT]
//! PACKAGE_IST:   [ID, PACKAGE_SYMBOL_ID, PACKAGE_TYPE]
//! ```
//!
//! The C implementation stores these in a flat array (`P->W.instruction[i]`)
//! and uses field indices to access specific words. We use a `Vec<u32>` with
//! named accessor methods.
//!
//! # Construct ID Values
//!
//! These numeric IDs are part of the Inter specification and appear in binary
//! Inter files. They must match the C implementation exactly. The gaps in
//! numbering (e.g., 14 to 20) are intentional — they separate the three groups.

// ---------------------------------------------------------------------------
// Construct ID
// ---------------------------------------------------------------------------

/// Identifies which kind of instruction a node represents.
///
/// The numeric values are stored in binary Inter files and must match the
/// C implementation in `Inter Constructs.w` exactly. The values are grouped:
///
/// - 0: Invalid (sentinel)
/// - 1-14: Top-level constructs (Chapter 4)
/// - 20-31: Code-level constructs (Chapter 5)
/// - 40-42: Wiring constructs (Chapter 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ConstructId {
    /// Sentinel value for unknown/corrupt instructions.
    Invalid = 0,

    // -- Chapter 4: Top-level constructs (define program structure) --

    /// A comment. No semantic effect; used for human-readable annotations.
    /// Written as `#` in textual Inter.
    Comment = 1,
    /// Defines a named constant. Frame: `[ID, SYMBOL_ID, VALUE_FORMAT, VALUE_CONTENT]`.
    Constant = 2,
    /// Defines a named instance of a kind. Frame: `[ID, SYMBOL_ID, KIND_TID, VALUE]`.
    Instance = 3,
    /// Inserts the contents of another package at this point. Used for linking.
    Insert = 4,
    /// No operation. Does nothing; used as a placeholder.
    Nop = 5,
    /// Defines a package (namespace). Frame: `[ID, SYMBOL_ID, PACKAGE_TYPE]`.
    /// Packages can contain other packages and instructions.
    Package = 6,
    /// Declares a package type (e.g., `_plain`, `_code`). Must appear at the
    /// root level before the type is used.
    Packagetype = 7,
    /// Grants permission for a kind to have a property.
    Permission = 8,
    /// A compiler pragma — platform-specific tuning. Frame: `[ID, TARGET_STRING_ID]`.
    Pragma = 9,
    /// Declares a primitive operation (e.g., `!print`, `!add`). Must appear at
    /// the root level. Frame: `[ID, SYMBOL_ID, SIGNATURE...]`.
    Primitive = 10,
    /// Defines a named property. Frame: `[ID, SYMBOL_ID, VALUE_TYPE]`.
    Property = 11,
    /// Sets the value of a property for an owner. Frame: `[ID, OWNER_ID, PROPERTY_ID, VALUE]`.
    Propertyvalue = 12,
    /// Declares a named type (type alias). Frame: `[ID, SYMBOL_ID, TYPE]`.
    /// The type can be a base type or a compound type.
    Typename = 13,
    /// Defines a named variable. Frame: `[ID, SYMBOL_ID, TYPE, INITIAL_VALUE?]`.
    Variable = 14,

    // -- Chapter 5: Code-level constructs (appear inside function bodies) --

    /// Inline assembly (target-specific code). Rarely used.
    Assembly = 20,
    /// Type cast: converts a value from one type to another.
    Cast = 21,
    /// Marks the beginning of executable code within a `_code` package.
    Code = 22,
    /// Evaluates an expression and discards the result. Used for side effects.
    Evaluation = 23,
    /// Invokes a primitive operation. Frame: `[ID, PRIMITIVE_SYMBOL_ID]`.
    /// Children of this node are the arguments to the primitive.
    Inv = 24,
    /// References a label (used with `!jump`). Frame: `[ID, LABEL_SYMBOL_ID]`.
    Lab = 25,
    /// Defines a label (target for jumps). Frame: `[ID, LABEL_SYMBOL_ID]`.
    Label = 26,
    /// Declares a local variable within a function body.
    Local = 27,
    /// References a variable or constant by symbol.
    Ref = 28,
    /// References a value by symbol (similar to `ref`).
    Reference = 29,
    /// A "splat" — raw Inform 6 code embedded inline. Rarely used.
    Splat = 30,
    /// A literal value. Frame: `[ID, VALUE_FORMAT, VALUE_CONTENT]`.
    /// This is the most common code-level construct.
    Val = 31,

    // -- Chapter 6: Wiring constructs (cross-package connections) --

    /// A plug — a symbol that needs to be connected to an external definition.
    /// Used during linking to resolve cross-tree references.
    Plug = 40,
    /// A socket — a symbol offered for external connection. The counterpart
    /// to a plug.
    Socket = 41,
    /// Records the Inter specification version used to create the file.
    /// This is a pseudo-construct: it affects the file, not the program.
    Version = 42,
}

impl ConstructId {
    /// Convert from the raw u32 value used in binary Inter files.
    ///
    /// Returns `None` for values in the gaps between groups or beyond the
    /// known range. This would indicate a corrupted file or a newer Inter
    /// version with additional constructs.
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::Comment),
            2 => Some(Self::Constant),
            3 => Some(Self::Instance),
            4 => Some(Self::Insert),
            5 => Some(Self::Nop),
            6 => Some(Self::Package),
            7 => Some(Self::Packagetype),
            8 => Some(Self::Permission),
            9 => Some(Self::Pragma),
            10 => Some(Self::Primitive),
            11 => Some(Self::Property),
            12 => Some(Self::Propertyvalue),
            13 => Some(Self::Typename),
            14 => Some(Self::Variable),
            20 => Some(Self::Assembly),
            21 => Some(Self::Cast),
            22 => Some(Self::Code),
            23 => Some(Self::Evaluation),
            24 => Some(Self::Inv),
            25 => Some(Self::Lab),
            26 => Some(Self::Label),
            27 => Some(Self::Local),
            28 => Some(Self::Ref),
            29 => Some(Self::Reference),
            30 => Some(Self::Splat),
            31 => Some(Self::Val),
            40 => Some(Self::Plug),
            41 => Some(Self::Socket),
            42 => Some(Self::Version),
            _ => None,
        }
    }

    /// The keyword used in textual Inter for this construct.
    ///
    /// For example, `Constant` → `"constant"`, `Inv` → `"inv"`.
    /// The `Comment` construct uses `"#"` as its keyword.
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Comment => "#",
            Self::Constant => "constant",
            Self::Instance => "instance",
            Self::Insert => "insert",
            Self::Nop => "nop",
            Self::Package => "package",
            Self::Packagetype => "packagetype",
            Self::Permission => "permission",
            Self::Pragma => "pragma",
            Self::Primitive => "primitive",
            Self::Property => "property",
            Self::Propertyvalue => "propertyvalue",
            Self::Typename => "typename",
            Self::Variable => "variable",
            Self::Assembly => "assembly",
            Self::Cast => "cast",
            Self::Code => "code",
            Self::Evaluation => "evaluation",
            Self::Inv => "inv",
            Self::Lab => "lab",
            Self::Label => "label",
            Self::Local => "local",
            Self::Ref => "ref",
            Self::Reference => "reference",
            Self::Splat => "splat",
            Self::Val => "val",
            Self::Plug => "plug",
            Self::Socket => "socket",
            Self::Version => "version",
        }
    }

    /// Look up a construct by its textual Inter keyword.
    ///
    /// Used when parsing `.intert` files. The lookup is O(n) but since there
    /// are only ~25 constructs and textual parsing is not performance-critical,
    /// this is fine.
    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "#" => Some(Self::Comment),
            "constant" => Some(Self::Constant),
            "instance" => Some(Self::Instance),
            "insert" => Some(Self::Insert),
            "nop" => Some(Self::Nop),
            "package" => Some(Self::Package),
            "packagetype" => Some(Self::Packagetype),
            "permission" => Some(Self::Permission),
            "pragma" => Some(Self::Pragma),
            "primitive" => Some(Self::Primitive),
            "property" => Some(Self::Property),
            "propertyvalue" => Some(Self::Propertyvalue),
            "typename" => Some(Self::Typename),
            "variable" => Some(Self::Variable),
            "assembly" => Some(Self::Assembly),
            "cast" => Some(Self::Cast),
            "code" => Some(Self::Code),
            "evaluation" => Some(Self::Evaluation),
            "inv" => Some(Self::Inv),
            "lab" => Some(Self::Lab),
            "label" => Some(Self::Label),
            "local" => Some(Self::Local),
            "ref" => Some(Self::Ref),
            "reference" => Some(Self::Reference),
            "splat" => Some(Self::Splat),
            "val" => Some(Self::Val),
            "plug" => Some(Self::Plug),
            "socket" => Some(Self::Socket),
            "version" => Some(Self::Version),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Instruction
// ---------------------------------------------------------------------------

/// A single Inter instruction — a node in the Inter tree.
///
/// Each instruction has a [`construct`] type and a **frame** of `u32` words.
/// The first word of the frame is always the construct ID. Subsequent words
/// have construct-specific meanings (symbol IDs, value pairs, type references,
/// etc.).
///
/// The `depth` field records the nesting depth of this instruction within
/// its package. This is used for textual Inter output (indentation).
/// Depth 0 means top-level in the package. Higher values mean deeper
/// nesting (e.g., a `val` inside an `inv` inside a `code` block).
///
/// This corresponds to `inter_tree_node` in the C implementation, though
/// our representation is simpler: we store the frame as a `Vec<u32>` rather
/// than using the C approach of a separately-allocated array with extent
/// tracking.
///
/// # Frame Layout
///
/// The frame always has the construct ID at index 0. Use [`field`] to access
/// words by their logical field index (0 = construct ID, 1 = first data word,
/// etc.).
///
/// ```text
/// CONSTANT_IST frame:
///   [0] = 2 (CONSTANT_IST)
///   [1] = symbol ID of the constant
///   [2] = value format (e.g., DECIMAL_IVAL)
///   [3] = value content (e.g., 42)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Which kind of instruction this is.
    pub construct: ConstructId,
    /// The full frame of words. `words[0]` is always the construct ID.
    /// The remaining words have construct-specific meanings.
    pub words: Vec<u32>,
    /// Nesting depth within the package. Used for textual Inter output (indentation).
    /// 0 = top-level in the package. Higher values mean deeper nesting
    /// (e.g., a `val` inside an `inv` inside a `code` block).
    ///
    /// Note: this records the *textual indentation* of the instruction
    /// relative to its containing package, not the structural depth in the
    /// Inter tree. For binary Inter, structural depth would be derived
    /// from the actual parent-child relationships, not this field.
    pub depth: usize,
    /// Optional type marker text (interned string ID) written before the
    /// name/value in textual Inter. Used for constructs like
    /// `constant (K_number) C_x = 1` or `val (int32) 17`.
    pub type_marker: Option<u32>,
}

impl Instruction {
    /// Create a new instruction with just the construct ID.
    ///
    /// The frame is initialized with the construct ID as word 0.
    /// Additional words can be set with [`set_field`].
    /// Depth defaults to 0 (top-level in the package).
    pub fn new(construct: ConstructId) -> Self {
        Self { construct, words: vec![construct as u32], depth: 0, type_marker: None }
    }

    /// Create an instruction with a pre-built frame.
    ///
    /// The construct ID is automatically inserted as word 0. The provided
    /// `words` should contain the data words only (without the construct ID).
    /// Depth defaults to 0 (top-level in the package).
    pub fn with_words(construct: ConstructId, mut words: Vec<u32>) -> Self {
        words.insert(0, construct as u32);
        Self { construct, words, depth: 0, type_marker: None }
    }

    /// The total number of words in this instruction's frame, including
    /// the construct ID word.
    ///
    /// This corresponds to `P->W.extent` in the C implementation.
    pub fn extent(&self) -> usize {
        self.words.len()
    }

    /// Get a word from the frame by field index.
    ///
    /// Field 0 is the construct ID. Field 1 is the first data word, etc.
    /// Returns `None` if the index is out of bounds.
    pub fn field(&self, index: usize) -> Option<u32> {
        self.words.get(index).copied()
    }

    /// Set a word in the frame, growing the frame if necessary.
    ///
    /// If `index` is beyond the current frame length, the frame is extended
    /// with zeros to accommodate it. This matches the flexible nature of
    /// Inter instruction frames, where different constructs have different
    /// numbers of fields.
    pub fn set_field(&mut self, index: usize, value: u32) {
        if index >= self.words.len() {
            self.words.resize(index + 1, 0);
        }
        self.words[index] = value;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_id_keyword_roundtrip() {
        // Every construct should round-trip through keyword and back
        let constructs = [
            ConstructId::Invalid,
            ConstructId::Comment,
            ConstructId::Constant,
            ConstructId::Instance,
            ConstructId::Insert,
            ConstructId::Nop,
            ConstructId::Package,
            ConstructId::Packagetype,
            ConstructId::Permission,
            ConstructId::Pragma,
            ConstructId::Primitive,
            ConstructId::Property,
            ConstructId::Propertyvalue,
            ConstructId::Typename,
            ConstructId::Variable,
            ConstructId::Assembly,
            ConstructId::Cast,
            ConstructId::Code,
            ConstructId::Evaluation,
            ConstructId::Inv,
            ConstructId::Lab,
            ConstructId::Label,
            ConstructId::Local,
            ConstructId::Ref,
            ConstructId::Reference,
            ConstructId::Splat,
            ConstructId::Val,
            ConstructId::Plug,
            ConstructId::Socket,
            ConstructId::Version,
        ];
        for &c in &constructs {
            let kw = c.keyword();
            let c2 = ConstructId::from_keyword(kw).unwrap_or(ConstructId::Invalid);
            assert_eq!(c, c2, "keyword '{}' should round-trip for {:?}", kw, c);
        }
    }

    #[test]
    fn test_construct_id_u32_roundtrip() {
        let constructs = [
            (0u32, ConstructId::Invalid),
            (1, ConstructId::Comment),
            (2, ConstructId::Constant),
            (3, ConstructId::Instance),
            (4, ConstructId::Insert),
            (5, ConstructId::Nop),
            (6, ConstructId::Package),
            (7, ConstructId::Packagetype),
            (8, ConstructId::Permission),
            (9, ConstructId::Pragma),
            (10, ConstructId::Primitive),
            (11, ConstructId::Property),
            (12, ConstructId::Propertyvalue),
            (13, ConstructId::Typename),
            (14, ConstructId::Variable),
            (20, ConstructId::Assembly),
            (21, ConstructId::Cast),
            (22, ConstructId::Code),
            (23, ConstructId::Evaluation),
            (24, ConstructId::Inv),
            (25, ConstructId::Lab),
            (26, ConstructId::Label),
            (27, ConstructId::Local),
            (28, ConstructId::Ref),
            (29, ConstructId::Reference),
            (30, ConstructId::Splat),
            (31, ConstructId::Val),
            (40, ConstructId::Plug),
            (41, ConstructId::Socket),
            (42, ConstructId::Version),
        ];
        for &(val, expected) in &constructs {
            let c = ConstructId::from_u32(val).unwrap();
            assert_eq!(c, expected, "u32 {} should map to {:?}", val, expected);
            assert_eq!(c as u32, val, "{:?} should map back to u32 {}", c, val);
        }
    }

    #[test]
    fn test_instruction_frame() {
        let mut instr = Instruction::new(ConstructId::Constant);
        assert_eq!(instr.extent(), 1);
        assert_eq!(instr.field(0), Some(ConstructId::Constant as u32));

        instr.set_field(1, 0x40000001); // symbol ID
        instr.set_field(2, 0x10000);    // value format
        instr.set_field(3, 42);          // value content
        assert_eq!(instr.extent(), 4);
        assert_eq!(instr.field(1), Some(0x40000001));
        assert_eq!(instr.field(2), Some(0x10000));
        assert_eq!(instr.field(3), Some(42));
    }

    #[test]
    fn test_instruction_with_words() {
        let instr = Instruction::with_words(ConstructId::Inv, vec![0x40000001]);
        assert_eq!(instr.extent(), 2);
        assert_eq!(instr.field(0), Some(ConstructId::Inv as u32));
        assert_eq!(instr.field(1), Some(0x40000001));
    }

    #[test]
    fn test_instruction_depth() {
        let mut instr = Instruction::new(ConstructId::Code);
        assert_eq!(instr.depth, 0);
        instr.depth = 2;
        assert_eq!(instr.depth, 2);
    }

    #[test]
    fn test_unknown_construct() {
        assert!(ConstructId::from_u32(99).is_none());
        assert!(ConstructId::from_keyword("nonexistent").is_none());
    }
}
