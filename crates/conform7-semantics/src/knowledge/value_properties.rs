/// The Valued Properties system — management functions for valued property data.
///
/// Corresponds to `ValueProperties` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`).
///
/// Valued properties store values of a specific kind. Each valued property
/// has an associated setting relation (a binary predicate) that sets its value.
///
/// Simplified:
/// - No Preform grammar for property name parsing
/// - No kind validation or problem messages
/// - No P_grammatical_gender special case
/// - No Properties::can_name_coincide_with_kind check (always calls make_kind_coincident)
/// - No Propositions::Abstract::to_set_property (assert deferred)
/// - No Assert::true_about (assert deferred)
/// - No RTProperties::dont_show_in_index (new_nameless deferred)
/// - No nameless property creation (new_nameless deferred)
use crate::calculus::binary_predicates::BinaryPredicate;
use crate::knowledge::instances::Instances;
use crate::knowledge::properties::{Properties, Property, ValuePropertyData};
use crate::knowledge::setting_property_relation::SettingPropertyRelations;

/// Operations on valued properties.
///
/// Corresponds to `ValueProperties` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`).
pub struct ValueProperties;

impl ValueProperties {
    /// Create new valued property data.
    ///
    /// Corresponds to `ValueProperties::new_value_data` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 19-27).
    pub fn new_value_data() -> ValuePropertyData {
        ValuePropertyData {
            property_value_kind: None,
            setting_bp: None,
            name_coincides_with_kind: false,
            as_condition_of_subject: None,
            relation_whose_state_this_stores: None,
        }
    }

