/// The setting property relation — sets a property value on a subject.
///
/// Corresponds to `SettingPropertyRelations` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`).
///
/// Creates one bp_family instance:
/// - property_setting_bp_family — for the setting-property relation
///
/// Each valued property gets one make_pair in this family. For example,
/// if there is a valued property "weight", then a relation "set-weight"
/// is created to serve as the meaning of "the weight of X is Y".
///
/// Timing problem: BPs are created before properties exist. The property
/// name is stored as pending text and resolved at stage 2 when all
/// properties have been created.
///
/// Simplified:
/// - No Preform grammar for property name resolution
/// - No RTProperties::iname (run-time compilation)
/// - No Calculus::Schemas (simplified string schemas)
/// - No Wordings::match (simplified string comparison)
use crate::calculus::binary_predicate_families::{BpFamily, BpFamilyMethods};
use crate::calculus::binary_predicates::{BinaryPredicate, BinaryPredicates};
use crate::calculus::bp_term_details::BPTerms;
use crate::knowledge::properties::Property;
use crate::knowledge::property_inferences::PropertyInferences;

// ---------------------------------------------------------------------------
// Global constants for family and predicate indices
// ---------------------------------------------------------------------------

/// Index of the setting property family in the family registry.
pub const PROPERTY_SETTING_FAMILY: usize = 0;

/// The setting property relation module.
///
/// Corresponds to `SettingPropertyRelations` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`).
pub struct SettingPropertyRelations;

// ---------------------------------------------------------------------------
// Family-specific data encoding
// ---------------------------------------------------------------------------

/// Prefix for family_specific data when the BP has pending property text.
const PENDING_PREFIX: &str = "pending:";

/// Prefix for family_specific data when the BP has a resolved property.
const PROPERTY_PREFIX: &str = "property:";

/// Data for a setting property BP, stored as an encoded string in
/// `family_specific`.
///
/// Corresponds to `property_setting_bp_data` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 54-58).
///
/// When a BP is created before the property exists, the property name
/// is stored as pending text (encoded as `"pending:<name>"`). At stage 2,
/// this is resolved to an actual property index (encoded as `"property:<idx>"`).
///
/// # Encoding
///
/// - `"pending:<property_name>"` — BP has pending text, property not yet resolved
/// - `"property:<index>"` — BP has a resolved property at the given index
pub struct PropertySettingBpData;

impl PropertySettingBpData {
    /// Create a pending encoding from a property name.
    fn encode_pending(name: &str) -> String {
        format!("{PENDING_PREFIX}{name}")
    }
    /// Create a resolved encoding from a property index.
    fn encode_property(idx: usize) -> String {
        format!("{PROPERTY_PREFIX}{idx}")
    }

    /// Extract the pending property name from family_specific data.
    fn get_pending(family_specific: &Option<String>) -> Option<&str> {
        match family_specific {
            Some(s) => s.strip_prefix(PENDING_PREFIX),
            None => None,
        }
    }
    /// Extract the resolved property index from family_specific data.
    fn get_property(family_specific: &Option<String>) -> Option<usize> {
        match family_specific {
            Some(s) => s.strip_prefix(PROPERTY_PREFIX).and_then(|n| n.parse().ok()),
            None => None,
        }
    }
}

// ---------------------------------------------------------------------------
// SettingPropertyRelations implementation
// ---------------------------------------------------------------------------

impl SettingPropertyRelations {
    /// Create the setting property family with its methods.
    ///
    /// Corresponds to `SettingPropertyRelations::start` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 11-21).
    ///
    /// Returns (families, bp_registry) where:
    /// - families[0] = property_setting_bp_family
    /// - bp_registry is empty (stocking fills it)
    pub fn start() -> (Vec<BpFamily>, Vec<BinaryPredicate>) {
        let setting_family = BpFamily {
            name: "property_setting",
            methods: BpFamilyMethods {
                stock: Some(SettingPropertyRelations::stock),
                typecheck: Some(SettingPropertyRelations::typecheck),
                assert: Some(SettingPropertyRelations::assert),
                schema: Some(SettingPropertyRelations::schema),
                ..BpFamilyMethods::default()
            },
        };

        (vec![setting_family], Vec::new())
    }

    /// Stock the setting property family (stage 2): resolve pending property text.
    ///
    /// Corresponds to `SettingPropertyRelations::stock` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 73-84).
    ///
    /// At stage 2, iterates over all BPs in the family and resolves any
    /// pending property text to actual property indices.
    ///
    /// The `property_registry` parameter is the list of all known properties.
    #[allow(clippy::ptr_arg)]
    pub fn stock(
        _family: &BpFamily,
        n: u8,
        bp_registry: &mut Vec<BinaryPredicate>,
        property_registry: &[Property],
    ) {
        if n == 2 {
            let family_idx = PROPERTY_SETTING_FAMILY;

            // Collect indices of BPs to fix (avoid borrow issues with mutable iteration).
            let mut to_fix: Vec<usize> = Vec::new();
            for (bp_idx, bp) in bp_registry.iter().enumerate() {
                if bp.relation_family == family_idx
                    && PropertySettingBpData::get_pending(&bp.family_specific).is_some()
                {
                    to_fix.push(bp_idx);
                }
            }

            // Resolve each pending BP.
            for bp_idx in to_fix {
                Self::fix_property_bp_internal(bp_idx, bp_registry, property_registry);
            }
        }
    }

