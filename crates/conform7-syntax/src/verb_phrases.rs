//! Verb phrase parsing — the verb-finding algorithm.
//!
//! Provides the core `VerbPhrases::seek` algorithm that locates the primary
//! verb in a sentence, identifies its subject and object phrases, and builds
//! the `VERB_NT` sentence diagram.
//!
//! # References
//!
//! - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
//!   the full verb-finding algorithm including viability map, seek loop,
//!   accept, default_verb, and corrective surgery.

use crate::linguistics::{Diagrams, NounPhrases};
use crate::parse_node::ParseNode;
use crate::preform::{InternalRegistry, PreformContext, match_nonterminal_impl};
use crate::verbs::{
    Verbs, VerbFormRef, VerbMeaning, VerbUsageRef,
    SVO_FS_BIT, VO_FS_BIT, SVOO_FS_BIT, VOO_FS_BIT,
};
use crate::{NodeType, Wording};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of words in the viability map.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
///   `VIABILITY_MAP_SIZE` constant.
pub const VIABILITY_MAP_SIZE: usize = 100;

/// Task constant for `ACCEPT_SMFT` — accept a special meaning.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Special Meanings.w`
pub const ACCEPT_SMFT: i32 = 0;

// ---------------------------------------------------------------------------
// ViabilityMap
// ---------------------------------------------------------------------------

/// A map of verb viability scores for each word in a sentence.
///
/// Each word is scored 0-3:
/// - 0: not a verb
/// - 1: verb outside brackets
/// - 2: verb inside brackets
/// - 3: negated non-copular verb
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
///   `viability_map` array and `@<Calculate the viability map@>`.
#[derive(Clone, Debug)]
pub struct ViabilityMap {
    /// Scores for each word position.
    pub scores: [i32; VIABILITY_MAP_SIZE],
    /// The wording this map was calculated for.
    pub wording: Wording,
}

impl ViabilityMap {
    /// Create a new viability map with all scores initialized to 0.
    pub fn new(wording: Wording) -> Self {
        ViabilityMap {
            scores: [0i32; VIABILITY_MAP_SIZE],
            wording,
        }
    }

    /// Get the score at a word position.
    pub fn get(&self, pos: usize) -> i32 {
        if pos < VIABILITY_MAP_SIZE {
            self.scores[pos]
        } else {
            0
        }
    }

    /// Set the score at a word position.
    pub fn set(&mut self, pos: usize, score: i32) {
        if pos < VIABILITY_MAP_SIZE {
            self.scores[pos] = score;
        }
    }
}

// ---------------------------------------------------------------------------
// VerbPhrases
// ---------------------------------------------------------------------------

/// Verb phrase parsing functions.
///
/// Provides the core verb-finding algorithm that locates the primary verb
/// in a sentence, identifies its subject and object phrases, and builds
/// the `VERB_NT` sentence diagram.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
pub struct VerbPhrases;

impl VerbPhrases {
    // -----------------------------------------------------------------------
    // Viability map calculation
    // -----------------------------------------------------------------------

    /// Calculate the viability map for a wording.
    ///
    /// Scores each word in the sentence:
    /// - 0: not a verb
    /// - 1: verb outside brackets
    /// - 2: verb inside brackets
    /// - 3: negated non-copular verb
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `@<Calculate the viability map@>` (lines 158-184).
    pub fn calculate_viability_map(
        wording: Wording,
        ctx: &PreformContext,
        registry: &InternalRegistry,
    ) -> ViabilityMap {
        let mut vm = ViabilityMap::new(wording);
        let mut bl: i32 = 0; // bracket depth

        let start = wording.start as usize;
        let end = wording.end as usize;

        let mut i = start;
        while i < end {
            // Track bracket depth.
            if let Some(word) = ctx.word_text.get(i) {
                match *word {
                    "(" | "{" => bl += 1,
                    ")" | "}" => bl -= 1,
                    _ => {}
                }
            }

            // Check if this word starts a negated non-copular verb phrase.
            let neg_wording = Wording::new(i as u32, end as u32);
            if let Some(result) = match_nonterminal_impl(
                ctx, registry, "negated-noncopular-verb-present", neg_wording,
            ) {
                if let Some(internal) = result.internal {
                    if let crate::preform::InternalPayload::Integer(consumed) = internal.payload {
                        // Score all words in the negated verb phrase as 3.
                        let consumed = consumed as usize;
                        for j in i..(i + consumed).min(end) {
                            vm.set(j - start, 3);
                        }
                        i += consumed;
                        continue;
                    }
                }
            }

            // Check if this word is a non-imperative verb.
            let verb_wording = Wording::new(i as u32, end as u32);
            if let Some(result) = match_nonterminal_impl(
                ctx, registry, "nonimperative-verb", verb_wording,
            ) {
                if result.internal.is_some() {
                    let score = if bl > 0 { 2 } else { 1 };
                    vm.set(i - start, score);
                    i += 1;
                    continue;
                }
            }

            i += 1;
        }

        vm
    }

