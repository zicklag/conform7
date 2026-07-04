use std::sync::Arc;

use std::fmt::{self, Display, Formatter};

use crate::calculus::terms::PcalcTerm;
use crate::calculus::unary_predicates::UnaryPredicate;

/// Maximum arity for atoms (matching MAX_ATOM_ARITY = 2).
///
/// Corresponds to `MAX_ATOM_ARITY` in the C reference
/// (`services/calculus-module/Chapter 4/Atomic Propositions.w`).
pub const MAX_ATOM_ARITY: usize = 2;

/// Atom element types (matching the *_ATOM constants in Atomic Propositions.w lines 30-35).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtomElement {
    /// A generalised quantifier (binds a variable).
    Quantifier,
    /// A regular predicate (unary or binary).
    Predicate,
    /// Logical negation opening bracket.
    NegationOpen,
    /// Logical negation closing bracket.
    NegationClose,
    /// Quantifier domain opening bracket.
    DomainOpen,
    /// Quantifier domain closing bracket.
    DomainClose,
}

impl AtomElement {
    /// Returns true if this element is an opening bracket.
    ///
    /// Corresponds to `Atoms::is_opener` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 40-45).
    pub fn is_opener(&self) -> bool {
        matches!(self, AtomElement::NegationOpen | AtomElement::DomainOpen)
    }

    /// Returns true if this element is a closing bracket.
    ///
    /// Corresponds to `Atoms::is_closer` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 47-52).
    pub fn is_closer(&self) -> bool {
        matches!(self, AtomElement::NegationClose | AtomElement::DomainClose)
    }

    /// Returns the matching bracket element, if this is a bracket.
    ///
    /// Corresponds to `Atoms::counterpart` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 54-60).
    pub fn counterpart(&self) -> Option<AtomElement> {
        match self {
            AtomElement::NegationOpen => Some(AtomElement::NegationClose),
            AtomElement::NegationClose => Some(AtomElement::NegationOpen),
            AtomElement::DomainOpen => Some(AtomElement::DomainClose),
            AtomElement::DomainClose => Some(AtomElement::DomainOpen),
            _ => None,
        }
    }
}

/// Reference to a predicate (unary or binary).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PredicateRef {
    /// A unary predicate, identified by name.
    Unary(Arc<str>),
    /// A binary predicate, identified by name.
    Binary(Arc<str>),
}

/// Reference to a quantifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuantifierRef {
    /// Existential quantifier (there exists).
    Exists,
    /// Universal quantifier (for all).
    ForAll,
    /// "Not exists" quantifier.
    NotExists,
    /// "Not for all" quantifier.
    NotForAll,
}

/// An atomic proposition — the building block of propositions.
///
/// Corresponds to `pcalc_prop` in the C reference
/// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 14-23).
///
/// Propositions are linked lists of these atoms. Conjunction is implied
/// by adjacency (no explicit AND atom). Negation is bracketed:
/// NOT< --> P --> NOT>. Quantification is bracketed:
/// QUANTIFIER --> IN< --> P --> IN>.
#[derive(Clone, Debug)]
pub struct PcalcProp {
    /// The element type of this atom.
    pub element: AtomElement,
    /// Arity: 1 for quantifiers and unary predicates; 2 for binary predicates; 0 otherwise.
    pub arity: u8,
    /// Predicate reference (for PREDICATE_ATOM): either a unary or binary predicate name.
    pub predicate: Option<PredicateRef>,
    /// Terms to which the predicate applies (up to MAX_ATOM_ARITY).
    pub terms: [Option<PcalcTerm>; MAX_ATOM_ARITY],
    /// Quantifier reference (for QUANTIFIER_ATOM).
    pub quantifier: Option<QuantifierRef>,
    /// Quantification parameter (e.g., the 3 in "all three").
    pub quantification_parameter: i32,
    /// Next atom in the proposition linked list.
    pub next: Option<Box<PcalcProp>>,
}

impl PcalcProp {
    /// Create a new atom with the given element type.
    ///
    /// Corresponds to `Atoms::new` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 65-70).
    pub fn new(element: AtomElement) -> Self {
        PcalcProp {
            element,
            arity: 0,
            predicate: None,
            terms: [None, None],
            quantifier: None,
            quantification_parameter: 0,
            next: None,
        }
    }

