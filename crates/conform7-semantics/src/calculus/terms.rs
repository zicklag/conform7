use std::fmt::{self, Display, Formatter};

/// Maximum number of variables (0-25 for letters x, y, z, a, b, ..., w).
///
/// Corresponds to `MAX_VARIABLES` in the C reference
/// (`services/calculus-module/Chapter 4/Terms.w`).
pub const MAX_VARIABLES: usize = 26;

/// Variable letters lookup: x=0, y=1, z=2, a=3, b=4, ..., w=25.
///
/// Corresponds to the variable letter mapping in the C reference
/// (`services/calculus-module/Chapter 4/Terms.w`).
pub const PCALC_VARS: [char; 26] = [
    'x', 'y', 'z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
];

/// A term in predicate calculus — a variable, constant, or function of another term.
///
/// Corresponds to `pcalc_term` in the C reference
/// (`services/calculus-module/Chapter 4/Terms.w`, lines 27-33).
///
/// At all times exactly one of `variable`, `constant`, or `function` is used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcalcTerm {
    /// Variable number 0-25, or -1 for "not a variable".
    pub variable: i8,
    /// Constant value (simplified: a string name for now).
    pub constant: Option<&'static str>,
    /// Function of another term, or None.
    pub function: Option<Box<PcalcFunc>>,
}

/// A function application inside a term.
///
/// Corresponds to `pcalc_func` in the C reference
/// (`services/calculus-module/Chapter 4/Terms.w`, lines 51-55).
///
/// Terms such as f_A(f_B(f_C(x))) are chains of these structures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PcalcFunc {
    /// The binary predicate (simplified: a string name for now).
    pub bp_name: &'static str,
    /// Which term of the predicate this derives from (0 or 1).
    pub from_term: u8,
    /// The term to which we apply the function.
    pub fn_of: Box<PcalcTerm>,
}

impl PcalcTerm {
    /// Create a variable term (0-25).
    ///
    /// Corresponds to `Terms::new_variable` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 63-68).
    pub fn new_variable(v: u8) -> Self {
        assert!(v < MAX_VARIABLES as u8, "variable must be 0-25");
        PcalcTerm {
            variable: v as i8,
            constant: None,
            function: None,
        }
    }

    /// Create a constant term.
    ///
    /// Corresponds to `Terms::new_constant` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 70-75).
    pub fn new_constant(c: &'static str) -> Self {
        PcalcTerm {
            variable: -1,
            constant: Some(c),
            function: None,
        }
    }

    /// Create a function term.
    ///
    /// Corresponds to `Terms::new_function` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 77-83).
    pub fn new_function(bp_name: &'static str, fn_of: PcalcTerm, from_term: u8) -> Self {
        PcalcTerm {
            variable: -1,
            constant: None,
            function: Some(Box::new(PcalcFunc {
                bp_name,
                from_term,
                fn_of: Box::new(fn_of),
            })),
        }
    }

    /// Returns the constant at the bottom of a function chain, if any.
    ///
    /// Follows function chains to find the underlying constant.
    ///
    /// Corresponds to `Terms::constant_underlying` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 127-135).
    pub fn constant_underlying(&self) -> Option<&'static str> {
        match &self.function {
            Some(f) => f.fn_of.constant_underlying(),
            None => self.constant,
        }
    }

    /// Returns the variable at the bottom of a function chain, if any.
    ///
    /// Follows function chains to find the underlying variable.
    ///
    /// Corresponds to `Terms::variable_underlying` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 137-144).
    pub fn variable_underlying(&self) -> Option<u8> {
        match &self.function {
            Some(f) => f.fn_of.variable_underlying(),
            None => {
                if self.variable >= 0 {
                    Some(self.variable as u8)
                } else {
                    None
                }
            }
        }
    }
    /// Returns true if this is a variable term (not a constant or function).
    pub fn is_variable(&self) -> bool {
        self.variable >= 0
    }

    /// Returns the variable index if this is a variable term.
    pub fn variable_index(&self) -> Option<usize> {
        if self.variable >= 0 {
            Some(self.variable as usize)
        } else {
            None
        }
    }
}

