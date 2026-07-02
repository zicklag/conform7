//! Kinds::Behaviour API — high-level kind queries and operations.
//!
//! This module provides the public API for querying and manipulating kinds.
//! Every part of the compiler that works with kinds — instances, properties,
//! assertions, the calculus module — uses this API to ask questions about kinds.
//!
//! Corresponds to `Kinds::Behaviour` in the C reference
//! (`services/kinds-module/Chapter 2/Using Kinds.w`).

use crate::kind_constructors::{ConstructorGroup, DimensionalRules, UnitSequence};
use crate::kinds::Kind;

// ---------------------------------------------------------------------------
// Names of kinds
// Corresponds to Using Kinds.w, lines 9-33
// ---------------------------------------------------------------------------

/// Get the name of a kind.
///
/// Returns the constructor's name (e.g., "number", "text", "object").
///
/// Corresponds to `Kinds::Behaviour::get_name` in Using Kinds.w lines 9-12.
pub fn get_name(k: &Kind) -> &'static str {
    k.construct.name
}

/// Get the noun for a kind.
///
/// Returns the constructor's name (same as get_name in this simplified version).
///
/// Corresponds to `Kinds::Behaviour::get_noun` in Using Kinds.w lines 14-17.
pub fn get_noun(k: &Kind) -> &'static str {
    k.construct.name
}

/// Get the range number for a kind.
///
/// The range number is used for indexing purposes.
///
/// Corresponds to `Kinds::Behaviour::get_range_number` in Using Kinds.w lines 25-28.
pub fn get_range_number(k: &Kind) -> i32 {
    k.construct.class_number
}

/// Set the range number for a kind.
///
/// Corresponds to `Kinds::Behaviour::set_range_number` in Using Kinds.w lines 30-33.
pub fn set_range_number(k: &mut Kind, r: i32) {
    // SAFETY: We need to mutate the static constructor. In the C reference,
    // the constructor is stored in a mutable location. Here we use pointer
    // mutation since KindConstructors are stored in LazyLock statics.
    // This is safe because we only mutate fields that are designed to be
    // mutable at runtime (class_number is a per-kind-instance field in C).
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).class_number = r;
    }
}

// ---------------------------------------------------------------------------
// Being an object
// Corresponds to Using Kinds.w, lines 38-57
// ---------------------------------------------------------------------------

/// Returns true if K is an object kind (conforms to K_object, not nil/void).
///
/// A kind is an object if its constructor has the object_kind flag set,
/// or if it conforms to K_object (the object kind itself).
///
/// Corresponds to `Kinds::Behaviour::is_object` in Using Kinds.w lines 38-43.
pub fn is_object(k: &Kind) -> bool {
    use crate::familiar_kinds::K_object;
    use crate::familiar_kinds::K_nil;
    use crate::familiar_kinds::K_void;

    // Nil and void are not objects
    if k == &*K_nil || k == &*K_void {
        return false;
    }

    // The object kind itself is an object
    if k == &*K_object {
        return true;
    }

    // Check if the constructor has the object_kind flag
    if k.construct.object_kind {
        return true;
    }

    // Check if the kind conforms to K_object
    k.conforms_to(&K_object)
}

/// Returns true if K is a proper subkind of object (not object itself, not nil/void).
///
/// Corresponds to `Kinds::Behaviour::is_subkind_of_object` in Using Kinds.w lines 45-50.
pub fn is_subkind_of_object(k: &Kind) -> bool {
    use crate::familiar_kinds::K_object;
    use crate::familiar_kinds::K_nil;
    use crate::familiar_kinds::K_void;

    // Nil, void, and object itself are not subkinds of object
    if k == &*K_nil || k == &*K_void || k == &*K_object {
        return false;
    }

    // Check if the kind conforms to K_object
    k.conforms_to(&K_object)
}

/// Returns true if K is an object and conforms to L.
///
/// Corresponds to `Kinds::Behaviour::is_object_of_kind` in Using Kinds.w lines 52-57.
pub fn is_object_of_kind(k: &Kind, l: &Kind) -> bool {
    is_object(k) && k.conforms_to(l)
}

// ---------------------------------------------------------------------------
// Definiteness
// Corresponds to Using Kinds.w, lines 64-114
// ---------------------------------------------------------------------------