    /// Internal helper: fix a single BP's pending text, given the property registry.
    ///
    /// This is the core logic of `fix_property_bp`, factored out so `stock` can
    /// pass the property registry directly.
    #[allow(non_snake_case, clippy::too_many_arguments)]
    fn fix_property_bp_internal(
        bp_idx: usize,
        bp_registry: &mut [BinaryPredicate],
        properties: &[Property],
    ) {
        let pending = {
            let bp = &bp_registry[bp_idx];
            if bp.relation_family != PROPERTY_SETTING_FAMILY {
                return;
            }
            match PropertySettingBpData::get_pending(&bp.family_specific) {
                Some(name) => name.to_string(),
                None => return,
            }
        };

        // Look up the property by name.
        let prn_idx = match properties.iter().position(|p| p.name == pending) {
            Some(idx) => idx,
            None => return, // Property not found — leave pending (deferred resolution).
        };
        // Update the BP's family_specific to the resolved property index.
        let encoded = PropertySettingBpData::encode_property(prn_idx);
        bp_registry[bp_idx].family_specific = Some(encoded.clone());

        // Also update the reversal's family_specific.
        if let Some(rev_idx) = bp_registry[bp_idx].reversal {
            if rev_idx < bp_registry.len() {
                bp_registry[rev_idx].family_specific = Some(encoded);
            }
        }

        // Set up schemas for the right-way-round BP.
        if bp_registry[bp_idx].right_way_round {
            Self::set_property_BP_schemas_internal(bp_idx, prn_idx, bp_registry, properties);
        } else if let Some(rev_idx) = bp_registry[bp_idx].reversal {
            Self::set_property_BP_schemas_internal(rev_idx, prn_idx, bp_registry, properties);
        }
    }

    #[allow(non_snake_case)]
    fn set_property_BP_schemas_internal(
        bp_idx: usize,
        prn_idx: usize,
        bp_registry: &mut [BinaryPredicate],
        properties: &[Property],
    ) {
        let prn = &properties[prn_idx];
        let prop_name = prn.name;

        // Set TEST_ATOM_TASK (index 1) and NOW_ATOM_TRUE_TASK (index 2).
        // Corresponds to Calculus::Schemas::new("*1.%n == *2", RTProperties::iname(prn))
        // and Calculus::Schemas::new("*1.%n = *2", RTProperties::iname(prn)).
        // Simplified: use string schemas with the property name.
        bp_registry[bp_idx].task_functions[1] = Some(format!("*1.{prop_name} == *2"));
        bp_registry[bp_idx].task_functions[2] = Some(format!("*1.{prop_name} = *2"));

        // Set the domain of the right term to the property's value kind.
        // Corresponds to BPTerms::set_domain(&(bp->term_details[1]),
        //     ValueProperties::kind(prn));
        // Simplified: use the property's kind_of_contents as a kind index hint.
        // Since we don't have a full kind system, we store the kind name as a
        // string in implies_kind (which is Option<usize>). For now, we leave it
        // as None since the kind system is not yet fully integrated.
        // BPTerms::set_domain(&mut bp_registry[bp_idx].term_details[1], None, None);
    }

    /// Typecheck the setting property relation.
    ///
    /// Corresponds to `SettingPropertyRelations::typecheck` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 145-154).
    ///
    /// Returns ALWAYS_MATCH (1) for fixed BPs (those with a resolved property),
    /// and DECLINE_TO_MATCH (-1) for unfixed BPs (those with pending text).
    /// The full implementation would also check that the value is type-safe
    /// for the property's kind and that the subject can have properties.
    pub fn typecheck(
        _family: &BpFamily,
        bp: &BinaryPredicate,
        _kinds_of_terms: &[Option<usize>],
        _kinds_required: &[Option<usize>],
    ) -> i8 {
        if PropertySettingBpData::get_property(&bp.family_specific).is_some() {
            1 // ALWAYS_MATCH — BP has a resolved property
        } else {
            -1 // DECLINE_TO_MATCH — BP has pending text, not yet resolvable
        }
    }

