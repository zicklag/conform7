//! Inter type system — primitive data types below the level of I7 kinds.
//!
//! Based on `Chapter 3/Inter Data Types.w` from the bytecode module.
//!
//! # Overview
//!
//! Inter has its own type system that operates below the level of Inform 7
//! "kinds" (which are a higher-level concept). An Inter type is a combination
//! of a **constructor** and zero or more **operand types**.
//!
//! For example:
//! - `int32` is a base type (constructor only, no operands)
//! - `list of int32` has constructor `list` with one operand (`int32`)
//! - `function int32 int2 -> void` has constructor `function` with three operands
//!
//! # Representation
//!
//! Types are stored in two forms:
//!
//! 1. **TID** ([`Tid`]) — a single `u32` word stored in bytecode instruction
//!    frames. Compact but requires package context to fully interpret (since
//!    it may reference a typename symbol).
//!
//! 2. **InterType** ([`InterType`]) — a full structural description independent
//!    of any package context. Used when passing types between functions.
//!
//! The C implementation uses the same two-level approach: `inter_ti` for TIDs
//! and `inter_type` for full descriptions.
//!
//! # Constructor IDs
//!
//! These numeric IDs are part of the Inter specification and must match the C
//! implementation exactly, since they appear in binary Inter files. Changing
//! any of these values would require bumping the Inter version.

use std::fmt;

// ---------------------------------------------------------------------------
// Type Constructor
// ---------------------------------------------------------------------------

/// Identifies which type constructor is used (e.g., `int32`, `list`, `function`).
///
/// These values are stored in binary Inter files and must match the C
/// implementation in `Inter Data Types.w` exactly. The numeric values
/// start at 1 because 0 is reserved for "no type".
///
/// The constructors are divided into:
/// - **Base types** (arity 0): stand-alone types like `int32`, `text`, `void`
/// - **Compound types** (arity > 0): parameterized types like `list of X`,
///   `function A B -> C`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum TypeConstructor {
    /// An unchecked/unknown type. Used when type information is not available.
    /// Equivalent to C's `UNCHECKED_ITCONC`.
    Unchecked = 1,

    /// 32-bit signed integer. Range: -2147483648 to 2147483647.
    Int32 = 2,

    /// 16-bit signed integer. Range: -32768 to 32767.
    Int16 = 3,

    /// 8-bit signed integer. Range: -128 to 127.
    Int8 = 4,

    /// 2-bit unsigned integer (boolean). Range: 0 to 1.
    Int2 = 5,

    /// IEEE 754 floating-point number (stored as a text representation).
    Real = 6,

    /// Text string (stored as a warehouse reference to a string resource).
    Text = 7,

    /// Enumerated type. Values are symbolic constants.
    Enum = 8,

    /// List of some element type. Arity 1.
    List = 9,

    /// Activity type. Arity 1 (the activity's value type).
    Activity = 10,

    /// Table column type. Arity 1 (the column's value type).
    Column = 11,

    /// Table type. Arity 1 (the table's column types).
    Table = 12,

    /// Function type. Arity 2+ (argument types... → return type).
    Function = 13,

    /// Struct type with named fields.
    Struct = 14,

    /// Relation type. Arity 2 (left and right value types).
    Relation = 15,

    /// Description type (a pattern matching values). Arity 1.
    Description = 16,

    /// Rule type. Arity 2 (rulebook and action types).
    Rule = 17,

    /// Rulebook type. Arity 1 (the rule type).
    Rulebook = 18,

    /// Equated type (a type alias). Arity 1.
    Equated = 19,

    /// Void type (no value). Used for function return types and primitives
    /// that don't produce values. Its range is intentionally empty (min > max)
    /// so no literal value can have this type.
    Void = 20,
}

impl TypeConstructor {
    /// Convert from the raw u32 value used in binary Inter files.
    ///
    /// Returns `None` if the value doesn't correspond to a known constructor.
    /// This would indicate either a corrupted file or a newer Inter version
    /// with additional constructors we don't support yet.
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

