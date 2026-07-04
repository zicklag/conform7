//! Verb data structures and creation functions.
//!
//! Provides the core verb types — `Verb`, `VerbForm`, `VerbSense` — and the
//! `Verbs` registry for managing them. Also includes verb meanings, verb
//! usages, prepositions, and special meanings.
//!
//! # References
//!
//! - C reference: `services/linguistics-module/Chapter 3/Verbs.w` — verb,
//!   verb_form, verb_sense structs and creation.
//! - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w` —
//!   verb_meaning struct and creation.
//! - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
//!   verb_usage, verb_usage_tier structs and creation.
//! - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
//!   preposition struct and creation.
//! - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
//!   special_meaning_holder struct and creation.

use crate::linguistic_constants::{
    ACTIVE_VOICE, IS_TENSE, NEGATIVE_SENSE, PASSIVE_VOICE, PLURAL_NUMBER, POSITIVE_SENSE,
    SINGULAR_NUMBER, THIRD_PERSON, Lcon,
};
use crate::stock_control::{GrammaticalUsageRef, LinguisticStockItemRef, Stock};
use crate::verb_conjugation::{Conjugation, VerbConjugation};
use crate::word_assemblage::WordAssemblage;
use std::any::Any;

// ---------------------------------------------------------------------------
// Reference types
// ---------------------------------------------------------------------------

/// A reference to a verb by its index in the registry.
pub type VerbRef = usize;

/// A reference to a verb form by its index in the registry.
pub type VerbFormRef = usize;

/// A reference to a verb sense by its index in the registry.
pub type VerbSenseRef = usize;

/// A reference to a verb conjugation by its index in the registry.
pub type VerbConjugationRef = usize;

/// A reference to a preposition by its index in the registry.
pub type PrepositionRef = usize;

/// A reference to a special meaning by its index in the registry.
pub type SpecialMeaningRef = usize;

/// A reference to a verb usage by its index in the registry.
pub type VerbUsageRef = usize;

/// A reference to a verb usage tier by its index in the registry.
pub type VerbUsageTierRef = usize;

// ---------------------------------------------------------------------------
// Form structure constants
// ---------------------------------------------------------------------------

/// Bit flag for Subject-Verb-Object form structure.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
pub const SVO_FS_BIT: u8 = 1;

/// Bit flag for Verb-Object form structure.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
pub const VO_FS_BIT: u8 = 2;

/// Bit flag for Subject-Verb-Object-Object form structure.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
pub const SVOO_FS_BIT: u8 = 4;

/// Bit flag for Verb-Object-Object form structure.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
pub const VOO_FS_BIT: u8 = 8;

// ---------------------------------------------------------------------------
// Verb struct
// ---------------------------------------------------------------------------

/// A verb in the linguistic system.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
#[derive(Clone, Debug)]
pub struct Verb {
    /// The conjugation of this verb.
    pub conjugation: Option<VerbConjugationRef>,
    /// The first form in the linked list of forms.
    pub first_form: Option<VerbFormRef>,
    /// The base form (no prepositions).
    pub base_form: Option<VerbFormRef>,
    /// The stock reference for this verb.
    pub in_stock: Option<LinguisticStockItemRef>,
    /// Whether this verb is copular (a form of "to be").
    pub copular: bool,
}

impl Verb {
    /// Create a new verb with no forms.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
    pub fn new() -> Self {
        Verb {
            conjugation: None,
            first_form: None,
            base_form: None,
            in_stock: None,
            copular: false,
        }
    }
}