    /// Assert the setting property relation: draw a property inference.
    ///
    /// Corresponds to `SettingPropertyRelations::assert` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 160-167).
    ///
    /// Extracts the property from the BP's family_specific data and draws
    /// a property inference on the subject using `PropertyInferences::draw`.
    #[allow(clippy::too_many_arguments)]
    pub fn assert(
        _family: &BpFamily,
        bp: &BinaryPredicate,
        subj0: usize,
        _spec0: Option<&'static str>,
        _subj1: usize,
        spec1: Option<&'static str>,
        subjects: &mut [crate::knowledge::inference_subjects::InferenceSubject],
        _permissions: &mut Vec<crate::knowledge::property_permissions::PropertyPermission>,
        inference_families: &[crate::knowledge::inferences::InferenceFamily],
        inferences: &mut Vec<crate::knowledge::inferences::Inference>,
        data_registry: &mut Vec<crate::knowledge::property_inferences::PropertyInferenceData>,
        property_registry: &[Property],
    ) -> bool {
        // Extract the property index from the BP's family_specific data.
        let prn_idx = match PropertySettingBpData::get_property(&bp.family_specific) {
            Some(idx) => idx,
            None => return false, // BP has no resolved property — cannot assert
        };

        // Look up the property name from the property registry.
        let prn = property_registry[prn_idx].name;

        // Draw a property inference on the subject.
        let result = PropertyInferences::draw(
            subj0,
            prn,
            spec1,
            inference_families,
            inferences,
            subjects,
            data_registry,
        );

        // Return true if the inference was successfully drawn or replaced an existing one.
        matches!(
            result,
            crate::knowledge::inferences::JoinResult::Joined
                | crate::knowledge::inferences::JoinResult::Replaced
        )
    }

    /// Compile run-time code for the setting property relation.
    ///
    /// Corresponds to `SettingPropertyRelations::schema` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 169-172).
    ///
    /// Simplified: returns false (decline to compile). The full implementation
    /// would use the BP's task_functions set by `set_property_BP_schemas`.
    pub fn schema(_family: &BpFamily, _task: u8, _bp: &BinaryPredicate) -> bool {
        false
    }

    /// Create a setting property BP with pending property text.
    ///
    /// Corresponds to `SettingPropertyRelations::make_set_property_BP` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 57-67).
    ///
    /// Creates a BP in the setting property family with the property name
    /// stored as pending text. The property may not exist yet — it will be
    /// resolved at stage 2 by `stock()`.
    ///
    /// # Arguments
    ///
    /// * `property_name` - The name of the property (pending text).
    /// * `bp_registry` - The BP registry to add to.
    ///
    /// # Returns
    ///
    /// The index of the right-way-round BP in the registry.
    #[allow(non_snake_case)]
    pub fn make_set_property_BP(
        property_name: &str,
        bp_registry: &mut Vec<BinaryPredicate>,
    ) -> usize {
        let family_idx = PROPERTY_SETTING_FAMILY;
        let left_term = BPTerms::new(None);
        let right_term = BPTerms::new(None);

        let bp_idx = BinaryPredicates::make_pair(
            family_idx,
            left_term,
            right_term,
            "set-property",
            "set-property-reversed",
            None, // no make-true schema (set at stage 2)
            None, // no test schema (set at stage 2)
            None, // no source name
            bp_registry,
        );

        // Store the pending property text in family_specific.
        let pending = PropertySettingBpData::encode_pending(property_name);
        bp_registry[bp_idx].family_specific = Some(pending.clone());

        // Also set the reversal's family_specific.
        if let Some(rev_idx) = bp_registry[bp_idx].reversal {
            if rev_idx < bp_registry.len() {
                bp_registry[rev_idx].family_specific = Some(pending);
            }
        }

        bp_idx
    }

    /// Find a setting property BP by its pending property text.
    ///
    /// Corresponds to `SettingPropertyRelations::find_set_property_BP` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 90-101).
    ///
    /// Searches all BPs in the setting property family for one whose pending
    /// text matches the given property name. Only returns right-way-round BPs.
    ///
    /// Returns `Some(bp_index)` if found, `None` otherwise.
    #[allow(non_snake_case)]
    pub fn find_set_property_BP(
        property_name: &str,
        bp_registry: &[BinaryPredicate],
    ) -> Option<usize> {
        for (bp_idx, bp) in bp_registry.iter().enumerate() {
            if bp.relation_family != PROPERTY_SETTING_FAMILY {
                continue;
            }
            if !bp.right_way_round {
                continue;
            }
            if let Some(pending) = PropertySettingBpData::get_pending(&bp.family_specific) {
                if pending == property_name {
                    return Some(bp_idx);
                }
            }
        }
        None
    }

    /// Fix a setting property BP: resolve its pending text to an actual property.
    ///
    /// Corresponds to `SettingPropertyRelations::fix_property_bp` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 107-125).
    ///
    /// Looks up the property by name in the property registry, updates the BP's
    /// family_specific data, and sets up schemas.
    #[allow(non_snake_case, clippy::ptr_arg)]
    pub fn fix_property_bp(
        bp_idx: usize,
        bp_registry: &mut Vec<BinaryPredicate>,
        property_registry: &[Property],
    ) {
        Self::fix_property_bp_internal(bp_idx, bp_registry, property_registry);
    }

