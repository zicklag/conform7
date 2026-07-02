/// Metadata for a binary predicate term.
///
/// Corresponds to `bp_term_details` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 25-31).
///
/// Records the domain of values allowed for this term, the kind of those
/// values, and optional function-of-other schema.
#[derive(Clone, Debug)]
pub struct BpTermDetails {
    /// The domain of values allowed (inference subject index).
    /// Simplified: uses an index into a subject registry instead of
    /// `TERM_DOMAIN_CALCULUS_TYPE*` pointer.
    pub implies_infs: Option<usize>,
    /// The kind of values allowed (kind index).
    /// Simplified: uses a kind index instead of `kind*` pointer.
    pub implies_kind: Option<usize>,
    /// The "(called...)" name, if any exists.
    /// Simplified: a string instead of `wording`.
    pub called_name: Option<&'static str>,
    /// Where one term can be deduced from the other.
    /// Simplified: a string schema instead of `i6_schema*`.
    pub function_of_other: Option<&'static str>,
    /// Text to use in the Phrasebook index (usually null).
    pub index_term_as: Option<&'static str>,
}

/// Functions for creating and manipulating binary predicate term details.
///
/// Corresponds to `BPTerms` in the C reference
/// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 33-142).
pub struct BPTerms;

impl BPTerms {
    /// Create term details with a domain inference subject.
    ///
    /// Corresponds to `BPTerms::new` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 33-42).
    #[allow(clippy::new_ret_no_self)]
    pub fn new(infs: Option<usize>) -> BpTermDetails {
        BpTermDetails {
            implies_infs: infs,
            implies_kind: None,
            called_name: None,
            function_of_other: None,
            index_term_as: None,
        }
    }

    /// Create term details with a domain kind.
    ///
    /// Corresponds to `BPTerms::new_kind` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 44-53).
    pub fn new_kind(kind: Option<usize>) -> BpTermDetails {
        BpTermDetails {
            implies_infs: None,
            implies_kind: kind,
            called_name: None,
            function_of_other: None,
            index_term_as: None,
        }
    }

    /// Create term details with all fields.
    ///
    /// Corresponds to `BPTerms::new_full` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 55-68).
    pub fn new_full(
        infs: Option<usize>,
        kind: Option<usize>,
        called_name: Option<&'static str>,
        function_of_other: Option<&'static str>,
    ) -> BpTermDetails {
        BpTermDetails {
            implies_infs: infs,
            implies_kind: kind,
            called_name,
            function_of_other,
            index_term_as: None,
        }
    }

    /// Fill in the domain later (for BPs created before domains are known).
    ///
    /// Corresponds to `BPTerms::set_domain` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 70-82).
    pub fn set_domain(
        bptd: &mut BpTermDetails,
        kind: Option<usize>,
        infs: Option<usize>,
    ) {
        bptd.implies_kind = kind;
        bptd.implies_infs = infs;
    }

    /// Set the function-of-other schema.
    ///
    /// Corresponds to `BPTerms::set_function` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 84-96).
    pub fn set_function(bptd: &mut BpTermDetails, f: Option<&'static str>) {
        bptd.function_of_other = f;
    }

    /// Return the kind of a term.
    ///
    /// Corresponds to `BPTerms::kind` in the C reference
    /// (`services/calculus-module/Chapter 3/Binary Predicate Term Details.w`, lines 98-108).
    pub fn kind(bptd: &BpTermDetails) -> Option<usize> {
        bptd.implies_kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_term_details_with_inference_subject() {
        let td = BPTerms::new(Some(42));
        assert_eq!(td.implies_infs, Some(42));
        assert_eq!(td.implies_kind, None);
        assert_eq!(td.called_name, None);
        assert_eq!(td.function_of_other, None);
        assert_eq!(td.index_term_as, None);
    }

    #[test]
    fn test_new_creates_term_details_with_none_subject() {
        let td = BPTerms::new(None);
        assert_eq!(td.implies_infs, None);
        assert_eq!(td.implies_kind, None);
    }

    #[test]
    fn test_new_kind_creates_term_details_with_kind() {
        let td = BPTerms::new_kind(Some(7));
        assert_eq!(td.implies_kind, Some(7));
        assert_eq!(td.implies_infs, None);
        assert_eq!(td.called_name, None);
        assert_eq!(td.function_of_other, None);
    }

    #[test]
    fn test_new_kind_creates_term_details_with_none_kind() {
        let td = BPTerms::new_kind(None);
        assert_eq!(td.implies_kind, None);
    }

    #[test]
    fn test_new_full_creates_term_details_with_all_fields() {
        let td = BPTerms::new_full(
            Some(1),
            Some(2),
            Some("called foo"),
            Some("function_of_other_schema"),
        );
        assert_eq!(td.implies_infs, Some(1));
        assert_eq!(td.implies_kind, Some(2));
        assert_eq!(td.called_name, Some("called foo"));
        assert_eq!(td.function_of_other, Some("function_of_other_schema"));
        assert_eq!(td.index_term_as, None);
    }

    #[test]
    fn test_new_full_creates_term_details_with_none_fields() {
        let td = BPTerms::new_full(None, None, None, None);
        assert_eq!(td.implies_infs, None);
        assert_eq!(td.implies_kind, None);
        assert_eq!(td.called_name, None);
        assert_eq!(td.function_of_other, None);
    }

    #[test]
    fn test_set_domain_updates_kind_and_infs() {
        let mut td = BPTerms::new(Some(42));
        assert_eq!(td.implies_kind, None);

        BPTerms::set_domain(&mut td, Some(99), Some(100));
        assert_eq!(td.implies_kind, Some(99));
        assert_eq!(td.implies_infs, Some(100));
    }

    #[test]
    fn test_set_domain_can_clear_fields() {
        let mut td = BPTerms::new_full(Some(1), Some(2), None, None);
        BPTerms::set_domain(&mut td, None, None);
        assert_eq!(td.implies_kind, None);
        assert_eq!(td.implies_infs, None);
    }

    #[test]
    fn test_set_function_updates_function_of_other() {
        let mut td = BPTerms::new(Some(42));
        assert_eq!(td.function_of_other, None);

        BPTerms::set_function(&mut td, Some("my_function"));
        assert_eq!(td.function_of_other, Some("my_function"));
    }

    #[test]
    fn test_set_function_can_clear() {
        let mut td = BPTerms::new_full(Some(1), Some(2), None, Some("old_fn"));
        BPTerms::set_function(&mut td, None);
        assert_eq!(td.function_of_other, None);
    }

    #[test]
    fn test_kind_returns_implies_kind() {
        let td = BPTerms::new_kind(Some(7));
        assert_eq!(BPTerms::kind(&td), Some(7));
    }

    #[test]
    fn test_kind_returns_none_when_not_set() {
        let td = BPTerms::new(Some(42));
        assert_eq!(BPTerms::kind(&td), None);
    }
}
