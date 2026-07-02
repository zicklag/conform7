/// Property Inferences — the `property_inf` inference family.
///
/// Property inferences say that a subject has a property, which can be
/// either-or or valued. This is the first inference family created in the
/// knowledge module startup sequence.
///
/// # C Reference
///
/// - `inform7/knowledge-module/Chapter 5/Property Inferences.w` — the full
///   property inference family (355 lines)
/// - `inform7/knowledge-module/Chapter 1/Knowledge Module.w` — module startup,
///   calls `PropertyInferences::start()` (line 37)
/// - `inform7/knowledge-module/Chapter 5/Inferences.w` — `InferenceFamily`
///   infrastructure, `create_inference`, `join_inference`, `cmp`
///
/// # Simplified
///
/// Uses string names for properties instead of the full `property` struct.
/// The full property struct will be integrated in a later plan.
use crate::knowledge::inference_subjects::InferenceSubject;
use crate::knowledge::inferences::{
    Certainty, Inference, InferenceFamily, InferenceFamilyMethods, JoinResult,
};

/// Data stored in a property inference.
///
/// Corresponds to `property_inference_data` in the C reference
/// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 23-27).
///
/// Simplified: uses string names for properties instead of the full
/// `property` struct. The full property struct will be integrated in a
/// later plan.
/// The kind of value stored in a property inference.
///
/// Corresponds to the value kind tracking in the C reference
/// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    /// Value kind is not yet determined.
    Unknown,
    /// Value is not an object reference (e.g., a literal or number).
    NonObject,
    /// Value is an object reference.
    Object,
}

/// Data stored in a property inference.
///
/// Corresponds to `property_inference_data` in the C reference
/// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 23-27).
///
/// Simplified: uses string names for properties instead of the full
/// `property` struct. The full property struct will be integrated in a
/// later plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyInferenceData {
    /// The property name (simplified: string instead of `property*`).
    pub property: &'static str,
    /// The property value, if any (simplified: string instead of `parse_node*`).
    pub value: Option<&'static str>,
    /// The kind of value stored (used for bibliographic data).
    pub value_kind: ValueKind,
}

/// The property inference family — operations for creating and managing
/// property inferences.
///
/// Corresponds to the `PropertyInferences` namespace in the C reference
/// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`).
pub struct PropertyInferences;

impl PropertyInferences {
    /// Create the property inference family.
    ///
    /// Corresponds to `PropertyInferences::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 12-18).
    ///
    /// Returns the `property_inf` family with methods for logging, comparison,
    /// and contradiction explanation.
    pub fn start() -> InferenceFamily {
        InferenceFamily {
            name: "property_inf",
            methods: InferenceFamilyMethods {
                log_details: |inf| {
                    // Simplified: log the property name and value
                    if let Some(data) = inf.data {
                        format!("({})", data)
                    } else {
                        String::new()
                    }
                },
                compare: |a, b| {
                    // Compare two property inferences
                    // Returns 0 for identical, non-zero for different
                    if a.data == b.data {
                        0
                    } else {
                        1 // CI_DIFFER_IN_TOPIC
                    }
                },
                explain_contradiction: |_a, _b, _similarity, _subject| {
                    // Simplified: return true to indicate the contradiction
                    // was handled. The full C version issues problem messages.
                    true
                },
            },
        }
    }

