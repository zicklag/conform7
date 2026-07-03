/// Measurement definitions — adjectives that compare a property value against a threshold.
///
/// Corresponds to `measurement_definition` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92).
///
/// A typical example would be:
///
/// > Definition: A container is roomy if its carrying capacity is 10 or more.
///
/// Here the domain of the definition is "container", and we assign an adjective
/// meaning for "roomy" which involves the comparison of a property (here "carrying
/// capacity") against a threshold value t (here, t=10). "roomy" is said to
/// be the headword; the comparative form would be "roomier", and the superlative
/// form "roomiest".
///
/// Simplified:
/// - No `parse_node *` (creation tracking deferred)
/// - No `measurement_compilation_data` (run-time compilation deferred)
/// - No `Grading::make_superlative` (superlative form deferred)
/// - No `Grading::make_comparative` (comparative form deferred)
/// - No `Grading::make_quiddity` (quiddity form deferred)
use crate::knowledge::properties::Property;

/// Region shape constants for measurement definitions.
///
/// Corresponds to `MEASURE_T_*` constants in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 31-33).
///
/// These define how a measurement adjective compares a property value
/// against a threshold:
/// - `MEASURE_T_OR_LESS` (-1): property value <= threshold (e.g., "10 or less")
/// - `MEASURE_T_EXACTLY` (0): property value == threshold (e.g., "exactly 10")
/// - `MEASURE_T_OR_MORE` (1): property value >= threshold (e.g., "10 or more")
pub const MEASURE_T_OR_LESS: i32 = -1;
pub const MEASURE_T_EXACTLY: i32 = 0;
pub const MEASURE_T_OR_MORE: i32 = 1;

/// A measurement definition — defines an adjective that compares a property
/// value against a threshold.
///
/// Corresponds to `measurement_definition` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 74-92).
///
/// Each such definition allows the property value to belong to a "region",
/// which takes one of three "shapes": or-less, exactly, or or-more.
#[derive(Clone, Debug)]
pub struct MeasurementDefinition {
    /// The adjective being defined (headword, must be a single word).
    /// Corresponds to `headword` in the C reference.
    pub headword: String,
    /// The adjective meaning index, set by MeasurementAdjectives.
    /// Corresponds to `headword_as_adjective` in the C reference.
    pub headword_as_adjective: Option<usize>,
    /// The superlative form (e.g., "roomiest").
    /// Corresponds to `superlative` in the C reference.
    /// Deferred: set by Grading::make_superlative.
    pub superlative: Option<String>,
    /// The property being compared, if any.
    /// Corresponds to `prop` in the C reference.
    pub prop: Option<usize>,
    /// The name of the property to compare (used before the property is resolved).
    /// Corresponds to `name_of_property_to_compare` in the C reference.
    pub name_of_property_to_compare: Option<String>,
    /// The region shape: one of MEASURE_T_OR_LESS, MEASURE_T_EXACTLY, MEASURE_T_OR_MORE.
    /// Corresponds to `region_shape` in the C reference.
    pub region_shape: i32,
    /// The numerical value of the threshold.
    /// Corresponds to `region_threshold` in the C reference.
    pub region_threshold: i32,
    /// The kind of the threshold value, if known.
    /// Corresponds to `region_kind` in the C reference.
    pub region_kind: Option<usize>,
    /// Whether the threshold has been evaluated.
    /// Corresponds to `region_threshold_evaluated` in the C reference.
    pub region_threshold_evaluated: bool,
    /// The text of the threshold value (e.g., "10").
    /// Corresponds to `region_threshold_text` in the C reference.
    pub region_threshold_text: Option<String>,
}

impl MeasurementDefinition {
    /// Create a new measurement definition with default values.
    pub fn new(headword: &str) -> Self {
        MeasurementDefinition {
            headword: headword.to_string(),
            headword_as_adjective: None,
            superlative: None,
            prop: None,
            name_of_property_to_compare: None,
            region_shape: MEASURE_T_EXACTLY,
            region_threshold: 0,
            region_kind: None,
            region_threshold_evaluated: false,
            region_threshold_text: None,
        }
    }
}

/// The Measurements management module.
///
/// Corresponds to `Measurements` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Measurements.w`).
pub struct Measurements;

