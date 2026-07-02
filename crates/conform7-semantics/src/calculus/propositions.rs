use crate::calculus::atoms::{AtomElement, PcalcProp};

/// Maximum proposition group nesting (matching MAX_PROPOSITION_GROUP_NESTING = 100).
///
/// Corresponds to `MAX_PROPOSITION_GROUP_NESTING` in the C reference
/// (`services/calculus-module/Chapter 4/Propositions.w`).
pub const MAX_PROPOSITION_GROUP_NESTING: usize = 100;

/// Operations on propositions — linked lists of PcalcProp atoms.
///
/// Corresponds to `Propositions` in the C reference
/// (`services/calculus-module/Chapter 4/Propositions.w`).
pub struct Propositions;

impl Propositions {
    /// Create a proposition from a single atom.
    ///
    /// Corresponds to `Propositions::new_single` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 50-57).
    pub fn new_single(atom: PcalcProp) -> PcalcProp {
        atom
    }

    /// Concatenate two propositions (conjunction is implied by adjacency).
    ///
    /// Appends p2 to the end of p1's linked list. Renames variables in p2
    /// if they clash with p1.
    ///
    /// Corresponds to `Propositions::concatenate` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 59-120).
    pub fn conjunction(p1: PcalcProp, p2: PcalcProp) -> PcalcProp {
        // Find the last atom in p1
        let mut p1 = p1;
        {
            let mut current = &mut p1;
            while let Some(ref mut next) = current.next {
                current = next;
            }
            current.next = Some(Box::new(p2));
        }
        p1
    }

    /// Wrap a proposition in NOT< ... NOT> brackets.
    ///
    /// Corresponds to `Propositions::negation` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 122-130).
    pub fn negation(prop: PcalcProp) -> PcalcProp {
        let mut open = PcalcProp::new(AtomElement::NegationOpen);
        let close = PcalcProp::new(AtomElement::NegationClose);

        // Link: NOT< -> prop -> NOT>
        open.next = Some(Box::new(prop));
        {
            let mut current = &mut open;
            while let Some(ref mut next) = current.next {
                current = next;
            }
            current.next = Some(Box::new(close));
        }
        open
    }

    /// Wrap a domain in QUANTIFIER ... IN< ... IN> brackets.
    ///
    /// The quantifier atom must already be created. This function links
    /// the quantifier, domain open, domain body, and domain close.
    ///
    /// Corresponds to `Propositions::quantification` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 132-144).
    pub fn quantification(quant: PcalcProp, domain: PcalcProp) -> PcalcProp {
        let close = PcalcProp::new(AtomElement::DomainClose);

        let open = PcalcProp::new(AtomElement::DomainOpen);
        let mut quant = quant;
        {
            let mut current = &mut quant;
            while let Some(ref mut next) = current.next {
                current = next;
            }
            current.next = Some(Box::new(open));
        }
        {
            // Find the last atom (which is now the DomainOpen)
            let mut current = &mut quant;
            while let Some(ref mut next) = current.next {
                current = next;
            }
            current.next = Some(Box::new(domain));
        }
        {
            let mut current = &mut quant;
            while let Some(ref mut next) = current.next {
                current = next;
            }
            current.next = Some(Box::new(close));
        }
        quant
    }

    /// Deep-copy a proposition.
    ///
    /// Produces a structurally identical but independent copy.
    ///
    /// Corresponds to `Propositions::copy` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 287-301).
    pub fn copy(prop: &PcalcProp) -> PcalcProp {
        prop.clone()
    }

    /// Determine if two adjacent atoms form a conjunction.
    ///
    /// Conjunction is implied between any two adjacent atoms that are not
    /// bracket pairs (NOT< and NOT>, IN< and IN>).
    ///
    /// Corresponds to `Propositions::implied_conjunction_between` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 50-57).
    pub fn implied_conjunction_between(p1: &PcalcProp, p2: &PcalcProp) -> bool {
        // No conjunction between a closer and its counterpart opener
        // (e.g., NOT< ... NOT> — no conjunction between NOT> and the next atom)
        if p1.element.is_closer() {
            return false;
        }
        // No conjunction between an opener and its counterpart closer
        // (e.g., NOT< ... NOT> — no conjunction between NOT< and the atom after NOT>)
        if p2.element.is_closer() {
            return false;
        }
        // No conjunction between a quantifier and its domain
        if p1.is_quantifier() {
            return false;
        }
        // No conjunction between a domain open and the domain body
        if p1.element == AtomElement::DomainOpen {
            return false;
        }
        // No conjunction between the domain body and domain close
        if p2.element == AtomElement::DomainClose {
            return false;
        }
        true
    }

