//! Verb conjugation system (simplified for English).
//!
//! The verb conjugation system turns a base verb form (infinitive) into all its
//! conjugated variants. This is a simplified version that handles the most
//! common English verbs using hardcoded conjugation tables.
//!
//! # References
//!
//! - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w` —
//!   the `verb_conjugation` and `verb_tabulation` types.

use crate::linguistic_constants::*;
use crate::word_assemblage::WordAssemblage;

// ---------------------------------------------------------------------------
// Dimension constants
// ---------------------------------------------------------------------------

/// Number of known voices (active, passive).
pub const NO_KNOWN_VOICES: usize = 2;

/// Number of known tenses (is, was, has, had, will, would).
pub const NO_KNOWN_TENSES: usize = 6;

/// Number of known senses (positive, negative).
pub const NO_KNOWN_SENSES: usize = 2;

/// Number of known persons (first, second, third).
pub const NO_KNOWN_PERSONS: usize = 3;

/// Number of known numbers (singular, plural).
pub const NO_KNOWN_NUMBERS: usize = 2;

// ---------------------------------------------------------------------------
// VerbTabulation
// ---------------------------------------------------------------------------

/// A tabulation of verb forms for one voice (active or passive).
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
#[derive(Clone, Debug)]
pub struct VerbTabulation {
    /// The "to be" auxiliary used in this tabulation.
    pub to_be_auxiliary: WordAssemblage,
    /// The conjugated verb text for each [tense][sense][person][number].
    pub vc_text: [[[[WordAssemblage; NO_KNOWN_NUMBERS]; NO_KNOWN_PERSONS]; NO_KNOWN_SENSES]; NO_KNOWN_TENSES],
}

impl VerbTabulation {
    /// Create a new empty verb tabulation.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn new() -> Self {
        VerbTabulation {
            to_be_auxiliary: WordAssemblage::lit_0(),
            vc_text: Default::default(),
        }
    }
}