    /// Set up schemas for a property's setting BP.
    ///
    /// Corresponds to `SettingPropertyRelations::set_property_BP_schemas` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 131-139).
    ///
    /// Sets the TEST_ATOM_TASK and NOW_ATOM_TRUE_TASK schemas, and the
    /// domain of the right term to the property's value kind.
    ///
    #[allow(non_snake_case)]
    pub fn set_property_BP_schemas(
        bp_idx: usize,
        prn_idx: usize,
        bp_registry: &mut [BinaryPredicate],
        property_registry: &[Property],
    ) {
        Self::set_property_BP_schemas_internal(bp_idx, prn_idx, bp_registry, property_registry);
    }

    /// Create a setting property BP for an existing (nameless) property.
    ///
    /// Corresponds to `SettingPropertyRelations::make_set_nameless_property_BP` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 69-71).
    ///
    /// Creates a BP in the setting property family with the property already
    /// resolved (no pending text). The property must already exist.
    ///
    /// # Arguments
    ///
    /// * `prn_idx` - The index of the property in the property registry.
    /// * `bp_registry` - The BP registry to add to.
    /// * `property_registry` - The property registry.
    ///
    /// # Returns
    ///
    /// The index of the right-way-round BP in the registry.
    #[allow(non_snake_case)]
    pub fn make_set_nameless_property_BP(
        prn_idx: usize,
        bp_registry: &mut Vec<BinaryPredicate>,
        property_registry: &[Property],
    ) -> usize {
        let family_idx = PROPERTY_SETTING_FAMILY;
        let left_term = BPTerms::new(None);
        let right_term = BPTerms::new(None);

        let bp_idx = BinaryPredicates::make_pair(
            family_idx,
            left_term,
            right_term,
            "set-property",
            "set-property-reversed",
            None,
            None,
            None,
            bp_registry,
        );

        // Store the resolved property index.
        let encoded = PropertySettingBpData::encode_property(prn_idx);
        bp_registry[bp_idx].family_specific = Some(encoded.clone());

        // Also set the reversal's family_specific.
        if let Some(rev_idx) = bp_registry[bp_idx].reversal {
            if rev_idx < bp_registry.len() {
                bp_registry[rev_idx].family_specific = Some(encoded);
            }
        }

        // Set up schemas.
        Self::set_property_BP_schemas_internal(bp_idx, prn_idx, bp_registry, property_registry);


        bp_idx
    }

    /// Check if a binary predicate belongs to the setting property family.
    ///
    /// Corresponds to `SettingPropertyRelations::bp_sets_a_property` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Setting Property Relation.w`, lines 174-176).
    ///
    /// Returns `true` if the BP belongs to the setting property family.
    pub fn bp_sets_a_property(bp: &BinaryPredicate) -> bool {
        bp.relation_family == PROPERTY_SETTING_FAMILY
    }

    /// Get the resolved property index from a setting property BP.
    ///
    /// Corresponds to accessing `PSD->set_property` in the C reference.
    ///
    /// Returns `Some(property_index)` if the BP has a resolved property,
    /// `None` if it still has pending text or doesn't belong to the family.
    pub fn bp_get_set_property(bp: &BinaryPredicate) -> Option<usize> {
        if bp.relation_family != PROPERTY_SETTING_FAMILY {
            return None;
        }
        PropertySettingBpData::get_property(&bp.family_specific)
    }

    /// Get the pending property text from a setting property BP.
    ///
    /// Returns `Some(property_name)` if the BP still has pending text,
    /// `None` if it has been resolved or doesn't belong to the family.
    pub fn bp_get_pending_text(bp: &BinaryPredicate) -> Option<&str> {
        if bp.relation_family != PROPERTY_SETTING_FAMILY {
            return None;
        }
        PropertySettingBpData::get_pending(&bp.family_specific)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculus::binary_predicate_families::BinaryPredicateFamilies;
    use crate::calculus::bp_term_details::BpTermDetails;
    use crate::knowledge::properties::ValuePropertyData;

    // -----------------------------------------------------------------------
    // start() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_creates_one_family() {
        let (families, bp_registry) = SettingPropertyRelations::start();
        assert_eq!(families.len(), 1);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_start_creates_family_with_correct_name() {
        let (families, _) = SettingPropertyRelations::start();
        assert_eq!(families[PROPERTY_SETTING_FAMILY].name, "property_setting");
    }

    #[test]
    fn test_setting_family_has_all_methods() {
        let (families, _) = SettingPropertyRelations::start();
        let sp = &families[PROPERTY_SETTING_FAMILY];
        assert!(sp.methods.stock.is_some());
        assert!(sp.methods.typecheck.is_some());
        assert!(sp.methods.assert.is_some());
        assert!(sp.methods.schema.is_some());
        // Should NOT have describe methods
        assert!(sp.methods.describe_for_problems.is_none());
        assert!(sp.methods.describe_for_index.is_none());
    }

    // -----------------------------------------------------------------------
    // PropertySettingBpData encoding tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_pending() {
        let encoded = PropertySettingBpData::encode_pending("weight");
        assert_eq!(encoded, "pending:weight");
    }