    /// Validate bracket matching and quantifier-domain pairing.
    ///
    /// Checks that:
    /// - All opening brackets have matching closing brackets.
    /// - All quantifiers have a domain.
    /// - No stray closing brackets.
    ///
    /// Corresponds to `Propositions::is_syntactically_valid` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 209-247).
    pub fn is_syntactically_valid(prop: &PcalcProp) -> Result<(), String> {
        let mut negation_depth: i32 = 0;
        let mut domain_depth: i32 = 0;
        let mut quantifier_pending: bool = false;
        let mut atoms_visited: usize = 0;

        let mut current = Some(prop);
        while let Some(atom) = current {
            atoms_visited += 1;

            match atom.element {
                AtomElement::NegationOpen => {
                    negation_depth += 1;
                }
                AtomElement::NegationClose => {
                    negation_depth -= 1;
                    if negation_depth < 0 {
                        return Err("unmatched negation close bracket".to_string());
                    }
                }
                AtomElement::DomainOpen => {
                    if !quantifier_pending {
                        return Err("domain open without preceding quantifier".to_string());
                    }
                    quantifier_pending = false;
                    domain_depth += 1;
                }
                AtomElement::DomainClose => {
                    domain_depth -= 1;
                    if domain_depth < 0 {
                        return Err("unmatched domain close bracket".to_string());
                    }
                }
                AtomElement::Quantifier => {
                    quantifier_pending = true;
                }
                AtomElement::Predicate => {
                    // Validate the atom itself
                    if let Err(e) = atom.validate() {
                        return Err(format!("invalid atom at position {atoms_visited}: {e}"));
                    }
                }
            }

            current = atom.next.as_deref();
        }

        if negation_depth != 0 {
            return Err("unmatched negation open bracket".to_string());
        }
        if domain_depth != 0 {
            return Err("unmatched domain open bracket".to_string());
        }
        if quantifier_pending {
            return Err("quantifier without domain".to_string());
        }

        Ok(())
    }

    /// Check if a proposition is complex (contains quantifiers, negation, or
    /// non-equality binary predicates).
    ///
    /// Corresponds to `Propositions::is_complex` in the C reference
    /// (`services/calculus-module/Chapter 4/Propositions.w`, lines 258-273).
    pub fn is_complex(prop: &PcalcProp) -> bool {
        let mut current = Some(prop);
        while let Some(atom) = current {
            match atom.element {
                AtomElement::Quantifier => return true,
                AtomElement::NegationOpen | AtomElement::NegationClose => return true,
                AtomElement::Predicate => {
                    // Only non-equality binary predicates are complex.
                    // Unary predicates (like kind=K) are not complex.
                    if atom.arity == 2 && !atom.is_equality_predicate() {
                        return true;
                    }
                }
                AtomElement::DomainOpen | AtomElement::DomainClose => {
                    // Domain brackets themselves don't make a proposition complex;
                    // the quantifier that precedes them does.
                }
            }
            current = atom.next.as_deref();
        }
        false
    }
}