impl Measurements {
    /// Create a new measurement definition.
    ///
    /// Corresponds to `Measurements::new` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 95-110).
    ///
    /// Returns the index of the new definition.
    ///
    /// Simplified:
    /// - No `parse_node *` (creation tracking deferred)
    /// - No `RTAdjectives::new_measurement_compilation_data` (run-time compilation deferred)
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        headword: &str,
        prop: Option<usize>,
        shape: i32,
        threshold_text: Option<&str>,
        definitions: &mut Vec<MeasurementDefinition>,
    ) -> usize {
        let idx = definitions.len();
        let mut mdef = MeasurementDefinition::new(headword);
        mdef.prop = prop;
        mdef.region_shape = shape;
        mdef.region_threshold_text = threshold_text.map(|s| s.to_string());
        definitions.push(mdef);
        idx
    }

    /// Validate a measurement definition.
    ///
    /// Corresponds to `Measurements::validate` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 150-155).
    ///
    /// This function tries to fill in missing data:
    /// 1. If the property is missing but the property name is set, try to resolve it
    /// 2. If the threshold hasn't been evaluated, try to evaluate it
    ///
    /// Simplified:
    /// - No `<property-name>` grammar parsing (uses string matching)
    /// - No `<s-literal>` grammar parsing (uses simple number parsing)
    /// - No `Rvalues::to_kind` or `Rvalues::to_encoded_notation`
    /// - No `Kinds::Behaviour::is_quasinumerical` check
    /// - No `Kinds::compatible` check
    /// - No problem message generation
    pub fn validate(
        mdef_idx: usize,
        definitions: &mut [MeasurementDefinition],
        properties: &[Property],
    ) {
        if let Some(mdef) = definitions.get_mut(mdef_idx) {
            // Fill in missing property from name
            if mdef.prop.is_none() {
                if let Some(prn_name) = &mdef.name_of_property_to_compare {
                    for (i, prn) in properties.iter().enumerate() {
                        if prn.name == prn_name.as_str() {
                            mdef.prop = Some(i);
                            break;
                        }
                    }
                }
            }

            // Fill in missing threshold value
            if !mdef.region_threshold_evaluated {
                if let Some(threshold_text) = &mdef.region_threshold_text {
                    if let Ok(val) = threshold_text.parse::<i32>() {
                        mdef.region_threshold = val;
                        mdef.region_threshold_evaluated = true;
                    }
                }
            }
        }
    }

    /// Check if a measurement definition is fully validated.
    ///
    /// Corresponds to `Measurements::is_valid` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 220-224).
    ///
    /// Returns true if both the property and threshold are resolved.
    pub fn is_valid(mdef_idx: usize, definitions: &[MeasurementDefinition]) -> bool {
        definitions.get(mdef_idx).is_some_and(|mdef| {
            mdef.prop.is_some() && mdef.region_threshold_evaluated
        })
    }

    /// Extract the property and shape from a measurement definition.
    ///
    /// Corresponds to `Measurements::read_property_details` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 115-119).
    ///
    /// Returns (property_index, region_shape).
    pub fn read_property_details(
        mdef_idx: usize,
        definitions: &[MeasurementDefinition],
    ) -> (Option<usize>, i32) {
        if let Some(mdef) = definitions.get(mdef_idx) {
            (mdef.prop, mdef.region_shape)
        } else {
            (None, MEASURE_T_EXACTLY)
        }
    }

    /// Find a measurement definition by property and shape.
    ///
    /// Corresponds to `Measurements::retrieve` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 124-133).
    ///
    /// Validates each definition, then checks if it matches the given
    /// property and shape. Returns the index of the matching definition,
    /// or None.
    pub fn retrieve(
        prn_idx: usize,
        shape: i32,
        definitions: &mut [MeasurementDefinition],
        properties: &[Property],
    ) -> Option<usize> {
        for i in 0..definitions.len() {
            Measurements::validate(i, definitions, properties);
            if let Some(mdef) = definitions.get(i) {
                if Measurements::is_valid(i, definitions)
                    && mdef.prop == Some(prn_idx)
                    && mdef.region_shape == shape
                {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Get the weak comparison operator for a region shape.
    ///
    /// Corresponds to `Measurements::weak_comparison_bp` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 40-49).
    ///
    /// Returns a string representation of the comparison operator:
    /// - MEASURE_T_OR_MORE -> ">="
    /// - MEASURE_T_EXACTLY -> "=="
    /// - MEASURE_T_OR_LESS -> "<="
    ///
    /// Simplified: returns a string instead of a `binary_predicate *`.
    pub fn weak_comparison_bp(shape: i32) -> &'static str {
        match shape {
            MEASURE_T_OR_MORE => ">=",
            MEASURE_T_EXACTLY => "==",
            MEASURE_T_OR_LESS => "<=",
            _ => panic!("unknown region for weak comparison"),
        }
    }

    /// Get the strict comparison operator string for a region shape.
    ///
    /// Corresponds to `Measurements::strict_comparison` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 51-59).
    ///
    /// Returns a string representation of the comparison operator:
    /// - MEASURE_T_OR_MORE -> ">"
    /// - MEASURE_T_OR_LESS -> "<"
    ///
    /// Panics for MEASURE_T_EXACTLY (exact measurements don't have strict comparisons).
    pub fn strict_comparison(shape: i32) -> &'static str {
        match shape {
            MEASURE_T_OR_MORE => ">",
            MEASURE_T_OR_LESS => "<",
            _ => panic!("unknown region for strict comparison"),
        }
    }

    /// Validate all measurement definitions.
    ///
    /// Corresponds to `Measurements::validate_definitions` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 141-145).
    pub fn validate_definitions(
        definitions: &mut [MeasurementDefinition],
        properties: &[Property],
    ) {
        for i in 0..definitions.len() {
            Measurements::validate(i, definitions, properties);
        }
    }

    /// Create comparative forms for all measurement definitions.
    ///
    /// Corresponds to `Measurements::create_comparatives` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Measurements.w`, lines 231-249).
    ///
    /// Simplified: no-op. Comparative creation depends on Grading, binary
    /// predicates, and ComparativeRelations — deferred to a later plan.
    pub fn create_comparatives(
        _definitions: &[MeasurementDefinition],
        _properties: &[Property],
    ) {
        // No-op: deferred to ComparativeRelations plan
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::properties::{Property, ValuePropertyData};

    #[test]
    fn test_region_shape_constants() {
        assert_eq!(MEASURE_T_OR_LESS, -1);
        assert_eq!(MEASURE_T_EXACTLY, 0);
        assert_eq!(MEASURE_T_OR_MORE, 1);
    }

    #[test]
    fn test_new_creates_definition() {
        let mut definitions = Vec::new();
        let idx = Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

        assert_eq!(idx, 0);
        assert_eq!(definitions[0].headword, "roomy");
        assert_eq!(definitions[0].prop, Some(0));
        assert_eq!(definitions[0].region_shape, MEASURE_T_OR_MORE);
        assert_eq!(definitions[0].region_threshold_text, Some("10".to_string()));
        assert_eq!(definitions[0].region_threshold, 0);
        assert!(!definitions[0].region_threshold_evaluated);
        assert!(definitions[0].headword_as_adjective.is_none());
    }

    #[test]
    fn test_new_without_threshold() {
        let mut definitions = Vec::new();
        let idx = Measurements::new("roomy", Some(0), MEASURE_T_EXACTLY, None, &mut definitions);

        assert_eq!(idx, 0);
        assert!(definitions[0].region_threshold_text.is_none());
    }

    #[test]
    fn test_validate_fills_in_threshold() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        let properties = Vec::new();

        assert!(!definitions[0].region_threshold_evaluated);
        Measurements::validate(0, &mut definitions, &properties);
        assert!(definitions[0].region_threshold_evaluated);
        assert_eq!(definitions[0].region_threshold, 10);
    }

    #[test]
    fn test_validate_fills_in_property_from_name() {
        let mut definitions = Vec::new();
        let mut mdef = MeasurementDefinition::new("roomy");
        mdef.name_of_property_to_compare = Some("carrying capacity".to_string());
        mdef.region_shape = MEASURE_T_OR_MORE;
        mdef.region_threshold_text = Some("10".to_string());
        definitions.push(mdef);

        let properties = vec![Property {
            name: "carrying capacity",
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(ValuePropertyData {
                property_value_kind: Some("number"),
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }];

        assert!(definitions[0].prop.is_none());
        Measurements::validate(0, &mut definitions, &properties);
        assert_eq!(definitions[0].prop, Some(0));
    }

    #[test]
    fn test_is_valid_returns_true_for_validated() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        let properties = Vec::new();

        assert!(!Measurements::is_valid(0, &definitions));
        Measurements::validate(0, &mut definitions, &properties);
        assert!(Measurements::is_valid(0, &definitions));
    }

    #[test]
    fn test_is_valid_returns_false_for_unvalidated() {
        let definitions = vec![MeasurementDefinition::new("roomy")];
        assert!(!Measurements::is_valid(0, &definitions));
    }

    #[test]
    fn test_is_valid_returns_false_for_missing_property() {
        let mut definitions = Vec::new();
        let mut mdef = MeasurementDefinition::new("roomy");
        mdef.region_threshold_evaluated = true;
        mdef.region_threshold = 10;
        definitions.push(mdef);

        assert!(!Measurements::is_valid(0, &definitions));
    }

    #[test]
    fn test_read_property_details() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(42), MEASURE_T_OR_MORE, Some("10"), &mut definitions);

        let (prop, shape) = Measurements::read_property_details(0, &definitions);
        assert_eq!(prop, Some(42));
        assert_eq!(shape, MEASURE_T_OR_MORE);
    }

    #[test]
    fn test_read_property_details_invalid_index() {
        let definitions = Vec::new();
        let (prop, shape) = Measurements::read_property_details(0, &definitions);
        assert_eq!(prop, None);
        assert_eq!(shape, MEASURE_T_EXACTLY);
    }

    #[test]
    fn test_retrieve_finds_matching_definition() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        Measurements::new("compact", Some(0), MEASURE_T_OR_LESS, Some("5"), &mut definitions);
        let properties = Vec::new();

        // Validate all
        Measurements::validate_definitions(&mut definitions, &properties);

        let found = Measurements::retrieve(0, MEASURE_T_OR_MORE, &mut definitions, &properties);
        assert_eq!(found, Some(0));

        let found = Measurements::retrieve(0, MEASURE_T_OR_LESS, &mut definitions, &properties);
        assert_eq!(found, Some(1));
    }

    #[test]
    fn test_retrieve_returns_none_for_no_match() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        let properties = Vec::new();
        Measurements::validate_definitions(&mut definitions, &properties);

        let found = Measurements::retrieve(1, MEASURE_T_OR_MORE, &mut definitions, &properties);
        assert_eq!(found, None);
    }

    #[test]
    fn test_weak_comparison_bp() {
        assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_OR_MORE), ">=");
        assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_EXACTLY), "==");
        assert_eq!(Measurements::weak_comparison_bp(MEASURE_T_OR_LESS), "<=");
    }

    #[test]
    #[should_panic(expected = "unknown region for weak comparison")]
    fn test_weak_comparison_bp_invalid_shape() {
        Measurements::weak_comparison_bp(42);
    }

    #[test]
    fn test_strict_comparison() {
        assert_eq!(Measurements::strict_comparison(MEASURE_T_OR_MORE), ">");
        assert_eq!(Measurements::strict_comparison(MEASURE_T_OR_LESS), "<");
    }

    #[test]
    #[should_panic(expected = "unknown region for strict comparison")]
    fn test_strict_comparison_exact() {
        Measurements::strict_comparison(MEASURE_T_EXACTLY);
    }

    #[test]
    fn test_validate_definitions_validates_all() {
        let mut definitions = Vec::new();
        Measurements::new("roomy", Some(0), MEASURE_T_OR_MORE, Some("10"), &mut definitions);
        Measurements::new("compact", Some(1), MEASURE_T_OR_LESS, Some("5"), &mut definitions);
        let properties = Vec::new();

        assert!(!definitions[0].region_threshold_evaluated);
        assert!(!definitions[1].region_threshold_evaluated);

        Measurements::validate_definitions(&mut definitions, &properties);

        assert!(definitions[0].region_threshold_evaluated);
        assert_eq!(definitions[0].region_threshold, 10);
        assert!(definitions[1].region_threshold_evaluated);
        assert_eq!(definitions[1].region_threshold, 5);
    }

    #[test]
    fn test_create_comparatives_is_noop() {
        let definitions = Vec::new();
        let properties = Vec::new();
        // Should not panic
        Measurements::create_comparatives(&definitions, &properties);
    }

    #[test]
    fn test_measurement_definition_defaults() {
        let mdef = MeasurementDefinition::new("tall");
        assert_eq!(mdef.headword, "tall");
        assert!(mdef.headword_as_adjective.is_none());
        assert!(mdef.superlative.is_none());
        assert!(mdef.prop.is_none());
        assert!(mdef.name_of_property_to_compare.is_none());
        assert_eq!(mdef.region_shape, MEASURE_T_EXACTLY);
        assert_eq!(mdef.region_threshold, 0);
        assert!(mdef.region_kind.is_none());
        assert!(!mdef.region_threshold_evaluated);
        assert!(mdef.region_threshold_text.is_none());
    }
}