    #[test]
    fn test_encode_property() {
        let encoded = PropertySettingBpData::encode_property(42);
        assert_eq!(encoded, "property:42");
    }

    #[test]
    fn test_get_pending_returns_none_for_resolved() {
        let data = Some("property:0".to_string());
        assert_eq!(PropertySettingBpData::get_pending(&data), None);
    }

    #[test]
    fn test_get_pending_returns_none_for_none() {
        let data: Option<String> = None;
        assert_eq!(PropertySettingBpData::get_pending(&data), None);
    }

    #[test]
    fn test_get_property_returns_index() {
        let data = Some("property:42".to_string());
        assert_eq!(
            PropertySettingBpData::get_property(&data),
            Some(42)
        );
    }

    #[test]
    fn test_get_property_returns_none_for_pending() {
        let data = Some("pending:weight".to_string());
        assert_eq!(PropertySettingBpData::get_property(&data), None);
    }

    #[test]
    fn test_get_property_returns_none_for_none() {
        let data: Option<String> = None;
        assert_eq!(PropertySettingBpData::get_property(&data), None);
    }

    // -----------------------------------------------------------------------
    // make_set_property_BP() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_set_property_bp_creates_pair() {
        let (_families, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // make_pair creates 2 BPs (original + reversal)
        assert_eq!(bp_registry.len(), 2);

        // Right-way-round BP
        let bp = &bp_registry[bp_idx];
        assert_eq!(bp.relation_family, PROPERTY_SETTING_FAMILY);
        assert!(bp.right_way_round);
        assert_eq!(bp.debugging_log_name, Some("set-property".to_string()));

        // Reversal
        let rev_idx = bp.reversal.unwrap();
        let rev = &bp_registry[rev_idx];
        assert_eq!(rev.relation_family, PROPERTY_SETTING_FAMILY);
        assert!(!rev.right_way_round);
        assert_eq!(rev.debugging_log_name, Some("set-property-reversed".to_string()));
    }

    #[test]
    fn test_make_set_property_bp_stores_pending_text() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        assert_eq!(
            bp_registry[bp_idx].family_specific,
            Some("pending:weight".to_string())
        );

