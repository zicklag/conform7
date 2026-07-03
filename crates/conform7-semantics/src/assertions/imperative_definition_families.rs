//! Imperative Definition Families — dispatch layer for imperative definitions.
//!
//! Corresponds to `ImperativeDefinitionFamilies` in the C reference
//! (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
//!
//! This module defines the family dispatch infrastructure for different
//! categories of imperative definition in Inform 7: adjectives (`Definition: ...`),
//! To phrases (`To ...`), and rules (`Every turn: ...`, `Instead of ...`).
//!
//! Simplified:
//! - No `imperative_defn` or `id_body` types yet — method slots take `&ImpDefFamily` only.
//! - No `id_runtime_context_data` — `to_rcd` is a no-op.
//! - No Preform grammar or Salsa integration.
//! - No problem messages or I6 compilation.

use std::sync::LazyLock;

/// Methods that can be implemented for an imperative definition family.
///
/// Corresponds to the method IDs in the C reference
/// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
///
/// All methods are optional — the default implementations are no-ops or
/// return `false` as appropriate.
///
/// Simplified: method signatures take `&ImpDefFamily` only. They will be
/// expanded to include `&ImperativeDefn`, `&mut IdBody`, `&mut RuntimeContextData`,
/// etc., when those types are introduced.
#[derive(Clone, Debug, Default)]
pub struct ImpDefFamilyMethods {
    /// Decide whether a definition preamble belongs to this family.
    /// Corresponds to `IDENTIFY_IMP_DEFN_MTID`.
    pub identify: Option<fn(&ImpDefFamily)>,
    /// Parse the preamble in more detail.
    /// Corresponds to `ASSESS_IMP_DEFN_MTID`.
    pub assess: Option<fn(&ImpDefFamily)>,
    /// Called after the definition body has been created.
    /// Corresponds to `GIVEN_BODY_IMP_DEFN_MTID`.
    pub given_body: Option<fn(&ImpDefFamily)>,
    /// Called after all assessments/bodies are registered.
    /// Corresponds to `REGISTER_IMP_DEFN_MTID`.
    pub register: Option<fn(&ImpDefFamily)>,
    /// Provide runtime context data for the body.
    /// Corresponds to `TO_RCD_IMP_DEFN_MTID`.
    pub to_rcd: Option<fn(&ImpDefFamily)>,
    /// Called when assessment is complete for all definitions.
    /// Corresponds to `ASSESSMENT_COMPLETE_IMP_DEFN_MTID`.
    pub assessment_complete: Option<fn(&ImpDefFamily)>,
    /// Whether phrases that end rules/rulebooks may be used in the body.
    /// Corresponds to `ALLOWS_RULE_ONLY_PHRASES_IMP_DEFN_MTID`.
    pub allows_rule_only_phrases: Option<fn(&ImpDefFamily) -> bool>,
    /// Whether the body is allowed to be empty.
    /// Corresponds to `ALLOWS_EMPTY_IMP_DEFN_MTID`.
    pub allows_empty: Option<fn(&ImpDefFamily) -> bool>,
    /// Whether the body can be given as `(- ... -)` inline I6 material.
    /// Corresponds to `ALLOWS_INLINE_IMP_DEFN_MTID`.
    pub allows_inline: Option<fn(&ImpDefFamily) -> bool>,
    /// Main compilation round for resources needed by the family.
    /// Corresponds to `COMPILE_IMP_DEFN_MTID`.
    pub compile: Option<fn(&ImpDefFamily, &mut i32, i32)>,
    /// Whether definitions in this family should appear in the Phrasebook index.
    /// Corresponds to `PHRASEBOOK_INDEX_IMP_DEFN_MTID`.
    pub phrasebook_index: Option<fn(&ImpDefFamily) -> bool>,
}

/// A family of related imperative definitions.
///
/// Corresponds to `imperative_defn_family` in the C reference
/// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
///
/// Each family represents a category of imperative definition (adjectives,
/// To phrases, rules) with method dispatch for identification, assessment,
/// registration, and compilation.
#[derive(Clone, Debug)]
pub struct ImpDefFamily {
    /// Name of this family (for debugging and identification).
    pub name: &'static str,
    /// Method implementations for this family.
    pub methods: ImpDefFamilyMethods,
    /// Whether this family should be compiled last.
    pub compile_last: bool,
}