/// Returns true if K is a kind of kind (protocol kind).
///
/// A kind is a "kind of kind" if its constructor is a protocol.
///
/// Corresponds to `Kinds::Behaviour::is_kind_of_kind` in Using Kinds.w lines 64-68.
pub fn is_kind_of_kind(k: &Kind) -> bool {
    k.construct.group == ConstructorGroup::Protocol
}

/// Returns true if K is definite (not a protocol, not a variable,
/// and all children are definite).
///
/// A definite kind is one that is fully specified — it is not a protocol
/// (indefinite kind), not a kind variable, and all of its child kinds
/// are also definite.
///
/// Corresponds to `Kinds::Behaviour::definite` in Using Kinds.w lines 75-83.
pub fn definite(k: &Kind) -> bool {
    use crate::familiar_kinds::CON_KIND_VARIABLE;

    // Protocol kinds are not definite
    if is_kind_of_kind(k) {
        return false;
    }

    // Kind variables are not definite
    if std::ptr::eq(k.construct, &*CON_KIND_VARIABLE) {
        return false;
    }

    // All children must be definite
    for child in k.kc_args.iter().flatten() {
        if !definite(child) {
            return false;
        }
    }

    true
}

/// Returns true if K is semidefinite (definite or involves a kind variable).
///
/// A semidefinite kind is one that is either definite or involves a kind
/// variable (making it potentially definite after substitution).
///
/// Corresponds to `Kinds::Behaviour::semidefinite` in Using Kinds.w lines 85-103.
pub fn semidefinite(k: &Kind) -> bool {
    // Definite kinds are semidefinite
    if definite(k) {
        return true;
    }

    // Kinds that involve a kind variable are semidefinite
    involves_var(k, 0) // Check if any variable is involved
}

/// Returns true if K involves kind variable v.
///
/// If v is 0, returns true if K involves any kind variable.
///
/// Corresponds to `Kinds::Behaviour::involves_var` in Using Kinds.w lines 105-114.
pub fn involves_var(k: &Kind, v: u8) -> bool {
    use crate::familiar_kinds::CON_KIND_VARIABLE;

    // Check if this kind itself is a kind variable
    if std::ptr::eq(k.construct, &*CON_KIND_VARIABLE) && (v == 0 || k.kind_variable_number == v) {
        return true;
    }

    // Recursively check children
    for child in k.kc_args.iter().flatten() {
        if involves_var(child, v) {
            return true;
        }
    }

    false
}

// ---------------------------------------------------------------------------
// How this came into being
// Corresponds to Using Kinds.w, lines 125-190
// ---------------------------------------------------------------------------

/// Returns true if K is built-in (not defined in source text).
///
/// A built-in kind has no creating sentence (where_defined_in_source_text is None).
///
/// Corresponds to `Kinds::Behaviour::is_built_in` in Using Kinds.w lines 125-129.
pub fn is_built_in(k: &Kind) -> bool {
    k.construct.where_defined_in_source_text.is_none()
}

/// Returns the creating sentence for K (None for built-in kinds).
///
/// Corresponds to `Kinds::Behaviour::get_creating_sentence` in Using Kinds.w lines 131-134.
pub fn get_creating_sentence(k: &Kind) -> Option<usize> {
    k.construct.where_defined_in_source_text
}

/// Returns true if K is uncertainly defined (incompletely defined).
///
/// Corresponds to `Kinds::Behaviour::is_uncertainly_defined` in Using Kinds.w lines 144-147.
pub fn is_uncertainly_defined(k: &Kind) -> bool {
    k.construct.is_incompletely_defined
}

/// Returns true if K is an enumeration kind.
///
/// Corresponds to `Kinds::Behaviour::is_an_enumeration` in Using Kinds.w lines 152-155.
pub fn is_an_enumeration(k: &Kind) -> bool {
    k.construct.enumeration
}

/// Convert K to a unit kind. Returns true if successful or already a unit.
///
/// A unit kind is a kind whose values are measured in units (like "length" or "mass").
/// This is a simplified version that always succeeds for non-enumeration kinds.
///
/// Corresponds to `Kinds::Behaviour::convert_to_unit` in Using Kinds.w lines 162-165.
pub fn convert_to_unit(k: &mut Kind) -> bool {
    // If already an enumeration, cannot convert to unit
    if k.construct.enumeration {
        return false;
    }
    // Mark as arithmetic (unit kinds are arithmetic)
    // SAFETY: Same pattern as set_range_number — mutating the static constructor.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).arithmetic = true;
    }
    true
}

