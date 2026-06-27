//! Inter type system — primitive data types below the level of I7 kinds.
//!
//! Based on the `Inter Data Types` chapter of the bytecode module.

/// Type constructor IDs. Must match the values in the C implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum TypeConstructor {
    Unchecked = 1,
    Int32,
    Int16,
    Int8,
    Int2,
    Real,
    Text,
    Enum,
    List,
    Activity,
    Column,
    Table,
    Function,
    Struct,
    Relation,
    Description,
    Rule,
    Rulebook,
    Equated,
    Void,
}

impl TypeConstructor {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(Self::Unchecked),
            2 => Some(Self::Int32),
            3 => Some(Self::Int16),
            4 => Some(Self::Int8),
            5 => Some(Self::Int2),
            6 => Some(Self::Real),
            7 => Some(Self::Text),
            8 => Some(Self::Enum),
            9 => Some(Self::List),
            10 => Some(Self::Activity),
            11 => Some(Self::Column),
            12 => Some(Self::Table),
            13 => Some(Self::Function),
            14 => Some(Self::Struct),
            15 => Some(Self::Relation),
            16 => Some(Self::Description),
            17 => Some(Self::Rule),
            18 => Some(Self::Rulebook),
            19 => Some(Self::Equated),
            20 => Some(Self::Void),
            _ => None,
        }
    }

    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Int32 => "int32",
            Self::Int16 => "int16",
            Self::Int8 => "int8",
            Self::Int2 => "int2",
            Self::Real => "real",
            Self::Text => "text",
            Self::Enum => "enum",
            Self::List => "list",
            Self::Activity => "activity",
            Self::Column => "column",
            Self::Table => "table",
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Relation => "relation",
            Self::Description => "description",
            Self::Rule => "rule",
            Self::Rulebook => "rulebook",
            Self::Equated => "",
            Self::Void => "void",
        }
    }

    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "unchecked" => Some(Self::Unchecked),
            "int32" => Some(Self::Int32),
            "int16" => Some(Self::Int16),
            "int8" => Some(Self::Int8),
            "int2" => Some(Self::Int2),
            "real" => Some(Self::Real),
            "text" => Some(Self::Text),
            "enum" => Some(Self::Enum),
            "list" => Some(Self::List),
            "activity" => Some(Self::Activity),
            "column" => Some(Self::Column),
            "table" => Some(Self::Table),
            "function" => Some(Self::Function),
            "struct" => Some(Self::Struct),
            "relation" => Some(Self::Relation),
            "description" => Some(Self::Description),
            "rule" => Some(Self::Rule),
            "rulebook" => Some(Self::Rulebook),
            "void" => Some(Self::Void),
            _ => None,
        }
    }

    /// Whether this is a base type (no operands).
    pub fn is_base(&self) -> bool {
        matches!(
            self,
            Self::Unchecked
                | Self::Int32
                | Self::Int16
                | Self::Int8
                | Self::Int2
                | Self::Real
                | Self::Text
                | Self::Enum
                | Self::Void
        )
    }

    /// Number of type operands this constructor takes.
    pub fn arity(&self) -> usize {
        match self {
            Self::Unchecked | Self::Int32 | Self::Int16 | Self::Int8 | Self::Int2
            | Self::Real | Self::Text | Self::Enum | Self::Void => 0,
            Self::List | Self::Activity | Self::Column | Self::Table
            | Self::Description | Self::Rulebook | Self::Equated => 1,
            Self::Function | Self::Relation | Self::Rule => 2,
            Self::Struct => 0, // struct has variable fields, arity 0 in base sense
        }
    }
}

/// A TID (Type ID) — a single word encoding a type.
///
/// In binary Inter, types are stored as single `inter_ti` words.
/// The encoding depends on the constructor:
/// - Base types: just the constructor ID
/// - Non-base types: constructor ID in high bits, operands packed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tid(pub u32);

impl Tid {
    pub const UNCHECKED: Self = Tid(TypeConstructor::Unchecked as u32);
    pub const INT32: Self = Tid(TypeConstructor::Int32 as u32);
    pub const INT16: Self = Tid(TypeConstructor::Int16 as u32);
    pub const INT8: Self = Tid(TypeConstructor::Int8 as u32);
    pub const INT2: Self = Tid(TypeConstructor::Int2 as u32);
    pub const REAL: Self = Tid(TypeConstructor::Real as u32);
    pub const TEXT: Self = Tid(TypeConstructor::Text as u32);
    pub const VOID: Self = Tid(TypeConstructor::Void as u32);
}

/// A full Inter type description, independent of any package context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterType {
    pub constructor: TypeConstructor,
    pub operands: Vec<InterType>,
    /// If this type is a named type (typename), the symbol name.
    pub type_name: Option<String>,
}

impl InterType {
    pub fn base(constructor: TypeConstructor) -> Self {
        assert!(constructor.is_base());
        Self { constructor, operands: vec![], type_name: None }
    }

    pub fn unchecked() -> Self {
        Self::base(TypeConstructor::Unchecked)
    }

    pub fn int32() -> Self {
        Self::base(TypeConstructor::Int32)
    }

    pub fn text() -> Self {
        Self::base(TypeConstructor::Text)
    }

    pub fn void() -> Self {
        Self::base(TypeConstructor::Void)
    }

    pub fn list_of(element: InterType) -> Self {
        Self { constructor: TypeConstructor::List, operands: vec![element], type_name: None }
    }

    /// Write the type in textual Inter format.
    pub fn to_text(&self) -> String {
        if let Some(ref name) = self.type_name {
            return name.clone();
        }
        if self.constructor.is_base() {
            return self.constructor.keyword().to_string();
        }
        let mut s = self.constructor.keyword().to_string();
        for op in &self.operands {
            s.push(' ');
            s.push_str(&op.to_text());
        }
        s
    }
}