impl Display for PcalcTerm {
    /// Display a term.
    ///
    /// Corresponds to `Terms::write` in the C reference
    /// (`services/calculus-module/Chapter 4/Terms.w`, lines 195-220).
    ///
    /// - Variables: print the letter (x, y, z, a, b, ...).
    /// - Constants: print the constant name.
    /// - Functions: print the function application.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(func) = &self.function {
            write!(f, "{}({})", func.bp_name, func.fn_of)
        } else if let Some(c) = self.constant {
            write!(f, "{c}")
        } else if self.variable >= 0 {
            let idx = self.variable as usize;
            if idx < PCALC_VARS.len() {
                write!(f, "{}", PCALC_VARS[idx])
            } else {
                write!(f, "?{}", self.variable)
            }
        } else {
            write!(f, "?")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_variable_creates_variable_term() {
        let t = PcalcTerm::new_variable(0);
        assert_eq!(t.variable, 0);
        assert!(t.constant.is_none());
        assert!(t.function.is_none());
    }

    #[test]
    fn test_new_constant_creates_constant_term() {
        let t = PcalcTerm::new_constant("hello");
        assert_eq!(t.variable, -1);
        assert_eq!(t.constant, Some("hello"));
        assert!(t.function.is_none());
    }

    #[test]
    fn test_new_function_creates_function_term() {
        let inner = PcalcTerm::new_variable(0);
        let t = PcalcTerm::new_function("f", inner, 0);
        assert_eq!(t.variable, -1);
        assert!(t.constant.is_none());
        assert!(t.function.is_some());
        let func = t.function.as_ref().unwrap();
        assert_eq!(func.bp_name, "f");
        assert_eq!(func.from_term, 0);
        assert_eq!(func.fn_of.variable, 0);
    }

    #[test]
    fn test_constant_underlying_follows_function_chain() {
        let inner = PcalcTerm::new_constant("hello");
        let t = PcalcTerm::new_function("f", inner, 0);
        assert_eq!(t.constant_underlying(), Some("hello"));
    }

    #[test]
    fn test_constant_underlying_direct_constant() {
        let t = PcalcTerm::new_constant("world");
        assert_eq!(t.constant_underlying(), Some("world"));
    }

    #[test]
    fn test_constant_underlying_variable_returns_none() {
        let t = PcalcTerm::new_variable(0);
        assert_eq!(t.constant_underlying(), None);
    }

    #[test]
    fn test_variable_underlying_follows_function_chain() {
        let inner = PcalcTerm::new_variable(0);
        let t = PcalcTerm::new_function("f", inner, 0);
        assert_eq!(t.variable_underlying(), Some(0));
    }

    #[test]
    fn test_variable_underlying_direct_variable() {
        let t = PcalcTerm::new_variable(5);
        assert_eq!(t.variable_underlying(), Some(5));
    }

    #[test]
    fn test_variable_underlying_constant_returns_none() {
        let t = PcalcTerm::new_constant("hello");
        assert_eq!(t.variable_underlying(), None);
    }

    #[test]
    fn test_display_variable() {
        let t = PcalcTerm::new_variable(0);
        assert_eq!(format!("{t}"), "x");
        let t = PcalcTerm::new_variable(1);
        assert_eq!(format!("{t}"), "y");
        let t = PcalcTerm::new_variable(2);
        assert_eq!(format!("{t}"), "z");
        let t = PcalcTerm::new_variable(3);
        assert_eq!(format!("{t}"), "a");
        let t = PcalcTerm::new_variable(25);
        assert_eq!(format!("{t}"), "w");
    }

    #[test]
    fn test_display_constant() {
        let t = PcalcTerm::new_constant("hello");
        assert_eq!(format!("{t}"), "hello");
    }

    #[test]
    fn test_display_function() {
        let inner = PcalcTerm::new_variable(0);
        let t = PcalcTerm::new_function("f", inner, 0);
        assert_eq!(format!("{t}"), "f(x)");
    }

    #[test]
    fn test_display_nested_function() {
        let inner = PcalcTerm::new_variable(0);
        let mid = PcalcTerm::new_function("g", inner, 0);
        let t = PcalcTerm::new_function("f", mid, 0);
        assert_eq!(format!("{t}"), "f(g(x))");
    }
}
