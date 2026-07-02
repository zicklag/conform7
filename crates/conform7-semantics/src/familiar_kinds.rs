#![allow(non_upper_case_globals)]
//! Familiar kinds and constructors — global constants for all built-in kinds.
//!
//! The names follow the C reference convention (K_value, CON_list_of, etc.)
//! to match the Inform 7 source code terminology.
//!
//! Corresponds to `Familiar Kinds.w` in the C reference
//! (`services/kinds-module/Chapter 2/Familiar Kinds.w`).
//!
use std::sync::LazyLock;

use crate::kind_constructors::{ConstructorGroup, KindConstructor, Variance};
use crate::kinds::Kind;

// ---------------------------------------------------------------------------
// Protocol kind constructors
// ---------------------------------------------------------------------------
// Corresponds to `Familiar Kinds.w`, lines 29-37 in the C reference
// (`services/kinds-module/Chapter 2/Familiar Kinds.w`).

/// Constructor for the `value` protocol kind.
pub static CON_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("value", ConstructorGroup::Protocol, 0);
    c.explicit_identifier = Some("K_value");
    c
});

/// Constructor for the `stored value` protocol kind.
pub static CON_STORED_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("stored value", ConstructorGroup::Protocol, 0);
    c.explicit_identifier = Some("K_stored_value");
    c
});

/// Constructor for the `sayable value` protocol kind.
pub static CON_SAYABLE_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("sayable value", ConstructorGroup::Protocol, 0);
    c.explicit_identifier = Some("K_sayable_value");
    c
});

/// Constructor for the `understandable value` protocol kind.
pub static CON_UNDERSTANDABLE_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("understandable value", ConstructorGroup::Protocol, 0);
    c.explicit_identifier = Some("K_understandable_value");
    c
});

/// Constructor for the `arithmetic value` protocol kind.
pub static CON_ARITHMETIC_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("arithmetic value", ConstructorGroup::Protocol, 0);
    c.arithmetic = true;
    c.explicit_identifier = Some("K_arithmetic_value");
    c
});

/// Constructor for the `real arithmetic value` protocol kind.
pub static CON_REAL_ARITHMETIC_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("real arithmetic value", ConstructorGroup::Protocol, 0);
    c.arithmetic = true;
    c.explicit_identifier = Some("K_real_arithmetic_value");
    c
});

/// Constructor for the `enumerated value` protocol kind.
pub static CON_ENUMERATED_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("enumerated value", ConstructorGroup::Protocol, 0);
    c.enumeration = true;
    c.explicit_identifier = Some("K_enumerated_value");
    c
});

/// Constructor for the `pointer value` protocol kind.
pub static CON_POINTER_VALUE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("pointer value", ConstructorGroup::Protocol, 0);
    c.explicit_identifier = Some("K_pointer_value");
    c
});

// ---------------------------------------------------------------------------
// Protocol kinds
// ---------------------------------------------------------------------------

/// The `value` kind — top of the indefinite kind hierarchy.
pub static K_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_VALUE)
});

/// The `stored value` kind — values that can be stored in memory.
pub static K_stored_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_STORED_VALUE)
});

/// The `sayable value` kind — values that can be printed.
pub static K_sayable_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_SAYABLE_VALUE)
});

/// The `understandable value` kind — values that can be understood from
/// player input.
pub static K_understandable_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_UNDERSTANDABLE_VALUE)
});

/// The `arithmetic value` kind — values that support arithmetic operations.
pub static K_arithmetic_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_ARITHMETIC_VALUE)
});

/// The `real arithmetic value` kind — values that support real-number
/// arithmetic.
pub static K_real_arithmetic_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_REAL_ARITHMETIC_VALUE)
});

/// The `enumerated value` kind — values that are enumerations.
pub static K_enumerated_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_ENUMERATED_VALUE)
});

/// The `pointer value` kind — values that are pointers to objects.
pub static K_pointer_value: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_POINTER_VALUE)
});

// ---------------------------------------------------------------------------
// Punctuation constructors
// ---------------------------------------------------------------------------
// Corresponds to `Familiar Kinds.w`, lines 43-89 in the C reference.

/// Constructor for tuple entries in function kinds.
pub static CON_TUPLE_ENTRY: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("tuple entry", ConstructorGroup::Punctuation, 2);
    c.explicit_identifier = Some("CON_TUPLE_ENTRY");
    c
});