    // -----------------------------------------------------------------------
    // Seek
    // -----------------------------------------------------------------------

    /// Find the primary verb in a sentence.
    ///
    /// Entry point that calculates the viability map, seeks verb usages,
    /// and applies corrective surgery.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::seek` (lines 102-119).
    pub fn seek(
        wording: Wording,
        ctx: &PreformContext,
        registry: &InternalRegistry,
        detect_occurrences: bool,
    ) -> Option<ParseNode> {
        let mut result = Self::seek_inner(wording, ctx, registry, 0, detect_occurrences)?;
        Self::corrective_surgery(&mut result);
        Some(result)
    }

    /// Inner seek function with existential OP edge parameter.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::seek_inner` (lines 121-128).
    pub fn seek_inner(
        wording: Wording,
        ctx: &PreformContext,
        registry: &InternalRegistry,
        _existential_op_edge: i32,
        _detect_occurrences: bool,
    ) -> Option<ParseNode> {
        let verbs_registry = ctx.verbs_registry?;
        let vm = Self::calculate_viability_map(wording, ctx, registry);

        // Iterate viability levels: 1 (outside brackets), then 2 (inside brackets).
        for &level in &[1, 2] {
            // Iterate tiers (excluding priority 0).
            let mut tier_current = verbs_registry.tier_list_head;
            while let Some(tier_ref) = tier_current {
                let tier = verbs_registry.tiers.get(tier_ref)?;
                if tier.priority == 0 {
                    tier_current = tier.next_tier;
                    continue;
                }

                // For each word position with matching viability.
                let start = wording.start as usize;
                let end = wording.end as usize;
                for pos in start..end {
                    if vm.get(pos - start) != level {
                        continue;
                    }

                    // Try each verb usage in the tier at this position.
                    for &vu_ref in &tier.tier_contents {
                        if let Some(node) = Self::try_verb_usage_at_position(
                            wording, pos, vu_ref, ctx, registry, verbs_registry,
                        ) {
                            return Some(node);
                        }
                    }
                }

                tier_current = tier.next_tier;
            }
        }

        None
    }

    /// Try a verb usage at a specific word position.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `@<Seek verb usage at position pos@>` and
    ///   `@<Consider whether this usage is being made at this position@>`.
    fn try_verb_usage_at_position(
        wording: Wording,
        pos: usize,
        vu_ref: VerbUsageRef,
        ctx: &PreformContext,
        registry: &InternalRegistry,
        verbs_registry: &Verbs,
    ) -> Option<ParseNode> {
        let _usage = verbs_registry.usages.get(vu_ref)?;
        let verb_ref = verbs_registry.get_verb_from_usage(vu_ref)?;
        let verb = verbs_registry.verbs.get(verb_ref)?;

        // Get the tail wording from the current position.
        let tail_start = pos;
        let tail_end = wording.end as usize;
        if tail_start >= tail_end {
            return None;
        }
        let tail_slice = ctx.word_text.get(tail_start..tail_end)?;

        // Check if the usage text appears at the front of the tail wording.
        let consumed = verbs_registry.parse_against_verb(tail_slice, vu_ref)?;

        // Iterate verb forms to find one that matches.
        let mut form_current = verb.first_form;
        while let Some(form_ref) = form_current {
            let form = verbs_registry.forms.get(form_ref)?;

            // Skip if the verb meaning is meaningless.
            let sense_ref = form.list_of_senses.first()?;
            let sense = verbs_registry.senses.get(*sense_ref)?;
            if sense.vm.is_meaningless() {
                form_current = form.next_form;
                continue;
            }

            // Check form structure compatibility.
            if !Self::check_form_structure(form_ref, pos, wording, verbs_registry) {
                form_current = form.next_form;
                continue;
            }

            // Build subject and object wordings.
            let (subject_wording, object_wording) = Self::build_subject_object_wordings(
                wording, pos, consumed, form_ref, verbs_registry,
            );

            // Check required prepositions.
            let (prep_consumed, second_prep_consumed) = Self::check_prepositions(
                object_wording, form_ref, ctx, verbs_registry,
            );

            // Build the diagram.
            let node = Self::build_diagram(
                wording, pos, consumed, vu_ref, form_ref, verb_ref,
                subject_wording, object_wording,
                prep_consumed, second_prep_consumed,
                ctx, registry, verbs_registry,
            )?;

            return Some(node);
        }

        None
    }

