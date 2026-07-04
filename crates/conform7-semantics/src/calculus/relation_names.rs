//! Built-in relation name constants.
//!
//! Corresponds to the relation name constants in the C reference
//! (`inform7/assertions-module/Chapter 2/Booting Verbs.w`, lines 147-169).

pub const EQUALITY_RELATION_NAME: usize = 0;
pub const UNIVERSAL_RELATION_NAME: usize = 1;
pub const MEANING_RELATION_NAME: usize = 2;
pub const EMPTY_RELATION_NAME: usize = 3;
pub const PROVISION_RELATION_NAME: usize = 4;
pub const GE_RELATION_NAME: usize = 5;
pub const GT_RELATION_NAME: usize = 6;
pub const LE_RELATION_NAME: usize = 7;
pub const LT_RELATION_NAME: usize = 8;
pub const ADJACENCY_RELATION_NAME: usize = 9;
pub const REGIONAL_CONTAINMENT_RELATION_NAME: usize = 10;
pub const CONTAINMENT_RELATION_NAME: usize = 11;
pub const SUPPORT_RELATION_NAME: usize = 12;
pub const INCORPORATION_RELATION_NAME: usize = 13;
pub const CARRYING_RELATION_NAME: usize = 14;
pub const HOLDING_RELATION_NAME: usize = 15;
pub const WEARING_RELATION_NAME: usize = 16;
pub const POSSESSION_RELATION_NAME: usize = 17;
pub const VISIBILITY_RELATION_NAME: usize = 18;
pub const TOUCHABILITY_RELATION_NAME: usize = 19;
pub const CONCEALMENT_RELATION_NAME: usize = 20;
pub const ENCLOSURE_RELATION_NAME: usize = 21;
pub const ROOM_CONTAINMENT_RELATION_NAME: usize = 22;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_name_constants_are_correct() {
        assert_eq!(EQUALITY_RELATION_NAME, 0);
        assert_eq!(UNIVERSAL_RELATION_NAME, 1);
        assert_eq!(MEANING_RELATION_NAME, 2);
        assert_eq!(EMPTY_RELATION_NAME, 3);
        assert_eq!(PROVISION_RELATION_NAME, 4);
        assert_eq!(GE_RELATION_NAME, 5);
        assert_eq!(GT_RELATION_NAME, 6);
        assert_eq!(LE_RELATION_NAME, 7);
        assert_eq!(LT_RELATION_NAME, 8);
        assert_eq!(ADJACENCY_RELATION_NAME, 9);
        assert_eq!(REGIONAL_CONTAINMENT_RELATION_NAME, 10);
        assert_eq!(CONTAINMENT_RELATION_NAME, 11);
        assert_eq!(SUPPORT_RELATION_NAME, 12);
        assert_eq!(INCORPORATION_RELATION_NAME, 13);
        assert_eq!(CARRYING_RELATION_NAME, 14);
        assert_eq!(HOLDING_RELATION_NAME, 15);
        assert_eq!(WEARING_RELATION_NAME, 16);
        assert_eq!(POSSESSION_RELATION_NAME, 17);
        assert_eq!(VISIBILITY_RELATION_NAME, 18);
        assert_eq!(TOUCHABILITY_RELATION_NAME, 19);
        assert_eq!(CONCEALMENT_RELATION_NAME, 20);
        assert_eq!(ENCLOSURE_RELATION_NAME, 21);
        assert_eq!(ROOM_CONTAINMENT_RELATION_NAME, 22);
    }
}
