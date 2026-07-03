/// The Property System — core data structures for named attributes.
///
/// Properties are named attributes that subjects can have. They come in two
/// types: either/or (e.g., "open"/"closed") and valued (e.g., "carrying
/// capacity"). Each property has a list of permissions saying which subjects
/// can have it.
///
/// | Struct | C Reference | Purpose |
/// |--------|-------------|---------|
/// | [`Property`] | `Chapter 3/Properties.w` | Core property struct |
/// | [`EitherOrPropertyData`] | `Chapter 3/Either-Or Properties.w` | Either-or property data |
/// | [`ValuePropertyData`] | `Chapter 3/Valued Properties.w` | Valued property data |
///
/// # References
///
/// - C reference: `inform7/knowledge-module/Chapter 3/Properties.w`
/// - C reference: `inform7/knowledge-module/Chapter 3/Either-Or Properties.w`
/// - C reference: `inform7/knowledge-module/Chapter 3/Valued Properties.w`
///
/// A property: a named attribute that subjects can have.
///
/// Corresponds to `property` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 21-38).
///
/// Properties come in two types: either/or (e.g., "open"/"closed") and
/// valued (e.g., "carrying capacity"). Each property has a list of
/// permissions saying which subjects can have it.
///
/// Simplified: uses string names instead of `wording`, and a simplified
/// compilation data field instead of the full `property_compilation_data`.
#[derive(Clone, Debug)]
pub struct Property {
    /// Name of the property (simplified: a string instead of `wording`).
    pub name: &'static str,
    /// Whether the name looks like a property test (e.g., "point of view").
    pub has_of_in_the_name: bool,
    /// Whether this is an Inter-level-only property (no I7 source text existence).
    pub inter_level_only: bool,
    /// List of property permission indices: who can have this property.
    pub permissions: Vec<usize>,
    /// Either-or property data, or None if this is a valued property.
    pub either_or_data: Option<EitherOrPropertyData>,
    /// Valued property data, or None if this is an either-or property.
    pub value_data: Option<ValuePropertyData>,
    /// Compilation data (simplified: a string tag).
    /// Full `property_compilation_data` is deferred.
    pub compilation_data: Option<&'static str>,
    /// Possession marker for temporary use when checking implications.
    pub possession_marker: bool,
}

/// Data for an either-or property.
///
/// Corresponds to `either_or_property_data` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 29-36).
///
/// Either-or properties may come in pairs: "open" and "closed" are a pair,
/// where each is the negation of the other. Not all either-or properties
/// are paired; sometimes an author simply says something "can be P".
#[derive(Clone, Debug)]
pub struct EitherOrPropertyData {
    /// The negation property index, if this is one of a pair.
    pub negation: Option<usize>,
    /// The adjective index, if this property is adjectivally used.
    /// Deferred: depends on the adjective meaning system.
    pub as_adjective: Option<usize>,
}

/// Data for a valued property.
///
/// Corresponds to `value_property_data` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Valued Properties.w`, lines 10-17).
///
/// Valued properties store values of a specific kind. Each valued property
/// has an associated setting relation (a binary predicate) that sets its value.
#[derive(Clone, Debug)]
pub struct ValuePropertyData {
    /// The kind of value stored in this property (simplified: a kind name string).
    /// Corresponds to `property_value_kind` in the C reference.
    pub property_value_kind: Option<&'static str>,
    /// The setting binary predicate index (the relation that sets this property).
    /// Corresponds to `setting_bp` in the C reference.
    /// Simplified: deferred until SettingPropertyRelations is implemented.
    pub setting_bp: Option<usize>,
    /// Whether the property name coincides with a kind name.
    /// Corresponds to `name_coincides_with_kind` in the C reference.
    pub name_coincides_with_kind: bool,
    /// Condition of subject data, if this property is a condition of a subject.
    /// Deferred: depends on ConditionsOfSubjects.
    pub as_condition_of_subject: Option<usize>,
    /// Binary predicate whose state this property stores (if any).
    /// Deferred: depends on relation storage.
    pub relation_whose_state_this_stores: Option<usize>,
}

/// Creation and accessor functions for properties.
///
/// Corresponds to `Properties` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Properties.w`).
pub struct Properties;