/// Constructor for the `void` kind.
pub static CON_VOID: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("void", ConstructorGroup::Punctuation, 0);
    c.explicit_identifier = Some("CON_VOID");
    c
});

/// Constructor for the `nil` kind.
pub static CON_NIL: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("nil", ConstructorGroup::Punctuation, 0);
    c.explicit_identifier = Some("CON_NIL");
    c
});

/// Constructor for the `unknown` kind.
pub static CON_UNKNOWN: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("unknown", ConstructorGroup::Punctuation, 0);
    c.explicit_identifier = Some("CON_UNKNOWN");
    c
});

/// Constructor for intermediate kinds (used during type inference).
pub static CON_INTERMEDIATE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("intermediate", ConstructorGroup::Punctuation, 0);
    c.explicit_identifier = Some("CON_INTERMEDIATE");
    c
});

/// Constructor for kind variables (A-Z).
pub static CON_KIND_VARIABLE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("kind variable", ConstructorGroup::Punctuation, 0);
    c.explicit_identifier = Some("CON_KIND_VARIABLE");
    c
});

// ---------------------------------------------------------------------------
// Punctuation kinds
// ---------------------------------------------------------------------------

/// The `void` kind — represents no value.
pub static K_void: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_VOID)
});

/// The `nil` kind — represents nothing (conforms to everything).
pub static K_nil: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_NIL)
});

/// The `unknown` kind — represents an unknown type.
pub static K_unknown: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_UNKNOWN)
});

// ---------------------------------------------------------------------------
// Base kind constructors
// ---------------------------------------------------------------------------
// Corresponds to `Familiar Kinds.w`, lines 95-109 in the C reference.

/// Constructor for the `number` kind.
pub static CON_NUMBER: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("number", ConstructorGroup::Base, 0);
    c.definite = true;
    c.arithmetic = true;
    c.explicit_identifier = Some("K_number");
    c.constant_compilation_method = 1; // NUMBER_CCM
    c.uses_block_values = false;
    c.uses_signed_comparisons = true;
    c
});

/// Constructor for the `text` kind.
pub static CON_TEXT: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("text", ConstructorGroup::Base, 0);
    c.definite = true;
    c.explicit_identifier = Some("K_text");
    c.uses_block_values = true;
    c.small_block_size = 64;
    c.heap_size_estimate = 128;
    c.distinguishing_routine = Some("TextDistinguisher");
    c
});

/// Constructor for the `object` kind.
pub static CON_OBJECT: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("object", ConstructorGroup::Base, 0);
    c.definite = true;
    c.object_kind = true;
    c.explicit_identifier = Some("K_object");
    c.uses_block_values = true;
    c.small_block_size = 32;
    c.heap_size_estimate = 64;
    c
});

/// Constructor for the `real number` kind.
pub static CON_REAL_NUMBER: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("real number", ConstructorGroup::Base, 0);
    c.definite = true;
    c.arithmetic = true;
    c.explicit_identifier = Some("K_real_number");
    c.constant_compilation_method = 2; // REAL_CCM
    c.uses_block_values = false;
    c.uses_signed_comparisons = true;
    c
});

/// Constructor for the `truth state` kind.
pub static CON_TRUTH_STATE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("truth state", ConstructorGroup::Base, 0);
    c.definite = true;
    c.enumeration = true;
    c.explicit_identifier = Some("K_truth_state");
    c.uses_block_values = false;
    c
});

/// Constructor for the `table` kind.
pub static CON_TABLE: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("table", ConstructorGroup::Base, 0);
    c.definite = true;
    c.explicit_identifier = Some("K_table");
    c.uses_block_values = true;
    c
});

/// Constructor for the `unicode character` kind.
pub static CON_UNICODE_CHARACTER: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("unicode character", ConstructorGroup::Base, 0);
    c.definite = true;
    c.arithmetic = true;
    c.explicit_identifier = Some("K_unicode_character");
    c.uses_block_values = false;
    c
});

/// Constructor for the `verb` kind.
pub static CON_VERB: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("verb", ConstructorGroup::Base, 0);
    c.definite = true;
    c.explicit_identifier = Some("K_verb");
    c.uses_block_values = false;
    c
});

// ---------------------------------------------------------------------------
// Base kinds
// ---------------------------------------------------------------------------

/// The `number` kind — integer values.
pub static K_number: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_NUMBER)
});

/// The `text` kind — string values.
pub static K_text: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_TEXT)
});

