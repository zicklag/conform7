//! Inter value pairs — two-word values used in bytecode.
//!
//! Based on `Chapter 3/Inter Value Pairs.w` from the bytecode module.
//!
//! # Overview
//!
//! In Inter bytecode, every constant value is represented by a **pair** of
//! `u32` words: a **format** word and a **content** word. The format tells
//! you how to interpret the content.
//!
//! This two-word design is a compromise between compactness and expressiveness:
//! - Simple integers fit in one word (the content), with the format indicating
//!   the preferred display base (decimal, hex, binary)
//! - Complex values (strings, symbols, globs) store a warehouse ID in the
//!   content word, pointing to a separately-stored resource
//!
//! # Format Codes
//!
//! The format codes start at `0x10000` to avoid collision with other uses
//! of the word. The C implementation calls these `*_IVAL` constants.
//!
//! | Format | Code | Content Meaning |
//! |--------|------|-----------------|
//! | `DECIMAL_IVAL` | 0x10000 | Unsigned integer, display as decimal |
//! | `HEX_IVAL` | 0x10001 | Unsigned integer, display as hex |
//! | `BINARY_IVAL` | 0x10002 | Unsigned integer, display as binary |
//! | `SIGNED_IVAL` | 0x10003 | Signed integer, display as decimal |
//! | `TEXTUAL_IVAL` | 0x10004 | Warehouse ID of a string resource |
//! | `REAL_IVAL` | 0x10005 | Warehouse ID of a real number string |
//! | `DWORD_IVAL` | 0x10006 | Warehouse ID of a dictionary word |
//! | `PDWORD_IVAL` | 0x10007 | Warehouse ID of a plural dictionary word |
//! | `SYMBOLIC_IVAL` | 0x10008 | Symbol ID (reference to a named constant/variable) |
//! | `GLOB_IVAL` | 0x10009 | Warehouse ID of a glob (raw I6 code snippet) |
//! | `UNDEF_IVAL` | 0x1000A | Undefined value (like `null` or `None`) |
//!
//! Note that `DECIMAL_IVAL`, `HEX_IVAL`, `BINARY_IVAL`, and `SIGNED_IVAL` are
//! all numerically equal — they differ only in how the value is *printed* in
//! textual Inter. This is a presentation concern, not a semantic one.

// ---------------------------------------------------------------------------
// Value Format
// ---------------------------------------------------------------------------

/// Identifies how to interpret the content word of an [`InterValue`].
///
/// These values are stored in binary Inter files and must match the C
/// implementation exactly. The values start at `0x10000` to stay clear
/// of other uses of the word space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ValueFormat {
    /// Unsigned integer, display as decimal. Content is the integer value.
    Decimal = 0x10000,
    /// Unsigned integer, display as hexadecimal. Content is the integer value.
    Hex = 0x10001,
    /// Unsigned integer, display as binary. Content is the integer value.
    Binary = 0x10002,
    /// Signed integer, display as decimal. Content is the two's complement value.
    Signed = 0x10003,
    /// Text string. Content is a warehouse ID pointing to a string resource.
    Textual = 0x10004,
    /// Real (floating-point) number. Content is a warehouse ID pointing to
    /// a string resource containing the number's text representation.
    Real = 0x10005,
    /// Dictionary word (singular). Content is a warehouse ID.
    Dword = 0x10006,
    /// Dictionary word (plural). Content is a warehouse ID.
    Pdword = 0x10007,
    /// Symbol reference. Content is a symbol ID in the current package's
    /// symbols table.
    Symbolic = 0x10008,
    /// Glob (raw I6 code snippet). Content is a warehouse ID pointing to
    /// a string resource containing the I6 code.
    Glob = 0x10009,
    /// Undefined value. Content is ignored (always 0).
    Undef = 0x1000A,
}