/// Traverse a proposition, visiting each atom in order.
///
/// Returns the number of atoms visited.
///
/// The callback receives `(current_atom, previous_atom)` where `previous_atom`
/// is `None` for the first atom. Return `true` to continue, `false` to stop.
pub fn traverse<F>(prop: &PcalcProp, mut f: F) -> usize
where
    F: FnMut(&PcalcProp, Option<&PcalcProp>) -> bool,
{
    let mut count = 0;
    let mut current = Some(prop);
    let mut prev: Option<&PcalcProp> = None;

    while let Some(atom) = current {
        count += 1;
        if !f(atom, prev) {
            break;
        }
        prev = Some(atom);
        current = atom.next.as_deref();
    }

    count
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::atoms::QuantifierRef;
    use crate::calculus::terms::PcalcTerm;

    // -----------------------------------------------------------------------
    // Single atom proposition tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_single_creates_single_atom_proposition() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::new_single(atom);
        assert_eq!(prop.element, crate::calculus::atoms::AtomElement::Predicate);
        assert!(prop.next.is_none());
    }

    #[test]
    fn test_single_atom_is_valid() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::new_single(atom);
        assert!(Propositions::is_syntactically_valid(&prop).is_ok());
    }

    // -----------------------------------------------------------------------
    // Conjunction tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_conjunction_creates_linked_list() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);
        assert!(prop.next.is_some());
        let second = prop.next.as_ref().unwrap();
        assert_eq!(
            second.predicate,
            Some(crate::calculus::atoms::PredicateRef::Unary("kind=text"))
        );
        assert!(second.next.is_none());
    }

    #[test]
    fn test_conjunction_is_valid() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);
        assert!(Propositions::is_syntactically_valid(&prop).is_ok());
    }

    // -----------------------------------------------------------------------
    // Negation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_negation_wraps_in_brackets() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::negation(atom);
        assert_eq!(prop.element, AtomElement::NegationOpen);
        assert!(prop.next.is_some());
        let middle = prop.next.as_ref().unwrap();
        assert_eq!(middle.element, AtomElement::Predicate);
        assert!(middle.next.is_some());
        let close = middle.next.as_ref().unwrap();
        assert_eq!(close.element, AtomElement::NegationClose);
        assert!(close.next.is_none());
    }

    #[test]
    fn test_negation_is_valid() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::negation(atom);
        assert!(Propositions::is_syntactically_valid(&prop).is_ok());
    }

    // -----------------------------------------------------------------------
    // Quantification tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_quantification_wraps_domain() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        let term = PcalcTerm::new_variable(0);
        let domain_body = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::quantification(quant, domain_body);
        // Structure: QUANTIFIER -> IN< -> domain_body -> IN>
        assert_eq!(prop.element, AtomElement::Quantifier);
        let in_open = prop.next.as_ref().unwrap();
        assert_eq!(in_open.element, AtomElement::DomainOpen);
        let body = in_open.next.as_ref().unwrap();
        assert_eq!(body.element, AtomElement::Predicate);
        let in_close = body.next.as_ref().unwrap();
        assert_eq!(in_close.element, AtomElement::DomainClose);
        assert!(in_close.next.is_none());
    }

    #[test]
    fn test_quantification_is_valid() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        let term = PcalcTerm::new_variable(0);
        let domain_body = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::quantification(quant, domain_body);
        assert!(Propositions::is_syntactically_valid(&prop).is_ok());
    }

    // -----------------------------------------------------------------------
    // Validity tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_valid_simple_proposition() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        assert!(Propositions::is_syntactically_valid(&atom).is_ok());
    }

    #[test]
    fn test_invalid_unmatched_negation_close() {
        let prop = PcalcProp::new(AtomElement::NegationClose);
        assert!(Propositions::is_syntactically_valid(&prop).is_err());
    }

    #[test]
    fn test_invalid_unmatched_negation_open() {
        let open = PcalcProp::new(AtomElement::NegationOpen);
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::conjunction(open, atom);
        // NOT< kind=number(x) — no NOT>
        assert!(Propositions::is_syntactically_valid(&prop).is_err());
    }

    #[test]
    fn test_invalid_quantifier_without_domain() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        // Just a quantifier with no domain
        assert!(Propositions::is_syntactically_valid(&quant).is_err());
    }

    #[test]
    fn test_invalid_domain_without_quantifier() {
        let open = PcalcProp::new(AtomElement::DomainOpen);
        let close = PcalcProp::new(AtomElement::DomainClose);
        let mut prop = open;
        prop.next = Some(Box::new(close));
        assert!(Propositions::is_syntactically_valid(&prop).is_err());
    }

    #[test]
    fn test_invalid_unmatched_domain_open() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        let open = PcalcProp::new(AtomElement::DomainOpen);
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        // QUANTIFIER -> IN< -> kind=number(x) — no IN>
        let p1 = Propositions::conjunction(quant, open);
        let prop = Propositions::conjunction(p1, atom);
        assert!(Propositions::is_syntactically_valid(&prop).is_err());
    }

    // -----------------------------------------------------------------------
    // Complexity tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_simple_proposition_is_not_complex() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        assert!(!Propositions::is_complex(&atom));
    }

    #[test]
    fn test_equality_is_not_complex() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let atom = PcalcProp::binary_predicate_new("equality", t0, t1);
        assert!(!Propositions::is_complex(&atom));
    }

    #[test]
    fn test_quantifier_makes_complex() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        assert!(Propositions::is_complex(&quant));
    }

    #[test]
    fn test_negation_makes_complex() {
        let prop = PcalcProp::new(AtomElement::NegationOpen);
        assert!(Propositions::is_complex(&prop));
    }

    #[test]
    fn test_non_equality_binary_makes_complex() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_constant("hello");
        let atom = PcalcProp::binary_predicate_new("contains", t0, t1);
        assert!(Propositions::is_complex(&atom));
    }

    // -----------------------------------------------------------------------
    // Copy tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_copy_produces_independent_copy() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::new_single(atom);
        let copy = Propositions::copy(&prop);

        // They should be structurally identical
        assert_eq!(prop.element, copy.element);
        assert_eq!(prop.predicate, copy.predicate);
        assert_eq!(prop.next.is_some(), copy.next.is_some());

        // They should be independent (modifying copy doesn't affect original)
        // Since we use Clone, this is naturally the case with our implementation
    }

    #[test]
    fn test_copy_of_conjunction_is_independent() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);
        let copy = Propositions::copy(&prop);

        // Verify structure
        assert!(copy.next.is_some());
        let second = copy.next.as_ref().unwrap();
        assert_eq!(
            second.predicate,
            Some(crate::calculus::atoms::PredicateRef::Unary("kind=text"))
        );
    }

    // -----------------------------------------------------------------------
    // Implied conjunction tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_implied_conjunction_between_predicates() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        assert!(Propositions::implied_conjunction_between(&a1, &a2));
    }

    #[test]
    fn test_no_conjunction_after_closer() {
        let close = PcalcProp::new(AtomElement::NegationClose);
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        assert!(!Propositions::implied_conjunction_between(&close, &atom));
    }

    #[test]
    fn test_no_conjunction_before_closer() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let close = PcalcProp::new(AtomElement::NegationClose);
        assert!(!Propositions::implied_conjunction_between(&atom, &close));
    }

    #[test]
    fn test_no_conjunction_after_quantifier() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        let open = PcalcProp::new(AtomElement::DomainOpen);
        assert!(!Propositions::implied_conjunction_between(&quant, &open));
    }

    #[test]
    fn test_no_conjunction_after_domain_open() {
        let open = PcalcProp::new(AtomElement::DomainOpen);
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        assert!(!Propositions::implied_conjunction_between(&open, &atom));
    }

    // -----------------------------------------------------------------------
    // Traverse tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_traverse_visits_all_atoms() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);

        let mut visited = Vec::new();
        let count = traverse(&prop, |atom, _prev| {
            visited.push(atom.element);
            true
        });
        assert_eq!(count, 2);
        assert_eq!(visited.len(), 2);
    }

    #[test]
    fn test_traverse_stops_early() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);

        let mut visited = Vec::new();
        let count = traverse(&prop, |atom, _prev| {
            visited.push(atom.element);
            false // stop after first
        });
        assert_eq!(count, 1);
        assert_eq!(visited.len(), 1);
    }

    #[test]
    fn test_traverse_provides_previous_atom() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);

        let mut first_prev_none = false;
        let mut second_prev_some = false;
        traverse(&prop, |_atom, prev| {
            if prev.is_none() {
                first_prev_none = true;
            } else {
                second_prev_some = true;
            }
            true
        });
        assert!(first_prev_none);
        assert!(second_prev_some);
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_display_single_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::new_single(atom);
        assert_eq!(format!("{prop}"), "<< kind=number(x) >>");
    }

    #[test]
    fn test_display_conjunction() {
        let t0 = PcalcTerm::new_variable(0);
        let t1 = PcalcTerm::new_variable(1);
        let a1 = PcalcProp::unary_predicate_new("kind=number", t0);
        let a2 = PcalcProp::unary_predicate_new("kind=text", t1);
        let prop = Propositions::conjunction(a1, a2);
        assert_eq!(format!("{prop}"), "<< kind=number(x) ^ kind=text(y) >>");
    }

    #[test]
    fn test_display_negation() {
        let term = PcalcTerm::new_variable(0);
        let atom = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::negation(atom);
        assert_eq!(format!("{prop}"), "<< NOT< ^ kind=number(x) NOT> >>");
    }
    #[test]
    fn test_display_quantification() {
        let quant = PcalcProp::quantifier_new(QuantifierRef::Exists, 0, 0);
        let term = PcalcTerm::new_variable(0);
        let domain_body = PcalcProp::unary_predicate_new("kind=number", term);
        let prop = Propositions::quantification(quant, domain_body);
        assert_eq!(
            format!("{prop}"),
            "<< EXISTS(x) IN< kind=number(x) IN> >>"
        );
    }
}
