//! Linguistic constants for grammatical attributes.
//!
//! The `Lcon` type encodes a linguistic constant — a compact representation of
//! a word form with grammatical attributes (voice, tense, sense, person,
//! number, case, gender). This is a simplified version that supports the
//! attributes needed by the verb system.
//!
//! # References
//!
//! - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w` —
//!   the `lcon_ti` type and constants for voice, tense, sense, person, number,
//!   case, and gender.

use std::fmt;

// ---------------------------------------------------------------------------
// Voice constants
// ---------------------------------------------------------------------------

/// Active voice constant.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const ACTIVE_VOICE: i32 = 0;

/// Passive voice constant.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const PASSIVE_VOICE: i32 = 1;

// ---------------------------------------------------------------------------
// Tense constants
// ---------------------------------------------------------------------------

/// "is" tense (present).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const IS_TENSE: i32 = 0;

/// "was" tense (past).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const WAS_TENSE: i32 = 1;

/// "has" tense (present perfect).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const HAS_TENSE: i32 = 2;

/// "had" tense (past perfect).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const HAD_TENSE: i32 = 3;

/// "will" tense (future).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const WILL_TENSE: i32 = 4;

/// "would" tense (conditional).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const WOULD_TENSE: i32 = 5;

// ---------------------------------------------------------------------------
// Sense constants
// ---------------------------------------------------------------------------

/// Positive sense.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const POSITIVE_SENSE: i32 = 0;

/// Negative sense.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const NEGATIVE_SENSE: i32 = 1;

// ---------------------------------------------------------------------------
// Person constants
// ---------------------------------------------------------------------------

/// First person (I, we).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const FIRST_PERSON: i32 = 0;

/// Second person (you).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const SECOND_PERSON: i32 = 1;

/// Third person (he, she, it, they).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const THIRD_PERSON: i32 = 2;

// ---------------------------------------------------------------------------
// Number constants
// ---------------------------------------------------------------------------

/// Singular number.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const SINGULAR_NUMBER: i32 = 0;

/// Plural number.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const PLURAL_NUMBER: i32 = 1;

// ---------------------------------------------------------------------------
// Case constants
// ---------------------------------------------------------------------------

/// Nominative case (subject).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const NOMINATIVE_CASE: i32 = 0;

/// Accusative case (direct object).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const ACCUSATIVE_CASE: i32 = 1;

/// Genitive case (possessive).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const GENITIVE_CASE: i32 = 2;

/// Dative case (indirect object).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const DATIVE_CASE: i32 = 3;

// ---------------------------------------------------------------------------
// Gender constants
// ---------------------------------------------------------------------------

/// Neuter gender (it).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const NEUTER_GENDER: i32 = 0;

/// Masculine gender (he).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const MASCULINE_GENDER: i32 = 1;

/// Feminine gender (she).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const FEMININE_GENDER: i32 = 2;

/// Common gender (they).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
pub const COMMON_GENDER: i32 = 3;

// ---------------------------------------------------------------------------
// Lcon struct
// ---------------------------------------------------------------------------

/// A linguistic constant — a compact representation of a word form with
/// grammatical attributes.
///
/// In the C reference, `lcon_ti` is a bit-packed integer. Here we use a struct
/// with explicit fields for clarity.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lcon {
    /// The allocation ID for stock references.
    pub id: Option<usize>,
    /// Voice: `ACTIVE_VOICE` or `PASSIVE_VOICE`.
    pub voice: i32,
    /// Tense: `IS_TENSE`, `WAS_TENSE`, etc.
    pub tense: i32,
    /// Sense: `POSITIVE_SENSE` or `NEGATIVE_SENSE`.
    pub sense: i32,
    /// Person: `FIRST_PERSON`, `SECOND_PERSON`, or `THIRD_PERSON`.
    pub person: i32,
    /// Number: `SINGULAR_NUMBER` or `PLURAL_NUMBER`.
    pub number: i32,
    /// Case: `NOMINATIVE_CASE`, `ACCUSATIVE_CASE`, etc.
    pub case: i32,
    /// Gender: `NEUTER_GENDER`, `MASCULINE_GENDER`, etc.
    pub gender: i32,
}

impl Lcon {
    /// Create a new Lcon with the given ID and default attributes.
    ///
    /// Default attributes are: active voice, present tense, positive sense,
    /// third person, singular number, nominative case, neuter gender.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
    pub fn of_id(id: usize) -> Self {
        Lcon {
            id: Some(id),
            voice: ACTIVE_VOICE,
            tense: IS_TENSE,
            sense: POSITIVE_SENSE,
            person: THIRD_PERSON,
            number: SINGULAR_NUMBER,
            case: NOMINATIVE_CASE,
            gender: NEUTER_GENDER,
        }
    }

    /// Create a new Lcon with no ID and default attributes.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
    pub fn new() -> Self {
        Lcon {
            id: None,
            voice: ACTIVE_VOICE,
            tense: IS_TENSE,
            sense: POSITIVE_SENSE,
            person: THIRD_PERSON,
            number: SINGULAR_NUMBER,
            case: NOMINATIVE_CASE,
            gender: NEUTER_GENDER,
        }
    }

    /// Get the ID from this Lcon.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Linguistic Constants.w`
    pub fn get_id(&self) -> Option<usize> {
        self.id
    }

    /// Get the voice attribute.
    pub fn get_voice(&self) -> i32 {
        self.voice
    }

    /// Get the tense attribute.
    pub fn get_tense(&self) -> i32 {
        self.tense
    }