    // -----------------------------------------------------------------------
    // Form structure checking
    // -----------------------------------------------------------------------

    /// Check that the verb form is compatible with the verb's position.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `@<Reject a match with verb in the wrong position@>` (lines 285-291).
    fn check_form_structure(
        form_ref: VerbFormRef,
        pos: usize,
        wording: Wording,
        verbs_registry: &Verbs,
    ) -> bool {
        let form = match verbs_registry.forms.get(form_ref) {
            Some(f) => f,
            None => return false,
        };

        let is_at_position_0 = pos == wording.start as usize;

        // If the verb form has no VO or VOO bits, the verb must not be at
        // position 0 (it needs a subject).
        let has_vo_or_voo = (form.form_structures & (VO_FS_BIT | VOO_FS_BIT)) != 0;
        if !has_vo_or_voo && is_at_position_0 {
            return false;
        }

        // If the verb form has no SVO or SVOO bits, the verb must be at
        // position 0 (it's imperative).
        let has_svo_or_svoo = (form.form_structures & (SVO_FS_BIT | SVOO_FS_BIT)) != 0;
        if !has_svo_or_svoo && !is_at_position_0 {
            return false;
        }

        true
    }

    // -----------------------------------------------------------------------
    // Subject/object wording building
    // -----------------------------------------------------------------------

    /// Build the subject and object wordings for a verb match.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w`
    fn build_subject_object_wordings(
        wording: Wording,
        pos: usize,
        consumed: usize,
        form_ref: VerbFormRef,
        verbs_registry: &Verbs,
    ) -> (Wording, Wording) {
        let form = match verbs_registry.forms.get(form_ref) {
            Some(f) => f,
            None => return (Wording::EMPTY, Wording::EMPTY),
        };

        let start = wording.start as usize;
        let end = wording.end as usize;
        let verb_end = pos + consumed;

        let has_svo_or_svoo = (form.form_structures & (SVO_FS_BIT | SVOO_FS_BIT)) != 0;
        let has_vo_or_voo = (form.form_structures & (VO_FS_BIT | VOO_FS_BIT)) != 0;

        let (subject_start, subject_end) = if has_svo_or_svoo {
            // Subject is before the verb.
            (start, pos)
        } else {
            // No subject (imperative).
            (0, 0)
        };

        let (object_start, object_end) = if has_vo_or_voo {
            // Object is after the verb.
            (verb_end, end)
        } else {
            // No object.
            (0, 0)
        };

        let subject_wording = if subject_start < subject_end {
            Wording::new(subject_start as u32, subject_end as u32)
        } else {
            Wording::EMPTY
        };

        let object_wording = if object_start < object_end {
            Wording::new(object_start as u32, object_end as u32)
        } else {
            Wording::EMPTY
        };

        (subject_wording, object_wording)
    }

    // -----------------------------------------------------------------------
    // Preposition checking
    // -----------------------------------------------------------------------