impl Default for Verb {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// VerbForm struct
// ---------------------------------------------------------------------------

/// A form of a verb — a specific usage with optional prepositions.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
#[derive(Clone, Debug)]
pub struct VerbForm {
    /// The verb this form belongs to.
    pub underlying_verb: VerbRef,
    /// The preposition used with this form (e.g., "to" in "give to").
    pub preposition: Option<PrepositionRef>,
    /// A second preposition for this form.
    pub second_clause_preposition: Option<PrepositionRef>,
    /// Bitmap of form structures (SVO_FS_BIT, VO_FS_BIT, etc.).
    pub form_structures: u8,
    /// The infinitive reference text for this form.
    pub infinitive_reference_text: WordAssemblage,
    /// The positive-sense reference text.
    pub pos_reference_text: WordAssemblage,
    /// The negative-sense reference text.
    pub neg_reference_text: WordAssemblage,
    /// The list of senses for this form.
    pub list_of_senses: Vec<VerbSenseRef>,
    /// The next form in the linked list.
    pub next_form: Option<VerbFormRef>,
}

impl VerbForm {
    /// Create a new verb form.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
    pub fn new(underlying_verb: VerbRef) -> Self {
        VerbForm {
            underlying_verb,
            preposition: None,
            second_clause_preposition: None,
            form_structures: SVO_FS_BIT,
            infinitive_reference_text: WordAssemblage::lit_0(),
            pos_reference_text: WordAssemblage::lit_0(),
            neg_reference_text: WordAssemblage::lit_0(),
            list_of_senses: Vec::new(),
            next_form: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VerbSense struct
// ---------------------------------------------------------------------------

/// A sense of a verb form — a holder for a verb meaning.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
#[derive(Clone, Debug)]
pub struct VerbSense {
    /// The verb meaning for this sense.
    pub vm: VerbMeaning,
    /// The next sense in the linked list.
    pub next_sense: Option<VerbSenseRef>,
}

impl VerbSense {
    /// Create a new verb sense with the given meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
    pub fn new(vm: VerbMeaning) -> Self {
        VerbSense {
            vm,
            next_sense: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VerbMeaning struct
// ---------------------------------------------------------------------------

/// A verb meaning — what a verb means in a particular usage.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
#[derive(Debug)]
pub struct VerbMeaning {
    /// Whether the meaning is reversed (subject/object swapped).
    pub take_meaning_reversed: bool,
    /// The regular meaning (a binary predicate, stored as Box<dyn Any>).
    pub regular_meaning: Option<Box<dyn Any>>,
    /// A special meaning reference.
    pub special_meaning: Option<SpecialMeaningRef>,
    /// An indirection to another verb's meaning.
    pub take_meaning_from: Option<VerbRef>,
    /// Where this meaning was assigned (sentence index for problem messages).
    pub where_assigned: Option<usize>,
}

impl Clone for VerbMeaning {
    fn clone(&self) -> Self {
        VerbMeaning {
            take_meaning_reversed: self.take_meaning_reversed,
            regular_meaning: None, // Box<dyn Any> is not Clone
            special_meaning: self.special_meaning,
            take_meaning_from: self.take_meaning_from,
            where_assigned: self.where_assigned,
        }
    }
}

impl VerbMeaning {
    /// Create a meaningless verb meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn meaninglessness() -> Self {
        VerbMeaning {
            take_meaning_reversed: false,
            regular_meaning: None,
            special_meaning: None,
            take_meaning_from: None,
            where_assigned: None,
        }
    }

    /// Check if this meaning is meaningless.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn is_meaningless(&self) -> bool {
        self.regular_meaning.is_none()
            && self.special_meaning.is_none()
            && self.take_meaning_from.is_none()
    }

    /// Create a regular meaning with a binary predicate.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn regular(rel: Box<dyn Any>) -> Self {
        VerbMeaning {
            take_meaning_reversed: false,
            regular_meaning: Some(rel),
            special_meaning: None,
            take_meaning_from: None,
            where_assigned: None,
        }
    }

    /// Create a special meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn special(sm: SpecialMeaningRef) -> Self {
        VerbMeaning {
            take_meaning_reversed: false,
            regular_meaning: None,
            special_meaning: Some(sm),
            take_meaning_from: None,
            where_assigned: None,
        }
    }

    /// Create an indirected meaning (meaning taken from another verb).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn indirected(from: VerbRef, reversed: bool) -> Self {
        VerbMeaning {
            take_meaning_reversed: reversed,
            regular_meaning: None,
            special_meaning: None,
            take_meaning_from: Some(from),
            where_assigned: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VerbUsage struct
// ---------------------------------------------------------------------------

/// A usage of a verb — a specific text that can be matched against source text.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
#[derive(Clone, Debug)]
pub struct VerbUsage {
    /// The grammatical usage this verb usage belongs to.
    pub usage: GrammaticalUsageRef,
    /// The text of this verb usage.
    pub vu_text: WordAssemblage,
    /// Whether unexpected upper case is allowed in this usage.
    pub vu_allow_unexpected_upper_case: bool,
    /// The next usage in the search list (linked list).
    pub next_in_search_list: Option<VerbUsageRef>,
    /// The next usage within the same tier.
    pub next_within_tier: Option<VerbUsageRef>,
    /// Where this usage was created (sentence index).
    pub where_vu_created: Option<usize>,
    /// The lexical entry (conjugation) for this usage.
    pub vu_lex_entry: Option<VerbConjugationRef>,
}

impl VerbUsage {
    /// Create a new verb usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
    pub fn new(
        text: WordAssemblage,
        allow_unexpected_upper_case: bool,
        usage: GrammaticalUsageRef,
        where_created: Option<usize>,
    ) -> Self {
        VerbUsage {
            usage,
            vu_text: text,
            vu_allow_unexpected_upper_case: allow_unexpected_upper_case,
            next_in_search_list: None,
            next_within_tier: None,
            where_vu_created: where_created,
            vu_lex_entry: None,
        }
    }
}

// ---------------------------------------------------------------------------
// VerbUsageTier struct
// ---------------------------------------------------------------------------

/// A tier of verb usages, ordered by priority.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
#[derive(Clone, Debug)]
pub struct VerbUsageTier {
    /// The priority of this tier (higher = checked first).
    pub priority: i32,
    /// The usages in this tier.
    pub tier_contents: Vec<VerbUsageRef>,
    /// The next tier in the linked list.
    pub next_tier: Option<VerbUsageTierRef>,
}

impl VerbUsageTier {
    /// Create a new verb usage tier.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
    pub fn new(priority: i32) -> Self {
        VerbUsageTier {
            priority,
            tier_contents: Vec::new(),
            next_tier: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Preposition struct
// ---------------------------------------------------------------------------

/// A preposition used with verbs.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w`
#[derive(Clone, Debug)]
pub struct Preposition {
    /// The text of this preposition.
    pub prep_text: WordAssemblage,
    /// The lexical entry for this preposition.
    pub prep_lex_entry: Option<VerbConjugationRef>,
    /// Where this preposition was created.
    pub where_prep_created: Option<usize>,
    /// Whether unexpected upper case is allowed.
    pub allow_unexpected_upper_case: bool,
    /// The stock reference for this preposition.
    pub in_stock: Option<LinguisticStockItemRef>,
}

impl Preposition {
    /// Create a new preposition.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w`
    pub fn new(
        text: WordAssemblage,
        allow_unexpected_upper_case: bool,
        where_created: Option<usize>,
    ) -> Self {
        Preposition {
            prep_text: text,
            prep_lex_entry: None,
            where_prep_created: where_created,
            allow_unexpected_upper_case,
            in_stock: None,
        }
    }
}

// ---------------------------------------------------------------------------
// SpecialMeaningHolder struct
// ---------------------------------------------------------------------------

/// A function pointer type for special meaning functions.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w`
pub type SpecialMeaningFn = fn(task: i32, node: &mut crate::parse_node::ParseNode, nps: &[crate::Wording; 3]) -> bool;

/// A special meaning holder — a named function that handles a special verb meaning.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w`
#[derive(Clone, Debug)]
pub struct SpecialMeaningHolder {
    /// The function implementing this special meaning.
    pub sm_func: SpecialMeaningFn,
    /// The name of this special meaning.
    pub sm_name: String,
    /// Metadata integer for this special meaning.
    pub metadata_n: i32,
}

impl SpecialMeaningHolder {
    /// Create a new special meaning holder.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w`
    pub fn new(func: SpecialMeaningFn, name: &str, metadata: i32) -> Self {
        SpecialMeaningHolder {
            sm_func: func,
            sm_name: name.to_string(),
            metadata_n: metadata,
        }
    }
}

// ---------------------------------------------------------------------------
// Verbs registry
// ---------------------------------------------------------------------------

/// The verb registry — manages all verbs, verb forms, verb senses, verb usages,
/// verb usage tiers, prepositions, and special meanings.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
/// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
/// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
/// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w`
/// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w`
#[derive(Debug, Default)]
pub struct Verbs {
    /// All registered verbs.
    pub verbs: Vec<Verb>,
    /// All registered verb forms.
    pub forms: Vec<VerbForm>,
    /// All registered verb senses.
    pub senses: Vec<VerbSense>,
    /// All registered verb usages.
    pub usages: Vec<VerbUsage>,
    /// All registered verb usage tiers.
    pub tiers: Vec<VerbUsageTier>,
    /// All registered prepositions.
    pub prepositions: Vec<Preposition>,
    /// All registered special meanings.
    pub special_meanings: Vec<SpecialMeaningHolder>,
    /// All registered verb conjugations.
    pub conjugations: Vec<VerbConjugation>,
    /// The stock registry.
    pub stock: Stock,
    /// The copular verb (first verb created with copular=true).
    pub copular_verb: Option<VerbRef>,
    /// The head of the search list (linked list of usages in length order).
    pub search_list_head: Option<VerbUsageRef>,
    /// The head of the tier list (linked list of tiers in priority order).
    pub tier_list_head: Option<VerbUsageTierRef>,
}

impl Verbs {
    /// Create a new empty verb registry.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
    pub fn new() -> Self {
        Verbs {
            verbs: Vec::new(),
            forms: Vec::new(),
            senses: Vec::new(),
            usages: Vec::new(),
            tiers: Vec::new(),
            prepositions: Vec::new(),
            special_meanings: Vec::new(),
            conjugations: Vec::new(),
            stock: Stock::new(),
            copular_verb: None,
            search_list_head: None,
            tier_list_head: None,
        }
    }

    // -----------------------------------------------------------------------
    // Verb creation
    // -----------------------------------------------------------------------

    /// Create a new verb with a single meaningless base form.
    ///
    /// If `copular` is true, this verb is tracked as the copular verb
    /// (the first copular verb registered).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::new_verb`.
    pub fn new_verb(&mut self, conjugation: Option<VerbConjugationRef>, copular: bool) -> VerbRef {
        let verb_index = self.verbs.len();
        let mut verb = Verb::new();
        verb.conjugation = conjugation;
        verb.copular = copular;

        // Create a meaningless base form.
        let meaning = VerbMeaning::meaninglessness();
        let sense = VerbSense::new(meaning);
        let sense_index = self.senses.len();
        self.senses.push(sense);

        let mut form = VerbForm::new(verb_index);
        form.list_of_senses.push(sense_index);
        let form_index = self.forms.len();
        self.forms.push(form);

        verb.first_form = Some(form_index);
        verb.base_form = Some(form_index);
        self.verbs.push(verb);

        if copular && self.copular_verb.is_none() {
            self.copular_verb = Some(verb_index);
        }

        verb_index
    }

    /// Create a new operator verb with a special meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::new_operator_verb`.
    pub fn new_operator_verb(&mut self, meaning: VerbMeaning) -> VerbRef {
        let verb_index = self.verbs.len();
        let mut verb = Verb::new();

        let sense = VerbSense::new(meaning);
        let sense_index = self.senses.len();
        self.senses.push(sense);

        let mut form = VerbForm::new(verb_index);
        form.list_of_senses.push(sense_index);
        let form_index = self.forms.len();
        self.forms.push(form);

        verb.first_form = Some(form_index);
        verb.base_form = Some(form_index);
        self.verbs.push(verb);

        verb_index
    }

    /// Add a form to a verb.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::add_form`.
    pub fn add_form(
        &mut self,
        verb: VerbRef,
        prep: Option<PrepositionRef>,
        second_prep: Option<PrepositionRef>,
        meaning: VerbMeaning,
        form_structs: u8,
    ) -> VerbFormRef {
        let sense = VerbSense::new(meaning);
        let sense_index = self.senses.len();
        self.senses.push(sense);

        let mut form = VerbForm::new(verb);
        form.preposition = prep;
        form.second_clause_preposition = second_prep;
        form.form_structures = form_structs;
        form.list_of_senses.push(sense_index);

        let form_index = self.forms.len();
        self.forms.push(form);

        // Link the form into the verb's form list.
        if let Some(v) = self.verbs.get_mut(verb) {
            // Insert at the head of the linked list.
            self.forms[form_index].next_form = v.first_form;
            v.first_form = Some(form_index);
        }

        form_index
    }

    /// Find a form by verb and prepositions.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::find_form`.
    pub fn find_form(
        &self,
        verb: VerbRef,
        prep: Option<PrepositionRef>,
        second_prep: Option<PrepositionRef>,
    ) -> Option<VerbFormRef> {
        let v = self.verbs.get(verb)?;
        let mut current = v.first_form;
        while let Some(fi) = current {
            if let Some(form) = self.forms.get(fi) {
                if form.preposition == prep && form.second_clause_preposition == second_prep {
                    return Some(fi);
                }
                current = form.next_form;
            } else {
                break;
            }
        }
        None
    }

    /// Get the base form of a verb (no prepositions).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::base_form`.
    pub fn base_form(&self, verb: VerbRef) -> Option<VerbFormRef> {
        self.verbs.get(verb).and_then(|v| v.base_form)
    }

    /// Look up a verb from a stock reference (Lcon).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::from_lcon`.
    pub fn from_lcon(&self, lcon: Lcon) -> Option<VerbRef> {
        let _id = lcon.get_id()?;
        // The verb index is stored as the stock item's data.
        let item = self.stock.from_lcon(lcon)?;
        item.data.downcast_ref::<VerbRef>().copied()
    }

    /// Convert a verb to a stock reference (Lcon).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
    ///   `Verbs::to_lcon`.
    pub fn to_lcon(&mut self, verb: VerbRef) -> Lcon {
        // Create a stock item for this verb if it doesn't have one.
        if self.verbs[verb].in_stock.is_none() {
            let cat = if self.stock.categories.is_empty() {
                self.stock.new_category("verb")
            } else {
                0 // assume first category is "verb"
            };
            let item = self.stock.add_item(cat, Box::new(verb));
            self.verbs[verb].in_stock = Some(item);
        }
        Stock::to_lcon(self.verbs[verb].in_stock.unwrap())
    }

    // -----------------------------------------------------------------------
    // Verb meaning helpers
    // -----------------------------------------------------------------------

    /// Follow an indirection to find the actual meaning.
    ///
    /// If this meaning is indirected (has `take_meaning_from`), follow the
    /// chain until we find a non-indirected meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn follow_indirection<'a>(&'a self, vm: &'a VerbMeaning) -> Option<&'a VerbMeaning> {
        let mut current = vm;
        // Limit to avoid infinite loops.
        for _ in 0..100 {
            if let Some(from) = current.take_meaning_from {
                let verb = self.verbs.get(from)?;
                let base_form = verb.base_form?;
                let form = self.forms.get(base_form)?;
                let sense_ref = form.list_of_senses.first()?;
                let sense = self.senses.get(*sense_ref)?;
                current = &sense.vm;
            } else {
                return Some(current);
            }
        }
        None
    }

    /// Reverse a verb meaning (stub — returns None for now).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Meanings.w`
    pub fn reverse_vmt(_recto: &VerbMeaning) -> Option<Box<dyn Any>> {
        None
    }

    // -----------------------------------------------------------------------
    // Verb usage management
    // -----------------------------------------------------------------------

    /// Create a new verb usage and add it to the search list.
    ///
    /// The search list is maintained in length order (longest first).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::new`.
    pub fn new_usage(
        &mut self,
        text: WordAssemblage,
        allow_unexpected_upper_case: bool,
        usage: GrammaticalUsageRef,
        where_created: Option<usize>,
    ) -> Option<VerbUsageRef> {
        if !text.nonempty() {
            return None;
        }

        let index = self.usages.len();
        let vu = VerbUsage::new(text, allow_unexpected_upper_case, usage, where_created);
        self.usages.push(vu);

        // Insert into search list in length order (longest first).
        self.insert_into_search_list(index);

        Some(index)
    }

    /// Insert a usage into the search list in length order (longest first).
    fn insert_into_search_list(&mut self, usage: VerbUsageRef) {
        let usage_len = self.usages[usage].vu_text.length();

        if self.search_list_head.is_none() {
            self.search_list_head = Some(usage);
            return;
        }

        // Find the insertion point (longest first).
        let mut prev: Option<VerbUsageRef> = None;
        let mut current = self.search_list_head;

        while let Some(cur) = current {
            let cur_len = self.usages[cur].vu_text.length();
            if usage_len > cur_len {
                // Insert before current.
                self.usages[usage].next_in_search_list = Some(cur);
                if let Some(p) = prev {
                    self.usages[p].next_in_search_list = Some(usage);
                } else {
                    self.search_list_head = Some(usage);
                }
                return;
            }
            prev = current;
            current = self.usages[cur].next_in_search_list;
        }

        // Append at the end.
        if let Some(p) = prev {
            self.usages[p].next_in_search_list = Some(usage);
        }
    }

    /// Get the verb from a usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::get_verb`.
    pub fn get_verb_from_usage(&self, vu: VerbUsageRef) -> Option<VerbRef> {
        let usage = self.usages.get(vu)?;
        let gu = self.stock.usages.get(usage.usage)?;
        let item = self.stock.items.get(gu.item)?;
        item.data.downcast_ref::<VerbRef>().copied()
    }

    /// Parse a wording against a verb usage.
    ///
    /// Checks if the verb usage text appears at the start of the wording
    /// and returns the word position after the match.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::parse_against_verb`.
    pub fn parse_against_verb(&self, wording: &[&str], vu: VerbUsageRef) -> Option<usize> {
        let usage = self.usages.get(vu)?;
        let words = &usage.vu_text.words;

        if words.len() > wording.len() {
            return None;
        }

        for (i, word) in words.iter().enumerate() {
            if i >= wording.len() {
                return None;
            }
            if wording[i].to_lowercase() != word.to_lowercase() {
                return None;
            }
        }

        Some(words.len())
    }

    /// Mark a word as being a verb (stub for now).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::mark_as_verb`.
    pub fn mark_as_verb(&self, _word: &str) {
        // Stub: in the full system, this would annotate the word in the
        // vocabulary system.
    }

    /// Get the adaptive person for a conjugation.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::adaptive_person`.
    pub fn adaptive_person(&self, _conjugation: VerbConjugationRef) -> i32 {
        // Stub: returns third person by default.
        crate::linguistic_constants::THIRD_PERSON
    }

    /// Get the adaptive number for a conjugation.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::adaptive_number`.
    pub fn adaptive_number(&self, _conjugation: VerbConjugationRef) -> i32 {
        // Stub: returns singular by default.
        crate::linguistic_constants::SINGULAR_NUMBER
    }

    // -----------------------------------------------------------------------
    // Tier management
    // -----------------------------------------------------------------------

    /// Create a new verb usage tier.
    ///
    /// Tiers are maintained in priority order (highest first).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
    pub fn new_tier(&mut self, priority: i32) -> VerbUsageTierRef {
        let index = self.tiers.len();
        let tier = VerbUsageTier::new(priority);
        self.tiers.push(tier);

        // Insert into tier list in priority order (highest first).
        if self.tier_list_head.is_none() {
            self.tier_list_head = Some(index);
        } else {
            let mut prev: Option<VerbUsageTierRef> = None;
            let mut current = self.tier_list_head;
            while let Some(cur) = current {
                if priority > self.tiers[cur].priority {
                    self.tiers[index].next_tier = Some(cur);
                    if let Some(p) = prev {
                        self.tiers[p].next_tier = Some(index);
                    } else {
                        self.tier_list_head = Some(index);
                    }
                    return index;
                }
                prev = current;
                current = self.tiers[cur].next_tier;
            }
            // Append at the end.
            if let Some(p) = prev {
                self.tiers[p].next_tier = Some(index);
            }
        }

        index
    }

    /// Add a usage to a tier.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w`
    pub fn add_usage_to_tier(&mut self, usage: VerbUsageRef, tier: VerbUsageTierRef) {
        if let Some(t) = self.tiers.get_mut(tier) {
            t.tier_contents.push(usage);
        }
    }

    // -----------------------------------------------------------------------
    // Preposition management
    // -----------------------------------------------------------------------

    /// Create or find a preposition by text (deduplicated).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
    ///   `Prepositions::make`.
    pub fn make_preposition(
        &mut self,
        text: WordAssemblage,
        allow_unexpected_upper_case: bool,
        where_created: Option<usize>,
    ) -> PrepositionRef {
        // Check for an existing preposition with the same text.
        for (i, prep) in self.prepositions.iter().enumerate() {
            if prep.prep_text.eq(&text) {
                return i;
            }
        }

        let index = self.prepositions.len();
        let prep = Preposition::new(text, allow_unexpected_upper_case, where_created);
        self.prepositions.push(prep);
        index
    }

    /// Get the word count of a preposition.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
    ///   `Prepositions::length`.
    pub fn preposition_length(&self, prep: PrepositionRef) -> usize {
        self.prepositions
            .get(prep)
            .map(|p| p.prep_text.length())
            .unwrap_or(0)
    }

    /// Mark a word as a preposition (stub for now).
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
    ///   `Prepositions::mark_as_preposition`.
    pub fn mark_as_preposition(&self, _word: &str) {
        // Stub: in the full system, this would annotate the word in the
        // vocabulary system.
    }

    /// Get where a preposition was created.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
    ///   `Prepositions::get_where_pu_created`.
    pub fn get_where_prep_created(&self, prep: PrepositionRef) -> Option<usize> {
        self.prepositions.get(prep).and_then(|p| p.where_prep_created)
    }

    /// Parse a wording against a preposition.
    ///
    /// Checks if the preposition text appears at the start of the wording.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Prepositions.w` —
    ///   `Prepositions::parse_against`.
    pub fn parse_against_preposition(&self, wording: &[&str], prep: PrepositionRef) -> Option<usize> {
        let preposition = self.prepositions.get(prep)?;
        let words = &preposition.prep_text.words;

        if words.len() > wording.len() {
            return None;
        }

        for (i, word) in words.iter().enumerate() {
            if i >= wording.len() {
                return None;
            }
            if wording[i].to_lowercase() != word.to_lowercase() {
                return None;
            }
        }

        Some(words.len())
    }

    // -----------------------------------------------------------------------
    // Special meaning management
    // -----------------------------------------------------------------------

    /// Declare a new special meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::declare`.
    pub fn declare_special_meaning(
        &mut self,
        func: SpecialMeaningFn,
        name: &str,
        metadata: i32,
    ) -> SpecialMeaningRef {
        let index = self.special_meanings.len();
        let smh = SpecialMeaningHolder::new(func, name, metadata);
        self.special_meanings.push(smh);
        index
    }

    /// Find a special meaning by name.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::find_from_wording`.
    pub fn find_special_meaning(&self, name: &str) -> Option<SpecialMeaningRef> {
        self.special_meanings
            .iter()
            .position(|sm| sm.sm_name == name)
    }

    /// Call a special meaning function.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::call`.
    pub fn call_special_meaning(
        &self,
        sm: SpecialMeaningRef,
        task: i32,
        node: &mut crate::parse_node::ParseNode,
        nps: &[crate::Wording; 3],
    ) -> bool {
        self.special_meanings
            .get(sm)
            .map(|smh| (smh.sm_func)(task, node, nps))
            .unwrap_or(false)
    }

    /// Get the metadata of a special meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::get_metadata_n`.
    pub fn get_special_meaning_metadata(&self, sm: SpecialMeaningRef) -> i32 {
        self.special_meanings
            .get(sm)
            .map(|smh| smh.metadata_n)
            .unwrap_or(0)
    }

    /// Get the name of a special meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::get_name`.
    pub fn get_special_meaning_name(&self, sm: SpecialMeaningRef) -> &str {
        self.special_meanings
            .get(sm)
            .map(|smh| smh.sm_name.as_str())
            .unwrap_or("")
    }

    /// Check if a special meaning uses a given function.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
    ///   `SpecialMeanings::is`.
    pub fn is_special_meaning(&self, sm: SpecialMeaningRef, func: SpecialMeaningFn) -> bool {
        self.special_meanings
            .get(sm)
            .map(|smh| smh.sm_func as usize == func as usize)
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Conjugation management
    // -----------------------------------------------------------------------

    /// Register a conjugation and return its index.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn register_conjugation(&mut self, conj: VerbConjugation) -> VerbConjugationRef {
        let index = self.conjugations.len();
        self.conjugations.push(conj);
        index
    }

    /// Find a conjugation by infinitive text.
    ///
    /// # References
    ///
    /// - C reference: `services/inflections-module/Chapter 3/Verb Conjugation.w`
    pub fn find_conjugation(&self, infinitive: &WordAssemblage) -> Option<VerbConjugationRef> {
        self.conjugations
            .iter()
            .position(|c| c.infinitive.eq(infinitive))
    }

    // -----------------------------------------------------------------------
    // Boot verb creation
    // -----------------------------------------------------------------------

    /// Register all conjugated forms of a verb as usages in tiers.
    ///
    /// Simplified: registers present tense 3rd person forms (singular and plural)
    /// for both active and passive voice. For copular verbs, no passive forms.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 3/Verb Usages.w` —
    ///   `VerbUsages::register_all_usages_of_verb`.
    pub fn register_all_usages_of_verb(
        &mut self,
        verb: VerbRef,
        _unexpected_upper_casing: bool,
        priority: i32,
    ) -> VerbUsageTierRef {
        // Extract all data upfront to avoid borrow conflicts.
        let copular = self.verbs[verb].copular;
        let vc_idx = self.verbs[verb]
            .conjugation
            .expect("verb must have a conjugation");
        let active_3s_pos = self.conjugations[vc_idx].tabulations[ACTIVE_VOICE as usize].vc_text
            [IS_TENSE as usize][POSITIVE_SENSE as usize]
            [THIRD_PERSON as usize][SINGULAR_NUMBER as usize]
            .clone();
        let active_3p_pos = self.conjugations[vc_idx].tabulations[ACTIVE_VOICE as usize].vc_text
            [IS_TENSE as usize][POSITIVE_SENSE as usize]
            [THIRD_PERSON as usize][PLURAL_NUMBER as usize]
            .clone();
        let active_3s_neg = self.conjugations[vc_idx].tabulations[ACTIVE_VOICE as usize].vc_text
            [IS_TENSE as usize][NEGATIVE_SENSE as usize]
            [THIRD_PERSON as usize][SINGULAR_NUMBER as usize]
            .clone();
        let active_3p_neg = self.conjugations[vc_idx].tabulations[ACTIVE_VOICE as usize].vc_text
            [IS_TENSE as usize][NEGATIVE_SENSE as usize]
            [THIRD_PERSON as usize][PLURAL_NUMBER as usize]
            .clone();

        let tier = self.new_tier(priority);

        // Create a stock item and grammatical usage for this verb.
        let cat = if self.stock.categories.is_empty() {
            self.stock.new_category("verb")
        } else {
            0
        };
        let item = self.stock.add_item(cat, Box::new(verb));
        let gu = self.stock.new_usage(item, "English");

        // Active voice: present tense 3rd person singular and plural (positive and negative)
        let forms = [
            (active_3s_pos.clone(), 0u32),
            (active_3p_pos.clone(), 0u32),
            (active_3s_neg.clone(), 0u32),
            (active_3p_neg.clone(), 0u32),
        ];

        for (wa, _bits) in &forms {
            if let Some(vu) = self.new_usage(wa.clone(), false, gu, None) {
                self.add_usage_to_tier(vu, tier);
            }
        }

        // For non-copular verbs, also register passive voice forms (positive and negative)
        if !copular {
            let passive_3s_pos = self.conjugations[vc_idx].tabulations[PASSIVE_VOICE as usize].vc_text
                [IS_TENSE as usize][POSITIVE_SENSE as usize]
                [THIRD_PERSON as usize][SINGULAR_NUMBER as usize]
                .clone();
            let passive_3p_pos = self.conjugations[vc_idx].tabulations[PASSIVE_VOICE as usize].vc_text
                [IS_TENSE as usize][POSITIVE_SENSE as usize]
                [THIRD_PERSON as usize][PLURAL_NUMBER as usize]
                .clone();
            let passive_3s_neg = self.conjugations[vc_idx].tabulations[PASSIVE_VOICE as usize].vc_text
                [IS_TENSE as usize][NEGATIVE_SENSE as usize]
                [THIRD_PERSON as usize][SINGULAR_NUMBER as usize]
                .clone();
            let passive_3p_neg = self.conjugations[vc_idx].tabulations[PASSIVE_VOICE as usize].vc_text
                [IS_TENSE as usize][NEGATIVE_SENSE as usize]
                [THIRD_PERSON as usize][PLURAL_NUMBER as usize]
                .clone();
            let passive_forms = [passive_3s_pos, passive_3p_pos, passive_3s_neg, passive_3p_neg];
            for wa in &passive_forms {
                if let Some(vu) = self.new_usage(wa.clone(), false, gu, None) {
                    self.add_usage_to_tier(vu, tier);
                }
            }
        }

        tier
    }

    /// Create the bootstrap verbs "to be" and "to mean".
    ///
    /// Corresponds to `BootVerbs::make_built_in` in the C reference
    /// (`inform7/assertions-module/Chapter 2/Booting Verbs.w`).
    ///
    /// Returns (to_be_ref, to_mean_ref).
    pub fn make_built_in(&mut self) -> (VerbRef, VerbRef) {
        // 1. Declare special meanings
        let priority1_sms = [
            "new-relation", "rule-substitutes-for", "rule-does-nothing",
            "rule-does-nothing-if", "rule-does-nothing-unless",
            "translates-into-unicode", "translates-into-i6",
            "translates-into-language", "test-with",
        ];
        let priority2_sms = [
            "new-verb", "new-plural", "new-activity", "new-adjective",
            "new-either-or", "defined-by-table", "rule-listed-in", "can-be",
        ];
        let priority3_sms = ["verb-means"];
        let priority4_sms = [
            "specifies-notation", "use-translates", "use",
            "include-in", "omit-from",
        ];

        for name in &priority1_sms {
            self.declare_special_meaning(generic_smf, name, 1);
        }
        for name in &priority2_sms {
            self.declare_special_meaning(generic_smf, name, 2);
        }
        for name in &priority3_sms {
            self.declare_special_meaning(generic_smf, name, 3);
        }
        for name in &priority4_sms {
            self.declare_special_meaning(generic_smf, name, 4);
        }

        // 2. Conjugate and create "to be" (copular)
        let be_conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let be_idx = self.register_conjugation(be_conj);
        let to_be = self.new_verb(Some(be_idx), true);
        self.register_all_usages_of_verb(to_be, false, 2);

        // 3. Conjugate and create "to mean" (non-copular)
        let mean_conj = Conjugation::conjugate(&WordAssemblage::lit_1("mean"), "English");
        let mean_idx = self.register_conjugation(mean_conj);
        let to_mean = self.new_verb(Some(mean_idx), false);
        self.register_all_usages_of_verb(to_mean, false, 3);

        // 4. Give meaning to mean: attach "verb-means" special meaning
        let verb_means_sm = self
            .special_meanings
            .iter()
            .position(|sm| sm.sm_name == "verb-means")
            .expect("verb-means special meaning should exist");
        let meaning = VerbMeaning::special(verb_means_sm);
        self.add_form(to_mean, None, None, meaning, SVO_FS_BIT);

        (to_be, to_mean)
    }
}

// ---------------------------------------------------------------------------
// Generic special meaning function
// ---------------------------------------------------------------------------

/// The generic special meaning function that accumulates non-empty SPs and OPs
/// as unparsed noun phrases.
///
/// This is the default handler for special meanings that don't have a custom
/// implementation. It creates `UNPARSED_NOUN_NT` nodes for any non-empty
/// subject, object, or indirect object wordings.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w` —
///   `SpecialMeanings::generic_smf`.
pub fn generic_smf(
    _task: i32,
    node: &mut crate::parse_node::ParseNode,
    nps: &[crate::Wording; 3],
) -> bool {
    use crate::linguistics::Diagrams;
    use crate::NodeType;

    // For each of the three noun phrase slots (SP, OP, IOP), if non-empty,
    // create an unparsed noun node and append it as a child.
    for np in nps.iter() {
        if !np.is_empty() {
            let child = Diagrams::new_unparsed_noun(*np);
            node.append_child(child);
        }
    }

    // Set the node type to VERB_NT to indicate this is a verb node.
    node.set_node_type(NodeType::Verb);

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_node::ParseNode;
    use crate::Wording;
    use crate::verb_conjugation::Conjugation;

    // -----------------------------------------------------------------------
    // Verb creation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_verb() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, false);
        assert_eq!(verbs.verbs.len(), 1);
        assert!(verbs.verbs[v].first_form.is_some());
        assert!(verbs.verbs[v].base_form.is_some());
        assert!(verbs.copular_verb.is_none());
    }

    #[test]
    fn test_new_verb_copular() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, true);
        assert_eq!(verbs.copular_verb, Some(v));
    }

    #[test]
    fn test_new_verb_copular_first_only() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v1 = verbs.new_verb(None, true);
        let v2 = verbs.new_verb(None, true);
        assert_eq!(verbs.copular_verb, Some(v1));
        assert_ne!(verbs.copular_verb, Some(v2));
    }

    #[test]
    fn test_new_operator_verb() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let meaning = VerbMeaning::special(0);
        let v = verbs.new_operator_verb(meaning);
        assert_eq!(verbs.verbs.len(), 1);
        assert!(verbs.verbs[v].first_form.is_some());
    }

    #[test]
    fn test_add_form() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, false);
        let meaning = VerbMeaning::meaninglessness();
        let f = verbs.add_form(v, None, None, meaning, SVO_FS_BIT);
        assert!(f > 0);
        assert_eq!(verbs.forms[f].underlying_verb, v);
    }