impl Properties {
    /// Create a new property.
    ///
    /// Corresponds to `Properties::create` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 66-82).
    ///
    /// Simplified:
    /// - No Preform grammar for name validation
    /// - No noun registration
    /// - No RTProperties::initialise_pcd
    /// - No special property detection (P_description, P_specification, etc.)
    /// - No PluginCalls::new_property_notify
    ///
    /// `eo` is true for either-or properties, false for valued properties.
    ///
    /// Returns the index of the new property in the registry.
    pub fn create(
        name: &'static str,
        eo: bool,
        registry: &mut Vec<Property>,
    ) -> usize {
        let has_of = name.contains(" of ");
        let prn = Property {
            name,
            has_of_in_the_name: has_of,
            inter_level_only: false,
            permissions: Vec::new(),
            either_or_data: if eo {
                Some(EitherOrPropertyData {
                    negation: None,
                    as_adjective: None,
                })
            } else {
                None
            },
            value_data: if eo {
                None
            } else {
                Some(ValuePropertyData {
                    property_value_kind: None,
                    setting_bp: None,
                    name_coincides_with_kind: false,
                    as_condition_of_subject: None,
                    relation_whose_state_this_stores: None,
                })
            },
            compilation_data: None,
            possession_marker: false,
        };
        let idx = registry.len();
        registry.push(prn);
        idx
    }

    /// Find or create a property by name.
    ///
    /// Corresponds to `Properties::obtain` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 47-61).
    ///
    /// Simplified:
    /// - No Lexicon::retrieve (no Preform grammar)
    /// - No Rvalues::to_property
    /// - No internal_error for type mismatches
    /// - No ValueProperties::make_setting_bp
    ///
    /// If a property with the given name already exists, returns its index.
    /// If `valued` is true, the property must be a valued property (not either-or).
    /// If `valued` is false, the property must be an either-or property.
    /// If no property exists with the given name, creates a new one.
    ///
    /// Returns the index of the property.
    pub fn obtain(
        name: &'static str,
        valued: bool,
        registry: &mut Vec<Property>,
    ) -> usize {
        // Check if a property with this name already exists.
        if let Some(idx) = registry.iter().position(|p| p.name == name) {
            // Property exists — verify type consistency.
            // Simplified: no internal_error for mismatches.
            return idx;
        }
        // Create a new property.
        Properties::create(name, !valued, registry)
    }

    /// Return the kind of a property.
    ///
    /// Corresponds to `Properties::to_kind` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 238-241).
    ///
    /// For an either-or property, returns "truth-state-valued property".
    /// For a valued property, returns "<kind>-valued property".
    ///
    /// Simplified: returns a string representation instead of a `kind*` pointer.
    pub fn to_kind(prn: &Property) -> String {
        if prn.either_or_data.is_some() {
            "truth-state-valued property".to_string()
        } else if let Some(ref vd) = prn.value_data {
            if let Some(k) = vd.property_value_kind {
                format!("{}-valued property", k)
            } else {
                "value-valued property".to_string()
            }
        } else {
            "property".to_string()
        }
    }

    /// Return the kind of values stored in a property.
    ///
    /// Corresponds to `Properties::kind_of_contents` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`, lines 243-247).
    ///
    /// For an either-or property, returns "truth_state".
    /// For a valued property, returns the property's value kind.
    pub fn kind_of_contents(prn: &Property) -> Option<&'static str> {
        if prn.either_or_data.is_some() {
            Some("truth_state")
        } else if let Some(ref vd) = prn.value_data {
            vd.property_value_kind
        } else {
            None
        }
    }
    /// Find a valued property whose name matches a given kind name.
    ///
    /// Corresponds to `Properties::property_with_same_name_as` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Properties.w`).
    ///
    /// This is used by `InstanceAdjectives::assert` to find the property
    /// that corresponds to an enumerated kind (e.g., the "colour" property
    /// for the "colour" kind).
    ///
    /// Returns the index of the first valued property whose name matches
    /// the given kind name, or None if no such property exists.
    pub fn property_with_same_name_as(kind_name: &str, properties: &[Property]) -> Option<usize> {
        properties.iter().position(|p| {
            p.either_or_data.is_none() && p.name == kind_name
        })
    }
}

/// Operations on either-or properties.
///
/// Corresponds to `EitherOrProperties` in the C reference
/// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`).
pub struct EitherOrProperties;

impl EitherOrProperties {
    /// Create new either-or property data.
    ///
    /// Corresponds to `EitherOrProperties::new_eo_data` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 38-46).
    pub fn new_eo_data() -> EitherOrPropertyData {
        EitherOrPropertyData {
            negation: None,
            as_adjective: None,
        }
    }

