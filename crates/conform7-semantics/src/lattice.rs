use std::ptr;

use crate::familiar_kinds;
use crate::kinds::{Kind, KindMutableState};

/// The superkind of a given kind in the conformance hierarchy.
///
/// Returns None for kinds that have no superkind (value, nil, void).
///
/// Corresponds to `Latticework::super` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 89-119).
pub fn superkind(k: &Kind) -> Option<&'static Kind> {
    use familiar_kinds::*;

    // Protocol kinds: hard-code the hierarchy
    if ptr::eq(k.construct, &*CON_VALUE) {
        return None;
    }
    if ptr::eq(k.construct, &*CON_STORED_VALUE) {
        return Some(&K_value);
    }
    if ptr::eq(k.construct, &*CON_SAYABLE_VALUE) {
        return Some(&K_stored_value);
    }
    if ptr::eq(k.construct, &*CON_UNDERSTANDABLE_VALUE) {
        return Some(&K_sayable_value);
    }
    if ptr::eq(k.construct, &*CON_ARITHMETIC_VALUE) {
        return Some(&K_understandable_value);
    }
    if ptr::eq(k.construct, &*CON_REAL_ARITHMETIC_VALUE) {
        return Some(&K_arithmetic_value);
    }
    if ptr::eq(k.construct, &*CON_ENUMERATED_VALUE) {
        return Some(&K_understandable_value);
    }
    if ptr::eq(k.construct, &*CON_POINTER_VALUE) {
        return Some(&K_sayable_value);
    }

    // Nil, void, unknown have no superkind
    if ptr::eq(k.construct, &*CON_NIL) {
        return None;
    }
    if ptr::eq(k.construct, &*CON_VOID) {
        return None;
    }
    if ptr::eq(k.construct, &*CON_UNKNOWN) {
        return None;
    }

    // Base kinds: hard-code the superkind for each base kind
    // using pointer comparison with the static constructors.
    if ptr::eq(k.construct, &*CON_NUMBER) {
        return Some(&K_real_number);
    }
    if ptr::eq(k.construct, &*CON_REAL_NUMBER) {
        return Some(&K_arithmetic_value);
    }
    if ptr::eq(k.construct, &*CON_TEXT) {
        return Some(&K_sayable_value);
    }
    if ptr::eq(k.construct, &*CON_TRUTH_STATE) {
        return Some(&K_enumerated_value);
    }
    if ptr::eq(k.construct, &*CON_OBJECT) {
        return Some(&K_pointer_value);
    }
    if ptr::eq(k.construct, &*CON_TABLE) {
        return Some(&K_stored_value);
    }
    if ptr::eq(k.construct, &*CON_UNICODE_CHARACTER) {
        return Some(&K_arithmetic_value);
    }
    if ptr::eq(k.construct, &*CON_VERB) {
        return Some(&K_value);
    }

    // Proper constructors: superkind is value
    if k.construct.group == crate::kind_constructors::ConstructorGroup::Proper {
        return Some(&K_value);
    }

    // Punctuation kinds (tuple entry, kind variable, intermediate): no superkind
    None
}

/// Compute the join (least upper bound) of two kinds in the lattice.
///
/// Returns None if no common superkind can be found.
///
/// Corresponds to `Latticework::join` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 134-140).
pub fn join(k1: &Kind, k2: &Kind) -> Option<Kind> {
    // If they're equal, the join is the kind itself
    if k1.eq(k2) {
        return Some(k1.clone());
    }

    // If one conforms to the other, the join is the more general one
    if k1.conforms_to(k2) {
        return Some(k2.clone());
    }
    if k2.conforms_to(k1) {
        return Some(k1.clone());
    }

    // For proper constructors with the same constructor, recursively join children
    if k1.arity() > 0
        && k1.arity() == k2.arity()
        && ptr::eq(k1.construct, k2.construct)
    {
        let mut joined_args: [Option<Box<Kind>>; 2] = [None, None];
        let mut all_joined = true;

        for (i, (a, b)) in k1.kc_args.iter().zip(k2.kc_args.iter()).enumerate() {
            if i >= k1.arity() as usize {
                break;
            }
            match (a, b) {
                (Some(a), Some(b)) => {
                    if let Some(j) = join(a, b) {
                        joined_args[i] = Some(Box::new(j));
                    } else {
                        all_joined = false;
                    }
                }
                (None, None) => {}
                _ => {
                    all_joined = false;
                }
            }
        }

        if all_joined {
            return Some(Kind {
                construct: k1.construct,
                kind_variable_number: 0,
                kc_args: joined_args,
                construct_id: usize::MAX,
                mutable_state: KindMutableState::from_constructor(k1.construct),
            });
        }
        }
    // Walk up k1's superkind chain and check if it conforms to k2
    if let Some(super_k) = superkind(k1) {
        if let Some(j) = join(super_k, k2) {
            return Some(j);
        }
    }

    // Walk up k2's superkind chain
    if let Some(super_k) = superkind(k2) {
        if let Some(j) = join(k1, super_k) {
            return Some(j);
        }
    }

    None
}