    /// The keyword used in textual Inter for this constructor.
    ///
    /// For example, `Int32` → `"int32"`, `List` → `"list"`.
    /// Note that `Equated` has an empty keyword because equated types
    /// are printed using their alias name, not a constructor keyword.
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

    /// Look up a constructor by its textual Inter keyword.
    ///
    /// This is used when parsing textual Inter files. The lookup is O(n) in
    /// the number of constructors, but since there are only 20 and textual
    /// parsing is not performance-critical, this is acceptable.
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

    /// Whether this is a base type (no type operands needed).
    ///
    /// Base types can stand alone: `int32`, `text`, `void`, etc.
    /// Non-base types require operands: `list of X`, `function A -> B`.
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

    /// How many type operands this constructor requires.
    ///
    /// - Base types: 0
    /// - `list of X`, `column of X`, etc.: 1
    /// - `function A B -> C`, `relation L R`: 2
    /// - `struct` is special: it has variable fields, so arity is 0 in the
    ///   base sense (fields are specified separately)
    pub fn arity(&self) -> usize {
        match self {
            Self::Unchecked | Self::Int32 | Self::Int16 | Self::Int8 | Self::Int2
            | Self::Real | Self::Text | Self::Enum | Self::Void => 0,
            Self::List | Self::Activity | Self::Column | Self::Table
            | Self::Description | Self::Rulebook | Self::Equated => 1,
            Self::Function | Self::Relation | Self::Rule => 2,
            Self::Struct => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// TID — Type ID
// ---------------------------------------------------------------------------

/// A TID (Type ID) — a single `u32` word encoding a type in bytecode.
///
/// In binary Inter, types are stored as single words in instruction frames.
/// The encoding depends on the constructor:
///
/// - **Base types**: the TID is just the constructor ID (e.g., `3` = `int16`)
/// - **Named types**: the TID is a symbol ID referencing a `typename` declaration
/// - **Compound types**: encoded with constructor in high bits and operands packed
///
/// You need to know which package (symbols table) a TID came from to fully
/// interpret it, since it may reference a typename symbol in that package.
///
/// This corresponds to `inter_ti` in the C implementation when used as a type ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tid(pub u32);

impl Tid {
    /// The unchecked type — used when type information is unavailable.
    pub const UNCHECKED: Self = Tid(TypeConstructor::Unchecked as u32);
    /// 32-bit signed integer.
    pub const INT32: Self = Tid(TypeConstructor::Int32 as u32);
    /// 16-bit signed integer.
    pub const INT16: Self = Tid(TypeConstructor::Int16 as u32);
    /// 8-bit signed integer.
    pub const INT8: Self = Tid(TypeConstructor::Int8 as u32);
    /// 2-bit unsigned (boolean).
    pub const INT2: Self = Tid(TypeConstructor::Int2 as u32);
    /// IEEE 754 floating-point.
    pub const REAL: Self = Tid(TypeConstructor::Real as u32);
    /// Text string.
    pub const TEXT: Self = Tid(TypeConstructor::Text as u32);
    /// Void (no value).
    pub const VOID: Self = Tid(TypeConstructor::Void as u32);
}

impl fmt::Display for Tid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TID({})", self.0)
    }
}

// ---------------------------------------------------------------------------
// InterType — Full Type Description
// ---------------------------------------------------------------------------

/// A full structural description of an Inter type, independent of any package.
///
/// Unlike [`Tid`], which is a compact word reference that may depend on
/// package context, an `InterType` is a self-contained description that can
/// be passed around freely. It corresponds to `inter_type` in the C
/// implementation.
///
/// # Examples
///
/// ```text
/// int32           → InterType { constructor: Int32, operands: [], type_name: None }
/// list of text    → InterType { constructor: List, operands: [InterType::text()], type_name: None }
/// K_number        → InterType { constructor: Equated, operands: [...], type_name: Some("K_number") }
/// ```
///
/// Named types (typenames) carry their symbol name in `type_name`. When
/// printed, the name is used instead of the underlying constructor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterType {
    /// Which type constructor this is.
    pub constructor: TypeConstructor,
    /// Type operands (e.g., the element type for `list of X`).
    pub operands: Vec<InterType>,
    /// If this is a named type (declared with `typename`), the symbol name.
    /// When present, this name is used for display instead of the constructor.
    pub type_name: Option<String>,
}