/// Convert K to an enumeration kind.
///
/// Corresponds to `Kinds::Behaviour::convert_to_enumeration` in Using Kinds.w lines 170-172.
pub fn convert_to_enumeration(k: &mut Kind) {
    // SAFETY: Mutating the static constructor's enumeration flag.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).enumeration = true;
    }
}

/// Convert K to use real arithmetic.
///
/// Corresponds to `Kinds::Behaviour::convert_to_real` in Using Kinds.w lines 177-179.
pub fn convert_to_real(k: &mut Kind) {
    // SAFETY: Mutating the static constructor's arithmetic flag.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).arithmetic = true;
    }
}

/// Create a new enumerated value for K. Returns the next free value index.
///
/// Increments and returns the next_free_value counter on the constructor.
///
/// Corresponds to `Kinds::Behaviour::new_enumerated_value` in Using Kinds.w lines 186-190.
pub fn new_enumerated_value(k: &mut Kind) -> i32 {
    // SAFETY: Mutating the static constructor's next_free_value field.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        let val = (*con_ptr).next_free_value;
        (*con_ptr).next_free_value += 1;
        val
    }
}

// ---------------------------------------------------------------------------
// Compatibility with other kinds
// Corresponds to Using Kinds.w, lines 198-206
// ---------------------------------------------------------------------------

/// Set where the superkind was set.
///
/// Corresponds to `Kinds::Behaviour::set_superkind_set_at` in Using Kinds.w lines 198-201.
pub fn set_superkind_set_at(k: &mut Kind, s: Option<usize>) {
    // SAFETY: Mutating the static constructor's superkind_set_at field.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).superkind_set_at = s;
    }
}

/// Get where the superkind was set.
///
/// Corresponds to `Kinds::Behaviour::get_superkind_set_at` in Using Kinds.w lines 203-206.
pub fn get_superkind_set_at(k: &Kind) -> Option<usize> {
    k.construct.superkind_set_at
}

// ---------------------------------------------------------------------------
// How constant values are expressed
// Corresponds to Using Kinds.w, lines 214-230
// ---------------------------------------------------------------------------

/// Returns true if K has named constant values.
///
/// A kind has named constant values if it is an enumeration or
/// uncertainly defined.
///
/// Corresponds to `Kinds::Behaviour::has_named_constant_values` in Using Kinds.w lines 214-219.
pub fn has_named_constant_values(k: &Kind) -> bool {
    k.construct.enumeration || k.construct.is_incompletely_defined
}

/// Get the constant compilation method for K.
///
/// Corresponds to `Kinds::Behaviour::get_constant_compilation_method` in Using Kinds.w lines 227-230.
pub fn get_constant_compilation_method(k: &Kind) -> i32 {
    k.construct.constant_compilation_method
}

// ---------------------------------------------------------------------------
// Performing arithmetic
// Corresponds to Using Kinds.w, lines 238-295
// ---------------------------------------------------------------------------

/// Returns true if K uses signed comparisons.
///
/// Corresponds to `Kinds::Behaviour::uses_signed_comparisons` in Using Kinds.w lines 238-241.
pub fn uses_signed_comparisons(k: &Kind) -> bool {
    k.construct.uses_signed_comparisons
}

/// Get the comparison routine identifier for K.
///
/// Corresponds to `Kinds::Behaviour::get_comparison_routine` in Using Kinds.w lines 243-246.
pub fn get_comparison_routine(k: &Kind) -> Option<&'static str> {
    k.construct.comparison_fn_identifier
}

/// Returns true if K is quasinumerical (supports arithmetic).
///
/// A kind is quasinumerical if its constructor has the arithmetic flag set.
///
/// Corresponds to `Kinds::Behaviour::is_quasinumerical` in Using Kinds.w lines 255-258.
pub fn is_quasinumerical(k: &Kind) -> bool {
    k.construct.arithmetic
}

/// Get the dimensional form of K.
///
/// Returns the dimensional form (unit sequence) for quasinumerical kinds.
///
/// Corresponds to `Kinds::Behaviour::get_dimensional_form` in Using Kinds.w lines 260-265.
pub fn get_dimensional_form(k: &Kind) -> Option<&UnitSequence> {
    k.construct.dimensional_form.as_deref()
}