impl ValueFormat {
    /// Convert from the raw u32 value used in binary Inter files.
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0x10000 => Some(Self::Decimal),
            0x10001 => Some(Self::Hex),
            0x10002 => Some(Self::Binary),
            0x10003 => Some(Self::Signed),
            0x10004 => Some(Self::Textual),
            0x10005 => Some(Self::Real),
            0x10006 => Some(Self::Dword),
            0x10007 => Some(Self::Pdword),
            0x10008 => Some(Self::Symbolic),
            0x10009 => Some(Self::Glob),
            0x1000A => Some(Self::Undef),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// InterValue
// ---------------------------------------------------------------------------

/// A two-word value pair as stored in Inter bytecode.
///
/// Every constant in an Inter program is represented this way. The [`format`]
/// field tells you how to interpret the [`content`] field.
///
/// This corresponds to `inter_pair` in the C implementation.
///
/// # Examples
///
/// ```text
/// InterValue::number(42)         → format=Decimal, content=42
/// InterValue::text(string_id)    → format=Textual, content=<warehouse ID>
/// InterValue::symbolic(sym_id)  → format=Symbolic, content=<symbol ID>
/// InterValue::undef()            → format=Undef,   content=0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterValue {
    /// How to interpret the content word.
    pub format: ValueFormat,
    /// The actual data, whose meaning depends on the format.
    pub content: u32,
}

impl InterValue {
    // ---------------------------------------------------------------
    // Constructors — each creates a value with the appropriate format
    // ---------------------------------------------------------------

    /// An unsigned integer displayed as decimal. This is the most common
    /// value type for numeric constants.
    pub fn number(n: u32) -> Self {
        Self { format: ValueFormat::Decimal, content: n }
    }

    /// An unsigned integer with a preferred display base.
    /// The base affects only textual Inter output, not semantics.
    pub fn number_in_base(n: u32, base: u32) -> Self {
        let format = match base {
            2 => ValueFormat::Binary,
            10 => ValueFormat::Decimal,
            16 => ValueFormat::Hex,
            _ => ValueFormat::Decimal, // fallback
        };
        Self { format, content: n }
    }

    /// A signed integer. The content is stored as two's complement.
    pub fn signed_number(n: i32) -> Self {
        Self { format: ValueFormat::Signed, content: n as u32 }
    }

    /// A text string. The content is a warehouse ID pointing to the
    /// string resource in the Inter tree's warehouse.
    pub fn text(text_id: u32) -> Self {
        Self { format: ValueFormat::Textual, content: text_id }
    }

    /// A real (floating-point) number. The content is a warehouse ID
    /// pointing to a string resource with the number's text representation.
    pub fn real(real_id: u32) -> Self {
        Self { format: ValueFormat::Real, content: real_id }
    }

    /// A dictionary word (singular form). The content is a warehouse ID.
    pub fn dword(dword_id: u32) -> Self {
        Self { format: ValueFormat::Dword, content: dword_id }
    }

    /// A dictionary word (plural form). The content is a warehouse ID.
    pub fn pdword(dword_id: u32) -> Self {
        Self { format: ValueFormat::Pdword, content: dword_id }
    }

    /// A reference to a symbol (constant, variable, etc.). The content is
    /// a symbol ID in the current package's symbols table.
    pub fn symbolic(symbol_id: u32) -> Self {
        Self { format: ValueFormat::Symbolic, content: symbol_id }
    }

    /// A glob — raw Inform 6 code embedded in the Inter program. The content
    /// is a warehouse ID pointing to the I6 code string.
    pub fn glob(glob_id: u32) -> Self {
        Self { format: ValueFormat::Glob, content: glob_id }
    }

    /// An undefined value. Used as a placeholder or "null" sentinel.
    /// The content is always 0 and is ignored.
    pub fn undef() -> Self {
        Self { format: ValueFormat::Undef, content: 0 }
    }

    // ---------------------------------------------------------------
    // Predicates — test the format without inspecting content
    // ---------------------------------------------------------------

    /// Whether this value is any kind of number (decimal, hex, binary, or signed).
    /// All numeric formats are semantically equivalent; they differ only in
    /// display preference.
    pub fn is_number(&self) -> bool {
        matches!(
            self.format,
            ValueFormat::Decimal | ValueFormat::Hex
                | ValueFormat::Binary | ValueFormat::Signed
        )
    }

    /// Whether this value is a text string.
    pub fn is_text(&self) -> bool { matches!(self.format, ValueFormat::Textual) }

    /// Whether this value is a real (floating-point) number.
    pub fn is_real(&self) -> bool { matches!(self.format, ValueFormat::Real) }

    /// Whether this value is a singular dictionary word.
    pub fn is_dword(&self) -> bool { matches!(self.format, ValueFormat::Dword) }

    /// Whether this value is a plural dictionary word.
    pub fn is_pdword(&self) -> bool { matches!(self.format, ValueFormat::Pdword) }

