//! To Phrase Family — imperative definitions of "To ..." phrases.
//!
//! Corresponds to `ToPhraseFamily` in the C reference
//! (`inform7/assertions-module/Chapter 5/To Phrase Family.w`).
//!
//! This family handles definitions of "To..." phrases: Inform's equivalent of
//! function definitions. For example, `To chime (N - a number) times: ...`.
//! The preamble is recognised by its opening word "To". It wires seven
//! methods into the `TO_PHRASE_EFF` imperative definition family:
//!
//! - `identify` — decide whether a preamble belongs to this family (C: `claim`)
//! - `assess` — take a closer look at the wording after identification
//! - `register` — enter phrases into the excerpt parser in logical order
//! - `given_body` — insert the body into the logical-order linked list
//! - `allows_inline` — whether the body may be `(- ... -)` inline I6
//! - `compile` — make compilation requests for the phrase bodies
//! - `phrasebook_index` — whether definitions appear in the Phrasebook index
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types — method signatures take `&ImpDefFamily` only.
//! - No `to_family_data` structure or `new_data` allocation in `identify`.
//! - No logical-order linked list, `cmp`, or `get_next`/`set_next` in `given_body`.
//! - No `ParseInvocations::register_excerpt` call in `register`.
//! - No `PhraseRequests` calls or problem messages in `compile`.
//! - No `parse_node` handling or Preform grammar.

use crate::assertions::imperative_definition_families::ImpDefFamily;

/// The To Phrase Family module.
///
/// Corresponds to `ToPhraseFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`).
pub struct ToPhraseFamily;

impl ToPhraseFamily {
    /// Wire the seven To-phrase-family methods into the given family.
    ///
    /// Corresponds to `ToPhraseFamily::create_family` in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 18-27).
    ///
    /// Simplified: all methods are stubs. The full logic (preamble matching,
    /// `to_family_data` allocation, logical-order insertion, excerpt-parser
    /// registration, compilation requests) is deferred until `imperative_defn`,
    /// `id_body`, and `parse_node` types are introduced.
    pub fn wire_methods(family: &mut ImpDefFamily) {
        family.methods.identify = Some(Self::identify);
        family.methods.assess = Some(Self::assess);
        family.methods.register = Some(Self::register);
        family.methods.given_body = Some(Self::given_body);
        family.methods.allows_inline = Some(Self::allows_inline);
        family.methods.compile = Some(Self::compile);
        family.methods.phrasebook_index = Some(Self::phrasebook_index);
    }

    /// Decide whether a definition preamble belongs to the To phrase family.
    ///
    /// Corresponds to `ToPhraseFamily::claim` (the `IDENTIFY_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 92-120).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Match the preamble text against `<to-phrase-preamble>` (lines 70-76)
    /// 2. Set `id->family = to_phrase_idf`
    /// 3. Allocate a `to_family_data` (lines 38-49) and store it on
    ///    `id->family_specific_data`
    /// 4. Parse out the constant name (`(this is ...)`), documentation symbol,
    ///    prototype text, and (for forms 1 and 2) the `explicit_name_for_inverse`
    fn identify(_self: &ImpDefFamily) {
        // No-op: preamble matching and to_family_data allocation deferred.
    }

    /// Take a closer look at the wording after identification.
    ///
    /// Corresponds to `ToPhraseFamily::assess` (the `ASSESS_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 134-148).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. If the preamble has a constant name, attach the `constant_phrase`
    ///    and reject indefinite kinds (`PM_NamedGeneric`, lines 160-178)
    /// 2. Detect `<now-phrase-preamble>` (`to now ...`) and reject redefinitions
    ///    (`PM_RedefinedNow`, lines 140-146)
    /// 3. Detect `<begin-phrase-preamble>` (`to begin`) and set `tfd->to_begin`
    fn assess(_self: &ImpDefFamily) {
        // No-op: wording assessment and problem reporting deferred.
    }

    /// Enter To phrases into the excerpt parser in logical order.
    ///
    /// Corresponds to `ToPhraseFamily::register` (the `REGISTER_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 285-293).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Walk the `first_in_logical_order` linked list
    /// 2. Call `ParseInvocations::register_excerpt(id->body_of_defn)` for each
    /// 3. Assign each a `sequence_count` from 0 upward
    fn register(_self: &ImpDefFamily) {
        // No-op: excerpt-parser registration deferred.
    }

    /// Insert the body into the logical-order linked list.
    ///
    /// Corresponds to `ToPhraseFamily::given_body` (the
    /// `GIVEN_BODY_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 193-219).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Call `CompileImperativeDefn::prepare_for_requests(body)`
    /// 2. Parse the prototype text via `ParsingIDTypeData::parse`
    /// 3. Insert `id` into the `first_in_logical_order` list at the position
    ///    determined by `ToPhraseFamily::cmp` (lines 245-278)
    fn given_body(_self: &ImpDefFamily) {
        // No-op: body preparation and logical-order insertion deferred.
    }