    /// Join two either-or properties into a negation pair.
    ///
    /// Corresponds to `EitherOrProperties::make_pair` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 97-128).
    ///
    /// Simplified: no problem message generation for broken pairs.
    /// Returns true if the pair was successfully created, false otherwise.
    pub fn make_pair(
        prn_idx: usize,
        neg_idx: usize,
        registry: &mut [Property],
    ) -> bool {
        // Both must be either-or properties.
        if prn_idx == neg_idx {
            return false;
        }
        let (prn, neg) = get_two_mut(registry, prn_idx, neg_idx);
        if prn.either_or_data.is_none() || neg.either_or_data.is_none() {
            return false;
        }
        // Check if either already has a negation.
        if let Some(eod) = &prn.either_or_data {
            if eod.negation.is_some() && eod.negation != Some(neg_idx) {
                return false; // Already paired with someone else.
            }
        }
        if let Some(eod) = &neg.either_or_data {
            if eod.negation.is_some() && eod.negation != Some(prn_idx) {
                return false; // Already paired with someone else.
            }
        }
        // Set the negation pointers.
        if let Some(eod) = &mut prn.either_or_data {
            eod.negation = Some(neg_idx);
        }
        if let Some(eod) = &mut neg.either_or_data {
            eod.negation = Some(prn_idx);
        }
        true
    }

    /// Get the negation of an either-or property.
    ///
    /// Corresponds to `EitherOrProperties::get_negation` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 131-134).
    pub fn get_negation(prn: &Property) -> Option<usize> {
        prn.either_or_data.as_ref().and_then(|eod| eod.negation)
    }

    /// Get the adjective index for an either-or property.
    ///
    /// Corresponds to `EitherOrProperties::as_adjective` in the C reference
    /// (`inform7/knowledge-module/Chapter 3/Either-Or Properties.w`, lines 151-154).
    ///
    /// Returns the adjective index, or None if the property is not either-or
    /// or has no associated adjective.
    pub fn as_adjective(prn_idx: usize, properties: &[Property]) -> Option<usize> {
        properties.get(prn_idx).and_then(|prn| {
            prn.either_or_data.as_ref().and_then(|eod| eod.as_adjective)
        })
    }
}