        // Reversal should also have the same family_specific
        let rev_idx = bp_registry[bp_idx].reversal.unwrap();
        assert_eq!(
            bp_registry[rev_idx].family_specific,
            Some("pending:weight".to_string())
        );
    }

    #[test]
    fn test_make_set_property_bp_creates_multiple_bps() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp0 = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        let bp1 = SettingPropertyRelations::make_set_property_BP("height", &mut bp_registry);

        // 2 properties × 2 BPs each = 4 BPs
        assert_eq!(bp_registry.len(), 4);

        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp0]),
            Some("weight")
        );
        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp1]),
            Some("height")
        );
    }

    // -----------------------------------------------------------------------
    // find_set_property_BP() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_set_property_bp_finds_by_pending_text() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        SettingPropertyRelations::make_set_property_BP("height", &mut bp_registry);

        let found = SettingPropertyRelations::find_set_property_BP("weight", &bp_registry);
        assert!(found.is_some());
        let bp = &bp_registry[found.unwrap()];
        assert!(bp.right_way_round);
        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(bp),
            Some("weight")
        );
    }

    #[test]
    fn test_find_set_property_bp_returns_none_for_missing() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        let found = SettingPropertyRelations::find_set_property_BP("nonexistent", &bp_registry);
        assert!(found.is_none());
    }

    #[test]
    fn test_find_set_property_bp_returns_none_for_empty_registry() {
        let bp_registry: Vec<BinaryPredicate> = Vec::new();
        let found = SettingPropertyRelations::find_set_property_BP("weight", &bp_registry);
        assert!(found.is_none());
    }

    #[test]
    fn test_find_set_property_bp_only_returns_right_way_round() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // The reversal should NOT be found by find_set_property_BP
        let rev_idx = bp_registry[bp_idx].reversal.unwrap();
        let found = SettingPropertyRelations::find_set_property_BP("weight", &bp_registry);
        assert_eq!(found, Some(bp_idx));
        assert_ne!(found, Some(rev_idx));
    }

    // -----------------------------------------------------------------------
    // fix_property_bp() tests
    // -----------------------------------------------------------------------

    /// Create a valued property for testing.
    fn make_valued_property(name: &'static str) -> Property {
        Property {
            name,
            has_of_in_the_name: false,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: None,
            value_data: Some(ValuePropertyData {
                property_value_kind: None,
                setting_bp: None,
                name_coincides_with_kind: false,
                as_condition_of_subject: None,
                relation_whose_state_this_stores: None,
            }),
            compilation_data: None,
            possession_marker: false,
        }
    }



    #[test]
    fn test_fix_property_bp_resolves_pending_text() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Fix the BP
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);

        // Should now have resolved property index
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );
        // Pending text should be gone
        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_idx]),
            None
        );
    }

    #[test]
    fn test_fix_property_bp_sets_schemas() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);

        // Should have TEST_ATOM_TASK (index 1) and NOW_ATOM_TRUE_TASK (index 2) schemas
        assert_eq!(
            bp_registry[bp_idx].task_functions[1],
            Some("*1.weight == *2".to_string())
        );
        assert_eq!(
            bp_registry[bp_idx].task_functions[2],
            Some("*1.weight = *2".to_string())
        );
    }

    #[test]
    fn test_fix_property_bp_skips_already_resolved() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Fix once
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );

        // Fix again — should be a no-op (no pending text to resolve)
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );
    }

    #[test]
    fn test_fix_property_bp_skips_nonexistent_property() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("nonexistent", &mut bp_registry);

        let properties: Vec<Property> = vec![];
        let prop_reg = &properties;

        // Fix — should leave pending since property doesn't exist
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);
        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_idx]),
            Some("nonexistent")
        );
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            None
        );
    }

    // -----------------------------------------------------------------------
    // stock() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_stock_skips_stage_1() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Stock at stage 1 should do nothing
        families[PROPERTY_SETTING_FAMILY]
            .methods
            .stock
            .unwrap()(&families[PROPERTY_SETTING_FAMILY], 1, &mut bp_registry, prop_reg);
        assert_eq!(bp_registry.len(), 0);
    }

    #[test]
    fn test_stock_resolves_pending_text() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Stock at stage 2
        families[PROPERTY_SETTING_FAMILY]
            .methods
            .stock
            .unwrap()(&families[PROPERTY_SETTING_FAMILY], 2, &mut bp_registry, prop_reg);

        // BP should now be resolved
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );
        assert_eq!(
            SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_idx]),
            None
        );
    }

    #[test]
    fn test_stock_resolves_multiple_bps() {
        let (mut families, mut bp_registry) = SettingPropertyRelations::start();
        let bp0 = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        let bp1 = SettingPropertyRelations::make_set_property_BP("height", &mut bp_registry);

        let properties = vec![
            make_valued_property("weight"),
            make_valued_property("height"),
        ];
        let prop_reg = &properties;

        // Stock at stage 2 via BinaryPredicateFamilies
        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, prop_reg);

        // Both BPs should be resolved
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp0]),
            Some(0)
        );
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp1]),
            Some(1)
        );
    }

    #[test]
    fn test_stock_skips_bps_without_pending_text() {
        let (mut families, mut bp_registry) = SettingPropertyRelations::start();
        // Create a BP with pending text
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // Manually set one BP to resolved state
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Stock at stage 2
        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, prop_reg);

        // The BP should be resolved
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );
    }

    #[test]
    fn test_stock_skips_bps_from_other_families() {
        let (mut families, mut bp_registry) = SettingPropertyRelations::start();
        // Create a BP in the setting property family
        SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // Create a BP that looks like it's from another family
        let other_bp = BinaryPredicate {
            relation_family: 999, // not PROPERTY_SETTING_FAMILY
            family_specific: Some("pending:weight".to_string()),
            relation_name: None,
            debugging_log_name: Some("other".to_string()),
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };
        bp_registry.push(other_bp);

        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        // Stock at stage 2
        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, prop_reg);

        // The other family's BP should still have pending text
        assert_eq!(bp_registry[2].family_specific, Some("pending:weight".to_string()));
    }

    // -----------------------------------------------------------------------
    // make_set_nameless_property_BP() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_set_nameless_property_bp_creates_pair() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        let bp_idx = SettingPropertyRelations::make_set_nameless_property_BP(
            0, &mut bp_registry, prop_reg,
        );

        assert_eq!(bp_registry.len(), 2);
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );
    }

    #[test]
    fn test_make_set_nameless_property_bp_sets_schemas() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        let bp_idx = SettingPropertyRelations::make_set_nameless_property_BP(
            0, &mut bp_registry, prop_reg,
        );

        assert_eq!(
            bp_registry[bp_idx].task_functions[1],
            Some("*1.weight == *2".to_string())
        );
        assert_eq!(
            bp_registry[bp_idx].task_functions[2],
            Some("*1.weight = *2".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // bp_sets_a_property() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bp_sets_a_property_returns_true_for_setting_family() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        assert!(SettingPropertyRelations::bp_sets_a_property(&bp_registry[bp_idx]));
    }

    #[test]
    fn test_bp_sets_a_property_returns_true_for_reversal() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        let rev_idx = bp_registry[bp_idx].reversal.unwrap();

        assert!(SettingPropertyRelations::bp_sets_a_property(&bp_registry[rev_idx]));
    }

    #[test]
    fn test_bp_sets_a_property_returns_false_for_other_family() {
        let dummy_bp = BinaryPredicate {
            relation_family: 999,
            family_specific: None,
            relation_name: None,
            debugging_log_name: None,
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        assert!(!SettingPropertyRelations::bp_sets_a_property(&dummy_bp));
    }

    // -----------------------------------------------------------------------
    // bp_get_set_property() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_bp_get_set_property_returns_none_for_pending() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            None
        );
    }

    #[test]
    fn test_bp_get_set_property_returns_none_for_wrong_family() {
        let dummy_bp = BinaryPredicate {
            relation_family: 999,
            family_specific: Some("property:0".to_string()),
            relation_name: None,
            debugging_log_name: None,
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: None,
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&dummy_bp),
            None
        );
    }

    // -----------------------------------------------------------------------
    // typecheck() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_typecheck_returns_always_match_for_fixed_bp() {
        let (families, _) = SettingPropertyRelations::start();
        let sp_family = &families[PROPERTY_SETTING_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROPERTY_SETTING_FAMILY,
            family_specific: Some("property:0".to_string()),
            relation_name: None,
            debugging_log_name: Some("set-property".to_string()),
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = SettingPropertyRelations::typecheck(
            sp_family,
            &dummy_bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
        );
        assert_eq!(result, 1); // ALWAYS_MATCH for fixed BP

        // Even with different kinds
        let result = SettingPropertyRelations::typecheck(
            sp_family,
            &dummy_bp,
            &[Some(99), Some(42)],
            &[None, None],
        );
        assert_eq!(result, 1);
    }

    #[test]
    fn test_typecheck_returns_decline_to_match_for_unfixed_bp() {
        let (families, _) = SettingPropertyRelations::start();
        let sp_family = &families[PROPERTY_SETTING_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROPERTY_SETTING_FAMILY,
            family_specific: None,
            relation_name: None,
            debugging_log_name: Some("set-property".to_string()),
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let result = SettingPropertyRelations::typecheck(
            sp_family,
            &dummy_bp,
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
        );
        assert_eq!(result, -1); // DECLINE_TO_MATCH for unfixed BP
    }

    #[test]
    fn test_typecheck_dispatch_via_family() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // Unfixed BP should return DECLINE_TO_MATCH.
        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[bp_idx],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, -1);

        // Fix the BP.
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);

        // Fixed BP should return ALWAYS_MATCH.
        let result = BinaryPredicateFamilies::typecheck(
            &bp_registry[bp_idx],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(result, 1);
    }

    // -----------------------------------------------------------------------
    // schema() tests
    // -----------------------------------------------------------------------