    ///
    /// Returns the number of words consumed by the first and second prepositions.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `@<Check whether we do indeed have these required prepositions in place@>`
    ///   (lines 374-407).
    fn check_prepositions(
        object_wording: Wording,
        form_ref: VerbFormRef,
        ctx: &PreformContext,
        verbs_registry: &Verbs,
    ) -> (usize, usize) {
        let form = match verbs_registry.forms.get(form_ref) {
            Some(f) => f,
            None => return (0, 0),
        };

        let mut prep_consumed = 0;
        let mut second_prep_consumed = 0;

        // Check first preposition.
        if let Some(prep_ref) = form.preposition {
            if !object_wording.is_empty() {
                let obj_start = object_wording.start as usize;
                let obj_slice = ctx.word_text.get(obj_start..).unwrap_or(&[]);
                if let Some(consumed) = verbs_registry.parse_against_preposition(obj_slice, prep_ref) {
                    prep_consumed = consumed;
                }
            }
        }

        // Check second preposition.
        if let Some(second_prep_ref) = form.second_clause_preposition {
            if !object_wording.is_empty() {
                let obj_start = object_wording.start as usize + prep_consumed;
                let obj_slice = ctx.word_text.get(obj_start..).unwrap_or(&[]);
                if let Some(consumed) = verbs_registry.parse_against_preposition(obj_slice, second_prep_ref) {
                    second_prep_consumed = consumed;
                }
            }
        }

        (prep_consumed, second_prep_consumed)
    }

    // -----------------------------------------------------------------------
    // Diagram building
    // -----------------------------------------------------------------------

    /// Build a VERB_NT parse node for a matched verb usage.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `@<Check whether any sense of this verb form will accept this usage
#[allow(clippy::too_many_arguments)]
    fn build_diagram(
        _wording: Wording,
        pos: usize,
        consumed: usize,
        vu_ref: VerbUsageRef,
        form_ref: VerbFormRef,
        verb_ref: crate::verbs::VerbRef,
        subject_wording: Wording,
        object_wording: Wording,
        _prep_consumed: usize,
        _second_prep_consumed: usize,
        ctx: &PreformContext,
        registry: &InternalRegistry,
        verbs_registry: &Verbs,
    ) -> Option<ParseNode> {
        let form = verbs_registry.forms.get(form_ref)?;
        let verb_wording = Wording::new(pos as u32, (pos + consumed) as u32);

        // Create the VERB_NT node.
        let mut vp_pn = ParseNode::new(NodeType::Verb, verb_wording);

        // Set verb usage reference.
        vp_pn.set_verb_usage(vu_ref);

        // Set preposition references.
        vp_pn.set_preposition(form.preposition);
        vp_pn.set_second_preposition(form.second_clause_preposition);

        // Check for certainty adverbs in the verb wording.
        // The <certainty> internal NT matches adverbs like "always", "never", etc.
        if let Some(result) = match_nonterminal_impl(ctx, registry, "certainty", verb_wording) {
            if let Some(internal) = result.internal {
                if let crate::preform::InternalPayload::Integer(level) = internal.payload {
                    vp_pn.add_annotation(crate::parse_node::Annotation::VerbalCertainty(level));
                }
            }
        }

        // Build the noun phrase wordings array for accept.
        let nps = [subject_wording, object_wording, Wording::EMPTY];

        // Call accept to try each sense.
        Self::accept(form_ref, verb_ref, &mut vp_pn, &nps, ctx, registry, verbs_registry)
    }
    // -----------------------------------------------------------------------
    // Accept
    // -----------------------------------------------------------------------

    /// Try each sense of a verb form, calling special meaning functions
    /// and falling back to the regular meaning.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::accept` (lines 472-496).
    pub fn accept(
        vf: VerbFormRef,
        verb_ref: crate::verbs::VerbRef,
        vp_pn: &mut ParseNode,
        nps: &[Wording; 3],
        ctx: &PreformContext,
        registry: &InternalRegistry,
        verbs_registry: &Verbs,
    ) -> Option<ParseNode> {
        let form = verbs_registry.forms.get(vf)?;

        // Try the first sense of the verb form.
        if let Some(&sense_ref) = form.list_of_senses.first() {
            let sense = verbs_registry.senses.get(sense_ref)?;

            // Check if this sense has a special meaning.
            if let Some(sm_ref) = sense.vm.special_meaning {
                // Try the special meaning function.
                let mut node = vp_pn.clone();
                if verbs_registry.call_special_meaning(sm_ref, ACCEPT_SMFT, &mut node, nps) {
                    return Some(node);
                }
            }

            // Fall back to the regular meaning.
            return Self::default_verb(&sense.vm, verb_ref, vp_pn, nps, ctx, registry, verbs_registry);
        }

        None
    }