/// The `object` kind — physical objects in the world model.
pub static K_object: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_OBJECT)
});

/// The `real number` kind — floating-point values.
pub static K_real_number: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_REAL_NUMBER)
});

/// The `truth state` kind — boolean values (true/false).
pub static K_truth_state: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_TRUTH_STATE)
});

/// The `table` kind — table values.
pub static K_table: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_TABLE)
});

/// The `unicode character` kind — single Unicode character values.
pub static K_unicode_character: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_UNICODE_CHARACTER)
});

/// The `verb` kind — verb values.
pub static K_verb: LazyLock<Kind> = LazyLock::new(|| {
    Kind::base_construction(&*CON_VERB)
});

// ---------------------------------------------------------------------------
// Proper constructors
// ---------------------------------------------------------------------------
// Corresponds to `Familiar Kinds.w`, lines 131-142 in the C reference.

/// Constructor for `list of K` — ordered sequences of values.
pub static CON_list_of: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("list of K", ConstructorGroup::Proper, 1);
    c.definite = true;
    c.explicit_identifier = Some("CON_list_of");
    c
});

/// Constructor for `description of K` — descriptions that match values.
pub static CON_description: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("description of K", ConstructorGroup::Proper, 1);
    c.definite = true;
    c.explicit_identifier = Some("CON_description");
    c
});

/// Constructor for `relation of K to L` — binary relations.
pub static CON_relation: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("relation of K to L", ConstructorGroup::Proper, 2);
    c.definite = true;
    c.explicit_identifier = Some("CON_relation");
    c
});

/// Constructor for `rule` — rules.
pub static CON_rule: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("rule", ConstructorGroup::Proper, 0);
    c.definite = true;
    c.explicit_identifier = Some("CON_rule");
    c
});

/// Constructor for `rulebook` — rulebooks.
pub static CON_rulebook: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("rulebook", ConstructorGroup::Proper, 0);
    c.definite = true;
    c.explicit_identifier = Some("CON_rulebook");
    c
});

/// Constructor for `activity` — activities.
pub static CON_activity: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("activity", ConstructorGroup::Proper, 0);
    c.definite = true;
    c.explicit_identifier = Some("CON_activity");
    c
});

/// Constructor for `phrase K -> L` — function kinds.
pub static CON_phrase: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("phrase K -> L", ConstructorGroup::Proper, 2);
    c.definite = true;
    c.set_variance(0, Variance::Contravariant);
    c.set_variance(1, Variance::Covariant);
    c.explicit_identifier = Some("CON_phrase");
    c
});

/// Constructor for `property of K` — properties on kinds.
pub static CON_property: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("property of K", ConstructorGroup::Proper, 1);
    c.definite = true;
    c.explicit_identifier = Some("CON_property");
    c
});

/// Constructor for `table column` — table columns.
pub static CON_table_column: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("table column", ConstructorGroup::Proper, 0);
    c.definite = true;
    c.explicit_identifier = Some("CON_table_column");
    c
});

/// Constructor for `combination of K` — combinations.
pub static CON_combination: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("combination of K", ConstructorGroup::Proper, 1);
    c.definite = true;
    c.explicit_identifier = Some("CON_combination");
    c
});