    /// Get the sense attribute.
    pub fn get_sense(&self) -> i32 {
        self.sense
    }

    /// Get the person attribute.
    pub fn get_person(&self) -> i32 {
        self.person
    }

    /// Get the number attribute.
    pub fn get_number(&self) -> i32 {
        self.number
    }

    /// Get the case attribute.
    pub fn get_case(&self) -> i32 {
        self.case
    }

    /// Get the gender attribute.
    pub fn get_gender(&self) -> i32 {
        self.gender
    }

    /// Set the voice attribute.
    pub fn with_voice(mut self, voice: i32) -> Self {
        self.voice = voice;
        self
    }

    /// Set the tense attribute.
    pub fn with_tense(mut self, tense: i32) -> Self {
        self.tense = tense;
        self
    }

    /// Set the sense attribute.
    pub fn with_sense(mut self, sense: i32) -> Self {
        self.sense = sense;
        self
    }

    /// Set the person attribute.
    pub fn with_person(mut self, person: i32) -> Self {
        self.person = person;
        self
    }

    /// Set the number attribute.
    pub fn with_number(mut self, number: i32) -> Self {
        self.number = number;
        self
    }

    /// Set the case attribute.
    pub fn with_case(mut self, case: i32) -> Self {
        self.case = case;
        self
    }

    /// Set the gender attribute.
    pub fn with_gender(mut self, gender: i32) -> Self {
        self.gender = gender;
        self
    }
}

impl Default for Lcon {
    fn default() -> Self {
        Lcon::new()
    }
}

impl fmt::Display for Lcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lcon(id={:?}, voice={}, tense={}, sense={}, person={}, number={}, case={}, gender={})",
            self.id, self.voice, self.tense, self.sense, self.person,
            self.number, self.case, self.gender)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcon_default() {
        // Reference: services/inflections-module/Chapter 3/Linguistic Constants.w
        let lcon = Lcon::new();
        assert_eq!(lcon.get_id(), None);
        assert_eq!(lcon.get_voice(), ACTIVE_VOICE);
        assert_eq!(lcon.get_tense(), IS_TENSE);
        assert_eq!(lcon.get_sense(), POSITIVE_SENSE);
        assert_eq!(lcon.get_person(), THIRD_PERSON);
        assert_eq!(lcon.get_number(), SINGULAR_NUMBER);
        assert_eq!(lcon.get_case(), NOMINATIVE_CASE);
        assert_eq!(lcon.get_gender(), NEUTER_GENDER);
    }

    #[test]
    fn test_lcon_of_id() {
        // Reference: services/inflections-module/Chapter 3/Linguistic Constants.w
        let lcon = Lcon::of_id(42);
        assert_eq!(lcon.get_id(), Some(42));
    }

    #[test]
    fn test_lcon_with_voice() {
        let lcon = Lcon::new().with_voice(PASSIVE_VOICE);
        assert_eq!(lcon.get_voice(), PASSIVE_VOICE);
    }

    #[test]
    fn test_lcon_with_tense() {
        let lcon = Lcon::new().with_tense(WAS_TENSE);
        assert_eq!(lcon.get_tense(), WAS_TENSE);
    }

    #[test]
    fn test_lcon_with_sense() {
        let lcon = Lcon::new().with_sense(NEGATIVE_SENSE);
        assert_eq!(lcon.get_sense(), NEGATIVE_SENSE);
    }

    #[test]
    fn test_lcon_with_person() {
        let lcon = Lcon::new().with_person(FIRST_PERSON);
        assert_eq!(lcon.get_person(), FIRST_PERSON);
    }

    #[test]
    fn test_lcon_with_number() {
        let lcon = Lcon::new().with_number(PLURAL_NUMBER);
        assert_eq!(lcon.get_number(), PLURAL_NUMBER);
    }

    #[test]
    fn test_lcon_with_case() {
        let lcon = Lcon::new().with_case(ACCUSATIVE_CASE);
        assert_eq!(lcon.get_case(), ACCUSATIVE_CASE);
    }

    #[test]
    fn test_lcon_with_gender() {
        let lcon = Lcon::new().with_gender(FEMININE_GENDER);
        assert_eq!(lcon.get_gender(), FEMININE_GENDER);
    }

    #[test]
    fn test_lcon_chained_builders() {
        let lcon = Lcon::of_id(7)
            .with_voice(PASSIVE_VOICE)
            .with_tense(WAS_TENSE)
            .with_sense(NEGATIVE_SENSE)
            .with_person(FIRST_PERSON)
            .with_number(PLURAL_NUMBER)
            .with_case(DATIVE_CASE)
            .with_gender(COMMON_GENDER);
        assert_eq!(lcon.get_id(), Some(7));
        assert_eq!(lcon.get_voice(), PASSIVE_VOICE);
        assert_eq!(lcon.get_tense(), WAS_TENSE);
        assert_eq!(lcon.get_sense(), NEGATIVE_SENSE);
        assert_eq!(lcon.get_person(), FIRST_PERSON);
        assert_eq!(lcon.get_number(), PLURAL_NUMBER);
        assert_eq!(lcon.get_case(), DATIVE_CASE);
        assert_eq!(lcon.get_gender(), COMMON_GENDER);
    }

    #[test]
    fn test_lcon_eq() {
        let a = Lcon::of_id(1).with_voice(PASSIVE_VOICE);
        let b = Lcon::of_id(1).with_voice(PASSIVE_VOICE);
        let c = Lcon::of_id(2).with_voice(PASSIVE_VOICE);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