    /// Whether the body may be given as `(- ... -)` inline I6 material.
    ///
    /// Corresponds to `ToPhraseFamily::allows_inline` (the
    /// `ALLOWS_INLINE_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 388-390).
    ///
    /// To phrases always allow inline I6 bodies.
    fn allows_inline(_self: &ImpDefFamily) -> bool {
        true
    }

    /// Make compilation requests for the To phrase bodies.
    ///
    /// Corresponds to `ToPhraseFamily::compile` (the `COMPILE_IMP_DEFN_MTID`
    /// method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 299-304).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Mark To phrases with definite kinds for future compilation via
    ///    `PhraseRequests::simple_request` (lines 306-313)
    /// 2. Throw `PM_ReturnKindVague` / `PM_ReturnKindUndetermined` problems for
    ///    phrases with return kinds too vaguely defined (lines 315-349)
    /// 3. Throw `PM_NamedInline` problems for inline phrases named as
    ///    constants (lines 351-365)
    fn compile(_self: &ImpDefFamily, _total: &mut i32, _target: i32) {
        // No-op: compilation requests and problem reporting deferred.
    }

    /// Whether definitions in this family appear in the Phrasebook index.
    ///
    /// Corresponds to `ToPhraseFamily::include_in_Phrasebook_index` (the
    /// `PHRASEBOOK_INDEX_IMP_DEFN_MTID` method) in the C reference
    /// (`inform7/assertions-module/Chapter 5/To Phrase Family.w`, lines 392-395).
    ///
    /// To phrases always appear in the Phrasebook index.
    fn phrasebook_index(_self: &ImpDefFamily) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        ImperativeDefinitionFamilies, BUILTIN_IMP_DEFN_FAMILIES,
    };

    /// Test that `wire_methods` installs all seven methods on a family.
    #[test]
    fn wire_methods_installs_all_seven_methods() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        assert!(family.methods.identify.is_some(), "identify should be installed");
        assert!(family.methods.assess.is_some(), "assess should be installed");
        assert!(family.methods.register.is_some(), "register should be installed");
        assert!(family.methods.given_body.is_some(), "given_body should be installed");
        assert!(family.methods.allows_inline.is_some(), "allows_inline should be installed");
        assert!(family.methods.compile.is_some(), "compile should be installed");
        assert!(family.methods.phrasebook_index.is_some(), "phrasebook_index should be installed");
    }

    /// Test that `allows_inline` returns `true`.
    #[test]
    fn allows_inline_returns_true() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        let result = ImperativeDefinitionFamilies::allows_inline(&family);
        assert!(result, "allows_inline should return true for To phrases");
    }

    /// Test that `phrasebook_index` returns `true`.
    #[test]
    fn phrasebook_index_returns_true() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        let result = ImperativeDefinitionFamilies::phrasebook_index(&family);
        assert!(result, "phrasebook_index should return true for To phrases");
    }

    /// Test that `identify` does not panic when dispatched.
    #[test]
    fn identify_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::identify(&family);
    }

    /// Test that `assess` does not panic when dispatched.
    #[test]
    fn assess_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::assess(&family);
    }

    /// Test that `register` does not panic when dispatched.
    #[test]
    fn register_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::register(&family);
    }

    /// Test that `given_body` does not panic when dispatched.
    #[test]
    fn given_body_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::given_body(&family);
    }

    /// Test that `compile` does not modify `total` (no-op stub).
    #[test]
    fn compile_does_not_modify_total() {
        let mut family = ImpDefFamily::new("test", false);
        ToPhraseFamily::wire_methods(&mut family);

        let mut total = 42;
        ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
        assert_eq!(total, 42, "compile should not modify total (no-op stub)");
    }

    /// Test that the builtin `TO_PHRASE_EFF` family has all seven methods wired.
    #[test]
    fn builtin_family_has_methods_wired() {
        // Force initialization of the builtin families
        ImperativeDefinitionFamilies::create();
        let family = &BUILTIN_IMP_DEFN_FAMILIES[2];

        assert_eq!(family.name, "TO_PHRASE_EFF");
        assert!(family.methods.identify.is_some(), "TO_PHRASE_EFF should have identify installed");
        assert!(family.methods.assess.is_some(), "TO_PHRASE_EFF should have assess installed");
        assert!(family.methods.register.is_some(), "TO_PHRASE_EFF should have register installed");
        assert!(family.methods.given_body.is_some(), "TO_PHRASE_EFF should have given_body installed");
        assert!(family.methods.allows_inline.is_some(), "TO_PHRASE_EFF should have allows_inline installed");
        assert!(family.methods.compile.is_some(), "TO_PHRASE_EFF should have compile installed");
        assert!(family.methods.phrasebook_index.is_some(), "TO_PHRASE_EFF should have phrasebook_index installed");
    }
}