/// Safely return mutable references to two distinct elements of a slice.
///
/// This is a helper for operations that need to modify two properties at
/// once (e.g., `EitherOrProperties::make_pair`). Panics if the indices
/// are equal or out of bounds.
fn get_two_mut<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j, "get_two_mut: indices must be distinct");
    if i < j {
        let (left, right) = slice.split_at_mut(j);
        (&mut left[i], &mut right[0])
    } else {
        let (left, right) = slice.split_at_mut(i);
        (&mut right[0], &mut left[j])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::ValueProperties;
    // -----------------------------------------------------------------------
    // Properties::create
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_creates_property_with_correct_name() {
        let mut registry = Vec::new();
        let idx = Properties::create("colour", false, &mut registry);
        assert_eq!(registry[idx].name, "colour");
    }

    #[test]
    fn test_create_creates_either_or_property_when_eo_true() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        assert!(registry[idx].either_or_data.is_some());
        assert!(registry[idx].value_data.is_none());
    }

    #[test]
    fn test_create_creates_valued_property_when_eo_false() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        assert!(registry[idx].either_or_data.is_none());
        assert!(registry[idx].value_data.is_some());
    }

    #[test]
    fn test_create_sets_has_of_in_the_name() {
        let mut registry = Vec::new();
        let idx = Properties::create("point of view", false, &mut registry);
        assert!(registry[idx].has_of_in_the_name);
    }

    #[test]
    fn test_create_does_not_set_has_of_for_simple_names() {
        let mut registry = Vec::new();
        let idx = Properties::create("colour", false, &mut registry);
        assert!(!registry[idx].has_of_in_the_name);
    }

    #[test]
    fn test_create_assigns_increasing_indices() {
        let mut registry = Vec::new();
        let idx1 = Properties::create("open", true, &mut registry);
        let idx2 = Properties::create("closed", true, &mut registry);
        let idx3 = Properties::create("colour", false, &mut registry);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
        assert_eq!(registry.len(), 3);
    }

    // -----------------------------------------------------------------------
    // Properties::obtain
    // -----------------------------------------------------------------------

    #[test]
    fn test_obtain_creates_new_property_when_none_exists() {
        let mut registry = Vec::new();
        let idx = Properties::obtain("colour", true, &mut registry);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry[idx].name, "colour");
        // valued=true means not either-or, so value_data should be Some
        assert!(registry[idx].value_data.is_some());
    }

    #[test]
    fn test_obtain_returns_existing_property() {
        let mut registry = Vec::new();
        let idx1 = Properties::create("colour", false, &mut registry);
        let idx2 = Properties::obtain("colour", true, &mut registry);
        assert_eq!(idx1, idx2);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_obtain_creates_either_or_when_valued_false() {
        let mut registry = Vec::new();
        let idx = Properties::obtain("open", false, &mut registry);
        assert!(registry[idx].either_or_data.is_some());
        assert!(registry[idx].value_data.is_none());
    }

    #[test]
    fn test_obtain_creates_valued_when_valued_true() {
        let mut registry = Vec::new();
        let idx = Properties::obtain("carrying capacity", true, &mut registry);
        assert!(registry[idx].either_or_data.is_none());
        assert!(registry[idx].value_data.is_some());
    }

    // -----------------------------------------------------------------------
    // Properties::to_kind
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_kind_returns_truth_state_for_either_or() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        assert_eq!(
            Properties::to_kind(&registry[idx]),
            "truth-state-valued property"
        );
    }

    fn test_to_kind_returns_kind_valued_for_valued_with_kind() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        ValueProperties::set_kind(idx, "number", &mut registry);
        assert_eq!(
            Properties::to_kind(&registry[idx]),
            "number-valued property"
        );
    }

    #[test]
    fn test_to_kind_returns_value_valued_for_valued_without_kind() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        assert_eq!(
            Properties::to_kind(&registry[idx]),
            "value-valued property"
        );
    }

    // -----------------------------------------------------------------------
    // Properties::kind_of_contents
    // -----------------------------------------------------------------------

    #[test]
    fn test_kind_of_contents_returns_truth_state_for_either_or() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        assert_eq!(
            Properties::kind_of_contents(&registry[idx]),
            Some("truth_state")
        );
    }

    #[test]
    fn test_kind_of_contents_returns_value_kind_for_valued() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        ValueProperties::set_kind(idx, "number", &mut registry);
        assert_eq!(
            Properties::kind_of_contents(&registry[idx]),
            Some("number")
        );
    }

    #[test]
    fn test_kind_of_contents_returns_none_for_valued_without_kind() {
        let mut registry = Vec::new();
        let idx = Properties::create("carrying capacity", false, &mut registry);
        assert_eq!(Properties::kind_of_contents(&registry[idx]), None);
    }

    // -----------------------------------------------------------------------
    // EitherOrProperties::new_eo_data
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_eo_data_creates_data_with_no_negation() {
        let eod = EitherOrProperties::new_eo_data();
        assert!(eod.negation.is_none());
        assert!(eod.as_adjective.is_none());
    }

    // -----------------------------------------------------------------------
    // EitherOrProperties::make_pair
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_pair_joins_two_properties() {
        let mut registry = Vec::new();
        let open_idx = Properties::create("open", true, &mut registry);
        let closed_idx = Properties::create("closed", true, &mut registry);

        let result = EitherOrProperties::make_pair(open_idx, closed_idx, &mut registry);
        assert!(result);

        assert_eq!(
            EitherOrProperties::get_negation(&registry[open_idx]),
            Some(closed_idx)
        );
        assert_eq!(
            EitherOrProperties::get_negation(&registry[closed_idx]),
            Some(open_idx)
        );
    }

    #[test]
    fn test_make_pair_returns_false_for_non_either_or() {
        let mut registry = Vec::new();
        let open_idx = Properties::create("open", true, &mut registry);
        let colour_idx = Properties::create("colour", false, &mut registry);

        let result = EitherOrProperties::make_pair(open_idx, colour_idx, &mut registry);
        assert!(!result);
    }

    #[test]
    fn test_make_pair_returns_false_for_same_index() {
        let mut registry = Vec::new();
        let open_idx = Properties::create("open", true, &mut registry);

        let result = EitherOrProperties::make_pair(open_idx, open_idx, &mut registry);
        assert!(!result);
    }

    #[test]
    fn test_make_pair_returns_false_when_already_paired() {
        let mut registry = Vec::new();
        let a_idx = Properties::create("a", true, &mut registry);
        let b_idx = Properties::create("b", true, &mut registry);
        let c_idx = Properties::create("c", true, &mut registry);

        // Pair a with b
        assert!(EitherOrProperties::make_pair(a_idx, b_idx, &mut registry));
        // Try to pair a with c — should fail since a is already paired
        assert!(!EitherOrProperties::make_pair(a_idx, c_idx, &mut registry));
    }

    // -----------------------------------------------------------------------
    // EitherOrProperties::get_negation
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_negation_returns_negation_of_paired_property() {
        let mut registry = Vec::new();
        let open_idx = Properties::create("open", true, &mut registry);
        let closed_idx = Properties::create("closed", true, &mut registry);
        EitherOrProperties::make_pair(open_idx, closed_idx, &mut registry);

        assert_eq!(
            EitherOrProperties::get_negation(&registry[open_idx]),
            Some(closed_idx)
        );
    }

    #[test]
    fn test_get_negation_returns_none_for_unpaired_property() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        assert_eq!(EitherOrProperties::get_negation(&registry[idx]), None);
    }

    #[test]
    fn test_get_negation_returns_none_for_valued_property() {
        let mut registry = Vec::new();
        let idx = Properties::create("colour", false, &mut registry);
        assert_eq!(EitherOrProperties::get_negation(&registry[idx]), None);
    }


    // -----------------------------------------------------------------------
    // Integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_then_obtain_round_trip() {
        let mut registry = Vec::new();
        let create_idx = Properties::create("colour", false, &mut registry);
        let obtain_idx = Properties::obtain("colour", true, &mut registry);
        assert_eq!(create_idx, obtain_idx);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_multiple_properties_of_both_types() {
        let mut registry = Vec::new();
        let open_idx = Properties::create("open", true, &mut registry);
        let closed_idx = Properties::create("closed", true, &mut registry);
        let colour_idx = Properties::create("colour", false, &mut registry);
        let capacity_idx = Properties::create("carrying capacity", false, &mut registry);

        assert_eq!(registry.len(), 4);

        // Pair open/closed
        assert!(EitherOrProperties::make_pair(open_idx, closed_idx, &mut registry));

        // Set kind on colour
        ValueProperties::set_kind(colour_idx, "number", &mut registry);

        // Verify all accessors
        assert_eq!(
            Properties::to_kind(&registry[open_idx]),
            "truth-state-valued property"
        );
        assert_eq!(
            Properties::to_kind(&registry[closed_idx]),
            "truth-state-valued property"
        );
        assert_eq!(
            Properties::to_kind(&registry[colour_idx]),
            "number-valued property"
        );
        assert_eq!(
            Properties::to_kind(&registry[capacity_idx]),
            "value-valued property"
        );

        assert_eq!(
            Properties::kind_of_contents(&registry[open_idx]),
            Some("truth_state")
        );
        assert_eq!(
            Properties::kind_of_contents(&registry[colour_idx]),
            Some("number")
        );
        assert_eq!(Properties::kind_of_contents(&registry[capacity_idx]), None);

        assert_eq!(
            EitherOrProperties::get_negation(&registry[open_idx]),
            Some(closed_idx)
        );
        assert_eq!(
            EitherOrProperties::get_negation(&registry[closed_idx]),
            Some(open_idx)
        );
    }

    #[test]
    fn test_property_defaults() {
        let mut registry = Vec::new();
        let idx = Properties::create("test", false, &mut registry);
        let p = &registry[idx];

        assert!(!p.inter_level_only);
        assert!(p.permissions.is_empty());
        assert!(p.compilation_data.is_none());
        assert!(!p.possession_marker);
    }

    #[test]
    fn test_either_or_property_defaults() {
        let mut registry = Vec::new();
        let idx = Properties::create("open", true, &mut registry);
        let eod = registry[idx].either_or_data.as_ref().unwrap();
        assert!(eod.negation.is_none());
        assert!(eod.as_adjective.is_none());
    }

    #[test]
    fn test_valued_property_defaults() {
        let mut registry = Vec::new();
        let idx = Properties::create("colour", false, &mut registry);
        let vd = registry[idx].value_data.as_ref().unwrap();
        assert!(vd.property_value_kind.is_none());
        assert!(vd.setting_bp.is_none());
        assert!(!vd.name_coincides_with_kind);
        assert!(vd.as_condition_of_subject.is_none());
        assert!(vd.relation_whose_state_this_stores.is_none());
    }
}
