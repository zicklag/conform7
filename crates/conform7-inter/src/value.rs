//! Inter value pairs — two-word values used in bytecode.
//!
//! Based on the `Inter Value Pairs` chapter of the bytecode module.

/// Format codes for value pairs. Must match the C implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ValueFormat {
    Decimal = 0x10000,
    Hex,
    Binary,
    Signed,
    Textual,
    Real,
    Dword,
    Pdword,
    Symbolic,
    Glob,
    Undef,
}

impl ValueFormat {
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

/// A two-word value pair as stored in Inter bytecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterValue {
    pub format: ValueFormat,
    pub content: u32,
}

impl InterValue {
    // --- Constructors ---

    pub fn number(n: u32) -> Self {
        Self { format: ValueFormat::Decimal, content: n }
    }

    pub fn number_in_base(n: u32, base: u32) -> Self {
        let format = match base {
            2 => ValueFormat::Binary,
            10 => ValueFormat::Decimal,
            16 => ValueFormat::Hex,
            _ => ValueFormat::Decimal,
        };
        Self { format, content: n }
    }

    pub fn signed_number(n: i32) -> Self {
        Self { format: ValueFormat::Signed, content: n as u32 }
    }

    pub fn text(text_id: u32) -> Self {
        Self { format: ValueFormat::Textual, content: text_id }
    }

    pub fn real(real_id: u32) -> Self {
        Self { format: ValueFormat::Real, content: real_id }
    }

    pub fn dword(dword_id: u32) -> Self {
        Self { format: ValueFormat::Dword, content: dword_id }
    }

    pub fn pdword(dword_id: u32) -> Self {
        Self { format: ValueFormat::Pdword, content: dword_id }
    }

    pub fn symbolic(symbol_id: u32) -> Self {
        Self { format: ValueFormat::Symbolic, content: symbol_id }
    }

    pub fn glob(glob_id: u32) -> Self {
        Self { format: ValueFormat::Glob, content: glob_id }
    }

    pub fn undef() -> Self {
        Self { format: ValueFormat::Undef, content: 0 }
    }

    // --- Predicates ---

    pub fn is_number(&self) -> bool {
        matches!(self.format, ValueFormat::Decimal | ValueFormat::Hex | ValueFormat::Binary | ValueFormat::Signed)
    }

    pub fn is_text(&self) -> bool {
        matches!(self.format, ValueFormat::Textual)
    }

    pub fn is_real(&self) -> bool {
        matches!(self.format, ValueFormat::Real)
    }

    pub fn is_dword(&self) -> bool {
        matches!(self.format, ValueFormat::Dword)
    }

    pub fn is_pdword(&self) -> bool {
        matches!(self.format, ValueFormat::Pdword)
    }

    pub fn is_symbolic(&self) -> bool {
        matches!(self.format, ValueFormat::Symbolic)
    }

    pub fn is_glob(&self) -> bool {
        matches!(self.format, ValueFormat::Glob)
    }

    pub fn is_undef(&self) -> bool {
        matches!(self.format, ValueFormat::Undef)
    }

    // --- Accessors ---

    pub fn to_number(&self) -> u32 {
        if self.is_number() { self.content } else { 0 }
    }

    pub fn to_base(&self) -> u32 {
        match self.format {
            ValueFormat::Decimal | ValueFormat::Signed => 10,
            ValueFormat::Hex => 16,
            ValueFormat::Binary => 2,
            _ => 10,
        }
    }

    /// Write the value in textual Inter format.
    pub fn to_text(&self, strings: &dyn Fn(u32) -> String, symbols: &dyn Fn(u32) -> String) -> String {
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

/// Escape a string for textual Inter (backslash escapes for `"`, `\`, tab, newline).
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

/// Unescape a string from textual Inter.
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
    }

    #[test]
    fn test_text_escape_roundtrip() {
        let original = "Hello, \"world\"!\n\tTabbed.";
        let escaped = escape_text(original);
        let unescaped = unescape_text(&escaped).unwrap();
        assert_eq!(original, unescaped);
    }
}
