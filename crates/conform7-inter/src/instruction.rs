//! Inter instruction constructs — the ~25 instruction types in Inter bytecode.
//!
//! Based on the `Inter Constructs` chapter of the bytecode module.

/// Construct IDs for Inter instructions. Must match the C implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ConstructId {
    Invalid = 0,

    // Chapter 4: Top-level constructs
    Comment = 1,
    Constant,
    Instance,
    Insert,
    Nop,
    Package,
    Packagetype,
    Permission,
    Pragma,
    Primitive,
    Property,
    Propertyvalue,
    Typename,
    Variable,

    // Chapter 5: Code-level constructs
    Assembly = 20,
    Cast,
    Code,
    Evaluation,
    Inv,
    Lab,
    Label,
    Local,
    Ref,
    Reference,
    Splat,
    Val,

    // Chapter 6: Wiring constructs
    Plug = 40,
    Socket,
    Version,
}

impl ConstructId {
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

    /// The textual keyword for this construct.
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

/// A single Inter instruction — a node in the Inter tree.
///
/// Each instruction has a construct ID and a frame of words (the bytecode).
/// The first word of the frame is always the construct ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub construct: ConstructId,
    /// The full frame of words, including the construct ID as word 0.
    pub words: Vec<u32>,
}

impl Instruction {
    pub fn new(construct: ConstructId) -> Self {
        Self { construct, words: vec![construct as u32] }
    }

    pub fn with_words(construct: ConstructId, mut words: Vec<u32>) -> Self {
        words.insert(0, construct as u32);
        Self { construct, words }
    }

    /// The extent (total number of words) of this instruction.
    pub fn extent(&self) -> usize {
        self.words.len()
    }

    /// Get a word from the frame by field index (0 = construct ID).
    pub fn field(&self, index: usize) -> Option<u32> {
        self.words.get(index).copied()
    }

    /// Set a word in the frame.
    pub fn set_field(&mut self, index: usize, value: u32) {
        if index >= self.words.len() {
            self.words.resize(index + 1, 0);
        }
        self.words[index] = value;
    }
}