impl ImpDefFamily {
    /// Create a new imperative definition family with default (no-op) methods.
    ///
    /// Corresponds to `ImperativeDefinitionFamilies::new` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
    pub fn new(name: &'static str, compile_last: bool) -> Self {
        ImpDefFamily {
            name,
            methods: ImpDefFamilyMethods::default(),
            compile_last,
        }
    }
}

/// Management functions for imperative definition families.
///
/// Corresponds to `ImperativeDefinitionFamilies` in the C reference
/// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
pub struct ImperativeDefinitionFamilies;

impl ImperativeDefinitionFamilies {
    /// Create a new family with the given name and compile-last flag.
    ///
    /// Corresponds to `ImperativeDefinitionFamilies::new` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
    pub fn new_family(name: &'static str, compile_last: bool) -> ImpDefFamily {
        ImpDefFamily::new(name, compile_last)
    }

    /// Force initialization of the built-in family registry.
    ///
    /// Corresponds to `ImperativeDefinitionFamilies::create` in the C reference
    /// (`inform7/assertions-module/Chapter 5/Imperative Definition Families.w`).
    ///
    /// In the C version, this creates the four built-in families in order:
    /// unknown-idf, adjectival-idf, TO_PHRASE_EFF, rule-idf.
    /// The order matters: rule-idf must come last because its grammar
    /// ends with a catch-all `...` production.
    pub fn create() {
        LazyLock::<Vec<ImpDefFamily>>::force(&BUILTIN_IMP_DEFN_FAMILIES);
    }

    /// Assertions-module startup hook.
    ///
    /// Corresponds to the `ImperativeDefinitionFamilies::create()` call in
    /// `AssertionsModule::start` in the C reference
    /// (`inform7/assertions-module/Chapter 1/Assertions Module.w`, line 33).
    pub fn start() {
        Self::create();
    }

    /// Dispatch to the family's identify method.
    pub fn identify(family: &ImpDefFamily) {
        if let Some(f) = family.methods.identify {
            f(family);
        }
    }

    pub fn assess(family: &ImpDefFamily) {
        if let Some(f) = family.methods.assess {
            f(family);
        }
    }

    pub fn given_body(family: &ImpDefFamily) {
        if let Some(f) = family.methods.given_body {
            f(family);
        }
    }

    pub fn register(family: &ImpDefFamily) {
        if let Some(f) = family.methods.register {
            f(family);
        }
    }

    pub fn to_rcd(family: &ImpDefFamily) {
        if let Some(f) = family.methods.to_rcd {
            f(family);
        }
    }

    pub fn assessment_complete(family: &ImpDefFamily) {
        if let Some(f) = family.methods.assessment_complete {
            f(family);
        }
    }

    /// Dispatch to the family's allows_rule_only_phrases method.
    ///
    /// Returns `false` if the family has no method installed.
    pub fn allows_rule_only_phrases(family: &ImpDefFamily) -> bool {
        family.methods.allows_rule_only_phrases.map(|f| f(family)).unwrap_or(false)
    }

    /// Dispatch to the family's allows_empty method.
    ///
    /// Returns `false` if the family has no method installed.
    pub fn allows_empty(family: &ImpDefFamily) -> bool {
        family.methods.allows_empty.map(|f| f(family)).unwrap_or(false)
    }

    /// Dispatch to the family's allows_inline method.
    ///
    /// Returns `false` if the family has no method installed.
    pub fn allows_inline(family: &ImpDefFamily) -> bool {
        family.methods.allows_inline.map(|f| f(family)).unwrap_or(false)
    }

    pub fn compile(family: &ImpDefFamily, total: &mut i32, target: i32) {
        if let Some(f) = family.methods.compile {
            f(family, total, target);
        }
    }

    /// Dispatch to the family's phrasebook_index method.
    ///
    /// Returns `false` if the family has no method installed.
    pub fn phrasebook_index(family: &ImpDefFamily) -> bool {
        family.methods.phrasebook_index.map(|f| f(family)).unwrap_or(false)
    }
}