/// Test if K is a derived kind.
///
/// A derived kind has a fixed dimensional form.
///
/// Corresponds to `Kinds::Behaviour::test_if_derived` in Using Kinds.w lines 267-270.
pub fn test_if_derived(k: &Kind) -> bool {
    k.construct.dimensional_form_fixed
}

/// Mark K as derived (fix the dimensional form).
///
/// Corresponds to `Kinds::Behaviour::now_derived` in Using Kinds.w lines 272-275.
pub fn now_derived(k: &mut Kind) {
    // SAFETY: Mutating the static constructor's dimensional_form_fixed field.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).dimensional_form_fixed = true;
    }
}

/// Get the scale factor for K.
///
/// The scale factor is the class number (simplified — in the C reference
/// this is a separate field on the kind structure).
///
/// Corresponds to `Kinds::Behaviour::scale_factor` in Using Kinds.w lines 277-286.
pub fn scale_factor(k: &Kind) -> i32 {
    k.construct.class_number
}

/// Get the dimensional rules for K.
///
/// Corresponds to `Kinds::Behaviour::get_dim_rules` in Using Kinds.w lines 292-295.
pub fn get_dim_rules(k: &Kind) -> &DimensionalRules {
    &k.construct.dim_rules
}

// ---------------------------------------------------------------------------
// Identifier
// Corresponds to Using Kinds.w, lines 300-303
// ---------------------------------------------------------------------------

/// Get the explicit identifier for K.
///
/// Returns the explicit identifier (e.g., "K_number", "K_text") if one has
/// been set.
///
/// Corresponds to `Kinds::Behaviour::get_identifier` in Using Kinds.w lines 300-303.
pub fn get_identifier(k: &Kind) -> Option<&'static str> {
    k.construct.explicit_identifier
}

// ---------------------------------------------------------------------------
// Storing values at run-time
// Corresponds to Using Kinds.w, lines 313-352
// ---------------------------------------------------------------------------

/// Returns true if K uses block (pointer) values.
///
/// Block values are stored on the heap and accessed via pointers.
///
/// Corresponds to `Kinds::Behaviour::uses_block_values` in Using Kinds.w lines 313-316.
pub fn uses_block_values(k: &Kind) -> bool {
    k.construct.uses_block_values
}

/// Get the small block size for K.
///
/// The small block size is the number of bytes allocated for small
/// block values.
///
/// Corresponds to `Kinds::Behaviour::get_small_block_size` in Using Kinds.w lines 321-324.
pub fn get_small_block_size(k: &Kind) -> i32 {
    k.construct.small_block_size
}

/// Get the heap size estimate for K.
///
/// The heap size estimate is the estimated number of bytes for heap
/// allocation of this kind's values.
///
/// Corresponds to `Kinds::Behaviour::get_heap_size_estimate` in Using Kinds.w lines 330-333.
pub fn get_heap_size_estimate(k: &Kind) -> i32 {
    k.construct.heap_size_estimate
}

/// Get the distinguishing routine for K.
///
/// Returns the name of the routine used to distinguish values of this kind,
/// or None if the default (~=) comparison is sufficient.
///
/// Corresponds to `Kinds::Behaviour::get_distinguisher` in Using Kinds.w lines 340-343.
pub fn get_distinguisher(k: &Kind) -> Option<&'static str> {
    k.construct.distinguishing_routine
}

/// Returns true if values of K can be serialized.
///
/// Corresponds to `Kinds::Behaviour::can_exchange` in Using Kinds.w lines 349-352.
pub fn can_exchange(k: &Kind) -> bool {
    k.construct.can_exchange
}

// ---------------------------------------------------------------------------
// Indexing and documentation
// Corresponds to Using Kinds.w, lines 357-408
// ---------------------------------------------------------------------------

/// Get the documentation reference for K.
///
/// Corresponds to `Kinds::Behaviour::get_documentation_reference` in Using Kinds.w lines 357-360.
pub fn get_documentation_reference(k: &Kind) -> Option<&'static str> {
    k.construct.documentation_reference
}

/// Set the documentation reference for K.
///
/// Corresponds to `Kinds::Behaviour::set_documentation_reference` in Using Kinds.w lines 362-365.
pub fn set_documentation_reference(k: &mut Kind, dr: &'static str) {
    // SAFETY: Mutating the static constructor's documentation_reference field.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).documentation_reference = Some(dr);
    }
}

/// Get the index default value for K.
///
/// Corresponds to `Kinds::Behaviour::get_index_default_value` in Using Kinds.w lines 371-374.
pub fn get_index_default_value(k: &Kind) -> Option<&'static str> {
    k.construct.index_default_value
}