/// Constructor for `variable of K` — variables.
pub static CON_variable: LazyLock<KindConstructor> = LazyLock::new(|| {
    let mut c = KindConstructor::new("variable of K", ConstructorGroup::Proper, 1);
    c.definite = true;
    c.explicit_identifier = Some("CON_variable");
    c
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_familiar_kinds_are_constructable() {
        // Protocol kinds
        let _ = &K_value;
        let _ = &K_stored_value;
        let _ = &K_sayable_value;
        let _ = &K_understandable_value;
        let _ = &K_arithmetic_value;
        let _ = &K_real_arithmetic_value;
        let _ = &K_enumerated_value;
        let _ = &K_pointer_value;

        // Punctuation kinds
        let _ = &K_void;
        let _ = &K_nil;
        let _ = &K_unknown;

        // Base kinds
        let _ = &K_number;
        let _ = &K_text;
        let _ = &K_object;
        let _ = &K_real_number;
        let _ = &K_truth_state;
        let _ = &K_table;
        let _ = &K_unicode_character;
        let _ = &K_verb;

        // All constructors
        let _ = &CON_VALUE;
        let _ = &CON_list_of;
        let _ = &CON_relation;
        let _ = &CON_phrase;
        let _ = &CON_description;
        let _ = &CON_rule;
        let _ = &CON_rulebook;
        let _ = &CON_activity;
        let _ = &CON_property;
        let _ = &CON_table_column;
        let _ = &CON_combination;
        let _ = &CON_variable;
        let _ = &CON_TUPLE_ENTRY;
        let _ = &CON_VOID;
        let _ = &CON_NIL;
        let _ = &CON_UNKNOWN;
        let _ = &CON_INTERMEDIATE;
        let _ = &CON_KIND_VARIABLE;
    }

    #[test]
    fn k_number_is_definite_arithmetic_not_enumeration_not_object() {
        assert!(K_number.get_construct().is_definite());
        assert!(K_number.get_construct().is_arithmetic());
        assert!(!K_number.get_construct().is_enumeration());
        assert!(!K_number.get_construct().is_object_kind());
    }

    #[test]
    fn k_text_is_definite_not_arithmetic_not_enumeration_not_object() {
        assert!(K_text.get_construct().is_definite());
        assert!(!K_text.get_construct().is_arithmetic());
        assert!(!K_text.get_construct().is_enumeration());
        assert!(!K_text.get_construct().is_object_kind());
    }

    #[test]
    fn k_object_is_definite_not_arithmetic_not_enumeration_is_object() {
        assert!(K_object.get_construct().is_definite());
        assert!(!K_object.get_construct().is_arithmetic());
        assert!(!K_object.get_construct().is_enumeration());
        assert!(K_object.get_construct().is_object_kind());
    }

    #[test]
    fn k_arithmetic_value_is_not_definite() {
        assert!(!K_arithmetic_value.get_construct().is_definite());
    }

    #[test]
    fn con_list_of_has_arity_one_and_is_covariant() {
        assert_eq!(CON_list_of.arity, 1);
        assert_eq!(CON_list_of.variance[0], Variance::Covariant);
    }

    #[test]
    fn con_relation_has_arity_two_and_is_covariant_in_both() {
        assert_eq!(CON_relation.arity, 2);
        assert_eq!(CON_relation.variance[0], Variance::Covariant);
        assert_eq!(CON_relation.variance[1], Variance::Covariant);
    }

    #[test]
    fn con_phrase_has_arity_two_contravariant_in_arg0_covariant_in_arg1() {
        assert_eq!(CON_phrase.arity, 2);
        assert_eq!(CON_phrase.variance[0], Variance::Contravariant);
        assert_eq!(CON_phrase.variance[1], Variance::Covariant);
    }

    #[test]
    fn list_of_numbers_can_be_constructed() {
        let list_of_num = Kind::unary_con(&CON_list_of, K_number.clone());
        assert!(list_of_num.get_construct().is_definite());
        assert_eq!(list_of_num.arity(), 1);
        assert!(list_of_num.unary_material().unwrap().eq(&K_number));
    }

    #[test]
    fn protocol_kinds_have_correct_groups() {
        assert_eq!(CON_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_STORED_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_SAYABLE_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_UNDERSTANDABLE_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_ARITHMETIC_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_REAL_ARITHMETIC_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_ENUMERATED_VALUE.group, ConstructorGroup::Protocol);
        assert_eq!(CON_POINTER_VALUE.group, ConstructorGroup::Protocol);
    }

    #[test]
    fn punctuation_constructors_have_correct_groups() {
        assert_eq!(CON_TUPLE_ENTRY.group, ConstructorGroup::Punctuation);
        assert_eq!(CON_VOID.group, ConstructorGroup::Punctuation);
        assert_eq!(CON_NIL.group, ConstructorGroup::Punctuation);
        assert_eq!(CON_UNKNOWN.group, ConstructorGroup::Punctuation);
        assert_eq!(CON_INTERMEDIATE.group, ConstructorGroup::Punctuation);
        assert_eq!(CON_KIND_VARIABLE.group, ConstructorGroup::Punctuation);
    }

    #[test]
    fn base_constructors_have_correct_groups() {
        assert_eq!(CON_NUMBER.group, ConstructorGroup::Base);
        assert_eq!(CON_TEXT.group, ConstructorGroup::Base);
        assert_eq!(CON_OBJECT.group, ConstructorGroup::Base);
        assert_eq!(CON_REAL_NUMBER.group, ConstructorGroup::Base);
        assert_eq!(CON_TRUTH_STATE.group, ConstructorGroup::Base);
        assert_eq!(CON_TABLE.group, ConstructorGroup::Base);
        assert_eq!(CON_UNICODE_CHARACTER.group, ConstructorGroup::Base);
        assert_eq!(CON_VERB.group, ConstructorGroup::Base);
    }

    #[test]
    fn proper_constructors_have_correct_groups() {
        assert_eq!(CON_list_of.group, ConstructorGroup::Proper);
        assert_eq!(CON_description.group, ConstructorGroup::Proper);
        assert_eq!(CON_relation.group, ConstructorGroup::Proper);
        assert_eq!(CON_rule.group, ConstructorGroup::Proper);
        assert_eq!(CON_rulebook.group, ConstructorGroup::Proper);
        assert_eq!(CON_activity.group, ConstructorGroup::Proper);
        assert_eq!(CON_phrase.group, ConstructorGroup::Proper);
        assert_eq!(CON_property.group, ConstructorGroup::Proper);
        assert_eq!(CON_table_column.group, ConstructorGroup::Proper);
        assert_eq!(CON_combination.group, ConstructorGroup::Proper);
        assert_eq!(CON_variable.group, ConstructorGroup::Proper);
    }

    #[test]
    fn all_constructors_have_explicit_identifiers() {
        assert_eq!(CON_VALUE.explicit_identifier, Some("K_value"));
        assert_eq!(CON_STORED_VALUE.explicit_identifier, Some("K_stored_value"));
        assert_eq!(CON_SAYABLE_VALUE.explicit_identifier, Some("K_sayable_value"));
        assert_eq!(CON_UNDERSTANDABLE_VALUE.explicit_identifier, Some("K_understandable_value"));
        assert_eq!(CON_ARITHMETIC_VALUE.explicit_identifier, Some("K_arithmetic_value"));
        assert_eq!(CON_REAL_ARITHMETIC_VALUE.explicit_identifier, Some("K_real_arithmetic_value"));
        assert_eq!(CON_ENUMERATED_VALUE.explicit_identifier, Some("K_enumerated_value"));
        assert_eq!(CON_POINTER_VALUE.explicit_identifier, Some("K_pointer_value"));
        assert_eq!(CON_NUMBER.explicit_identifier, Some("K_number"));
        assert_eq!(CON_TEXT.explicit_identifier, Some("K_text"));
        assert_eq!(CON_OBJECT.explicit_identifier, Some("K_object"));
        assert_eq!(CON_REAL_NUMBER.explicit_identifier, Some("K_real_number"));
        assert_eq!(CON_TRUTH_STATE.explicit_identifier, Some("K_truth_state"));
        assert_eq!(CON_TABLE.explicit_identifier, Some("K_table"));
        assert_eq!(CON_UNICODE_CHARACTER.explicit_identifier, Some("K_unicode_character"));
        assert_eq!(CON_VERB.explicit_identifier, Some("K_verb"));
        assert_eq!(CON_TUPLE_ENTRY.explicit_identifier, Some("CON_TUPLE_ENTRY"));
        assert_eq!(CON_VOID.explicit_identifier, Some("CON_VOID"));
        assert_eq!(CON_NIL.explicit_identifier, Some("CON_NIL"));
        assert_eq!(CON_UNKNOWN.explicit_identifier, Some("CON_UNKNOWN"));
        assert_eq!(CON_INTERMEDIATE.explicit_identifier, Some("CON_INTERMEDIATE"));
        assert_eq!(CON_KIND_VARIABLE.explicit_identifier, Some("CON_KIND_VARIABLE"));
        assert_eq!(CON_list_of.explicit_identifier, Some("CON_list_of"));
        assert_eq!(CON_description.explicit_identifier, Some("CON_description"));
        assert_eq!(CON_relation.explicit_identifier, Some("CON_relation"));
        assert_eq!(CON_rule.explicit_identifier, Some("CON_rule"));
        assert_eq!(CON_rulebook.explicit_identifier, Some("CON_rulebook"));
        assert_eq!(CON_activity.explicit_identifier, Some("CON_activity"));
        assert_eq!(CON_phrase.explicit_identifier, Some("CON_phrase"));
        assert_eq!(CON_property.explicit_identifier, Some("CON_property"));
        assert_eq!(CON_table_column.explicit_identifier, Some("CON_table_column"));
        assert_eq!(CON_combination.explicit_identifier, Some("CON_combination"));
        assert_eq!(CON_variable.explicit_identifier, Some("CON_variable"));
    }
}