/// The built-in imperative definition families registry.
///
/// Created in the C-mandated order:
/// 1. `unknown-idf` — placeholder for unclassified definitions
/// 2. `adjectival-idf` — adjectival definitions (`Definition: ...`)
/// 3. `TO_PHRASE_EFF` — To phrase definitions (`To ...`)
/// 4. `rule-idf` — rule definitions (`Every turn: ...`, `Instead of ...`)
///
/// The order matters: `rule-idf` must come last because
/// `ImperativeDefinitionFamilies::identify` iterates the list in creation
/// order and the rule family claims anything not already claimed.
pub static BUILTIN_IMP_DEFN_FAMILIES: LazyLock<Vec<ImpDefFamily>> = LazyLock::new(|| {
    vec![
        ImpDefFamily::new("unknown-idf", false),
        ImpDefFamily::new("adjectival-idf", false),
        ImpDefFamily::new("TO_PHRASE_EFF", true),
        ImpDefFamily::new("rule-idf", false),
    ]
});

/// Return a reference to the `unknown-idf` built-in family.
///
/// Forces initialization of the registry if not yet initialized.
pub fn unknown_idf() -> &'static ImpDefFamily {
    &BUILTIN_IMP_DEFN_FAMILIES[0]
}

/// Return a reference to the `adjectival-idf` built-in family.
///
/// Forces initialization of the registry if not yet initialized.
pub fn adjectival_idf() -> &'static ImpDefFamily {
    &BUILTIN_IMP_DEFN_FAMILIES[1]
}

/// Return a reference to the `TO_PHRASE_EFF` built-in family.
///
/// Forces initialization of the registry if not yet initialized.
pub fn to_phrase_idf() -> &'static ImpDefFamily {
    &BUILTIN_IMP_DEFN_FAMILIES[2]
}