    /// Create a quantifier atom.
    ///
    /// Corresponds to `Atoms::QUANTIFIER_new` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 72-90).
    pub fn quantifier_new(quant: QuantifierRef, v: u8, parameter: i32) -> Self {
        PcalcProp {
            element: AtomElement::Quantifier,
            arity: 1,
            predicate: None,
            terms: [Some(PcalcTerm::new_variable(v)), None],
            quantifier: Some(quant),
            quantification_parameter: parameter,
            next: None,
        }
    }

    /// Create a unary predicate atom.
    ///
    /// Corresponds to `Atoms::unary_PREDICATE_new` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 92-120).
    pub fn unary_predicate_new(predicate_name: impl Into<Arc<str>>, term: PcalcTerm) -> Self {
        PcalcProp {
            element: AtomElement::Predicate,
            arity: 1,
            predicate: Some(PredicateRef::Unary(predicate_name.into())),
            terms: [Some(term), None],
            quantifier: None,
            quantification_parameter: 0,
            next: None,
        }
    }

    /// Create a binary predicate atom.
    ///
    /// Corresponds to `Atoms::binary_PREDICATE_new` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 122-188).
    pub fn binary_predicate_new(
        predicate_name: impl Into<Arc<str>>,
        term0: PcalcTerm,
        term1: PcalcTerm,
    ) -> Self {
        PcalcProp {
            element: AtomElement::Predicate,
            arity: 2,
            predicate: Some(PredicateRef::Binary(predicate_name.into())),
            terms: [Some(term0), Some(term1)],
            quantifier: None,
            quantification_parameter: 0,
            next: None,
        }
    }

    /// Create an empty proposition (no atoms).
    ///
    /// Used as a placeholder or sentinel value.
    pub fn new_empty() -> Self {
        PcalcProp {
            element: AtomElement::Predicate,
            arity: 0,
            predicate: None,
            terms: [None, None],
            quantifier: None,
            quantification_parameter: 0,
            next: None,
        }
    }

    /// Create a unary predicate atom from an owned `UnaryPredicate`.
    ///
    /// Corresponds to `Atoms::unary_PREDICATE_new` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 92-120),
    /// but takes an owned `UnaryPredicate` instead of a string name.
    ///
    /// The predicate is stored using a static name derived from the family.
    pub fn unary_predicate_new_from_up(up: UnaryPredicate, term: PcalcTerm) -> Self {
        PcalcProp {
            element: AtomElement::Predicate,
            arity: 1,
            predicate: Some(PredicateRef::Unary(Arc::from(up.family.name))),
            terms: [Some(term), None],
            quantifier: None,
            quantification_parameter: 0,
            next: None,
        }
    }

    /// Returns true if this atom is a quantifier.
    ///
    /// Corresponds to `Atoms::is_quantifier` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 104-108).
    pub fn is_quantifier(&self) -> bool {
        self.element == AtomElement::Quantifier
    }

    /// Returns the quantifier reference, if this is a quantifier atom.
    ///
    /// Corresponds to `Atoms::get_quantifier` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 110-115).
    pub fn get_quantifier(&self) -> Option<QuantifierRef> {
        if self.is_quantifier() {
            self.quantifier.clone()
        } else {
            None
        }
    }

    /// Returns true if this is an existential quantifier.
    ///
    /// Corresponds to `Atoms::is_existence_quantifier` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 117-122).
    pub fn is_existence_quantifier(&self) -> bool {
        matches!(self.quantifier, Some(QuantifierRef::Exists))
    }

    /// Returns true if this is a universal quantifier.
    ///
    /// Corresponds to `Atoms::is_forall_quantifier` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 124-129).
    pub fn is_forall_quantifier(&self) -> bool {
        matches!(self.quantifier, Some(QuantifierRef::ForAll))
    }

    /// Returns the binary predicate name if this is a binary predicate atom.
    ///
    /// Corresponds to `Atoms::is_binary_predicate` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 131-140).
    pub fn is_binary_predicate(&self) -> Option<&str> {
        match &self.predicate {
            Some(PredicateRef::Binary(name)) => Some(name.as_ref()),
            _ => None,
        }
    }

