use std::sync::LazyLock;

use crate::calculus::atoms::{AtomElement, PcalcProp, PredicateRef};
use crate::calculus::terms::PcalcTerm;
use crate::calculus::unary_predicate_families::{UpFamily, UpFamilyMethods};
use crate::calculus::unary_predicates::UnaryPredicate;

// ---------------------------------------------------------------------------
// Predicate name constants
// ---------------------------------------------------------------------------

/// Predicate name for the calling family.
///
/// Calling predicates may encode a calling name and optional kind in the
/// predicate name string using the format `"calling"`, `"calling=name"`, or
/// `"calling=name:kind"`.
pub const CALLING_PREDICATE: &str = "calling";

/// Predicate name for the is-a-var family.
pub const IS_A_VAR_PREDICATE: &str = "is-a-var";

/// Predicate name for the is-a-const family.
pub const IS_A_CONST_PREDICATE: &str = "is-a-const";

/// Predicate name for the is-a-kind family.
///
/// Is-a-kind predicates may encode a kind name in the predicate name string
/// using the format `"is-a-kind"` or `"is-a-kind=kind"`.
pub const IS_A_KIND_PREDICATE: &str = "is-a-kind";

// ---------------------------------------------------------------------------
// Family statics
// ---------------------------------------------------------------------------

/// The calling unary predicate family — for naming bound variables.
///
/// Corresponds to `calling_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, line 9).
pub static CALLING_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "calling",
        UpFamilyMethods {
            log: log_calling,
            infer_kind: |_, up| up.assert_kind,
            testable: |_, _| false,
            test: |_, _| false,
            ..Default::default()
        },
    )
});

/// The is-a-var unary predicate family — for variable declarations.
///
/// Corresponds to `is_a_var_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, line 10).
pub static IS_A_VAR_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "is-a-var",
        UpFamilyMethods {
            log: log_is_a_var,
            infer_kind: |_, _| None,
            testable: |_, _| false,
            test: |_, _| false,
            ..Default::default()
        },
    )
});

/// The is-a-const unary predicate family — for constant declarations.
///
/// Corresponds to `is_a_const_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, line 11).
pub static IS_A_CONST_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "is-a-const",
        UpFamilyMethods {
            log: log_is_a_const,
            infer_kind: |_, _| None,
            testable: |_, _| false,
            test: |_, _| false,
            ..Default::default()
        },
    )
});

/// The is-a-kind unary predicate family — for kind declarations.
///
/// Corresponds to `is_a_kind_up_family` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, line 12).
pub static IS_A_KIND_UP_FAMILY: LazyLock<UpFamily> = LazyLock::new(|| {
    UpFamily::new(
        "is-a-kind",
        UpFamilyMethods {
            log: log_is_a_kind,
            infer_kind: |_, up| up.assert_kind,
            testable: |_, _| false,
            test: |_, _| false,
            ..Default::default()
        },
    )
});

// ---------------------------------------------------------------------------
// Log methods
// ---------------------------------------------------------------------------

/// Log method for the calling family.
///
/// Corresponds to `CreationPredicates::log_calling` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 177-180).
fn log_calling(_family: &UpFamily, up: &UnaryPredicate) -> String {
    let mut result = format!("called='{}'", up.calling_name.unwrap_or(""));
    if let Some(kind) = up.assert_kind {
        result.push_str(&format!(":{}", kind));
    }
    result
}

/// Log method for the is-a-var family.
///
/// Corresponds to `CreationPredicates::log_is_a_var` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 182-184).
fn log_is_a_var(_family: &UpFamily, _up: &UnaryPredicate) -> String {
    "is-a-var".to_string()
}

/// Log method for the is-a-const family.
///
/// Corresponds to `CreationPredicates::log_is_a_const` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 186-188).
fn log_is_a_const(_family: &UpFamily, _up: &UnaryPredicate) -> String {
    "is-a-const".to_string()
}

/// Log method for the is-a-kind family.
///
/// Corresponds to `CreationPredicates::log_is_a_kind` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 190-193).
fn log_is_a_kind(_family: &UpFamily, up: &UnaryPredicate) -> String {
    let mut result = "is-a-kind".to_string();
    if let Some(kind) = up.assert_kind {
        result.push_str(&format!("={}", kind));
    }
    result
}

