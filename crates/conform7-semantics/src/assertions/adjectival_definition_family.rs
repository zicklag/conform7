//! Adjectival Definition Family — imperative definitions of "Definition: X is Y: ..." adjectives.
//!
//! Corresponds to `AdjectivalDefinitionFamily` in the C reference
//! (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`).
//!
//! This family is used for adjective definitions, whether or not they run on
//! into substantial amounts of code. It wires four methods into the
//! `adjectival-idf` imperative definition family:
//!
//! - `identify` — decide whether a definition preamble belongs to this family
//! - `given_body` — set up the body of a definition
//! - `allows_empty` — whether the body is allowed to be empty
//! - `compile` — compile the definition body
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types — method signatures take `&ImpDefFamily` only.
//! - No `AdjectivesByPhrase::define_adjective_by_phrase` call in `given_body`.
//! - No `RTAdjectives` or `CompileImperativeDefn` calls in `compile`.
//! - No `parse_node` handling or Preform grammar.
//! - No problem messages.

use crate::assertions::imperative_definition_families::ImpDefFamily;

/// The Adjectival Definition Family module.
///
/// Corresponds to `AdjectivalDefinitionFamily` in the C reference
/// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`).
pub struct AdjectivalDefinitionFamily;

impl AdjectivalDefinitionFamily {
    /// Wire the four adjectival-family methods into the given family.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::create_family` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 11-17).
    ///
    /// Simplified: all methods are stubs. The full logic (preamble matching,
    /// body setup, definition iteration, I6 compilation) is deferred until
    /// `imperative_defn`, `id_body`, and `parse_node` types are introduced.
    pub fn wire_methods(family: &mut ImpDefFamily) {
        family.methods.identify = Some(Self::identify);
        family.methods.given_body = Some(Self::given_body);
        family.methods.allows_empty = Some(Self::allows_empty);
        family.methods.compile = Some(Self::compile);
    }

    /// Decide whether a definition preamble belongs to the adjectival family.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::identify` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 48-59).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Check if the preamble text matches `<definition-preamble>` ("definition")
    /// 2. Set `id->family = adjectival_idf`
    /// 3. Handle continuation-node rewriting (`IMPERATIVE_NT` -> `DEFN_CONT_NT`)
    /// 4. Call `look_for_headers` to parse and register the adjective definition
    fn identify(_self: &ImpDefFamily) {
        // No-op: full preamble matching and node manipulation deferred.
    }

    /// Set up the body of an adjectival definition.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::given_body` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 76-86).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Set `body->type_data` to `DECIDES_CONDITION_MOR`
    /// 2. Call `AdjectivesByPhrase::define_adjective_by_phrase` to parse the body
    /// 3. Enable the `it` pronoun via `Frames::enable_it`
    fn given_body(_self: &ImpDefFamily) {
        // No-op: body setup and phrase parsing deferred.
    }

    /// Whether the body is allowed to be empty for adjectival definitions.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::allows_empty` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 65-67).
    ///
    /// Adjectival definitions always allow empty bodies because the code may
    /// be under a continuation node (`DEFN_CONT_NT`) rather than directly
    /// under the `Definition:` node.
    fn allows_empty(_self: &ImpDefFamily) -> bool {
        true
    }

    /// Compile the adjectival definition bodies.
    ///
    /// Corresponds to `AdjectivalDefinitionFamily::compile` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Adjectival Definition Family.w`, lines 91-101).
    ///
    /// Simplified: no-op stub. The full implementation will:
    /// 1. Loop over all `imperative_defn` objects belonging to `adjectival_idf`
    /// 2. Call `RTAdjectives::make_adjective_phrase_package` for each
    /// 3. Call `CompileImperativeDefn::not_from_phrase` for each
    /// 4. Call `RTAdjectives::compile()` at the end
    fn compile(_self: &ImpDefFamily, _total: &mut i32, _target: i32) {
        // No-op: definition iteration and I6 compilation deferred.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::imperative_definition_families::{
        ImperativeDefinitionFamilies, BUILTIN_IMP_DEFN_FAMILIES,
    };

    /// Test that `wire_methods` installs all four methods on a family.
    #[test]
    fn wire_methods_installs_all_four_methods() {
        let mut family = ImpDefFamily::new("test", false);
        AdjectivalDefinitionFamily::wire_methods(&mut family);

        assert!(family.methods.identify.is_some(), "identify should be installed");
        assert!(family.methods.given_body.is_some(), "given_body should be installed");
        assert!(family.methods.allows_empty.is_some(), "allows_empty should be installed");
        assert!(family.methods.compile.is_some(), "compile should be installed");
    }

    /// Test that `allows_empty` returns `true`.
    #[test]
    fn allows_empty_returns_true() {
        let mut family = ImpDefFamily::new("test", false);
        AdjectivalDefinitionFamily::wire_methods(&mut family);

        let result = ImperativeDefinitionFamilies::allows_empty(&family);
        assert!(result, "allows_empty should return true for adjectival definitions");
    }

    /// Test that `identify` does not panic when dispatched.
    #[test]
    fn identify_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        AdjectivalDefinitionFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::identify(&family);
    }

    /// Test that `given_body` does not panic when dispatched.
    #[test]
    fn given_body_does_not_panic() {
        let mut family = ImpDefFamily::new("test", false);
        AdjectivalDefinitionFamily::wire_methods(&mut family);

        // Should not panic
        ImperativeDefinitionFamilies::given_body(&family);
    }

    /// Test that `compile` does not modify `total` (no-op stub).
    #[test]
    fn compile_does_not_modify_total() {
        let mut family = ImpDefFamily::new("test", false);
        AdjectivalDefinitionFamily::wire_methods(&mut family);

        let mut total = 42;
        ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
        assert_eq!(total, 42, "compile should not modify total (no-op stub)");
    }

    /// Test that the builtin `adjectival-idf` family has methods wired.
    #[test]
    fn builtin_family_has_methods_wired() {
        // Force initialization of the builtin families
        ImperativeDefinitionFamilies::create();
        let family = &BUILTIN_IMP_DEFN_FAMILIES[1];

        assert_eq!(family.name, "adjectival-idf");
        assert!(family.methods.identify.is_some(), "adjectival-idf should have identify installed");
        assert!(family.methods.given_body.is_some(), "adjectival-idf should have given_body installed");
        assert!(family.methods.allows_empty.is_some(), "adjectival-idf should have allows_empty installed");
        assert!(family.methods.compile.is_some(), "adjectival-idf should have compile installed");
    }
}