/// Return a reference to the `rule-idf` built-in family.
///
/// Forces initialization of the registry if not yet initialized.
pub fn rule_idf() -> &'static ImpDefFamily {
    &BUILTIN_IMP_DEFN_FAMILIES[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that `new_family` creates a family with the given name and compile-last flag.
    #[test]
    fn new_family_creates_family_with_name_and_flag() {
        let family = ImperativeDefinitionFamilies::new_family("test-family", true);
        assert_eq!(family.name, "test-family");
        assert!(family.compile_last);

        let family2 = ImperativeDefinitionFamilies::new_family("another-family", false);
        assert_eq!(family2.name, "another-family");
        assert!(!family2.compile_last);
    }

    /// Test that all method slots default to `None`.
    #[test]
    fn default_methods_are_none() {
        let family = ImpDefFamily::new("test", false);
        assert!(family.methods.identify.is_none());
        assert!(family.methods.assess.is_none());
        assert!(family.methods.given_body.is_none());
        assert!(family.methods.register.is_none());
        assert!(family.methods.to_rcd.is_none());
        assert!(family.methods.assessment_complete.is_none());
        assert!(family.methods.allows_rule_only_phrases.is_none());
        assert!(family.methods.allows_empty.is_none());
        assert!(family.methods.allows_inline.is_none());
        assert!(family.methods.compile.is_none());
        assert!(family.methods.phrasebook_index.is_none());
    }

    /// Test that `create` initializes the built-in families in the correct order
    /// with the correct flags.
    #[test]
    fn create_initializes_builtin_families_in_order() {
        ImperativeDefinitionFamilies::create();
        let families = &*BUILTIN_IMP_DEFN_FAMILIES;
        assert_eq!(families.len(), 4);
        assert_eq!(families[0].name, "unknown-idf");
        assert!(!families[0].compile_last);
        assert_eq!(families[1].name, "adjectival-idf");
        assert!(!families[1].compile_last);
        assert_eq!(families[2].name, "TO_PHRASE_EFF");
        assert!(families[2].compile_last);
        assert_eq!(families[3].name, "rule-idf");
        assert!(!families[3].compile_last);
    }

    /// Test that `start` runs without panic and initializes the registry.
    #[test]
    fn start_runs_without_panic() {
        ImperativeDefinitionFamilies::start();
        let families = &*BUILTIN_IMP_DEFN_FAMILIES;
        assert_eq!(families.len(), 4);
    }

    /// Test that accessor functions return the correct built-in families.
    #[test]
    fn accessors_return_correct_families() {
        // Force initialization
        ImperativeDefinitionFamilies::create();

        assert_eq!(unknown_idf().name, "unknown-idf");
        assert!(!unknown_idf().compile_last);

        assert_eq!(adjectival_idf().name, "adjectival-idf");
        assert!(!adjectival_idf().compile_last);

        assert_eq!(to_phrase_idf().name, "TO_PHRASE_EFF");
        assert!(to_phrase_idf().compile_last);

        assert_eq!(rule_idf().name, "rule-idf");
        assert!(!rule_idf().compile_last);
    }

    /// Test that dispatch helpers call installed methods.
    #[test]
    fn dispatch_helpers_call_installed_methods() {
        use std::cell::Cell;
        thread_local! {
            static IDENTIFY_CALLED: Cell<bool> = const { Cell::new(false) };
            static ALLOWS_EMPTY_CALLED: Cell<bool> = const { Cell::new(false) };
            static COMPILE_CALLED: Cell<bool> = const { Cell::new(false) };
        }

        fn identify_fn(_: &ImpDefFamily) { IDENTIFY_CALLED.with(|c| c.set(true)); }
        fn allows_empty_fn(_: &ImpDefFamily) -> bool { ALLOWS_EMPTY_CALLED.with(|c| c.set(true)); true }
        fn compile_fn(_: &ImpDefFamily, total: &mut i32, target: i32) { COMPILE_CALLED.with(|c| c.set(true)); *total += target; }

        // Test a void method: identify
        {
            let mut family = ImpDefFamily::new("test", false);
            family.methods.identify = Some(identify_fn);
            ImperativeDefinitionFamilies::identify(&family);
        }
        assert!(IDENTIFY_CALLED.with(|c| c.get()), "identify method should have been called");

        // Test a bool-returning method: allows_empty
        {
            let mut family = ImpDefFamily::new("test", false);
            family.methods.allows_empty = Some(allows_empty_fn);
            let result = ImperativeDefinitionFamilies::allows_empty(&family);
            assert!(ALLOWS_EMPTY_CALLED.with(|c| c.get()), "allows_empty method should have been called");
            assert!(result, "allows_empty should return the installed method's value");
        }

        // Test compile with arguments
        {
            let mut family = ImpDefFamily::new("test", false);
            family.methods.compile = Some(compile_fn);
            let mut total = 10;
            ImperativeDefinitionFamilies::compile(&family, &mut total, 5);
            assert!(COMPILE_CALLED.with(|c| c.get()), "compile method should have been called");
            assert_eq!(total, 15, "compile should have modified total");
        }
    }

    /// Test that dispatch helpers return safe defaults when no method is installed.
    #[test]
    fn dispatch_helpers_return_defaults_when_unset() {
        let family = ImpDefFamily::new("test", false);

        // Void methods should not panic
        ImperativeDefinitionFamilies::identify(&family);
        ImperativeDefinitionFamilies::assess(&family);
        ImperativeDefinitionFamilies::given_body(&family);
        ImperativeDefinitionFamilies::register(&family);
        ImperativeDefinitionFamilies::to_rcd(&family);
        ImperativeDefinitionFamilies::assessment_complete(&family);

        // Bool-returning methods should return false
        assert!(!ImperativeDefinitionFamilies::allows_rule_only_phrases(&family));
        assert!(!ImperativeDefinitionFamilies::allows_empty(&family));
        assert!(!ImperativeDefinitionFamilies::allows_inline(&family));
        assert!(!ImperativeDefinitionFamilies::phrasebook_index(&family));

        // compile should not modify total
        let mut total = 42;
        ImperativeDefinitionFamilies::compile(&family, &mut total, 10);
        assert_eq!(total, 42, "compile should not modify total when unset");
    }
}