/// Compute the meet (greatest lower bound) of two kinds in the lattice.
///
/// Returns None if no common subkind can be found.
///
/// Corresponds to `Latticework::meet` in the C reference
/// (`services/kinds-module/Chapter 2/The Lattice of Kinds.w`, lines 134-140).
pub fn meet(k1: &Kind, k2: &Kind) -> Option<Kind> {
    // If they're equal, the meet is the kind itself
    if k1.eq(k2) {
        return Some(k1.clone());
    }

    // If one conforms to the other, the meet is the more specific one
    if k1.conforms_to(k2) {
        return Some(k1.clone());
    }
    if k2.conforms_to(k1) {
        return Some(k2.clone());
    }

    // For proper constructors with the same constructor, recursively meet children
    if k1.arity() > 0
        && k1.arity() == k2.arity()
        && ptr::eq(k1.construct, k2.construct)
    {
        let mut met_args: [Option<Box<Kind>>; 2] = [None, None];
        let mut all_met = true;

        for (i, (a, b)) in k1.kc_args.iter().zip(k2.kc_args.iter()).enumerate() {
            if i >= k1.arity() as usize {
                break;
            }
            match (a, b) {
                (Some(a), Some(b)) => {
                    if let Some(m) = meet(a, b) {
                        met_args[i] = Some(Box::new(m));
                    } else {
                        all_met = false;
                    }
                }
                (None, None) => {}
                _ => {
                    all_met = false;
                }
            }
        }

        if all_met {
            return Some(Kind {
                construct: k1.construct,
                kind_variable_number: 0,
                kc_args: met_args,
                construct_id: usize::MAX,
                mutable_state: KindMutableState::from_constructor(k1.construct),
            });
        }
    }

    // Unrelated kinds have no common subkind
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar_kinds::*;
    use crate::kinds::Kind;

    #[test]
    fn k_number_conforms_to_k_arithmetic_value() {
        assert!(K_number.conforms_to(&K_arithmetic_value));
    }

    #[test]
    fn k_number_conforms_to_k_value() {
        assert!(K_number.conforms_to(&K_value));
    }

    #[test]
    fn k_number_does_not_conform_to_k_text() {
        assert!(!K_number.conforms_to(&K_text));
    }

    #[test]
    fn k_text_conforms_to_k_sayable_value() {
        assert!(K_text.conforms_to(&K_sayable_value));
    }

    #[test]
    fn k_object_conforms_to_k_value() {
        assert!(K_object.conforms_to(&K_value));
    }

    #[test]
    fn list_of_numbers_conforms_to_list_of_arithmetic_values() {
        let list_of_num = Kind::unary_con(&CON_list_of, (*K_number).clone());
        let list_of_arith = Kind::unary_con(&CON_list_of, (*K_arithmetic_value).clone());
        assert!(list_of_num.conforms_to(&list_of_arith));
    }

    #[test]
    fn list_of_numbers_does_not_conform_to_list_of_texts() {
        let list_of_num = Kind::unary_con(&CON_list_of, (*K_number).clone());
        let list_of_text = Kind::unary_con(&CON_list_of, (*K_text).clone());
        assert!(!list_of_num.conforms_to(&list_of_text));
    }

    #[test]
    fn k_nil_conforms_to_everything() {
        assert!(K_nil.conforms_to(&K_number));
        assert!(K_nil.conforms_to(&K_text));
        assert!(K_nil.conforms_to(&K_value));
        assert!(K_nil.conforms_to(&K_void));
    }

    #[test]
    fn k_value_does_not_conform_to_k_number() {
        assert!(!K_value.conforms_to(&K_number));
    }

    #[test]
    fn join_number_and_text_is_sayable_value() {
        let j = join(&K_number, &K_text);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn join_number_and_real_number_is_real_number() {
        let j = join(&K_number, &K_real_number);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_real_number));
    }

    #[test]
    fn meet_number_and_arithmetic_value_is_number() {
        let m = meet(&K_number, &K_arithmetic_value);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_number));
    }

    #[test]
    fn meet_number_and_text_is_none() {
        // number and text have no common subkind, so meet should be None
        let m = meet(&K_number, &K_text);
        assert!(m.is_none());
    }

    #[test]
    fn k_number_is_always_compatible_with_k_real_number() {
        use crate::kinds::Compatibility;
        assert_eq!(
            K_number.compatible(&K_real_number),
            Compatibility::Always
        );
    }

    #[test]
    fn k_number_is_never_compatible_with_k_text() {
        use crate::kinds::Compatibility;
        assert_eq!(
            K_number.compatible(&K_text),
            Compatibility::Never
        );
    }

    #[test]
    fn k_object_is_always_compatible_with_k_pointer_value() {
        use crate::kinds::Compatibility;
        assert_eq!(
            K_object.compatible(&K_pointer_value),
            Compatibility::Always
        );
    }

    #[test]
    fn superkind_of_value_is_none() {
        assert!(superkind(&K_value).is_none());
    }

    #[test]
    fn superkind_of_stored_value_is_value() {
        let sup = superkind(&K_stored_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_value));
    }

    #[test]
    fn superkind_of_sayable_value_is_stored_value() {
        let sup = superkind(&K_sayable_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_stored_value));
    }

    #[test]
    fn superkind_of_understandable_value_is_sayable_value() {
        let sup = superkind(&K_understandable_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn superkind_of_arithmetic_value_is_understandable_value() {
        let sup = superkind(&K_arithmetic_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_understandable_value));
    }

    #[test]
    fn superkind_of_real_arithmetic_value_is_arithmetic_value() {
        let sup = superkind(&K_real_arithmetic_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_arithmetic_value));
    }

    #[test]
    fn superkind_of_enumerated_value_is_understandable_value() {
        let sup = superkind(&K_enumerated_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_understandable_value));
    }

    #[test]
    fn superkind_of_pointer_value_is_sayable_value() {
        let sup = superkind(&K_pointer_value);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn superkind_of_number_is_real_number() {
        let sup = superkind(&K_number);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_real_number));
    }

    #[test]
    fn superkind_of_real_number_is_arithmetic_value() {
        let sup = superkind(&K_real_number);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_arithmetic_value));
    }

    #[test]
    fn superkind_of_text_is_sayable_value() {
        let sup = superkind(&K_text);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn superkind_of_object_is_pointer_value() {
        let sup = superkind(&K_object);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_pointer_value));
    }

    #[test]
    fn superkind_of_truth_state_is_enumerated_value() {
        let sup = superkind(&K_truth_state);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_enumerated_value));
    }

    #[test]
    fn superkind_of_table_is_stored_value() {
        let sup = superkind(&K_table);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_stored_value));
    }

    #[test]
    fn superkind_of_unicode_character_is_arithmetic_value() {
        let sup = superkind(&K_unicode_character);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_arithmetic_value));
    }

    #[test]
    fn superkind_of_verb_is_value() {
        let sup = superkind(&K_verb);
        assert!(sup.is_some());
        assert!(sup.unwrap().eq(&K_value));
    }

    #[test]
    fn join_same_kind_returns_that_kind() {
        let j = join(&K_number, &K_number);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_number));
    }

    #[test]
    fn join_kind_and_its_superkind_returns_superkind() {
        let j = join(&K_number, &K_arithmetic_value);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_arithmetic_value));
    }

    #[test]
    fn join_list_of_number_and_list_of_text_is_list_of_value() {
        let list_of_num = Kind::unary_con(&CON_list_of, (*K_number).clone());
        let list_of_text = Kind::unary_con(&CON_list_of, (*K_text).clone());
        let j = join(&list_of_num, &list_of_text);
        // join(number, text) = sayable_value, so join(list of number, list of text) = list of sayable_value
        let expected = Kind::unary_con(&CON_list_of, (*K_sayable_value).clone());
        assert!(j.unwrap().eq(&expected));
    }

    #[test]
    fn meet_same_kind_returns_that_kind() {
        let m = meet(&K_number, &K_number);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_number));
    }

    #[test]
    fn meet_kind_and_its_superkind_returns_kind() {
        let m = meet(&K_number, &K_arithmetic_value);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_number));
    }

    #[test]
    fn conformance_is_reflexive() {
        assert!(K_number.conforms_to(&K_number));
        assert!(K_text.conforms_to(&K_text));
        assert!(K_value.conforms_to(&K_value));
    }

    #[test]
    fn conformance_is_transitive() {
        // number -> real_number -> arithmetic_value -> understandable_value
        // -> sayable_value -> stored_value -> value
        assert!(K_number.conforms_to(&K_understandable_value));
        assert!(K_number.conforms_to(&K_sayable_value));
        assert!(K_number.conforms_to(&K_stored_value));
    }

    #[test]
    fn conformance_chain_via_superkind() {
        // Walk the chain explicitly
        let sup = superkind(&K_number).unwrap();
        assert!(sup.eq(&K_real_number));
        let sup2 = superkind(sup).unwrap();
        assert!(sup2.eq(&K_arithmetic_value));
        let sup3 = superkind(sup2).unwrap();
        assert!(sup3.eq(&K_understandable_value));
        let sup4 = superkind(sup3).unwrap();
        assert!(sup4.eq(&K_sayable_value));
        let sup5 = superkind(sup4).unwrap();
        assert!(sup5.eq(&K_stored_value));
        let sup6 = superkind(sup5).unwrap();
        assert!(sup6.eq(&K_value));
        assert!(superkind(sup6).is_none());
    }

    #[test]
    fn join_of_unrelated_base_kinds() {
        // number and object: number -> real_number -> arithmetic_value
        //   -> understandable_value -> sayable_value -> stored_value -> value
        // object -> pointer_value -> sayable_value -> stored_value -> value
        // Common superkind: sayable_value
        let j = join(&K_number, &K_object);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn compatible_always_for_related_kinds() {
        use crate::kinds::Compatibility;
        assert_eq!(
            K_number.compatible(&K_arithmetic_value),
            Compatibility::Always
        );
        assert_eq!(
            K_arithmetic_value.compatible(&K_number),
            Compatibility::Always
        );
    }

    #[test]
    fn compatible_never_for_unrelated_kinds() {
        use crate::kinds::Compatibility;
        assert_eq!(
            K_number.compatible(&K_text),
            Compatibility::Never
        );
        assert_eq!(
            K_text.compatible(&K_object),
            Compatibility::Never
        );
    }

    #[test]
    fn nil_is_always_compatible() {
        use crate::kinds::Compatibility;
        assert_eq!(K_nil.compatible(&K_number), Compatibility::Always);
        assert_eq!(K_nil.compatible(&K_text), Compatibility::Always);
        assert_eq!(K_number.compatible(&K_nil), Compatibility::Always);
    }

    #[test]
    fn void_is_always_compatible() {
        use crate::kinds::Compatibility;
        assert_eq!(K_void.compatible(&K_number), Compatibility::Always);
        assert_eq!(K_number.compatible(&K_void), Compatibility::Always);
    }

    #[test]
    fn unknown_is_always_compatible() {
        use crate::kinds::Compatibility;
        assert_eq!(K_unknown.compatible(&K_number), Compatibility::Always);
        assert_eq!(K_number.compatible(&K_unknown), Compatibility::Always);
    }

    #[test]
    fn number_conforms_to_real_number() {
        assert!(K_number.conforms_to(&K_real_number));
    }

    #[test]
    fn real_number_does_not_conform_to_number() {
        assert!(!K_real_number.conforms_to(&K_number));
    }

    #[test]
    fn meet_number_and_real_number_is_number() {
        let m = meet(&K_number, &K_real_number);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_number));
    }

    #[test]
    fn meet_real_number_and_arithmetic_value_is_real_number() {
        let m = meet(&K_real_number, &K_arithmetic_value);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_real_number));
    }

    #[test]
    fn join_text_and_object_is_sayable_value() {
        // text -> sayable_value -> stored_value -> value
        // object -> pointer_value -> sayable_value -> stored_value -> value
        // Common superkind: sayable_value
        let j = join(&K_text, &K_object);
        assert!(j.is_some());
        assert!(j.unwrap().eq(&K_sayable_value));
    }

    #[test]
    fn meet_text_and_sayable_value_is_text() {
        let m = meet(&K_text, &K_sayable_value);
        assert!(m.is_some());
        assert!(m.unwrap().eq(&K_text));
    }
}