impl InterType {
    /// Create a base type (no operands).
    ///
    /// Panics if the constructor is not a base type — use [`is_base`](TypeConstructor::is_base)
    /// to check first if the constructor is dynamic.
    pub fn base(constructor: TypeConstructor) -> Self {
        assert!(constructor.is_base(), "{} is not a base type", constructor.keyword());
        Self { constructor, operands: vec![], type_name: None }
    }

    /// The unchecked type — used when type information is not available.
    /// This is the default type in many contexts where the type is unknown.
    pub fn unchecked() -> Self { Self::base(TypeConstructor::Unchecked) }

    /// 32-bit signed integer. The most common numeric type in Inter.
    pub fn int32() -> Self { Self::base(TypeConstructor::Int32) }

    /// Text string type.
    pub fn text() -> Self { Self::base(TypeConstructor::Text) }

    /// Void type — used for function returns and primitives with no output.
    pub fn void() -> Self { Self::base(TypeConstructor::Void) }

    /// Create a `list of X` type.
    ///
    /// Lists are the primary collection type in Inter. They represent
    /// Inform 7 lists like `{2, 3, 5, 7, 11}`.
    pub fn list_of(element: InterType) -> Self {
        Self { constructor: TypeConstructor::List, operands: vec![element], type_name: None }
    }