    // -----------------------------------------------------------------------
    // Default verb
    // -----------------------------------------------------------------------
    ///
    /// Build the default sentence diagram for regular verb meanings.
    /// Parses the subject wording as `<np-as-subject>` and the object wording
    /// as `<np-as-object>`, then builds the sentence diagram.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::default_verb` (lines 503-527).
    pub fn default_verb(
        _vm: &VerbMeaning,
        verb_ref: crate::verbs::VerbRef,
        vp_pn: &mut ParseNode,
        nps: &[Wording; 3],
        ctx: &PreformContext,
        registry: &InternalRegistry,
        verbs_registry: &Verbs,
    ) -> Option<ParseNode> {
        let subject_wording = nps[0];
        let object_wording = nps[1];

        // Parse the subject wording as a noun phrase.
        if !subject_wording.is_empty() {
            if let Some(subject_node) = NounPhrases::parse_np_unparsed(ctx, registry, subject_wording) {
                vp_pn.append_child(subject_node);
            }
        }

        // Parse the object wording as a noun phrase.
        if !object_wording.is_empty() {
            if let Some(object_node) = NounPhrases::parse_np_unparsed(ctx, registry, object_wording) {
                // For non-copular verbs, wrap the object in a RELATIONSHIP_NT.
                let is_copular = verbs_registry.copular_verb == Some(verb_ref);

                if is_copular {
                    vp_pn.append_child(object_node);
                } else {
                    let rel_node = Diagrams::new_relationship(object_wording, object_node);
                    vp_pn.append_child(rel_node);
                }
            }
        }

        // Extend the verb node's wording to span all children.
        vp_pn.extend_wording_from_children();

        Some(vp_pn.clone())
    }

    // -----------------------------------------------------------------------
    // Corrective surgery
    // -----------------------------------------------------------------------