/// Get the index minimum value for K.
///
/// Corresponds to `Kinds::Behaviour::get_index_minimum_value` in Using Kinds.w lines 376-379.
pub fn get_index_minimum_value(k: &Kind) -> Option<&'static str> {
    k.construct.index_minimum_value
}

/// Get the index maximum value for K.
///
/// Corresponds to `Kinds::Behaviour::get_index_maximum_value` in Using Kinds.w lines 381-384.
pub fn get_index_maximum_value(k: &Kind) -> Option<&'static str> {
    k.construct.index_maximum_value
}

/// Get the index priority for K.
///
/// Corresponds to `Kinds::Behaviour::get_index_priority` in Using Kinds.w lines 386-389.
pub fn get_index_priority(k: &Kind) -> i32 {
    k.construct.index_priority
}

/// Returns true if K should be greyed out in the index when empty.
///
/// Corresponds to `Kinds::Behaviour::indexed_grey_if_empty` in Using Kinds.w lines 391-394.
pub fn indexed_grey_if_empty(k: &Kind) -> bool {
    k.construct.indexed_grey_if_empty
}

/// Set the specification text for K.
///
/// Corresponds to `Kinds::Behaviour::set_specification_text` in Using Kinds.w lines 400-403.
pub fn set_specification_text(k: &mut Kind, desc: &'static str) {
    // SAFETY: Mutating the static constructor's specification_text field.
    let con_ptr = k.construct as *const _ as *mut crate::kind_constructors::KindConstructor;
    unsafe {
        (*con_ptr).specification_text = Some(desc);
    }
}