    /// Return the kind of a valued property.
    ///
    /// Corresponds to `ValueProperties::kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 139-142).
    ///
    /// Returns None for either-or properties or properties without value data.
    pub fn kind(prn_idx: usize, properties: &[Property]) -> Option<&'static str> {
        let prn = &properties[prn_idx];
        if prn.either_or_data.is_some() {
            return None;
        }
        prn.value_data.as_ref()?.property_value_kind
    }

    /// Set the kind of a valued property.
    ///
    /// Corresponds to `ValueProperties::set_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 144-173).
    ///
    /// Simplified:
    /// - No kind validation (definite check, problem messages)
    /// - No RTProperties::can_be_compiled check
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn set_kind(prn_idx: usize, kind_name: &'static str, properties: &mut [Property]) {
        let prn = &mut properties[prn_idx];
        assert!(prn.either_or_data.is_none(), "non-value property");
        let vd = prn.value_data.as_mut().expect("non-value property");
        vd.property_value_kind = Some(kind_name);
    }
    /// Make a property name coincide with a kind name.
    ///
    /// Corresponds to `ValueProperties::make_coincide_with_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 182-189).
    ///
    /// Simplified:
    /// - No P_grammatical_gender special case
    /// - No Properties::can_name_coincide_with_kind check (always calls make_kind_coincident)
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn make_coincide_with_kind(
        prn_idx: usize,
        kind_name: &'static str,
        properties: &mut [Property],
        instances: &mut [crate::knowledge::instances::Instance],
    ) {
        Self::set_kind(prn_idx, kind_name, properties);
        let prn = &mut properties[prn_idx];
        let vd = prn.value_data.as_mut().expect("non-value property");
        vd.name_coincides_with_kind = true;
        Instances::make_kind_coincident(kind_name, prn_idx, instances);
    }

    /// Check if a property name coincides with a kind name.
    ///
    /// Corresponds to `ValueProperties::coincides_with_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 191-194).
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn coincides_with_kind(prn_idx: usize, properties: &[Property]) -> bool {
        let prn = &properties[prn_idx];
        assert!(prn.either_or_data.is_none(), "non-value property");
        prn.value_data
            .as_ref()
            .expect("non-value property")
            .name_coincides_with_kind
    }

    /// Create the setting binary predicate for a valued property.
    ///
    /// Corresponds to `ValueProperties::make_setting_bp` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 121-128).
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn make_setting_bp(
        prn_idx: usize,
        property_name: &str,
        bp_registry: &mut Vec<BinaryPredicate>,
        properties: &mut [Property],
    ) {
        {
            let prn = &properties[prn_idx];
            assert!(prn.either_or_data.is_none(), "non-value property");
            assert!(prn.value_data.is_some(), "non-value property");
        }

        // Find or create the setting BP.
        let bp_idx =
            match SettingPropertyRelations::find_set_property_BP(property_name, bp_registry) {
                Some(idx) => idx,
                None => SettingPropertyRelations::make_set_property_BP(property_name, bp_registry),
            };

        // Fix the BP and its reversal.
        // SAFETY: fix_property_bp casts &[()] back to &[Property] internally.
        let property_registry: &[()] =
            unsafe { &*(properties as *const [Property] as *const [()]) };
        SettingPropertyRelations::fix_property_bp(bp_idx, bp_registry, property_registry);
        if let Some(rev_idx) = bp_registry[bp_idx].reversal {
            SettingPropertyRelations::fix_property_bp(rev_idx, bp_registry, property_registry);
        }

        // Store the BP index in the property's value data.
        let prn = &mut properties[prn_idx];
        let vd = prn.value_data.as_mut().expect("non-value property");
        vd.setting_bp = Some(bp_idx);
    }

    /// Get the setting binary predicate for a valued property.
    ///
    /// Corresponds to `ValueProperties::get_setting_bp` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 130-133).
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn get_setting_bp(prn_idx: usize, properties: &[Property]) -> Option<usize> {
        let prn = &properties[prn_idx];
        assert!(prn.either_or_data.is_none(), "non-value property");
        prn.value_data
            .as_ref()
            .expect("non-value property")
            .setting_bp
    }

    /// Set the stored relation for a valued property.
    ///
    /// Corresponds to `ValueProperties::set_stored_relation` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 200-203).
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn set_stored_relation(prn_idx: usize, bp_idx: usize, properties: &mut [Property]) {
        let prn = &mut properties[prn_idx];
        assert!(prn.either_or_data.is_none(), "non-value property");
        let vd = prn.value_data.as_mut().expect("non-value property");
        vd.relation_whose_state_this_stores = Some(bp_idx);
    }

    /// Get the stored relation for a valued property.
    ///
    /// Corresponds to `ValueProperties::get_stored_relation` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 205-208).
    ///
    /// Panics if the property is either-or or has no value data.
    pub fn get_stored_relation(prn_idx: usize, properties: &[Property]) -> Option<usize> {
        let prn = &properties[prn_idx];
        assert!(prn.either_or_data.is_none(), "non-value property");
        prn.value_data
            .as_ref()
            .expect("non-value property")
            .relation_whose_state_this_stores
    }

    /// Find or create a valued property by name.
    ///
    /// Corresponds to `ValueProperties::obtain` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 35-37).
    ///
    /// Wraps `Properties::obtain` with `valued = true`.
    pub fn obtain(name: &'static str, properties: &mut Vec<Property>) -> usize {
        Properties::obtain(name, true, properties)
    }

    /// Find or create a valued property with a specific kind.
    ///
    /// Corresponds to `ValueProperties::obtain_within_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 44-66).
    ///
    /// Simplified:
    /// - No Preform grammar for property name parsing
    /// - No kind compatibility checking (SOMETIMES_MATCH, NEVER_MATCH)
    /// - No problem messages for incompatible kinds
    /// - No Kinds::weaken
    ///
    /// If a property with the given name already exists and has value data,
    /// sets its kind if not already set. If no property exists, creates one
    /// and sets its kind.
    pub fn obtain_within_kind(
        name: &'static str,
        kind_name: &'static str,
        properties: &mut Vec<Property>,
    ) -> usize {
        // Check if a property with this name already exists.
        if let Some(idx) = properties.iter().position(|p| p.name == name) {
            // Property exists — set the kind if not already set.
            if let Some(vd) = &mut properties[idx].value_data {
                if vd.property_value_kind.is_none() {
                    vd.property_value_kind = Some(kind_name);
                }
            }
            return idx;
        }
        // Create a new valued property with the given kind.
        let idx = Properties::obtain(name, true, properties);
        if let Some(vd) = &mut properties[idx].value_data {
            vd.property_value_kind = Some(kind_name);
        }
        idx
    }

    /// Check if a kind name can coincide with a property name.
    ///
    /// Corresponds to `Properties::can_name_coincide_with_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 548-551).
    ///
    /// Simplified: returns true for all kinds (the C reference delegates to
    /// `K->construct->can_coincide_with_property` which is a kind system detail).
    pub fn can_name_coincide_with_kind(_kind_name: &str) -> bool {
        true
    }

    /// Assert a property value for a subject.
    ///
    /// Corresponds to `ValueProperties::assert` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 213-217).
    ///
    /// Deferred: depends on `Propositions::Abstract::to_set_property` and
    /// `Assert::true_about` from the assertions module.
    ///
    /// Returns false (no-op).
    pub fn assert(
        _prn_idx: usize,
        _owner_idx: usize,
        _val: &str,
        _certainty: i32,
    ) -> bool {
        false
    }
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::properties::Properties;
    use crate::knowledge::setting_property_relation::SettingPropertyRelations;

    // -----------------------------------------------------------------------
    // ValueProperties::new_value_data
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_value_data_creates_data_with_no_kind() {
        let vd = ValueProperties::new_value_data();
        assert!(vd.property_value_kind.is_none());
        assert!(vd.setting_bp.is_none());
        assert!(!vd.name_coincides_with_kind);
        assert!(vd.as_condition_of_subject.is_none());
        assert!(vd.relation_whose_state_this_stores.is_none());
    }

    // -----------------------------------------------------------------------
    // ValueProperties::kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_kind_returns_value_kind() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        ValueProperties::set_kind(idx, "number", &mut registry);
        assert_eq!(ValueProperties::kind(idx, &registry), Some("number"));
    }

    #[test]
    fn test_kind_returns_none_when_not_set() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        assert_eq!(ValueProperties::kind(idx, &registry), None);
    }

    #[test]
    fn test_kind_returns_none_for_either_or() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        assert_eq!(ValueProperties::kind(idx, &registry), None);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::set_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_kind_sets_value_kind() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        ValueProperties::set_kind(idx, "number", &mut registry);
        assert_eq!(
            ValueProperties::kind(idx, &registry),
            Some("number")
        );
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_set_kind_panics_for_either_or() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        ValueProperties::set_kind(idx, "number", &mut registry);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::make_coincide_with_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_coincide_with_kind_sets_kind_and_marks_coincidence() {
        let mut registry = Vec::new();
        let mut instances = Vec::new();
        let idx = Properties::create("number", false, &mut registry);
        ValueProperties::make_coincide_with_kind(idx, "number", &mut registry, &mut instances);
        assert_eq!(
            ValueProperties::kind(idx, &registry),
            Some("number")
        );
        assert!(ValueProperties::coincides_with_kind(idx, &registry));
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_make_coincide_with_kind_panics_for_either_or() {
        let mut registry = Vec::new();
        let mut instances = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        ValueProperties::make_coincide_with_kind(idx, "truth_state", &mut registry, &mut instances);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::coincides_with_kind
    // -----------------------------------------------------------------------
    #[test]
    fn test_coincides_with_kind_returns_true_after_make_coincide() {
        let mut registry = Vec::new();
        let mut instances = Vec::new();
        let idx = Properties::create("number", false, &mut registry);
        ValueProperties::make_coincide_with_kind(idx, "number", &mut registry, &mut instances);
        assert!(ValueProperties::coincides_with_kind(idx, &registry));
    }

    #[test]
    fn test_coincides_with_kind_returns_false_for_normal_property() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        assert!(!ValueProperties::coincides_with_kind(idx, &registry));
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_coincides_with_kind_panics_for_either_or() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        let _ = ValueProperties::coincides_with_kind(idx, &registry);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::make_setting_bp
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_setting_bp_creates_bp_and_stores_in_value_data() {
        // Need a setting property family for BP creation.
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let _families = families; // keep alive

        let mut properties = Vec::new();
        let idx = Properties::create("weight", false, &mut properties);

        ValueProperties::make_setting_bp(idx, "weight", &mut bp_registry, &mut properties);

        // The setting BP should be stored in the property's value data.
        let stored_bp = ValueProperties::get_setting_bp(idx, &properties);
        assert!(stored_bp.is_some());

        // The BP should be in the registry.
        let bp_idx = stored_bp.unwrap();
        assert!(bp_idx < bp_registry.len());
    }

    #[test]
    fn test_make_setting_bp_creates_new_bp_for_different_properties() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let _families = families;

        let mut properties = Vec::new();
        let weight_idx = Properties::create("weight", false, &mut properties);
        let height_idx = Properties::create("height", false, &mut properties);

        ValueProperties::make_setting_bp(weight_idx, "weight", &mut bp_registry, &mut properties);
        ValueProperties::make_setting_bp(height_idx, "height", &mut bp_registry, &mut properties);

        // Each property should have its own setting BP.
        let weight_bp = ValueProperties::get_setting_bp(weight_idx, &properties);
        let height_bp = ValueProperties::get_setting_bp(height_idx, &properties);
        assert!(weight_bp.is_some());
        assert!(height_bp.is_some());
        // Different properties should have different BPs.
        assert_ne!(weight_bp, height_bp);
    }
    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_make_setting_bp_panics_for_either_or() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let _families = families;

        let mut properties = Vec::new();
        let idx = Properties::create("open", true, &mut properties);

        ValueProperties::make_setting_bp(idx, "open", &mut bp_registry, &mut properties);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::get_setting_bp
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_setting_bp_returns_none_when_not_set() {
        let mut properties = Vec::new();
        let idx = Properties::create("weight", false, &mut properties);
        assert_eq!(ValueProperties::get_setting_bp(idx, &properties), None);
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_get_setting_bp_panics_for_either_or() {
        let mut properties = Vec::new();
        let idx = Properties::create("open", true, &mut properties);
        let _ = ValueProperties::get_setting_bp(idx, &properties);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::set_stored_relation / get_stored_relation
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_stored_relation_round_trip() {
        let mut properties = Vec::new();
        let idx = Properties::create("weight", false, &mut properties);

        // Initially no stored relation.
        assert_eq!(ValueProperties::get_stored_relation(idx, &properties), None);

        // Set a stored relation.
        ValueProperties::set_stored_relation(idx, 42, &mut properties);
        assert_eq!(ValueProperties::get_stored_relation(idx, &properties), Some(42));

        // Overwrite with a different value.
        ValueProperties::set_stored_relation(idx, 99, &mut properties);
        assert_eq!(ValueProperties::get_stored_relation(idx, &properties), Some(99));
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_set_stored_relation_panics_for_either_or() {
        let mut properties = Vec::new();
        let idx = Properties::create("open", true, &mut properties);
        ValueProperties::set_stored_relation(idx, 42, &mut properties);
    }

    #[test]
    #[should_panic(expected = "non-value property")]
    fn test_get_stored_relation_panics_for_either_or() {
        let mut properties = Vec::new();
        let idx = Properties::create("open", true, &mut properties);
        let _ = ValueProperties::get_stored_relation(idx, &properties);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::obtain
    // -----------------------------------------------------------------------

    #[test]
    fn test_obtain_creates_new_valued_property() {
        let mut properties = Vec::new();
        let idx = ValueProperties::obtain("weight", &mut properties);
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[idx].name, "weight");
        assert!(properties[idx].value_data.is_some());
        assert!(properties[idx].either_or_data.is_none());
    }

    #[test]
    fn test_obtain_returns_existing_valued_property() {
        let mut properties = Vec::new();
        let idx1 = ValueProperties::obtain("weight", &mut properties);
        let idx2 = ValueProperties::obtain("weight", &mut properties);
        assert_eq!(idx1, idx2);
        assert_eq!(properties.len(), 1);
    }

    // -----------------------------------------------------------------------
    // ValueProperties::obtain_within_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_obtain_within_kind_creates_new_property_with_kind() {
        let mut properties = Vec::new();
        let idx = ValueProperties::obtain_within_kind("weight", "number", &mut properties);
        assert_eq!(properties[idx].name, "weight");
        assert_eq!(
            ValueProperties::kind(idx, &properties),
            Some("number")
        );
    }

    #[test]
    fn test_obtain_within_kind_sets_kind_on_existing_property() {
        let mut properties = Vec::new();
        let idx = ValueProperties::obtain("weight", &mut properties);
        // Kind should be None initially.
        assert_eq!(ValueProperties::kind(idx, &properties), None);

        // Now obtain within kind — should set the kind.
        let idx2 = ValueProperties::obtain_within_kind("weight", "number", &mut properties);
        assert_eq!(idx, idx2);
        assert_eq!(
            ValueProperties::kind(idx, &properties),
            Some("number")
        );
    }

    #[test]
    fn test_obtain_within_kind_does_not_overwrite_existing_kind() {
        let mut properties = Vec::new();
        let idx = ValueProperties::obtain_within_kind("weight", "number", &mut properties);
        assert_eq!(
            ValueProperties::kind(idx, &properties),
            Some("number")
        );

        // Try to obtain with a different kind — should not overwrite.
        let idx2 = ValueProperties::obtain_within_kind("weight", "text", &mut properties);
        assert_eq!(idx, idx2);
        // Kind should still be "number" (not overwritten).
        assert_eq!(
            ValueProperties::kind(idx, &properties),
            Some("number")
        );
    }

    // -----------------------------------------------------------------------
    // ValueProperties::can_name_coincide_with_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_name_coincide_with_kind_returns_true() {
        assert!(ValueProperties::can_name_coincide_with_kind("number"));
        assert!(ValueProperties::can_name_coincide_with_kind("text"));
        assert!(ValueProperties::can_name_coincide_with_kind(""));
    }

    // -----------------------------------------------------------------------
    // ValueProperties::assert
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_returns_false() {
        assert!(!ValueProperties::assert(0, 0, "test", 1));
    }
}