#[test]
    fn test_schema_returns_false() {
        let (families, _) = SettingPropertyRelations::start();
        let sp_family = &families[PROPERTY_SETTING_FAMILY];

        let dummy_bp = BinaryPredicate {
            relation_family: PROPERTY_SETTING_FAMILY,
            family_specific: None,
            relation_name: None,
            debugging_log_name: Some("set-property".to_string()),
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        assert!(!SettingPropertyRelations::schema(sp_family, 1, &dummy_bp));
        assert!(!SettingPropertyRelations::schema(sp_family, 2, &dummy_bp));
        assert!(!SettingPropertyRelations::schema(sp_family, 3, &dummy_bp));
    }

    #[test]
    fn test_schema_dispatch_via_family() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        assert!(!BinaryPredicateFamilies::get_schema(
            1,
            &bp_registry[bp_idx],
            &families,
        ));
    }

    // -----------------------------------------------------------------------
    // assert() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_returns_true() {
        let (families, _bp_registry) = SettingPropertyRelations::start();
        let sp_family = &families[PROPERTY_SETTING_FAMILY];

        // Set up inference families.
        let inf_families = vec![PropertyInferences::start()];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();

        // Set up property registry.
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;

        let dummy_bp = BinaryPredicate {
            relation_family: PROPERTY_SETTING_FAMILY,
            family_specific: Some("property:0".to_string()),
            relation_name: None,
            debugging_log_name: Some("set-property".to_string()),
            term_details: [
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
                BpTermDetails {
                    implies_infs: None,
                    implies_kind: None,
                    called_name: None,
                    function_of_other: None,
                    index_term_as: None,
                },
            ],
            reversal: Some(1),
            right_way_round: true,
            task_functions: [None, None, None, None],
            loop_parent_optimisation_proviso: None,
            loop_parent_optimisation_ranger: None,
            knowledge_about_bp: None,
        };

        let mut subjects = vec![crate::knowledge::inference_subjects::InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some("test_subject"),
        }];
        let mut permissions = Vec::new();

        let result = SettingPropertyRelations::assert(
            sp_family,
            &dummy_bp,
            0,
            None,
            0,
            Some("10 kg"),
            &mut subjects,
            &mut permissions,
            &inf_families,
            &mut inferences,
            &mut data_registry,
            prop_reg,
        );
        assert!(result);
        assert_eq!(inferences.len(), 1);
        assert_eq!(subjects[0].inf_list.len(), 1);
    }

    #[test]
    fn test_assert_dispatch_via_family() {
        let (families, mut bp_registry) = SettingPropertyRelations::start();
        let bp_idx = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);

        // Fix the BP so it has a resolved property.
        let properties = vec![make_valued_property("weight")];
        let prop_reg = &properties;
        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);

        // Set up inference families.
        let inf_families = vec![PropertyInferences::start()];
        let mut inferences = Vec::new();
        let mut data_registry = Vec::new();

        let mut subjects = vec![crate::knowledge::inference_subjects::InferenceSubject {
            broader_than: None,
            infs_family: 0,
            represents: Some("test_subject"),
            inf_list: Vec::new(),
            imp_list: Vec::new(),
            permissions_list: Vec::new(),
            alias_variable: None,
            log_name: Some("test_subject"),
        }];
        let mut permissions = Vec::new();

        let result = BinaryPredicateFamilies::assert(
            &bp_registry[bp_idx],
            0,
            None,
            0,
            Some("10 kg"),
            &families,
            &mut subjects,
            &mut permissions,
            &inf_families,
            &mut inferences,
            &mut data_registry,
            prop_reg,
        );
        assert!(result);
        assert_eq!(inferences.len(), 1);
        assert_eq!(subjects[0].inf_list.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Integration test: full lifecycle
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_lifecycle() {
        let (mut families, mut bp_registry) = SettingPropertyRelations::start();

        // Create BPs with pending text before properties exist.
        let bp_weight = SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        let bp_height = SettingPropertyRelations::make_set_property_BP("height", &mut bp_registry);

        // At this point, BPs have pending text but no resolved property.
        assert!(SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_weight]).is_some());
        assert!(SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_weight]).is_none());
        assert!(SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_height]).is_some());
        assert!(SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_height]).is_none());

        // Create properties.
        let properties = vec![
            make_valued_property("weight"),
            make_valued_property("height"),
        ];
        let prop_reg = &properties;

        // Stage 1: nothing happens.
        BinaryPredicateFamilies::first_stock(&mut families, &mut bp_registry);
        assert!(SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_weight]).is_some());

        // Stage 2: resolve pending text.
        BinaryPredicateFamilies::second_stock(&mut families, &mut bp_registry, prop_reg);

        // BPs should now be resolved.
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_weight]),
            Some(0)
        );
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_height]),
            Some(1)
        );
        assert!(SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_weight]).is_none());
        assert!(SettingPropertyRelations::bp_get_pending_text(&bp_registry[bp_height]).is_none());

        // Schemas should be set.
        assert_eq!(
            bp_registry[bp_weight].task_functions[1],
            Some("*1.weight == *2".to_string())
        );
        assert_eq!(
            bp_registry[bp_weight].task_functions[2],
            Some("*1.weight = *2".to_string())
        );
        assert_eq!(
            bp_registry[bp_height].task_functions[1],
            Some("*1.height == *2".to_string())
        );
        assert_eq!(
            bp_registry[bp_height].task_functions[2],
            Some("*1.height = *2".to_string())
        );

        // Typecheck returns ALWAYS_MATCH.
        let tc = BinaryPredicateFamilies::typecheck(
            &bp_registry[bp_weight],
            &[Some(0), Some(1)],
            &[Some(0), Some(1)],
            &families,
        );
        assert_eq!(tc, 1);

        // Schema returns false.
        assert!(!BinaryPredicateFamilies::get_schema(1, &bp_registry[bp_weight], &families));

        // bp_sets_a_property returns true for both original and reversal.
        assert!(SettingPropertyRelations::bp_sets_a_property(&bp_registry[bp_weight]));
        let rev_weight = bp_registry[bp_weight].reversal.unwrap();
        assert!(SettingPropertyRelations::bp_sets_a_property(&bp_registry[rev_weight]));

        // find_set_property_BP returns None for resolved BPs (pending text is gone).
        assert!(SettingPropertyRelations::find_set_property_BP("weight", &bp_registry).is_none());
    }

    #[test]
    fn test_find_and_fix_round_trip() {
        let (_, mut bp_registry) = SettingPropertyRelations::start();

        // Create BPs before properties exist.
        SettingPropertyRelations::make_set_property_BP("weight", &mut bp_registry);
        SettingPropertyRelations::make_set_property_BP("height", &mut bp_registry);

        // Find by pending text.
        let found = SettingPropertyRelations::find_set_property_BP("weight", &bp_registry);
        assert!(found.is_some());
        let bp_idx = found.unwrap();

        // Create properties and fix.
        let properties = vec![
            make_valued_property("weight"),
            make_valued_property("height"),
        ];
        let prop_reg = &properties;

        SettingPropertyRelations::fix_property_bp(bp_idx, &mut bp_registry, prop_reg);

        // After fix, the BP should have a resolved property.
        assert_eq!(
            SettingPropertyRelations::bp_get_set_property(&bp_registry[bp_idx]),
            Some(0)
        );

        // find_set_property_BP should no longer find it (pending text is gone).
        assert!(SettingPropertyRelations::find_set_property_BP("weight", &bp_registry).is_none());
    }
}