/// Get the specification text for K.
///
/// Corresponds to `Kinds::Behaviour::get_specification_text` in Using Kinds.w lines 405-408.
pub fn get_specification_text(k: &Kind) -> Option<&'static str> {
    k.construct.specification_text
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar_kinds::*;
    use crate::kind_constructors::*;
    use crate::kinds::Kind;
    use std::sync::LazyLock;

    // -----------------------------------------------------------------------
    // Name tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_name_returns_constructor_name() {
        assert_eq!(get_name(&K_number), "number");
        assert_eq!(get_name(&K_text), "text");
        assert_eq!(get_name(&K_object), "object");
        assert_eq!(get_name(&K_value), "value");
    }

    #[test]
    fn get_noun_returns_constructor_name() {
        assert_eq!(get_noun(&K_number), "number");
        assert_eq!(get_noun(&K_text), "text");
    }

    #[test]
    fn get_range_number_defaults_to_zero() {
        assert_eq!(get_range_number(&K_number), 0);
        assert_eq!(get_range_number(&K_text), 0);
    }

    #[test]
    fn set_range_number_updates_value() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        set_range_number(&mut k, 42);
        assert_eq!(get_range_number(&k), 42);
    }

    // -----------------------------------------------------------------------
    // Object tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_object_returns_true_for_object_kind() {
        assert!(is_object(&K_object));
    }

    #[test]
    fn is_object_returns_false_for_non_object_kinds() {
        assert!(!is_object(&K_number));
        assert!(!is_object(&K_text));
        assert!(!is_object(&K_real_number));
        assert!(!is_object(&K_truth_state));
        assert!(!is_object(&K_value));
    }

    #[test]
    fn is_object_returns_false_for_nil_and_void() {
        assert!(!is_object(&K_nil));
        assert!(!is_object(&K_void));
    }

    #[test]
    fn is_subkind_of_object_returns_false_for_object_itself() {
        assert!(!is_subkind_of_object(&K_object));
    }

    #[test]
    fn is_subkind_of_object_returns_false_for_nil_and_void() {
        assert!(!is_subkind_of_object(&K_nil));
        assert!(!is_subkind_of_object(&K_void));
    }

    #[test]
    fn is_subkind_of_object_returns_false_for_non_object_kinds() {
        assert!(!is_subkind_of_object(&K_number));
        assert!(!is_subkind_of_object(&K_text));
    }

    #[test]
    fn is_object_of_kind_returns_true_for_object_conforming_to_value() {
        assert!(is_object_of_kind(&K_object, &K_value));
    }

    #[test]
    fn is_object_of_kind_returns_false_for_non_object() {
        assert!(!is_object_of_kind(&K_number, &K_value));
    }

    // -----------------------------------------------------------------------
    // Definiteness tests
    // -----------------------------------------------------------------------

    #[test]
    fn definite_returns_true_for_base_kinds() {
        assert!(definite(&K_number));
        assert!(definite(&K_text));
        assert!(definite(&K_object));
        assert!(definite(&K_real_number));
        assert!(definite(&K_truth_state));
        assert!(definite(&K_table));
    }

    #[test]
    fn definite_returns_false_for_protocol_kinds() {
        assert!(!definite(&K_value));
        assert!(!definite(&K_stored_value));
        assert!(!definite(&K_sayable_value));
        assert!(!definite(&K_understandable_value));
        assert!(!definite(&K_arithmetic_value));
        assert!(!definite(&K_enumerated_value));
    }

    #[test]
    fn definite_returns_true_for_list_of_numbers() {
        let list_of_numbers = Kind::unary_con(&*CON_list_of, K_number.clone());
        assert!(definite(&list_of_numbers));
    }

    #[test]
    fn definite_returns_false_for_list_of_values() {
        let list_of_values = Kind::unary_con(&*CON_list_of, K_value.clone());
        assert!(!definite(&list_of_values));
    }

    #[test]
    fn is_kind_of_kind_returns_true_for_protocol_kinds() {
        assert!(is_kind_of_kind(&K_value));
        assert!(is_kind_of_kind(&K_arithmetic_value));
        assert!(is_kind_of_kind(&K_sayable_value));
    }

    #[test]
    fn is_kind_of_kind_returns_false_for_base_kinds() {
        assert!(!is_kind_of_kind(&K_number));
        assert!(!is_kind_of_kind(&K_text));
        assert!(!is_kind_of_kind(&K_object));
    }

    #[test]
    fn semidefinite_returns_true_for_definite_kinds() {
        assert!(semidefinite(&K_number));
        assert!(semidefinite(&K_text));
    }

    #[test]
    fn semidefinite_returns_false_for_protocol_kinds() {
        assert!(!semidefinite(&K_value));
        assert!(!semidefinite(&K_arithmetic_value));
    }

    #[test]
    fn involves_var_returns_true_for_matching_variable() {
        let var = Kind::var_construction(1, None);
        assert!(involves_var(&var, 1));
    }

    #[test]
    fn involves_var_returns_false_for_non_matching_variable() {
        let var = Kind::var_construction(1, None);
        assert!(!involves_var(&var, 2));
    }

    #[test]
    fn involves_var_returns_false_for_base_kind() {
        assert!(!involves_var(&K_number, 1));
    }

    #[test]
    fn involves_var_with_zero_checks_any_variable() {
        let var = Kind::var_construction(3, None);
        assert!(involves_var(&var, 0));
        assert!(!involves_var(&K_number, 0));
    }

    #[test]
    fn semidefinite_returns_true_for_kind_involving_variable() {
        let var = Kind::var_construction(1, None);
        assert!(semidefinite(&var));
    }

    // -----------------------------------------------------------------------
    // Definition status tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_built_in_returns_true_for_familiar_kinds() {
        assert!(is_built_in(&K_number));
        assert!(is_built_in(&K_text));
        assert!(is_built_in(&K_object));
        assert!(is_built_in(&K_value));
    }

    #[test]
    fn is_uncertainly_defined_returns_false_for_familiar_kinds() {
        assert!(!is_uncertainly_defined(&K_number));
        assert!(!is_uncertainly_defined(&K_text));
    }

    #[test]
    fn is_an_enumeration_returns_false_for_number() {
        assert!(!is_an_enumeration(&K_number));
    }

    #[test]
    fn is_an_enumeration_returns_true_for_truth_state() {
        assert!(is_an_enumeration(&K_truth_state));
    }

    #[test]
    fn convert_to_enumeration_marks_kind_as_enumeration() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        assert!(!is_an_enumeration(&k));
        convert_to_enumeration(&mut k);
        assert!(is_an_enumeration(&k));
    }

    #[test]
    fn new_enumerated_value_returns_incrementing_values() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            let mut c = KindConstructor::new("test", ConstructorGroup::Base, 0);
            c.enumeration = true;
            c
        });
        let mut k = Kind::base_construction(&CON);
        let v0 = new_enumerated_value(&mut k);
        let v1 = new_enumerated_value(&mut k);
        let v2 = new_enumerated_value(&mut k);
        assert_eq!(v0, 0);
        assert_eq!(v1, 1);
        assert_eq!(v2, 2);
    }

    #[test]
    fn has_named_constant_values_returns_true_for_enumeration() {
        assert!(has_named_constant_values(&K_truth_state));
    }

    #[test]
    fn has_named_constant_values_returns_false_for_number() {
        assert!(!has_named_constant_values(&K_number));
    }

    #[test]
    fn convert_to_unit_returns_true_for_non_enumeration() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        assert!(convert_to_unit(&mut k));
    }

    #[test]
    fn convert_to_unit_returns_false_for_enumeration() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            let mut c = KindConstructor::new("test", ConstructorGroup::Base, 0);
            c.enumeration = true;
            c
        });
        let mut k = Kind::base_construction(&CON);
        assert!(!convert_to_unit(&mut k));
    }

    #[test]
    fn get_creating_sentence_returns_none_for_built_in() {
        assert!(get_creating_sentence(&K_number).is_none());
    }

    // -----------------------------------------------------------------------
    // Arithmetic tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_quasinumerical_returns_true_for_arithmetic_kinds() {
        assert!(is_quasinumerical(&K_number));
        assert!(is_quasinumerical(&K_real_number));
        assert!(is_quasinumerical(&K_unicode_character));
    }

    #[test]
    fn is_quasinumerical_returns_false_for_non_arithmetic_kinds() {
        assert!(!is_quasinumerical(&K_text));
        assert!(!is_quasinumerical(&K_object));
        assert!(!is_quasinumerical(&K_table));
    }

    #[test]
    fn uses_signed_comparisons_returns_true_for_number() {
        assert!(uses_signed_comparisons(&K_number));
    }

    #[test]
    fn uses_signed_comparisons_returns_false_for_truth_state() {
        assert!(!uses_signed_comparisons(&K_truth_state));
    }

    #[test]
    fn get_comparison_routine_returns_none_by_default() {
        assert!(get_comparison_routine(&K_number).is_none());
    }

    #[test]
    fn get_dimensional_form_returns_none_by_default() {
        assert!(get_dimensional_form(&K_number).is_none());
    }

    #[test]
    fn test_if_derived_returns_false_by_default() {
        assert!(!test_if_derived(&K_number));
    }

    #[test]
    fn now_derived_marks_kind_as_derived() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        assert!(!test_if_derived(&k));
        now_derived(&mut k);
        assert!(test_if_derived(&k));
    }

    #[test]
    fn scale_factor_returns_class_number() {
        assert_eq!(scale_factor(&K_number), 0);
    }

    #[test]
    fn get_dim_rules_returns_empty_by_default() {
        let rules = get_dim_rules(&K_number);
        assert!(rules.rules.is_empty());
    }

    // -----------------------------------------------------------------------
    // Identifier tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_identifier_returns_explicit_identifier() {
        assert_eq!(get_identifier(&K_number), Some("K_number"));
        assert_eq!(get_identifier(&K_text), Some("K_text"));
        assert_eq!(get_identifier(&K_object), Some("K_object"));
        assert_eq!(get_identifier(&K_value), Some("K_value"));
        assert_eq!(get_identifier(&K_truth_state), Some("K_truth_state"));
    }

    // -----------------------------------------------------------------------
    // Storage tests
    // -----------------------------------------------------------------------

    #[test]
    fn uses_block_values_returns_false_for_number() {
        assert!(!uses_block_values(&K_number));
    }

    #[test]
    fn uses_block_values_returns_true_for_text() {
        assert!(uses_block_values(&K_text));
    }

    #[test]
    fn uses_block_values_returns_true_for_object() {
        assert!(uses_block_values(&K_object));
    }

    #[test]
    fn get_small_block_size_returns_correct_value_for_text() {
        assert_eq!(get_small_block_size(&K_text), 64);
    }

    #[test]
    fn get_heap_size_estimate_returns_correct_value_for_text() {
        assert_eq!(get_heap_size_estimate(&K_text), 128);
    }

    #[test]
    fn get_small_block_size_returns_correct_value_for_object() {
        assert_eq!(get_small_block_size(&K_object), 32);
    }

    #[test]
    fn get_heap_size_estimate_returns_correct_value_for_object() {
        assert_eq!(get_heap_size_estimate(&K_object), 64);
    }

    #[test]
    fn get_distinguisher_returns_routine_for_text() {
        assert_eq!(get_distinguisher(&K_text), Some("TextDistinguisher"));
    }

    #[test]
    fn get_distinguisher_returns_none_for_number() {
        assert!(get_distinguisher(&K_number).is_none());
    }

    #[test]
    fn can_exchange_returns_false_by_default() {
        assert!(!can_exchange(&K_number));
    }

    // -----------------------------------------------------------------------
    // Indexing and documentation tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_index_priority_defaults_to_zero() {
        assert_eq!(get_index_priority(&K_number), 0);
    }

    #[test]
    fn get_documentation_reference_defaults_to_none() {
        assert!(get_documentation_reference(&K_number).is_none());
    }

    #[test]
    fn set_documentation_reference_updates_value() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        set_documentation_reference(&mut k, "doc/number");
        assert_eq!(get_documentation_reference(&k), Some("doc/number"));
    }

    #[test]
    fn set_specification_text_and_get_specification_text_round_trip() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        assert!(get_specification_text(&k).is_none());
        set_specification_text(&mut k, "A numeric kind");
        assert_eq!(get_specification_text(&k), Some("A numeric kind"));
    }

    #[test]
    fn get_index_default_value_defaults_to_none() {
        assert!(get_index_default_value(&K_number).is_none());
    }

    #[test]
    fn get_index_minimum_value_defaults_to_none() {
        assert!(get_index_minimum_value(&K_number).is_none());
    }

    #[test]
    fn get_index_maximum_value_defaults_to_none() {
        assert!(get_index_maximum_value(&K_number).is_none());
    }

    #[test]
    fn indexed_grey_if_empty_defaults_to_false() {
        assert!(!indexed_grey_if_empty(&K_number));
    }

    // -----------------------------------------------------------------------
    // Superkind tracking tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_superkind_set_at_defaults_to_none() {
        assert!(get_superkind_set_at(&K_number).is_none());
    }

    #[test]
    fn set_superkind_set_at_updates_value() {
        static CON: LazyLock<KindConstructor> = LazyLock::new(|| {
            KindConstructor::new("test", ConstructorGroup::Base, 0)
        });
        let mut k = Kind::base_construction(&CON);
        set_superkind_set_at(&mut k, Some(42));
        assert_eq!(get_superkind_set_at(&k), Some(42));
        set_superkind_set_at(&mut k, None);
        assert!(get_superkind_set_at(&k).is_none());
    }

    // -----------------------------------------------------------------------
    // Constant compilation method tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_constant_compilation_method_returns_correct_value() {
        assert_eq!(get_constant_compilation_method(&K_number), 1);
        assert_eq!(get_constant_compilation_method(&K_real_number), 2);
    }

    // -----------------------------------------------------------------------
    // Composite kind tests
    // -----------------------------------------------------------------------

    #[test]
    fn definite_works_for_list_of_numbers() {
        let list_of_numbers = Kind::unary_con(&*CON_list_of, K_number.clone());
        assert!(definite(&list_of_numbers));
    }

    #[test]
    fn definite_works_for_list_of_values() {
        let list_of_values = Kind::unary_con(&*CON_list_of, K_value.clone());
        assert!(!definite(&list_of_values));
    }

    #[test]
    fn involves_var_works_in_nested_kinds() {
        let var = Kind::var_construction(5, None);
        let list_of_var = Kind::unary_con(&*CON_list_of, var);
        assert!(involves_var(&list_of_var, 5));
        assert!(!involves_var(&list_of_var, 3));
    }

    #[test]
    fn is_object_works_for_list_of_objects() {
        let list_of_objects = Kind::unary_con(&*CON_list_of, K_object.clone());
        // list of objects is not itself an object
        assert!(!is_object(&list_of_objects));
    }

    #[test]
    fn is_kind_of_kind_works_for_proper_kinds() {
        let list_of_numbers = Kind::unary_con(&*CON_list_of, K_number.clone());
        assert!(!is_kind_of_kind(&list_of_numbers));
    }

    #[test]
    fn semidefinite_works_for_list_of_variable() {
        let var = Kind::var_construction(1, None);
        let list_of_var = Kind::unary_con(&*CON_list_of, var);
        assert!(semidefinite(&list_of_var));
    }

    #[test]
    fn semidefinite_works_for_list_of_values() {
        let list_of_values = Kind::unary_con(&*CON_list_of, K_value.clone());
        assert!(!semidefinite(&list_of_values));
    }
}