    /// Post-process the diagram tree to handle special cases.
    ///
    /// Iterates until no more surgeries are possible, trying "of surgery",
    /// "location surgery", and "called surgery" on each node.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::corrective_surgery` (lines 582-601).
    pub fn corrective_surgery(p: &mut ParseNode) {
        loop {
            let mut changed = false;

            // Try surgeries on this node.
            if Self::perform_of_surgery(p) {
                changed = true;
            }
            if Self::perform_location_surgery(p) {
                changed = true;
            }
            if Self::perform_called_surgery(p) {
                changed = true;
            }

            // Recurse into children.
            // We need to collect children first to avoid borrow issues.
            let child_types: Vec<NodeType> = p.children().map(|c| c.node_type()).collect();
            for child_type in child_types {
                // Find the child by type and recurse.
                if let Some(child) = p.find_child_mut(child_type) {
                    Self::corrective_surgery(child);
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// Perform "of surgery" — split `X of Y` noun phrases.
    ///
    /// Checks if the node is an `UNPARSED_NOUN_NT` and searches for "of"
    /// in the wording. If found, splits into `X_OF_Y_NT` with left and
    /// right children.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::perform_of_surgery` (lines 610-629).
    pub fn perform_of_surgery(p: &mut ParseNode) -> bool {
        // DEFERRED: This surgery requires access to the word text to find "of"
        // in the wording. When the word text context is available, implement:
        // 1. Get the word text for the wording range
        // 2. Find the position of "of" in the words
        // 3. Split into X_OF_Y_NT with left and right children
        //
        // C reference: services/linguistics-module/Chapter 4/Verb Phrases.w lines 610-629
        if p.node_type() != NodeType::UnparsedNoun {
            return false;
        }

        let wording = p.wording();
        if wording.is_empty() {
            return false;
        }

        false
    }

    /// Perform "location surgery" — handle `X is on Y and under Z`.
    ///
    /// Checks if the node is a `RELATIONSHIP_NT` with an `AND_NT` child
    /// and restructures the tree.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    ///   `VerbPhrases::perform_location_surgery` (lines 651-673).
    pub fn perform_location_surgery(p: &mut ParseNode) -> bool {
        // DEFERRED: This surgery restructures RELATIONSHIP_NT -> AND_NT -> [left, right]
        // into AND_NT -> [RELATIONSHIP_NT -> left, RELATIONSHIP_NT -> right].
        // Requires full tree restructuring logic.
        //
        // C reference: services/linguistics-module/Chapter 4/Verb Phrases.w lines 651-673
        if p.node_type() != NodeType::Relationship {
            return false;
        }

        // Check for AND_NT child.
        if p.find_child(NodeType::And).is_none() {
            return false;
        }

        false
    }
    /// Checks if the node is a `CALLED_NT` with a `RELATIONSHIP_NT` child
    /// and swaps the node types to fix the ordering.
    ///
    /// # References
    ///
    /// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
    pub fn perform_called_surgery(p: &mut ParseNode) -> bool {
        // DEFERRED: This surgery swaps node types to fix ordering in CALLED_NT nodes.
        // Requires full tree restructuring logic.
        //
        // C reference: services/linguistics-module/Chapter 4/Verb Phrases.w lines 682-696
        if p.node_type() != NodeType::Called {
            return false;
        }

        // Check for RELATIONSHIP_NT child.
        if p.find_child(NodeType::Relationship).is_none() {
            return false;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preform::{parse_preform_grammar, InternalRegistry, PreformContext};
    use crate::verbs::VerbMeaning;
    use crate::word_assemblage::WordAssemblage;
    use crate::Wording;

    fn make_test_verbs() -> Verbs {
        let mut v = Verbs::new();

        // Create a copular verb "to be".
        let be_verb = v.new_verb(None, true);

        // Add forms for "is", "are", "was", "were".
        // Note: VerbMeaning::clone sets regular_meaning=None because Box<dyn Any> is not Clone.
        // So we create separate meanings for each add_form call instead of cloning.
        v.add_form(be_verb, None, None, VerbMeaning::regular(Box::new("to be")), SVO_FS_BIT);

        // Create usages for "is", "are", "was", "were".
        let cat = v.stock.new_category("verb");
        let item = v.stock.add_item(cat, Box::new(be_verb));
        let usage = v.stock.new_usage(item, "English");

        let is_usage = v.new_usage(WordAssemblage::lit_1("is"), false, usage, None).unwrap();
        v.new_usage(WordAssemblage::lit_1("are"), false, usage, None);
        v.new_usage(WordAssemblage::lit_1("was"), false, usage, None);
        v.new_usage(WordAssemblage::lit_1("were"), false, usage, None);

        // Create a non-copular verb "to carry".
        let carry_verb = v.new_verb(None, false);
        v.add_form(carry_verb, None, None, VerbMeaning::regular(Box::new("to carry")), SVO_FS_BIT | VO_FS_BIT);

        let item2 = v.stock.add_item(cat, Box::new(carry_verb));
        let usage2 = v.stock.new_usage(item2, "English");

        let carry_usage = v.new_usage(WordAssemblage::lit_1("carry"), false, usage2, None).unwrap();
        v.new_usage(WordAssemblage::lit_1("carries"), false, usage2, None);
        v.new_usage(WordAssemblage::lit_1("carried"), false, usage2, None);

        // Create tiers and add usages to them.
        let tier = v.new_tier(100);
        v.add_usage_to_tier(is_usage, tier);
        v.add_usage_to_tier(carry_usage, tier);

        v
    }

    #[test]
    fn test_viability_map_simple() {
        // Reference: services/linguistics-module/Chapter 4/Verb Phrases.w
        let grammar = parse_preform_grammar("<nonimperative-verb> internal\n<negated-noncopular-verb-present> internal").unwrap();
        let verbs = make_test_verbs();
        let words = &["The", "cat", "is", "on", "the", "mat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
            verbs_registry: Some(&verbs),
        };
        let registry = InternalRegistry::linguistics();
        let vm = VerbPhrases::calculate_viability_map(Wording::new(0, 6), &ctx, &registry);
        assert_eq!(vm.get(0), 0, "'The' should be score 0");
        assert_eq!(vm.get(1), 0, "'cat' should be score 0");
        assert_eq!(vm.get(2), 1, "'is' should be score 1 (verb outside brackets)");
        assert_eq!(vm.get(3), 0, "'on' should be score 0");
        assert_eq!(vm.get(4), 0, "'the' should be score 0");
        assert_eq!(vm.get(5), 0, "'mat' should be score 0");
    }

    #[test]
    fn test_viability_map_negated() {
        // Reference: services/linguistics-module/Chapter 4/Verb Phrases.w
        let grammar = parse_preform_grammar("<nonimperative-verb> internal\n<negated-noncopular-verb-present> internal").unwrap();
        let verbs = make_test_verbs();
        let words = &["The", "cat", "does", "not", "carry", "the", "box"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
            verbs_registry: Some(&verbs),
        };
        let registry = InternalRegistry::linguistics();
        let vm = VerbPhrases::calculate_viability_map(Wording::new(0, 7), &ctx, &registry);
        assert_eq!(vm.get(0), 0, "'The' should be score 0");
        assert_eq!(vm.get(1), 0, "'cat' should be score 0");
        assert_eq!(vm.get(2), 3, "'does' should be score 3 (negated verb)");
        assert_eq!(vm.get(3), 3, "'not' should be score 3 (negated verb)");
        assert_eq!(vm.get(4), 3, "'carry' should be score 3 (negated verb)");
        assert_eq!(vm.get(5), 0, "'the' should be score 0");
        assert_eq!(vm.get(6), 0, "'box' should be score 0");
    }

    #[test]
    fn test_viability_map_copular_negated() {
        // Reference: services/linguistics-module/Chapter 4/Verb Phrases.w
        let grammar = parse_preform_grammar("<nonimperative-verb> internal\n<negated-noncopular-verb-present> internal").unwrap();
        let verbs = make_test_verbs();
        let words = &["The", "cat", "is", "not", "a", "dog"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
            verbs_registry: Some(&verbs),
        };
        let registry = InternalRegistry::linguistics();
        let vm = VerbPhrases::calculate_viability_map(Wording::new(0, 6), &ctx, &registry);
        // "is" is copular, so it should be score 1, not 3.
        assert_eq!(vm.get(2), 1, "'is' should be score 1 (copular, not negated)");
    }

    // -----------------------------------------------------------------------
    // Seek tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_seek_finds_copular_verb() {
        // Reference: services/linguistics-module/Chapter 4/Verb Phrases.w
        let grammar = parse_preform_grammar(
            "<nonimperative-verb> internal\n<negated-noncopular-verb-present> internal\n<np-unparsed> ::= ..."
        ).unwrap();
        let verbs = make_test_verbs();
        let words = &["The", "cat", "is", "on", "the", "mat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
            verbs_registry: Some(&verbs),
        };
        let registry = InternalRegistry::linguistics();
        let result = VerbPhrases::seek(Wording::new(0, 6), &ctx, &registry, false);
        assert!(result.is_some(), "seek should find a verb in 'The cat is on the mat'");
        if let Some(node) = result {
            assert_eq!(node.node_type(), NodeType::Verb, "result should be a VERB_NT");
        }
    }

    #[test]
    fn test_seek_fails_no_verb() {
        // Reference: services/linguistics-module/Chapter 4/Verb Phrases.w
        let grammar = parse_preform_grammar(
            "<nonimperative-verb> internal\n<negated-noncopular-verb-present> internal"
        ).unwrap();
        let verbs = make_test_verbs();
        let words = &["The", "cat"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
            verbs_registry: Some(&verbs),
        };
        let registry = InternalRegistry::linguistics();
        let result = VerbPhrases::seek(Wording::new(0, 2), &ctx, &registry, false);
        assert!(result.is_none(), "seek should fail on 'The cat' (no verb)");
    }
}