    /// Returns true if this is the equality predicate.
    ///
    /// Corresponds to `Atoms::is_equality_predicate` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 142-150).
    pub fn is_equality_predicate(&self) -> bool {
        matches!(&self.predicate, Some(PredicateRef::Binary(name)) if name.as_ref() == "equality")
    }

    /// Validate this atom.
    ///
    /// Checks arity, predicate presence, and quantifier variable.
    ///
    /// Corresponds to `Atoms::validate` in the C reference
    /// (`services/calculus-module/Chapter 4/Atomic Propositions.w`, lines 223-239).
    pub fn validate(&self) -> Result<(), String> {
        match self.element {
            AtomElement::Quantifier => {
                if self.quantifier.is_none() {
                    return Err("quantifier atom must have a quantifier reference".to_string());
                }
                if self.arity != 1 {
                    return Err("quantifier atom must have arity 1".to_string());
                }
                if self.terms[0].is_none() {
                    return Err("quantifier atom must have a variable term".to_string());
                }
                Ok(())
            }
            AtomElement::Predicate => {
                if self.predicate.is_none() {
                    return Err("predicate atom must have a predicate reference".to_string());
                }
                if self.arity == 0 {
                    return Err("predicate atom must have arity > 0".to_string());
                }
                if self.arity > MAX_ATOM_ARITY as u8 {
                    return Err(format!(
                        "predicate atom arity {} exceeds max {}",
                        self.arity, MAX_ATOM_ARITY
                    ));
                }
                for i in 0..self.arity as usize {
                    if self.terms[i].is_none() {
                        return Err(format!("predicate atom missing term at position {i}"));
                    }
                }
                Ok(())
            }
            AtomElement::NegationOpen | AtomElement::NegationClose => {
                if self.arity != 0 {
                    return Err("negation bracket must have arity 0".to_string());
                }
                Ok(())
            }
            AtomElement::DomainOpen | AtomElement::DomainClose => {
                if self.arity != 0 {
                    return Err("domain bracket must have arity 0".to_string());
                }
                Ok(())
            }
        }
    }
}

impl Display for PcalcProp {
    /// Display a proposition (linked list of atoms).
    ///
    /// Prints `<< atom1 ^ atom2 ^ atom3 >>` with conjunction markers (`^`)
    /// between implied conjunctions.
    ///
    /// Corresponds to `Propositions::write` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 146-156).
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Collect all atoms in the linked list
        let mut atoms: Vec<&PcalcProp> = Vec::new();
        let mut current = Some(self);
        while let Some(atom) = current {
            atoms.push(atom);
            current = atom.next.as_deref();
        }

        write!(f, "<<")?;
        for (i, atom) in atoms.iter().enumerate() {
            if i > 0 {
                // Simple heuristic for conjunction: no conjunction after a closer
                // or before a closer, or between quantifier and domain
                if atom.element.is_closer()
                    || atoms[i - 1].element.is_closer()
                    || atoms[i - 1].is_quantifier()
                    || atoms[i - 1].element == AtomElement::DomainOpen
                {
                    write!(f, " ")?;
                } else {
                    write!(f, " ^ ")?;
                }
            } else {
                write!(f, " ")?;
            }
            atom.write_atom(f)?;
        }
        write!(f, " >>")?;
        Ok(())
    }
}