impl Default for VerbTabulation {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// VerbConjugation
// ---------------------------------------------------------------------------

/// A conjugation of a verb — all its forms across tenses, persons, and numbers.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
#[derive(Clone, Debug)]
pub struct VerbConjugation {
    /// The infinitive form (e.g., "be", "have").
    pub infinitive: WordAssemblage,
    /// The past participle (e.g., "been", "had").
    pub past_participle: WordAssemblage,
    /// The present participle (e.g., "being", "having").
    pub present_participle: WordAssemblage,
    /// Tabulations for active and passive voices.
    pub tabulations: [VerbTabulation; 2],
    /// Whether this verb is auxiliary-only (like "be" and "have" as auxiliaries).
    pub auxiliary_only: bool,
}

impl VerbConjugation {
    /// Create a new verb conjugation with the given infinitive.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn new(infinitive: WordAssemblage) -> Self {
        VerbConjugation {
            infinitive,
            past_participle: WordAssemblage::lit_0(),
            present_participle: WordAssemblage::lit_0(),
            tabulations: [VerbTabulation::new(), VerbTabulation::new()],
            auxiliary_only: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Conjugation helper
// ---------------------------------------------------------------------------

/// Helper to set a single cell in a tabulation.
fn set_cell(
    tab: &mut VerbTabulation,
    tense: usize,
    sense: usize,
    person: usize,
    number: usize,
    text: &str,
) {
    tab.vc_text[tense][sense][person][number] = WordAssemblage::lit_1(text);
}

/// Helper to set a cell for both positive and negative senses.
fn set_cell_both_senses(
    tab: &mut VerbTabulation,
    tense: usize,
    person: usize,
    number: usize,
    positive: &str,
    negative: &str,
) {
    set_cell(tab, tense, POSITIVE_SENSE as usize, person, number, positive);
    set_cell(tab, tense, NEGATIVE_SENSE as usize, person, number, negative);
}

/// Helper to set a cell for both senses with the same text.
fn set_cell_same(
    tab: &mut VerbTabulation,
    tense: usize,
    person: usize,
    number: usize,
    text: &str,
) {
    set_cell_both_senses(tab, tense, person, number, text, text);
}

// ---------------------------------------------------------------------------
// Conjugation creation
// ---------------------------------------------------------------------------

/// The conjugation system — provides functions for creating and looking up
/// verb conjugations.
///
/// # References
///
/// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
pub struct Conjugation;

impl Conjugation {
    /// Conjugate a verb given its base text and language.
    ///
    /// For now, this only handles the hardcoded conjugations for "to be" and
    /// "to have". Other verbs use a simplified regular conjugation.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn conjugate(base_text: &WordAssemblage, _language: &str) -> VerbConjugation {
        let infinitive_str = base_text.to_string();

        match infinitive_str.as_str() {
            "be" => Conjugation::conjugate_to_be(),
            "have" => Conjugation::conjugate_to_have(),
            _ => Conjugation::conjugate_regular(base_text),
        }
    }

    /// Conjugate "to be" with hardcoded forms.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    fn conjugate_to_be() -> VerbConjugation {
        let mut conj = VerbConjugation::new(WordAssemblage::lit_1("be"));
        conj.present_participle = WordAssemblage::lit_1("being");
        conj.past_participle = WordAssemblage::lit_1("been");
        conj.auxiliary_only = true;

        // Active voice tabulation
        let active = &mut conj.tabulations[ACTIVE_VOICE as usize];

        // Present tense (IS_TENSE = 0)
        // Positive: am, are, is / are
        // Negative: am not, are not, is not / are not
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "am");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "are");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "is");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "are");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "are");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "are");

        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "am not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "are not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "is not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "are not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "are not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "are not");

        // Past tense (WAS_TENSE = 1)
        // Positive: was, were, was / were
        // Negative: was not, were not, was not / were not
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "was");
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "were");
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "was");
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "were");
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "were");
        set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "were");

        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "was not");
        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "were not");
        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "was not");
        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "were not");
        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "were not");
        set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "were not");

        // For remaining tenses, use "be" with auxiliaries
        // HAS_TENSE: has been / has not been
        // HAD_TENSE: had been / had not been
        // WILL_TENSE: will be / will not be
        // WOULD_TENSE: would be / would not be
        for tense in &[HAS_TENSE, HAD_TENSE, WILL_TENSE, WOULD_TENSE] {
            let t = *tense as usize;
            let (aux_pos, aux_neg) = match *tense {
                HAS_TENSE => ("has", "has not"),
                HAD_TENSE => ("had", "had not"),
                WILL_TENSE => ("will", "will not"),
                WOULD_TENSE => ("would", "would not"),
                _ => unreachable!(),
            };
            for person in 0..NO_KNOWN_PERSONS {
                for number in 0..NO_KNOWN_NUMBERS {
                    let participle = if *tense == WILL_TENSE || *tense == WOULD_TENSE { "be" } else { "been" };
                    set_cell(active, t, POSITIVE_SENSE as usize, person, number, &format!("{} {}", aux_pos, participle));
                    set_cell(active, t, NEGATIVE_SENSE as usize, person, number, &format!("{} {}", aux_neg, participle));
                }
            }
        }

        conj
    }

    /// Conjugate "to have" with hardcoded forms.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    fn conjugate_to_have() -> VerbConjugation {
        let mut conj = VerbConjugation::new(WordAssemblage::lit_1("have"));
        conj.present_participle = WordAssemblage::lit_1("having");
        conj.past_participle = WordAssemblage::lit_1("had");
        conj.auxiliary_only = true;

        // Active voice tabulation
        let active = &mut conj.tabulations[ACTIVE_VOICE as usize];

        // Present tense (IS_TENSE = 0)
        // Positive: have, have, has / have
        // Negative: have not, have not, has not / have not
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "have");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "have");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "has");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "have");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "have");
        set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "have");

        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, SINGULAR_NUMBER as usize, "have not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, SINGULAR_NUMBER as usize, "have not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, SINGULAR_NUMBER as usize, "has not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, FIRST_PERSON as usize, PLURAL_NUMBER as usize, "have not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, SECOND_PERSON as usize, PLURAL_NUMBER as usize, "have not");
        set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, THIRD_PERSON as usize, PLURAL_NUMBER as usize, "have not");

        // Past tense (WAS_TENSE = 1)
        // Positive: had / had
        // Negative: had not / had not
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, person, number, "had");
                set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, person, number, "had not");
            }
        }

        // HAS_TENSE: has had / has not had
        // HAD_TENSE: had had / had not had
        // WILL_TENSE: will have / will not have
        // WOULD_TENSE: would have / would not have
        for tense in &[HAS_TENSE, HAD_TENSE, WILL_TENSE, WOULD_TENSE] {
            let t = *tense as usize;
            let (aux_pos, aux_neg, participle) = match *tense {
                HAS_TENSE => ("has", "has not", "had"),
                HAD_TENSE => ("had", "had not", "had"),
                WILL_TENSE => ("will", "will not", "have"),
                WOULD_TENSE => ("would", "would not", "have"),
                _ => unreachable!(),
            };
            for person in 0..NO_KNOWN_PERSONS {
                for number in 0..NO_KNOWN_NUMBERS {
                    set_cell(active, t, POSITIVE_SENSE as usize, person, number, &format!("{} {}", aux_pos, participle));
                    set_cell(active, t, NEGATIVE_SENSE as usize, person, number, &format!("{} {}", aux_neg, participle));
                }
            }
        }

        conj
    }

    /// Conjugate a regular verb with simplified rules.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    fn conjugate_regular(base_text: &WordAssemblage) -> VerbConjugation {
        let infinitive_str = base_text.to_string();
        let mut conj = VerbConjugation::new(base_text.clone());

        // Present participle: infinitive + "ing"
        conj.present_participle = WordAssemblage::lit_1(&format!("{}ing", infinitive_str));

        // Past participle: infinitive + "ed" (simplified)
        conj.past_participle = WordAssemblage::lit_1(&format!("{}ed", infinitive_str));

        let active = &mut conj.tabulations[ACTIVE_VOICE as usize];

        // Present tense: base form, with -s for third person singular
        let third_sg = if infinitive_str.ends_with('s')
            || infinitive_str.ends_with('x')
            || infinitive_str.ends_with('z')
            || infinitive_str.ends_with("ch")
            || infinitive_str.ends_with("sh")
        {
            format!("{}es", infinitive_str)
        } else if infinitive_str.ends_with('y') && infinitive_str.len() > 2
            && !matches!(infinitive_str.chars().nth(infinitive_str.len() - 2), Some(c) if "aeiou".contains(c))
        {
            format!("{}ies", &infinitive_str[..infinitive_str.len() - 1])
        } else {
            format!("{}s", infinitive_str)
        };

        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                let pos = if person == THIRD_PERSON as usize && number == SINGULAR_NUMBER as usize {
                    &third_sg
                } else {
                    &infinitive_str
                };
                set_cell(active, IS_TENSE as usize, POSITIVE_SENSE as usize, person, number, pos);
                set_cell(active, IS_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("do not {}", infinitive_str));
            }
        }

        // Past tense: infinitive + "ed" (simplified)
        let past = format!("{}ed", infinitive_str);
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                set_cell(active, WAS_TENSE as usize, POSITIVE_SENSE as usize, person, number, &past);
                set_cell(active, WAS_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("did not {}", infinitive_str));
            }
        }

        // HAS_TENSE: has/have + past participle
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                let aux = if person == THIRD_PERSON as usize && number == SINGULAR_NUMBER as usize {
                    "has"
                } else {
                    "have"
                };
                set_cell(active, HAS_TENSE as usize, POSITIVE_SENSE as usize, person, number, &format!("{} {}", aux, conj.past_participle));
                set_cell(active, HAS_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("{} not {}", aux, conj.past_participle));
            }
        }

        // HAD_TENSE: had + past participle
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                set_cell(active, HAD_TENSE as usize, POSITIVE_SENSE as usize, person, number, &format!("had {}", conj.past_participle));
                set_cell(active, HAD_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("had not {}", conj.past_participle));
            }
        }

        // WILL_TENSE: will + infinitive
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                set_cell(active, WILL_TENSE as usize, POSITIVE_SENSE as usize, person, number, &format!("will {}", infinitive_str));
                set_cell(active, WILL_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("will not {}", infinitive_str));
            }
        }

        // WOULD_TENSE: would + infinitive
        for person in 0..NO_KNOWN_PERSONS {
            for number in 0..NO_KNOWN_NUMBERS {
                set_cell(active, WOULD_TENSE as usize, POSITIVE_SENSE as usize, person, number, &format!("would {}", infinitive_str));
                set_cell(active, WOULD_TENSE as usize, NEGATIVE_SENSE as usize, person, number, &format!("would not {}", infinitive_str));
            }
        }

        conj
    }

    /// Find a conjugation by its infinitive text.
    ///
    /// This is a simplified lookup that creates a new conjugation on demand.
    /// In the full system, this would look up a pre-registered conjugation.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn find_by_infinitive(assemblage: &WordAssemblage) -> VerbConjugation {
        Conjugation::conjugate(assemblage, "English")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjugate_to_be_infinitive() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        assert_eq!(conj.infinitive.to_string(), "be");
        assert_eq!(conj.present_participle.to_string(), "being");
        assert_eq!(conj.past_participle.to_string(), "been");
        assert!(conj.auxiliary_only);
    }

    #[test]
    fn test_conjugate_to_be_present_third_singular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(cell.to_string(), "is");
    }

    #[test]
    fn test_conjugate_to_be_present_third_plural() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][PLURAL_NUMBER as usize];
        assert_eq!(cell.to_string(), "are");
    }

    #[test]
    fn test_conjugate_to_be_past_third_singular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[WAS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(cell.to_string(), "was");
    }

    #[test]
    fn test_conjugate_to_be_past_third_plural() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[WAS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][PLURAL_NUMBER as usize];
        assert_eq!(cell.to_string(), "were");
    }

    #[test]
    fn test_conjugate_to_have_infinitive() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("have"), "English");
        assert_eq!(conj.infinitive.to_string(), "have");
        assert_eq!(conj.present_participle.to_string(), "having");
        assert_eq!(conj.past_participle.to_string(), "had");
    }

    #[test]
    fn test_conjugate_to_have_present_third_singular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("have"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(cell.to_string(), "has");
    }

    #[test]
    fn test_conjugate_regular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("walk"), "English");
        assert_eq!(conj.infinitive.to_string(), "walk");
        assert_eq!(conj.present_participle.to_string(), "walking");
        assert_eq!(conj.past_participle.to_string(), "walked");

        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let present_third_sg = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][THIRD_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(present_third_sg.to_string(), "walks");

        let present_first_sg = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][FIRST_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(present_first_sg.to_string(), "walk");
    }

    #[test]
    fn test_find_by_infinitive() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::find_by_infinitive(&WordAssemblage::lit_1("be"));
        assert_eq!(conj.infinitive.to_string(), "be");
    }

    #[test]
    fn test_conjugate_to_be_present_first_singular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[IS_TENSE as usize][POSITIVE_SENSE as usize][FIRST_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(cell.to_string(), "am");
    }

    #[test]
    fn test_conjugate_to_be_negative_present_third_singular() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let active = &conj.tabulations[ACTIVE_VOICE as usize];
        let cell = &active.vc_text[IS_TENSE as usize][NEGATIVE_SENSE as usize][THIRD_PERSON as usize][SINGULAR_NUMBER as usize];
        assert_eq!(cell.to_string(), "is not");
    }
}