    /// Write the type in textual Inter format.
    ///
    /// For named types, this outputs the type name (e.g., `K_number`).
    /// For base types, it outputs the keyword (e.g., `int32`).
    /// For compound types, it outputs constructor + operands using the
    /// correct Inform prepositions:
    /// - `list of X`
    /// - `activity on X`
    /// - `column of X`
    /// - `table of X`
    /// - `description of X`
    /// - `rulebook of X`
    /// - `relation of X to Y`
    /// - `function A B -> C` (and similarly `rule A -> B`)
    pub fn to_text(&self) -> String {
        if let Some(ref name) = self.type_name {
            return name.clone();
        }
        if self.constructor.is_base() {
            return self.constructor.keyword().to_string();
        }
        match self.constructor {
            TypeConstructor::List => {
                format!(
                    "list of {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Activity => {
                format!(
                    "activity on {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Column => {
                format!(
                    "column of {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Table => {
                format!(
                    "table of {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Description => {
                format!(
                    "description of {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Rulebook => {
                format!(
                    "rulebook of {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Relation => {
                format!(
                    "relation of {} to {}",
                    self.operands.first().map(|t| t.to_text()).unwrap_or_default(),
                    self.operands.get(1).map(|t| t.to_text()).unwrap_or_default()
                )
            }
            TypeConstructor::Function | TypeConstructor::Rule if self.operands.len() >= 2 => {
                let args = &self.operands[..self.operands.len() - 1];
                let ret = &self.operands[self.operands.len() - 1];
                let args_str = args.iter().map(|t| t.to_text()).collect::<Vec<_>>().join(" ");
                format!("{} {} -> {}", self.constructor.keyword(), args_str, ret.to_text())
            }
            _ => {
                let mut s = self.constructor.keyword().to_string();
                for op in &self.operands {
                    s.push(' ');
                    s.push_str(&op.to_text());
                }
                s
            }
        }
    }
}

impl fmt::Display for InterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tid_constants() {
        assert_eq!(Tid::UNCHECKED.0, 1);
        assert_eq!(Tid::INT32.0, 2);
        assert_eq!(Tid::INT16.0, 3);
        assert_eq!(Tid::INT8.0, 4);
        assert_eq!(Tid::INT2.0, 5);
        assert_eq!(Tid::REAL.0, 6);
        assert_eq!(Tid::TEXT.0, 7);
        assert_eq!(Tid::VOID.0, 20);
    }

    #[test]
    fn test_type_constructor_keyword_roundtrip() {
        let constructors = [
            TypeConstructor::Unchecked,
            TypeConstructor::Int32,
            TypeConstructor::Int16,
            TypeConstructor::Int8,
            TypeConstructor::Int2,
            TypeConstructor::Real,
            TypeConstructor::Text,
            TypeConstructor::Enum,
            TypeConstructor::List,
            TypeConstructor::Activity,
            TypeConstructor::Column,
            TypeConstructor::Table,
            TypeConstructor::Function,
            TypeConstructor::Struct,
            TypeConstructor::Relation,
            TypeConstructor::Description,
            TypeConstructor::Rule,
            TypeConstructor::Rulebook,
            TypeConstructor::Equated,
            TypeConstructor::Void,
        ];
        for &c in &constructors {
            let kw = c.keyword();
            if kw.is_empty() {
                // Equated has an empty keyword and can't be parsed from text
                continue;
            }
            let c2 = TypeConstructor::from_keyword(kw);
            assert_eq!(Some(c), c2, "keyword '{}' should round-trip for {:?}", kw, c);
        }
    }

    #[test]
    fn test_type_constructor_u32_roundtrip() {
        for i in 1..=20u32 {
            let c = TypeConstructor::from_u32(i).unwrap();
            assert_eq!(c as u32, i, "u32 {} should round-trip", i);
        }
        assert!(TypeConstructor::from_u32(0).is_none());
        assert!(TypeConstructor::from_u32(99).is_none());
    }

    #[test]
    fn test_inter_type_to_text() {
        assert_eq!(InterType::int32().to_text(), "int32");
        assert_eq!(InterType::text().to_text(), "text");
        assert_eq!(InterType::void().to_text(), "void");
        assert_eq!(InterType::unchecked().to_text(), "unchecked");
        assert_eq!(InterType::list_of(InterType::int32()).to_text(), "list of int32");

        // Named type
        let named = InterType {
            constructor: TypeConstructor::Equated,
            operands: vec![InterType::int32()],
            type_name: Some("K_number".to_string()),
        };
        assert_eq!(named.to_text(), "K_number");

        // Prepositional compound types
        let column = InterType {
            constructor: TypeConstructor::Column,
            operands: vec![InterType {
                constructor: TypeConstructor::Equated,
                operands: vec![InterType::int32()],
                type_name: Some("K_number".to_string()),
            }],
            type_name: None,
        };
        assert_eq!(column.to_text(), "column of K_number");

        let relation = InterType {
            constructor: TypeConstructor::Relation,
            operands: vec![
                InterType {
                    constructor: TypeConstructor::Equated,
                    operands: vec![InterType::int32()],
                    type_name: Some("K_player".to_string()),
                },
                InterType {
                    constructor: TypeConstructor::Equated,
                    operands: vec![InterType::int32()],
                    type_name: Some("K_room".to_string()),
                },
            ],
            type_name: None,
        };
        assert_eq!(relation.to_text(), "relation of K_player to K_room");

        // Function and rule types use -> between arguments and return type
        let func = InterType {
            constructor: TypeConstructor::Function,
            operands: vec![InterType::int32(), InterType::text(), InterType::void()],
            type_name: None,
        };
        assert_eq!(func.to_text(), "function int32 text -> void");

        let rule = InterType {
            constructor: TypeConstructor::Rule,
            operands: vec![InterType::int32(), InterType::text()],
            type_name: None,
        };
        assert_eq!(rule.to_text(), "rule int32 -> text");
    }

    #[test]
    fn test_constructor_arity() {
        assert_eq!(TypeConstructor::Int32.arity(), 0);
        assert_eq!(TypeConstructor::List.arity(), 1);
        assert_eq!(TypeConstructor::Function.arity(), 2);
        assert_eq!(TypeConstructor::Relation.arity(), 2);
        assert_eq!(TypeConstructor::Struct.arity(), 0);
    }

    #[test]
    fn test_constructor_is_base() {
        assert!(TypeConstructor::Int32.is_base());
        assert!(TypeConstructor::Text.is_base());
        assert!(TypeConstructor::Void.is_base());
        assert!(!TypeConstructor::List.is_base());
        assert!(!TypeConstructor::Function.is_base());
    }
}