// ---------------------------------------------------------------------------
// Helper: parse a calling predicate name into (calling_name, kind)
// ---------------------------------------------------------------------------

/// Parse a calling predicate name into the calling name and optional kind.
///
/// Format: `"calling"`, `"calling=name"`, or `"calling=name:kind"`.
fn parse_calling_name(predicate_name: &str) -> (Option<&str>, Option<&str>) {
    let rest = predicate_name.strip_prefix("calling").unwrap_or("");
    if rest.is_empty() {
        return (None, None);
    }
    // rest is either "=name" or "=name:kind"
    let rest = rest.strip_prefix('=').unwrap_or("");
    if rest.is_empty() {
        return (None, None);
    }
    if let Some((name, kind)) = rest.split_once(':') {
        (Some(name), Some(kind))
    } else {
        (Some(rest), None)
    }
}

/// Parse an is-a-kind predicate name into the optional kind name.
///
/// Format: `"is-a-kind"` or `"is-a-kind=kind"`.
fn parse_is_a_kind_name(predicate_name: &str) -> Option<&str> {
    let rest = predicate_name.strip_prefix("is-a-kind").unwrap_or("");
    if rest.is_empty() {
        return None;
    }
    rest.strip_prefix('=').filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// CreationPredicates struct
// ---------------------------------------------------------------------------

/// The Creation Predicates system — unary predicate families for creating
/// instances, variables, constants, and kinds during proposition assertion.
///
/// Corresponds to `CreationPredicates` in the C reference
/// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`).
///
/// Creates four unary predicate families:
/// - `calling_up_family` — for naming bound variables (called atoms)
/// - `is_a_var_up_family` — for variable declarations
/// - `is_a_const_up_family` — for constant declarations
/// - `is_a_kind_up_family` — for kind declarations
///
/// These families are used by the Assert Propositions module when processing
/// `QUANTIFIER_ATOM` atoms during proposition assertion. The calling family
/// provides a way to name bound variables (e.g., "called the den"), while
/// the is-a-var, is-a-const, and is-a-kind families mark what kind of thing
/// is being created by an existential quantifier.
///
/// Simplified:
/// - No `#ifdef CORE_MODULE` typecheck/schema methods (deferred)
/// - No `PreformUtilities::wording` (uses string names)
/// - No `wording` type (uses `&str` for names)
/// - No `LocalVariables::ensure_calling` (deferred)
/// - No `RTAdjectives::task_fn_iname` (deferred)
pub struct CreationPredicates;

impl CreationPredicates {
    /// Create the four creation predicate families with their methods.
    ///
    /// Corresponds to `CreationPredicates::start` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 22-42).
    ///
    /// Returns (families, predicates) where:
    /// - families[0] = calling_up_family
    /// - families[1] = is_a_var_up_family
    /// - families[2] = is_a_const_up_family
    /// - families[3] = is_a_kind_up_family
    /// - predicates is empty (stock methods fill it)
    pub fn start() -> (Vec<&'static UpFamily>, Vec<UnaryPredicate>) {
        (
            vec![
                &CALLING_UP_FAMILY,
                &IS_A_VAR_UP_FAMILY,
                &IS_A_CONST_UP_FAMILY,
                &IS_A_KIND_UP_FAMILY,
            ],
            Vec::new(),
        )
    }

    /// Check if a proposition atom is a calling atom.
    ///
    /// Corresponds to `CreationPredicates::is_calling_up_atom` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 59-65).
    pub fn is_calling_up_atom(prop: &PcalcProp) -> bool {
        if prop.element != AtomElement::Predicate || prop.arity != 1 {
            return false;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => *name == CALLING_PREDICATE
                || name.starts_with("calling="),
            _ => false,
        }
    }

    /// Return the kind from a calling atom.
    ///
    /// Corresponds to `CreationPredicates::what_kind_of_calling` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 67-73).
    pub fn what_kind_of_calling(prop: &PcalcProp) -> Option<&'static str> {
        if !Self::is_calling_up_atom(prop) {
            return None;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => {
                let (_, kind) = parse_calling_name(name);
                kind
            }
            _ => None,
        }
    }

    /// Create a calling atom with a name and optional kind.
    ///
    /// Corresponds to `CreationPredicates::calling_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 75-80).
    ///
    /// Simplified: uses `&'static str` for the name instead of wording.
    /// The calling name and kind are encoded in the predicate name string
    /// using the format `"calling=name"` or `"calling=name:kind"`, and
    /// interned via a global cache to produce a `&'static str`.
    pub fn calling_up(
        name: &'static str,
        term: PcalcTerm,
        kind_name: Option<&'static str>,
    ) -> PcalcProp {
        let predicate_name = match kind_name {
            Some(kind) => format!("calling={}:{}", name, kind),
            None => format!("calling={}", name),
        };
        let predicate_name = intern_predicate_name(&predicate_name);
        PcalcProp::unary_predicate_new(predicate_name, term)
    }

    /// Return the calling name from a calling atom.
    ///
    /// Corresponds to `CreationPredicates::get_calling_name` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 82-88).
    pub fn get_calling_name(prop: &PcalcProp) -> Option<&'static str> {
        if !Self::is_calling_up_atom(prop) {
            return None;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => {
                let (calling_name, _) = parse_calling_name(name);
                calling_name
            }
            _ => None,
        }
    }

    /// Stock method for the is-a-var family.
    ///
    /// Corresponds to `CreationPredicates::stock_is_a_var` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 95-99).
    ///
    /// At stock stage 1, creates the singleton is_a_var_up predicate.
    pub fn stock_is_a_var(
        _families: &mut [&'static UpFamily],
        predicates: &mut Vec<UnaryPredicate>,
        n: usize,
    ) {
        if n == 1 {
            let up = UnaryPredicate::new(&IS_A_VAR_UP_FAMILY);
            predicates.push(up);
        }
    }

    /// Stock method for the is-a-const family.
    ///
    /// Corresponds to `CreationPredicates::stock_is_a_const` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 101-105).
    ///
    /// At stock stage 1, creates the singleton is_a_const_up predicate.
    pub fn stock_is_a_const(
        _families: &mut [&'static UpFamily],
        predicates: &mut Vec<UnaryPredicate>,
        n: usize,
    ) {
        if n == 1 {
            let up = UnaryPredicate::new(&IS_A_CONST_UP_FAMILY);
            predicates.push(up);
        }
    }

    /// Create an is-a-var atom.
    ///
    /// Corresponds to `CreationPredicates::is_a_var_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 107-109).
    ///
    /// Uses the singleton is_a_var_up predicate (assumes stock has been called).
    /// Simplified: takes the predicate index from the caller.
    pub fn is_a_var_up(
        term: PcalcTerm,
        _is_a_var_idx: usize,
        _predicates: &[UnaryPredicate],
    ) -> PcalcProp {
        PcalcProp::unary_predicate_new(IS_A_VAR_PREDICATE, term)
    }

    /// Create an is-a-const atom.
    ///
    /// Corresponds to `CreationPredicates::is_a_const_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 111-113).
    ///
    /// Uses the singleton is_a_const_up predicate (assumes stock has been called).
    /// Simplified: takes the predicate index from the caller.
    pub fn is_a_const_up(
        term: PcalcTerm,
        _is_a_const_idx: usize,
        _predicates: &[UnaryPredicate],
    ) -> PcalcProp {
        PcalcProp::unary_predicate_new(IS_A_CONST_PREDICATE, term)
    }

    /// Create an is-a-kind atom with a specific kind.
    ///
    /// Corresponds to `CreationPredicates::is_a_kind_up` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 115-119).
    pub fn is_a_kind_up(term: PcalcTerm, kind_name: &'static str) -> PcalcProp {
        // Encode the kind name in the predicate name: "is-a-kind=kind"
        let predicate_name = intern_predicate_name(&format!("is-a-kind={}", kind_name));
        PcalcProp::unary_predicate_new(predicate_name, term)
    }

    /// Return the kind from an is-a-kind atom.
    ///
    /// Corresponds to `CreationPredicates::what_kind` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 121-127).
    pub fn what_kind(prop: &PcalcProp) -> Option<&'static str> {
        if prop.element != AtomElement::Predicate || prop.arity != 1 {
            return None;
        }
        match &prop.predicate {
            Some(PredicateRef::Unary(name)) => {
                if name.starts_with("is-a-kind") {
                    parse_is_a_kind_name(name)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a proposition contains any calling atoms.
    ///
    /// Corresponds to `CreationPredicates::contains_callings` in the C reference
    /// (`inform7/assertions-module/Chapter 8/The Creation Predicates.w`, lines 195-200).
    ///
    /// Simplified: takes a slice of PcalcProp atoms instead of a linked list.
    pub fn contains_callings(prop: &[PcalcProp]) -> bool {
        prop.iter().any(Self::is_calling_up_atom)
    }
}

// ---------------------------------------------------------------------------
// Global predicate name cache
// ---------------------------------------------------------------------------
use std::sync::Mutex;

/// A cache for interned predicate name strings.
///
/// This allows us to create `&'static str` predicate names from dynamic
/// strings without leaking memory. The cache is never cleared, but it
/// only stores a bounded number of predicate names.
static PREDICATE_NAME_CACHE: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Intern a predicate name string, returning a `&'static str`.
///
/// This stores the string in a global cache and returns a reference
/// with a static lifetime. The cache is never cleared, so the string
/// lives for the duration of the process.
fn intern_predicate_name(name: &str) -> &'static str {
    let mut cache = PREDICATE_NAME_CACHE.lock().unwrap();
    // Check if we already have this name
    if let Some(existing) = cache.iter().find(|s| s.as_str() == name) {
        // Safety: the cache is never cleared, so the reference is valid
        // for the lifetime of the process.
        let ptr = existing.as_str();
        // We need to extend the lifetime to 'static.
        // Since the cache owns the String and is never cleared,
        // the reference is valid for 'static.
        //
        // We use unsafe to extend the lifetime. This is safe because:
        // 1. The cache owns the String and is never cleared
        // 2. The reference points to heap memory that lives forever
        // 3. The cache is protected by a Mutex, so no concurrent modification
        unsafe { &*(ptr as *const str) }
    } else {
        cache.push(name.to_string());
        let ptr = cache.last().unwrap().as_str();
        unsafe { &*(ptr as *const str) }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::atoms::AtomElement;

    #[test]
    fn test_start_creates_four_families() {
        let (families, predicates) = CreationPredicates::start();
        assert_eq!(families.len(), 4);
        assert_eq!(families[0].name, "calling");
        assert_eq!(families[1].name, "is-a-var");
        assert_eq!(families[2].name, "is-a-const");
        assert_eq!(families[3].name, "is-a-kind");
        assert!(predicates.is_empty());
    }

    #[test]
    fn test_calling_family_has_log_method() {
        let up = UnaryPredicate::new(&CALLING_UP_FAMILY);
        let log_result = (CALLING_UP_FAMILY.methods.log)(&CALLING_UP_FAMILY, &up);
        assert_eq!(log_result, "called=''");
    }

    #[test]
    fn test_is_a_var_family_has_log_method() {
        let up = UnaryPredicate::new(&IS_A_VAR_UP_FAMILY);
        let log_result = (IS_A_VAR_UP_FAMILY.methods.log)(&IS_A_VAR_UP_FAMILY, &up);
        assert_eq!(log_result, "is-a-var");
    }

    #[test]
    fn test_is_a_const_family_has_log_method() {
        let up = UnaryPredicate::new(&IS_A_CONST_UP_FAMILY);
        let log_result = (IS_A_CONST_UP_FAMILY.methods.log)(&IS_A_CONST_UP_FAMILY, &up);
        assert_eq!(log_result, "is-a-const");
    }

    #[test]
    fn test_is_a_kind_family_has_log_method() {
        let up = UnaryPredicate::new(&IS_A_KIND_UP_FAMILY);
        let log_result = (IS_A_KIND_UP_FAMILY.methods.log)(&IS_A_KIND_UP_FAMILY, &up);
        assert_eq!(log_result, "is-a-kind");
    }

    #[test]
    fn test_is_calling_up_atom_returns_true_for_calling() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::calling_up("den", term, None);
        assert!(CreationPredicates::is_calling_up_atom(&atom));
    }

    #[test]
    fn test_is_calling_up_atom_returns_false_for_non_calling() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_var_up(term, 0, &[]);
        assert!(!CreationPredicates::is_calling_up_atom(&atom));
    }

    #[test]
    fn test_calling_up_creates_atom_with_name() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::calling_up("den", term, None);
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
        assert_eq!(
            CreationPredicates::get_calling_name(&atom),
            Some("den")
        );
    }

    #[test]
    fn test_calling_up_creates_atom_with_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::calling_up("den", term, Some("number"));
        assert_eq!(
            CreationPredicates::get_calling_name(&atom),
            Some("den")
        );
        assert_eq!(
            CreationPredicates::what_kind_of_calling(&atom),
            Some("number")
        );
    }

    #[test]
    fn test_get_calling_name_returns_none_for_non_calling() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_var_up(term, 0, &[]);
        assert_eq!(CreationPredicates::get_calling_name(&atom), None);
    }

    #[test]
    fn test_what_kind_of_calling_returns_none_for_no_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::calling_up("den", term, None);
        assert_eq!(CreationPredicates::what_kind_of_calling(&atom), None);
    }

    #[test]
    fn test_what_kind_of_calling_returns_none_for_non_calling() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_var_up(term, 0, &[]);
        assert_eq!(CreationPredicates::what_kind_of_calling(&atom), None);
    }

    #[test]
    fn test_stock_is_a_var_at_stage_1_creates_predicate() {
        let mut families = vec![&*IS_A_VAR_UP_FAMILY];
        let mut predicates = Vec::new();
        CreationPredicates::stock_is_a_var(&mut families, &mut predicates, 1);
        assert_eq!(predicates.len(), 1);
        assert_eq!(predicates[0].family.name, "is-a-var");
    }

    #[test]
    fn test_stock_is_a_var_at_other_stages_does_nothing() {
        let mut families = vec![&*IS_A_VAR_UP_FAMILY];
        let mut predicates = Vec::new();
        CreationPredicates::stock_is_a_var(&mut families, &mut predicates, 0);
        assert!(predicates.is_empty());
        CreationPredicates::stock_is_a_var(&mut families, &mut predicates, 2);
        assert!(predicates.is_empty());
    }

    #[test]
    fn test_stock_is_a_const_at_stage_1_creates_predicate() {
        let mut families = vec![&*IS_A_CONST_UP_FAMILY];
        let mut predicates = Vec::new();
        CreationPredicates::stock_is_a_const(&mut families, &mut predicates, 1);
        assert_eq!(predicates.len(), 1);
        assert_eq!(predicates[0].family.name, "is-a-const");
    }

    #[test]
    fn test_stock_is_a_const_at_other_stages_does_nothing() {
        let mut families = vec![&*IS_A_CONST_UP_FAMILY];
        let mut predicates = Vec::new();
        CreationPredicates::stock_is_a_const(&mut families, &mut predicates, 0);
        assert!(predicates.is_empty());
        CreationPredicates::stock_is_a_const(&mut families, &mut predicates, 2);
        assert!(predicates.is_empty());
    }

    #[test]
    fn test_is_a_var_up_creates_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_var_up(term, 0, &[]);
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
        match &atom.predicate {
            Some(PredicateRef::Unary(name)) => assert_eq!(*name, IS_A_VAR_PREDICATE),
            _ => panic!("expected unary predicate"),
        }
    }

    #[test]
    fn test_is_a_const_up_creates_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_const_up(term, 0, &[]);
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
        match &atom.predicate {
            Some(PredicateRef::Unary(name)) => assert_eq!(*name, IS_A_CONST_PREDICATE),
            _ => panic!("expected unary predicate"),
        }
    }

    #[test]
    fn test_is_a_kind_up_creates_atom() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_kind_up(term, "number");
        assert_eq!(atom.element, AtomElement::Predicate);
        assert_eq!(atom.arity, 1);
        assert!(atom.terms[0].is_some());
    }

    #[test]
    fn test_is_a_kind_up_creates_atom_with_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_kind_up(term, "number");
        assert_eq!(CreationPredicates::what_kind(&atom), Some("number"));
    }

    #[test]
    fn test_what_kind_returns_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_kind_up(term, "number");
        assert_eq!(CreationPredicates::what_kind(&atom), Some("number"));
    }

    #[test]
    fn test_what_kind_returns_none_for_non_is_a_kind() {
        let term = PcalcTerm::new_variable(0);
        let atom = CreationPredicates::is_a_var_up(term, 0, &[]);
        assert_eq!(CreationPredicates::what_kind(&atom), None);
    }

    #[test]
    fn test_contains_callings_returns_true_if_any_calling() {
        let term = PcalcTerm::new_variable(0);
        let calling = CreationPredicates::calling_up("den", term, None);
        let var = CreationPredicates::is_a_var_up(PcalcTerm::new_variable(1), 0, &[]);
        assert!(CreationPredicates::contains_callings(&[calling, var]));
    }

    #[test]
    fn test_contains_callings_returns_false_if_no_callings() {
        let var = CreationPredicates::is_a_var_up(PcalcTerm::new_variable(0), 0, &[]);
        let konst = CreationPredicates::is_a_const_up(PcalcTerm::new_variable(1), 0, &[]);
        assert!(!CreationPredicates::contains_callings(&[var, konst]));
    }

    #[test]
    fn test_contains_callings_returns_false_for_empty() {
        assert!(!CreationPredicates::contains_callings(&[]));
    }

    #[test]
    fn test_log_calling_with_name() {
        let mut up = UnaryPredicate::new(&CALLING_UP_FAMILY);
        up.calling_name = Some("den");
        let result = (CALLING_UP_FAMILY.methods.log)(&CALLING_UP_FAMILY, &up);
        assert_eq!(result, "called='den'");
    }

    #[test]
    fn test_log_calling_with_name_and_kind() {
        let mut up = UnaryPredicate::new(&CALLING_UP_FAMILY);
        up.calling_name = Some("den");
        up.assert_kind = Some("number");
        let result = (CALLING_UP_FAMILY.methods.log)(&CALLING_UP_FAMILY, &up);
        assert_eq!(result, "called='den':number");
    }

    #[test]
    fn test_log_is_a_var() {
        let up = UnaryPredicate::new(&IS_A_VAR_UP_FAMILY);
        let result = (IS_A_VAR_UP_FAMILY.methods.log)(&IS_A_VAR_UP_FAMILY, &up);
        assert_eq!(result, "is-a-var");
    }

    #[test]
    fn test_log_is_a_const() {
        let up = UnaryPredicate::new(&IS_A_CONST_UP_FAMILY);
        let result = (IS_A_CONST_UP_FAMILY.methods.log)(&IS_A_CONST_UP_FAMILY, &up);
        assert_eq!(result, "is-a-const");
    }

    #[test]
    fn test_log_is_a_kind() {
        let up = UnaryPredicate::new(&IS_A_KIND_UP_FAMILY);
        let result = (IS_A_KIND_UP_FAMILY.methods.log)(&IS_A_KIND_UP_FAMILY, &up);
        assert_eq!(result, "is-a-kind");
    }

    #[test]
    fn test_log_is_a_kind_with_kind() {
        let mut up = UnaryPredicate::new(&IS_A_KIND_UP_FAMILY);
        up.assert_kind = Some("number");
        let result = (IS_A_KIND_UP_FAMILY.methods.log)(&IS_A_KIND_UP_FAMILY, &up);
        assert_eq!(result, "is-a-kind=number");
    }

    #[test]
    fn test_parse_calling_name_no_name() {
        assert_eq!(parse_calling_name("calling"), (None, None));
    }

    #[test]
    fn test_parse_calling_name_with_name() {
        assert_eq!(parse_calling_name("calling=den"), (Some("den"), None));
    }

    #[test]
    fn test_parse_calling_name_with_name_and_kind() {
        assert_eq!(
            parse_calling_name("calling=den:number"),
            (Some("den"), Some("number"))
        );
    }

    #[test]
    fn test_parse_is_a_kind_name_no_kind() {
        assert_eq!(parse_is_a_kind_name("is-a-kind"), None);
    }

    #[test]
    fn test_parse_is_a_kind_name_with_kind() {
        assert_eq!(parse_is_a_kind_name("is-a-kind=number"), Some("number"));
    }
}