    /// Whether this value is a symbol reference.
    pub fn is_symbolic(&self) -> bool { matches!(self.format, ValueFormat::Symbolic) }

    /// Whether this value is a glob (raw I6 code).
    pub fn is_glob(&self) -> bool { matches!(self.format, ValueFormat::Glob) }

    /// Whether this value is undefined.
    pub fn is_undef(&self) -> bool { matches!(self.format, ValueFormat::Undef) }

    // ---------------------------------------------------------------
    // Accessors — extract the content with the right interpretation
    // ---------------------------------------------------------------

    /// Get the numeric value. Returns 0 if this is not a number.
    pub fn to_number(&self) -> u32 {
        if self.is_number() { self.content } else { 0 }
    }

    /// Get the preferred display base for numeric values.
    /// Returns 10 for non-numeric values.
    pub fn to_base(&self) -> u32 {
        match self.format {
            ValueFormat::Decimal | ValueFormat::Signed => 10,
            ValueFormat::Hex => 16,
            ValueFormat::Binary => 2,
            _ => 10,
        }
    }

    /// Write the value in textual Inter format.
    ///
    /// This requires two callbacks because the value may reference resources
    /// that need to be looked up:
    ///
    /// - `strings`: given a warehouse ID, returns the string content
    /// - `symbols`: given a symbol ID, returns the symbol name
    ///
    /// The callbacks are used instead of direct tree access to avoid
    /// coupling this module to the tree structure.
    pub fn to_text(
        &self,
        strings: &dyn Fn(u32) -> String,
        symbols: &dyn Fn(u32) -> String,
    ) -> String {
        match self.format {
            ValueFormat::Decimal => format!("{}", self.content as i32),
            ValueFormat::Signed => format!("{}", self.content as i32),
            ValueFormat::Hex => format!("0x{:x}", self.content),
            ValueFormat::Binary => format!("0b{:b}", self.content),
            ValueFormat::Textual => {
                let s = strings(self.content);
                format!("\"{}\"", escape_text(&s))
            }
            ValueFormat::Real => {
                let s = strings(self.content);
                format!("r\"{}\"", escape_text(&s))
            }
            ValueFormat::Dword => {
                let s = strings(self.content);
                format!("dw\"{}\"", escape_text(&s))
            }
            ValueFormat::Pdword => {
                let s = strings(self.content);
                format!("dwp\"{}\"", escape_text(&s))
            }
            ValueFormat::Symbolic => symbols(self.content),
            ValueFormat::Glob => {
                let s = strings(self.content);
                format!("glob\"{}\"", escape_text(&s))
            }
            ValueFormat::Undef => "!undef".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Text escaping — for string literals in textual Inter
// ---------------------------------------------------------------------------

/// Escape special characters in a string for textual Inter output.
///
/// The textual Inter format uses backslash escapes for:
/// - `"` → `\"` (so quotes inside strings don't terminate the string)
/// - `\` → `\\` (so backslashes are preserved)
/// - tab → `\t`
/// - newline → `\n`
///
/// This matches the behavior of `TextualInter::write_text` in the C implementation.
fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

/// Unescape a string read from textual Inter.
///
/// Reverses the escaping done by [`escape_text`]. Returns an error if an
/// unrecognized escape sequence is encountered.
///
/// This matches the behavior of `TextualInter::parse_literal_text` in the
/// C implementation.
pub fn unescape_text(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('t') => out.push('\t'),
                Some('n') => out.push('\n'),
                Some(other) => return Err(format!("unknown backslash escape: \\{}", other)),
                None => return Err("trailing backslash".to_string()),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_values() {
        let v = InterValue::number(42);
        assert!(v.is_number());
        assert_eq!(v.to_number(), 42);
        assert_eq!(v.to_base(), 10);
    }

    #[test]
    fn test_hex_values() {
        let v = InterValue::number_in_base(0xff, 16);
        assert!(v.is_number());
        assert_eq!(v.to_number(), 0xff);
        assert_eq!(v.to_base(), 16);
    }

    #[test]
    fn test_undef() {
        let v = InterValue::undef();
        assert!(v.is_undef());
        assert!(!v.is_number());
    }

    #[test]
    fn test_text_escape_roundtrip() {
        let original = "Hello, \"world\"!\n\tTabbed.";
        let escaped = escape_text(original);
        let unescaped = unescape_text(&escaped).unwrap();
        assert_eq!(original, unescaped);
    }
}