impl PcalcProp {
    /// Write just this single atom (no linked list traversal).
    fn write_atom(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.element {
            AtomElement::Quantifier => {
                let qname = match self.quantifier {
                    Some(QuantifierRef::Exists) => "EXISTS",
                    Some(QuantifierRef::ForAll) => "FORALL",
                    Some(QuantifierRef::NotExists) => "NOTEXISTS",
                    Some(QuantifierRef::NotForAll) => "NOTFORALL",
                    None => "QUANTIFIER",
                };
                write!(f, "{qname}")?;
                if let Some(term) = &self.terms[0] {
                    write!(f, "({term})")?;
                }
                if self.quantification_parameter != 0 {
                    write!(f, " [{}]", self.quantification_parameter)?;
                }
                Ok(())
            }
            AtomElement::Predicate => {
                match &self.predicate {
                    Some(PredicateRef::Unary(name)) => {
                        write!(f, "{name}")?;
                        if let Some(term) = &self.terms[0] {
                            write!(f, "({term})")?;
                        }
                    }
                    Some(PredicateRef::Binary(name)) => {
                        if name.as_ref() == "equality" {
                            // Special notation for equality
                            if let (Some(t0), Some(t1)) = (&self.terms[0], &self.terms[1]) {
                                write!(f, "({t0} == {t1})")?;
                            } else {
                                write!(f, "{name}")?;
                            }
                        } else {
                            write!(f, "{name}")?;
                            if let Some(t0) = &self.terms[0] {
                                write!(f, "({t0}")?;
                                if let Some(t1) = &self.terms[1] {
                                    write!(f, ", {t1}")?;
                                }
                                write!(f, ")")?;
                            }
                        }
                    }
                    None => {
                        write!(f, "PREDICATE")?;
                    }
                }
                Ok(())
            }
            AtomElement::NegationOpen => write!(f, "NOT<"),
            AtomElement::NegationClose => write!(f, "NOT>"),
            AtomElement::DomainOpen => write!(f, "IN<"),
            AtomElement::DomainClose => write!(f, "IN>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // AtomElement tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_atom_element_is_opener() {
        assert!(AtomElement::NegationOpen.is_opener());
        assert!(AtomElement::DomainOpen.is_opener());
        assert!(!AtomElement::NegationClose.is_opener());
        assert!(!AtomElement::DomainClose.is_opener());
        assert!(!AtomElement::Quantifier.is_opener());
        assert!(!AtomElement::Predicate.is_opener());
    }

    #[test]
    fn test_atom_element_is_closer() {
        assert!(AtomElement::NegationClose.is_closer());
        assert!(AtomElement::DomainClose.is_closer());
        assert!(!AtomElement::NegationOpen.is_closer());
        assert!(!AtomElement::DomainOpen.is_closer());
        assert!(!AtomElement::Quantifier.is_closer());
        assert!(!AtomElement::Predicate.is_closer());
    }

    #[test]
    fn test_atom_element_counterpart() {
        assert_eq!(
            AtomElement::NegationOpen.counterpart(),
            Some(AtomElement::NegationClose)
        );
        assert_eq!(
            AtomElement::NegationClose.counterpart(),
            Some(AtomElement::NegationOpen)
        );
        assert_eq!(
            AtomElement::DomainOpen.counterpart(),
            Some(AtomElement::DomainClose)
        );
        assert_eq!(
            AtomElement::DomainClose.counterpart(),
            Some(AtomElement::DomainOpen)
        );
        assert_eq!(AtomElement::Quantifier.counterpart(), None);
        assert_eq!(AtomElement::Predicate.counterpart(), None);
    }

    // -----------------------------------------------------------------------
    // PcalcProp creation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_negation_open() {
        let a = PcalcProp::new(AtomElement::NegationOpen);
        assert_eq!(a.element, AtomElement::NegationOpen);
        assert_eq!(a.arity, 0);
        assert!(a.predicate.is_none());
        assert!(a.quantifier.is_none());
        assert!(a.next.is_none());
    }

    #[test]
    fn test_quantifier_new_creates_quantifier_atom() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert_eq!(a.element, AtomElement::Quantifier);
        assert_eq!(a.arity, 1);
        assert_eq!(a.quantifier, Some(QuantifierRef::Exists));
        assert!(a.terms[0].is_some());
        assert_eq!(a.terms[0].as_ref().unwrap().variable, 0);
    }

    #[test]
    fn test_unary_predicate_new_creates_unary_atom() {
        let term = PcalcTerm::new_variable(0);
        let a = PcalcProp::unary_predicate_new("kind=number", term);
        assert_eq!(a.element, AtomElement::Predicate);
        assert_eq!(a.arity, 1);
        assert_eq!(a.predicate, Some(PredicateRef::Unary(Arc::from("kind=number"))));
        assert!(a.terms[0].is_some());
    }