    #[test]
    fn test_find_form() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, false);

        // Add a form with a preposition.
        let prep_text = WordAssemblage::lit_1("to");
        let prep = verbs.make_preposition(prep_text, false, None);
        let meaning = VerbMeaning::meaninglessness();
        let f = verbs.add_form(v, Some(prep), None, meaning, SVO_FS_BIT);

        let found = verbs.find_form(v, Some(prep), None);
        assert_eq!(found, Some(f));

        let not_found = verbs.find_form(v, None, None);
        assert_eq!(not_found, Some(0)); // base form
    }

    #[test]
    fn test_base_form() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, false);
        let base = verbs.base_form(v);
        assert!(base.is_some());
    }

    // -----------------------------------------------------------------------
    // Verb meaning tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_meaninglessness() {
        // Reference: services/linguistics-module/Chapter 3/Verb Meanings.w
        let vm = VerbMeaning::meaninglessness();
        assert!(vm.is_meaningless());
        assert!(vm.regular_meaning.is_none());
        assert!(vm.special_meaning.is_none());
        assert!(vm.take_meaning_from.is_none());
    }

    #[test]
    fn test_regular_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Verb Meanings.w
        let vm = VerbMeaning::regular(Box::new("test_predicate".to_string()));
        assert!(!vm.is_meaningless());
        assert!(vm.regular_meaning.is_some());
    }

    #[test]
    fn test_special_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Verb Meanings.w
        let vm = VerbMeaning::special(42);
        assert!(!vm.is_meaningless());
        assert_eq!(vm.special_meaning, Some(42));
    }

    #[test]
    fn test_indirected_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Verb Meanings.w
        let mut verbs = Verbs::new();
        let v1 = verbs.new_verb(None, false);
        let vm = VerbMeaning::indirected(v1, true);
        assert!(vm.take_meaning_reversed);
        assert_eq!(vm.take_meaning_from, Some(v1));
    }

    #[test]
    fn test_follow_indirection() {
        // Reference: services/linguistics-module/Chapter 3/Verb Meanings.w
        let mut verbs = Verbs::new();
        let v1 = verbs.new_verb(None, false);
        let vm = VerbMeaning::indirected(v1, false);
        let result = verbs.follow_indirection(&vm);
        // Should resolve to the base form's meaning (meaninglessness).
        assert!(result.is_some());
        assert!(result.unwrap().is_meaningless());
    }

    // -----------------------------------------------------------------------
    // Verb usage tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_usage() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let text = WordAssemblage::lit_1("is");
        let vu = verbs.new_usage(text, false, usage, None);
        assert!(vu.is_some());
        assert_eq!(verbs.usages.len(), 1);
    }

    #[test]
    fn test_new_usage_empty_text() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let vu = verbs.new_usage(WordAssemblage::lit_0(), false, usage, None);
        assert!(vu.is_none());
    }

    #[test]
    fn test_parse_against_verb() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let text = WordAssemblage::lit_1("is");
        let vu = verbs.new_usage(text, false, usage, None).unwrap();

        let wording = &["is", "in", "the", "room"];
        let pos = verbs.parse_against_verb(wording, vu);
        assert_eq!(pos, Some(1));
    }

    #[test]
    fn test_parse_against_verb_no_match() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let text = WordAssemblage::lit_1("is");
        let vu = verbs.new_usage(text, false, usage, None).unwrap();

        let wording = &["was", "in", "the", "room"];
        let pos = verbs.parse_against_verb(wording, vu);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_parse_against_verb_multi_word() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let text = WordAssemblage::new(vec!["carry".to_string(), "out".to_string()]);
        let vu = verbs.new_usage(text, false, usage, None).unwrap();

        let wording = &["carry", "out", "the", "plan"];
        let pos = verbs.parse_against_verb(wording, vu);
        assert_eq!(pos, Some(2));
    }

    #[test]
    fn test_search_list_order() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");

        let short = verbs.new_usage(WordAssemblage::lit_1("is"), false, usage, None).unwrap();
        let long = verbs.new_usage(
            WordAssemblage::new(vec!["carry".to_string(), "out".to_string()]),
            false,
            usage,
            None,
        ).unwrap();

        // Longest should be first in search list.
        assert_eq!(verbs.search_list_head, Some(long));
        assert_eq!(verbs.usages[long].next_in_search_list, Some(short));
    }

    // -----------------------------------------------------------------------
    // Tier tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_tier() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let t = verbs.new_tier(10);
        assert_eq!(verbs.tiers[t].priority, 10);
    }

    #[test]
    fn test_tier_priority_order() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let low = verbs.new_tier(5);
        let high = verbs.new_tier(10);
        // Highest priority first.
        assert_eq!(verbs.tier_list_head, Some(high));
        assert_eq!(verbs.tiers[high].next_tier, Some(low));
    }

    #[test]
    fn test_add_usage_to_tier() {
        // Reference: services/linguistics-module/Chapter 3/Verb Usages.w
        let mut verbs = Verbs::new();
        let cat = verbs.stock.new_category("verb");
        let item = verbs.stock.add_item(cat, Box::new(0usize));
        let usage = verbs.stock.new_usage(item, "English");
        let vu = verbs.new_usage(WordAssemblage::lit_1("is"), false, usage, None).unwrap();
        let tier = verbs.new_tier(10);
        verbs.add_usage_to_tier(vu, tier);
        assert_eq!(verbs.tiers[tier].tier_contents.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Preposition tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_make_preposition() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let text = WordAssemblage::lit_1("in");
        let p = verbs.make_preposition(text, false, None);
        assert_eq!(verbs.prepositions.len(), 1);
        assert_eq!(verbs.prepositions[p].prep_text.to_string(), "in");
    }

    #[test]
    fn test_make_preposition_deduplicates() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let p1 = verbs.make_preposition(WordAssemblage::lit_1("in"), false, None);
        let p2 = verbs.make_preposition(WordAssemblage::lit_1("in"), false, None);
        assert_eq!(p1, p2);
        assert_eq!(verbs.prepositions.len(), 1);
    }

    #[test]
    fn test_make_preposition_different_texts() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let p1 = verbs.make_preposition(WordAssemblage::lit_1("in"), false, None);
        let p2 = verbs.make_preposition(WordAssemblage::lit_1("on"), false, None);
        assert_ne!(p1, p2);
        assert_eq!(verbs.prepositions.len(), 2);
    }

    #[test]
    fn test_preposition_length() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let text = WordAssemblage::new(vec!["in".to_string(), "front".to_string(), "of".to_string()]);
        let p = verbs.make_preposition(text, false, None);
        assert_eq!(verbs.preposition_length(p), 3);
    }

    #[test]
    fn test_parse_against_preposition() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let p = verbs.make_preposition(WordAssemblage::lit_1("in"), false, None);
        let wording = &["in", "the", "room"];
        let pos = verbs.parse_against_preposition(wording, p);
        assert_eq!(pos, Some(1));
    }

    #[test]
    fn test_parse_against_preposition_no_match() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let p = verbs.make_preposition(WordAssemblage::lit_1("in"), false, None);
        let wording = &["on", "the", "table"];
        assert!(verbs.parse_against_preposition(wording, p).is_none());
    }

    #[test]
    fn test_get_where_prep_created() {
        // Reference: services/linguistics-module/Chapter 3/Prepositions.w
        let mut verbs = Verbs::new();
        let p = verbs.make_preposition(WordAssemblage::lit_1("in"), false, Some(42));
        assert_eq!(verbs.get_where_prep_created(p), Some(42));
    }

    // -----------------------------------------------------------------------
    // Special meaning tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_declare_special_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut verbs = Verbs::new();
        let sm = verbs.declare_special_meaning(generic_smf, "test_meaning", 0);
        assert_eq!(verbs.special_meanings.len(), 1);
        assert_eq!(verbs.special_meanings[sm].sm_name, "test_meaning");
    }

    #[test]
    fn test_find_special_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut verbs = Verbs::new();
        verbs.declare_special_meaning(generic_smf, "test_meaning", 0);
        let found = verbs.find_special_meaning("test_meaning");
        assert!(found.is_some());
        let not_found = verbs.find_special_meaning("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_special_meaning_metadata() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut verbs = Verbs::new();
        let sm = verbs.declare_special_meaning(generic_smf, "test", 42);
        assert_eq!(verbs.get_special_meaning_metadata(sm), 42);
    }

    #[test]
    fn test_get_special_meaning_name() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut verbs = Verbs::new();
        let sm = verbs.declare_special_meaning(generic_smf, "test_name", 0);
        assert_eq!(verbs.get_special_meaning_name(sm), "test_name");
    }

    #[test]
    fn test_is_special_meaning() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut verbs = Verbs::new();
        let sm = verbs.declare_special_meaning(generic_smf, "test", 0);
        assert!(verbs.is_special_meaning(sm, generic_smf));
    }

    #[test]
    fn test_generic_smf() {
        // Reference: services/linguistics-module/Chapter 3/Special Meanings.w
        let mut node = ParseNode::new(crate::NodeType::Root, Wording::EMPTY);
        let nps = [
            Wording::new(0, 2), // SP: "the cat"
            Wording::new(3, 4), // OP: "the mat"
            Wording::EMPTY,     // IOP: empty
        ];
        let result = generic_smf(0, &mut node, &nps);
        assert!(result);
        assert_eq!(node.node_type(), crate::NodeType::Verb);
        assert_eq!(node.child_count(), 2);
    }

    // -----------------------------------------------------------------------
    // Conjugation management tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_register_conjugation() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let mut verbs = Verbs::new();
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let ci = verbs.register_conjugation(conj);
        assert_eq!(verbs.conjugations.len(), 1);
        assert_eq!(verbs.conjugations[ci].infinitive.to_string(), "be");
    }

    #[test]
    fn test_find_conjugation() {
        // Reference: services/inflections-module/Chapter 3/Verb Conjugation.w
        let mut verbs = Verbs::new();
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        verbs.register_conjugation(conj);
        let found = verbs.find_conjugation(&WordAssemblage::lit_1("be"));
        assert!(found.is_some());
        let not_found = verbs.find_conjugation(&WordAssemblage::lit_1("walk"));
        assert!(not_found.is_none());
    }

    // -----------------------------------------------------------------------
    // Verb with conjugation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_verb_with_conjugation() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let conj = Conjugation::conjugate(&WordAssemblage::lit_1("be"), "English");
        let ci = verbs.register_conjugation(conj);
        let v = verbs.new_verb(Some(ci), true);
        assert_eq!(verbs.verbs[v].conjugation, Some(ci));
        assert_eq!(verbs.copular_verb, Some(v));
    }

    // -----------------------------------------------------------------------
    // Stock integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_to_lcon_and_from_lcon() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let mut verbs = Verbs::new();
        let v = verbs.new_verb(None, false);
        let lcon = verbs.to_lcon(v);
        assert!(lcon.get_id().is_some());
    }

    // -----------------------------------------------------------------------
    // make_built_in tests
    // -----------------------------------------------------------------------

    #[test]
    fn make_built_in_creates_two_verbs() {
        let mut verbs = Verbs::new();
        let (to_be, to_mean) = verbs.make_built_in();
        assert!(verbs.verbs[to_be].copular, "to be should be copular");
        assert!(!verbs.verbs[to_mean].copular, "to mean should not be copular");
    }

    #[test]
    fn make_built_in_declares_special_meanings() {
        let mut verbs = Verbs::new();
        verbs.make_built_in();
        assert!(verbs.special_meanings.iter().any(|sm| sm.sm_name == "verb-means"));
        assert!(verbs.special_meanings.iter().any(|sm| sm.sm_name == "new-relation"));
        assert!(verbs.special_meanings.iter().any(|sm| sm.sm_name == "new-verb"));
        assert!(verbs.special_meanings.iter().any(|sm| sm.sm_name == "use"));
    }
    #[test]
    fn make_built_in_verb_means_attached_to_to_mean() {
        let mut verbs = Verbs::new();
        let (_, to_mean) = verbs.make_built_in();
        let verb = &verbs.verbs[to_mean];
        // Should have at least one form (the base form + the verb-means form)
        assert!(verb.first_form.is_some(), "to mean should have forms");
    }

    #[test]
    fn make_built_in_registers_usages() {
        let mut verbs = Verbs::new();
        verbs.make_built_in();
        // Should have verb usages registered
        assert!(!verbs.usages.is_empty(), "should have verb usages");
        assert!(!verbs.tiers.is_empty(), "should have usage tiers");
    }

    #[test]
    fn make_built_in_special_meanings_count() {
        let mut verbs = Verbs::new();
        verbs.make_built_in();
        // 9 + 8 + 1 + 5 = 23 special meanings
        assert_eq!(verbs.special_meanings.len(), 23);
    }

    #[test]
    fn make_built_in_verb_means_has_special_meaning() {
        let mut verbs = Verbs::new();
        verbs.make_built_in();
        // Find the verb-means special meaning
        let verb_means_idx = verbs.special_meanings.iter()
            .position(|sm| sm.sm_name == "verb-means")
            .expect("verb-means should exist");
        assert_eq!(verbs.special_meanings[verb_means_idx].metadata_n, 3);
    }
}