    /// Create a new property inference.
    ///
    /// Corresponds to `PropertyInferences::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 29-40).
    ///
    /// Creates a `PropertyInferenceData` with the given property and value,
    /// stores it in the data registry, then creates an `Inference` with the
    /// prevailing mood (defaulting to `Likely` certainty).
    ///
    /// Returns the new `Inference`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        _subj: &InferenceSubject,
        prn: &'static str,
        val: Option<&'static str>,
        families: &[InferenceFamily],
        data_registry: &mut Vec<PropertyInferenceData>,
    ) -> Inference {
        // Find the property_inf family index
        let family_idx = families
            .iter()
            .position(|f| f.name == "property_inf")
            .expect("PropertyInferences::start must be called first");

        // Determine certainty: use the subject's default certainty
        // (simplified: default to LIKELY_CE for now)
        let certainty = Certainty::Likely;

        // Store the full PropertyInferenceData in the registry
        let data_idx = data_registry.len();
        data_registry.push(PropertyInferenceData {
            property: prn,
            value: val,
            value_kind: ValueKind::Unknown,
        });

        // Create the inference with the data index
        Inference {
            family: family_idx,
            data: Some(prn),
            data_index: Some(data_idx),
            certainty,
            inferred_from: None,
            drawn_during_stage: 0,
            drawn_from_metadata: false,
        }
    }

    pub fn draw(
        subj_idx: usize,
        prn: &'static str,
        val: Option<&'static str>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        subjects: &mut [InferenceSubject],
        data_registry: &mut Vec<PropertyInferenceData>,
    ) -> JoinResult {
        let inf = PropertyInferences::new(&subjects[subj_idx], prn, val, families, data_registry);
        inf.join(&mut subjects[subj_idx], families, inferences)
    }

    /// Create a property inference from metadata and join it to a subject.
    ///
    /// Corresponds to `PropertyInferences::draw_from_metadata` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 50-55).
    pub fn draw_from_metadata(
        subj_idx: usize,
        prn: &'static str,
        val: Option<&'static str>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        subjects: &mut [InferenceSubject],
        data_registry: &mut Vec<PropertyInferenceData>,
    ) -> JoinResult {
        let mut inf = PropertyInferences::new(&subjects[subj_idx], prn, val, families, data_registry);
        inf.drawn_from_metadata = true;
        inf.join(&mut subjects[subj_idx], families, inferences)
    }

    /// Create a negated property inference and join it to a subject.
    ///
    /// Corresponds to `PropertyInferences::draw_negated` in the C reference
    pub fn draw_negated(
        subj_idx: usize,
        prn: &'static str,
        val: Option<&'static str>,
        families: &[InferenceFamily],
        inferences: &mut Vec<Inference>,
        subjects: &mut [InferenceSubject],
        data_registry: &mut Vec<PropertyInferenceData>,
    ) -> JoinResult {
        let mut inf = PropertyInferences::new(&subjects[subj_idx], prn, val, families, data_registry);
        // Negate the certainty
        inf.certainty = match inf.certainty {
            Certainty::Impossible => Certainty::Certain,
            Certainty::Unlikely => Certainty::Likely,
            Certainty::Likely => Certainty::Unlikely,
            Certainty::Certain => Certainty::Impossible,
            other => other,
        };
        inf.join(&mut subjects[subj_idx], families, inferences)
    }

    /// Get the property name from a property inference.
    ///
    /// Corresponds to `PropertyInferences::get_property` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 188-192).
    pub fn get_property(inf: &Inference, data_registry: &[PropertyInferenceData]) -> Option<&'static str> {
        let idx = inf.data_index?;
        data_registry.get(idx).map(|d| d.property)
    }

    /// Get the property value from a property inference.
    ///
    /// Corresponds to `PropertyInferences::get_value` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`, lines 194-198).
    pub fn get_value(inf: &Inference, data_registry: &[PropertyInferenceData]) -> Option<&'static str> {
        let idx = inf.data_index?;
        data_registry.get(idx).and_then(|d| d.value)
    }

    /// Set the value kind for a property inference.
    ///
    /// Corresponds to `PropertyInferences::set_value_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 5/Property Inferences.w`).
    ///
    /// Forces the value's kind (used for bibliographic data).
    pub fn set_value_kind(
        inf: &Inference,
        kind: ValueKind,
        data_registry: &mut [PropertyInferenceData],
    ) {
        if let Some(idx) = inf.data_index {
            if let Some(data) = data_registry.get_mut(idx) {
                data.value_kind = kind;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::inference_subjects::InferenceSubject;
    use crate::knowledge::inferences::InferenceComparison;

    /// Helper: create a property_inf family for testing.
    fn make_property_family() -> InferenceFamily {
        PropertyInferences::start()
    }

    /// Helper: create a subject for testing.
    fn make_subject() -> InferenceSubject {
        InferenceSubject::new(0, None, None, None)
    }

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let family = PropertyInferences::start();
        assert_eq!(family.name, "property_inf");
    }

    #[test]
    fn test_start_creates_family_with_log_details_method() {
        let family = PropertyInferences::start();
        let inf = Inference::new(0, Some("colour"), Certainty::Likely);
        let log = (family.methods.log_details)(&inf);
        assert_eq!(log, "(colour)");
    }

    #[test]
    fn test_start_creates_family_with_log_details_empty_for_no_data() {
        let family = PropertyInferences::start();
        let inf = Inference::new(0, None, Certainty::Likely);
        let log = (family.methods.log_details)(&inf);
        assert_eq!(log, "");
    }

    #[test]
    fn test_new_creates_inference_with_correct_property() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        assert_eq!(inf.data, Some("colour"));
        assert_eq!(inf.data_index, Some(0));
    }

    #[test]
    fn test_new_creates_inference_with_likely_certainty() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        assert_eq!(inf.certainty, Certainty::Likely);
    }

    #[test]
    fn test_new_creates_inference_with_correct_family() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        assert_eq!(inf.family, 0);
    }

    #[test]
    fn test_new_accepts_value() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", Some("red"), &families, &mut data_registry);
        assert_eq!(inf.data, Some("colour"));
        assert_eq!(inf.data_index, Some(0));
    }

    #[test]
    fn test_draw_creates_and_joins_inference() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut inferences = Vec::new();
        let mut subjects = vec![subject];
        let mut data_registry = Vec::new();

        let result = PropertyInferences::draw(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(result, JoinResult::Joined);
        assert_eq!(subjects[0].inf_list.len(), 1);
        assert_eq!(inferences.len(), 1);
        assert_eq!(inferences[0].data, Some("colour"));
        assert_eq!(inferences[0].data_index, Some(0));
    }

    #[test]
    fn test_draw_returns_joined_for_new_inference() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut inferences = Vec::new();
        let mut subjects = vec![subject];
        let mut data_registry = Vec::new();

        let result = PropertyInferences::draw(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(result, JoinResult::Joined);
    }

    #[test]
    fn test_draw_returns_discarded_redundant_for_duplicate() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        // First draw
        PropertyInferences::draw(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);

        // Second draw with same data
        let result = PropertyInferences::draw(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(result, JoinResult::DiscardedRedundant);
    }

    #[test]
    fn test_draw_from_metadata_sets_flag() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        PropertyInferences::draw_from_metadata(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert!(inferences[0].drawn_from_metadata);
    }

    #[test]
    fn test_draw_negated_negates_certainty() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        PropertyInferences::draw_negated(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        // Likely negated -> Unlikely
        assert_eq!(inferences[0].certainty, Certainty::Unlikely);
    }

    #[test]
    fn test_draw_negated_joins_inference() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        let result = PropertyInferences::draw_negated(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(result, JoinResult::Joined);
        assert_eq!(subjects[0].inf_list.len(), 1);
    }

    #[test]
    fn test_get_property_returns_correct_name() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        assert_eq!(
            PropertyInferences::get_property(&inf, &data_registry),
            Some("colour")
        );
    }

    #[test]
    fn test_get_value_returns_stored_value() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", Some("red"), &families, &mut data_registry);
        assert_eq!(
            PropertyInferences::get_value(&inf, &data_registry),
            Some("red")
        );
    }

    #[test]
    fn test_get_value_returns_none_when_no_value() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        assert_eq!(
            PropertyInferences::get_value(&inf, &data_registry),
            None
        );
    }

    #[test]
    fn test_compare_identical_inferences() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf1 = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        let inf2 = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);

        let cmp = inf1.cmp(&inf2, &families);
        assert_eq!(cmp, InferenceComparison::Identical);
    }

    #[test]
    fn test_compare_different_inferences() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf1 = PropertyInferences::new(&subject, "colour", None, &families, &mut data_registry);
        let inf2 = PropertyInferences::new(&subject, "size", None, &families, &mut data_registry);

        let cmp = inf1.cmp(&inf2, &families);
        assert_eq!(cmp, InferenceComparison::DifferInTopic);
    }

    #[test]
    fn test_family_compare_returns_zero_for_identical() {
        let family = make_property_family();
        let a = Inference::new(0, Some("colour"), Certainty::Likely);
        let b = Inference::new(0, Some("colour"), Certainty::Likely);

        assert_eq!((family.methods.compare)(&a, &b), 0);
    }

    #[test]
    fn test_family_compare_returns_non_zero_for_different() {
        let family = make_property_family();
        let a = Inference::new(0, Some("colour"), Certainty::Likely);
        let b = Inference::new(0, Some("size"), Certainty::Likely);

        assert_eq!((family.methods.compare)(&a, &b), 1);
    }

    #[test]
    fn test_explain_contradiction_returns_true() {
        let family = make_property_family();
        let a = Inference::new(0, Some("colour"), Certainty::Likely);
        let b = Inference::new(0, Some("size"), Certainty::Likely);

        assert!((family.methods.explain_contradiction)(&a, &b, 0, 0));
    }

    #[test]
    fn test_draw_with_value_creates_inference() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        let result = PropertyInferences::draw(
            0,
            "colour",
            Some("red"),
            &families,
            &mut inferences,
            &mut subjects,
            &mut data_registry,
        );
        assert_eq!(result, JoinResult::Joined);
        assert_eq!(inferences[0].data, Some("colour"));
        assert_eq!(inferences[0].data_index, Some(0));
    }

    #[test]
    fn test_draw_negated_twice_is_redundant() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        // First negation: Likely -> Unlikely
        PropertyInferences::draw_negated(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(inferences[0].certainty, Certainty::Unlikely);

        // Second negation: creates a new inference with Likely, negates to Unlikely,
        // then join finds existing Unlikely and discards as redundant
        let result = PropertyInferences::draw_negated(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        assert_eq!(result, JoinResult::DiscardedRedundant);
        assert_eq!(inferences[0].certainty, Certainty::Unlikely);
    }

    #[test]
    fn test_draw_from_metadata_joins_inference() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        let result = PropertyInferences::draw_from_metadata(
            0,
            "colour",
            None,
            &families,
            &mut inferences,
            &mut subjects,
            &mut data_registry,
        );
        assert_eq!(result, JoinResult::Joined);
        assert_eq!(subjects[0].inf_list.len(), 1);
    }

    #[test]
    fn test_multiple_properties_on_same_subject() {
        let families = vec![make_property_family()];
        let mut inferences = Vec::new();
        let mut subjects = vec![make_subject()];
        let mut data_registry = Vec::new();

        PropertyInferences::draw(0, "colour", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        PropertyInferences::draw(0, "size", None, &families, &mut inferences, &mut subjects, &mut data_registry);
        PropertyInferences::draw(0, "weight", None, &families, &mut inferences, &mut subjects, &mut data_registry);

        assert_eq!(subjects[0].inf_list.len(), 3);
        assert_eq!(inferences.len(), 3);
    }

    #[test]
    fn test_property_inference_data_struct() {
        let data = PropertyInferenceData {
            property: "colour",
            value: Some("red"),
            value_kind: ValueKind::Unknown,
        };
        assert_eq!(data.property, "colour");
        assert_eq!(data.value, Some("red"));
        assert_eq!(data.value_kind, ValueKind::Unknown);
    }

    #[test]
    fn test_property_inference_data_no_value() {
        let data = PropertyInferenceData {
            property: "colour",
            value: None,
            value_kind: ValueKind::Unknown,
        };
        assert_eq!(data.property, "colour");
        assert_eq!(data.value, None);
    }

    #[test]
    fn test_set_value_kind_updates_kind() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let inf = PropertyInferences::new(&subject, "colour", Some("red"), &families, &mut data_registry);
        assert_eq!(data_registry[0].value_kind, ValueKind::Unknown);

        PropertyInferences::set_value_kind(&inf, ValueKind::Object, &mut data_registry);
        assert_eq!(data_registry[0].value_kind, ValueKind::Object);
    }

    #[test]
    fn test_set_value_kind_noop_for_missing_index() {
        let mut data_registry = vec![PropertyInferenceData {
            property: "colour",
            value: None,
            value_kind: ValueKind::Unknown,
        }];
        let inf = Inference::new(0, Some("colour"), Certainty::Likely);
        // No data_index set — should be a no-op
        PropertyInferences::set_value_kind(&inf, ValueKind::Object, &mut data_registry);
        assert_eq!(data_registry[0].value_kind, ValueKind::Unknown);
    }

    #[test]
    fn test_data_registry_stores_full_data() {
        let families = vec![make_property_family()];
        let subject = make_subject();
        let mut data_registry = Vec::new();

        let _inf = PropertyInferences::new(&subject, "colour", Some("red"), &families, &mut data_registry);
        assert_eq!(data_registry.len(), 1);
        assert_eq!(data_registry[0].property, "colour");
        assert_eq!(data_registry[0].value, Some("red"));
        assert_eq!(data_registry[0].value_kind, ValueKind::Unknown);
    }
}