    #[test]
    fn test_binary_predicate_new_creates_binary_atom() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert_eq!(a.element, AtomElement::Predicate);
        assert_eq!(a.arity, 2);
        assert_eq!(a.predicate, Some(PredicateRef::Binary(Arc::from("equality"))));
        assert!(a.terms[0].is_some());
        assert!(a.terms[1].is_some());
    }

    // -----------------------------------------------------------------------
    // Accessor tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert!(a.is_quantifier());
        let a = PcalcProp::new(AtomElement::Predicate);
        assert!(!a.is_quantifier());
    }

    #[test]
    fn test_is_existence_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert!(a.is_existence_quantifier());
        let a = PcalcProp::quantifier_new(QuantifierRef::ForAll, 0, 0);
        assert!(!a.is_existence_quantifier());
    }

    #[test]
    fn test_is_forall_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::ForAll, 0, 0);
        assert!(a.is_forall_quantifier());
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert!(!a.is_forall_quantifier());
    }

    #[test]
    fn test_get_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert_eq!(a.get_quantifier(), Some(QuantifierRef::Exists));
        let a = PcalcProp::new(AtomElement::Predicate);
        assert_eq!(a.get_quantifier(), None);
    }

    #[test]
    fn test_is_binary_predicate_returns_name() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert_eq!(a.is_binary_predicate(), Some("equality"));
    }

    #[test]
    fn test_is_binary_predicate_returns_none_for_unary() {
        let term = PcalcTerm::new_variable(0);
        let a = PcalcProp::unary_predicate_new("kind=number", term);
        assert_eq!(a.is_binary_predicate(), None);
    }

    #[test]
    fn test_is_equality_predicate() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert!(a.is_equality_predicate());
    }

    #[test]
    fn test_is_equality_predicate_false_for_other() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("contains", t0, t1);
        assert!(!a.is_equality_predicate());
    }

    // -----------------------------------------------------------------------
    // Validation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_valid_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_unary_predicate() {
        let term = PcalcTerm::new_variable(0);
        let a = PcalcProp::unary_predicate_new("kind=number", term);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_binary_predicate() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_brackets() {
        let a = PcalcProp::new(AtomElement::NegationOpen);
        assert!(a.validate().is_ok());
        let a = PcalcProp::new(AtomElement::NegationClose);
        assert!(a.validate().is_ok());
        let a = PcalcProp::new(AtomElement::DomainOpen);
        assert!(a.validate().is_ok());
        let a = PcalcProp::new(AtomElement::DomainClose);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn test_validate_quantifier_missing_quantifier() {
        let mut a = PcalcProp::new(AtomElement::Quantifier);
        a.arity = 1;
        a.terms[0] = Some(PcalcTerm::new_variable(0));
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_validate_predicate_missing_predicate() {
        let mut a = PcalcProp::new(AtomElement::Predicate);
        a.arity = 1;
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_validate_predicate_missing_term() {
        let mut a = PcalcProp::new(AtomElement::Predicate);
        a.arity = 1;
        a.predicate = Some(PredicateRef::Unary(Arc::from("test")));
        assert!(a.validate().is_err());
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_display_quantifier() {
        let a = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert_eq!(format!("{a}"), "<< EXISTS(x) >>");
    }

    #[test]
    fn test_display_forall() {
        let a = PcalcProp::quantifier_new(QuantifierRef::ForAll, 1, 0);
        assert_eq!(format!("{a}"), "<< FORALL(y) >>");
    }

    #[test]
    fn test_display_unary_predicate() {
        let term = PcalcTerm::new_variable(0);
        let a = PcalcProp::unary_predicate_new("kind=number", term);
        assert_eq!(format!("{a}"), "<< kind=number(x) >>");
    }

    #[test]
    fn test_display_binary_predicate() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("contains", t0, t1);
        assert_eq!(format!("{a}"), "<< contains(x, hello) >>");
    }

    #[test]
    fn test_display_equality() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let a = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert_eq!(format!("{a}"), "<< (x == hello) >>");
    }

    #[test]
    fn test_display_negation_brackets() {
        let a = PcalcProp::new(AtomElement::NegationOpen);
        assert_eq!(format!("{a}"), "<< NOT< >>");
        let a = PcalcProp::new(AtomElement::NegationClose);
        assert_eq!(format!("{a}"), "<< NOT> >>");
    }

    #[test]
    fn test_display_domain_brackets() {
        let a = PcalcProp::new(AtomElement::DomainOpen);
        assert_eq!(format!("{a}"), "<< IN< >>");
        let a = PcalcProp::new(AtomElement::DomainClose);
        assert_eq!(format!("{a}"), "<< IN> >>");
    }
}
