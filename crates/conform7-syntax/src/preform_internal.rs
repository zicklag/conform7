//! Internal nonterminal implementations for the Preform matching engine.
//!
//! This module provides Rust implementations of the `internal` nonterminals
//! declared in `Syntax.preform`. Each internal NT is a struct that implements
//! the [`InternalNonterminal`] trait, and they are registered in an
//! [`InternalRegistry`] under their Preform name (without angle brackets).
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 4/Basic Nonterminals.w` —
//!   the three simple internals `<if-start-of-paragraph>`, `<if-not-cap>`,
//!   and `<preform-nonterminal>`.
//! - C reference: `services/words-module/Chapter 4/Nonterminals.w` — the
//!   `INTERNAL_NONTERMINAL` macro and `internal_definition` function pointer.
//! - C reference: `services/words-module/Chapter 4/Preform.w` — how
//!   `Preform::parse_nt_against_word_range` dispatches to internal NTs.

use crate::preform::{InternalNonterminal, InternalPayload, InternalRegistry, InternalResult, PreformContext};
use crate::Wording;

// ---------------------------------------------------------------------------
// <if-start-of-paragraph>
// ---------------------------------------------------------------------------

/// Zero-width internal nonterminal that succeeds when the first word of the
/// wording is at the start of a paragraph.
///
/// A word is at the start of a paragraph if it is the first word of the
/// source text (word index 0) or if the preceding word is a paragraph break.
///
/// In the C reference, this is implemented in
/// `services/words-module/Chapter 4/Basic Nonterminals.w`:
///
/// ```c
/// <if-start-of-paragraph> internal 0 {
///     int w1 = Wordings::first_wn(W);
///     if ((w1 == 0) || (compare_word(w1-1, PARBREAK_V))) return TRUE;
///     ==> { fail nonterminal };
/// }
/// ```
///
/// Our implementation uses `ctx.is_paragraph_start` which must be set by the
/// caller based on the token stream's paragraph break information.
#[derive(Clone, Debug)]
pub struct IfStartOfParagraph;

impl InternalNonterminal for IfStartOfParagraph {
    fn match_nonterminal(&self, ctx: &PreformContext, _wording: Wording) -> Option<InternalResult> {
        if ctx.is_paragraph_start {
            Some(InternalResult {
                payload: InternalPayload::None,
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// <if-not-cap>
// ---------------------------------------------------------------------------

/// Zero-width internal nonterminal that succeeds when the first word of the
/// wording does not begin with an upper-case letter.
///
/// In the C reference, this is implemented in
/// `services/words-module/Chapter 4/Basic Nonterminals.w`:
///
/// ```c
/// <if-not-cap> internal 0 {
///     int w1 = Wordings::first_wn(W);
///     if (Word::unexpectedly_upper_case(w1) == FALSE) return TRUE;
///     ==> { fail nonterminal };
/// }
/// ```
///
/// The C routine `Word::unexpectedly_upper_case` (Numbered Words.w:75-86)
/// treats upper-case as EXPECTED (returns FALSE) in these positions:
///
/// - When `wn < 1` (before the first word)
/// - After a fullstop (`FULLSTOP_V`)
/// - After a paragraph break (`PARBREAK_V`)
/// - After a colon (`COLON_V`)
/// - After a quoted text that ends a sentence (`Word::text_ending_sentence(wn-1)`)
///
/// In all other positions, an upper-case initial letter IS unexpected
/// (returns TRUE), so `<if-not-cap>` fails.
///
/// **Simplification:** The Rust stand-in lacks all of this position-based
/// logic. It simply checks if the first character of the first word is an
/// upper-case ASCII letter. This means it will wrongly reject capitalized
/// words at sentence starts (e.g., "The cat" after a fullstop), whereas C
/// would allow them. When the vocabulary system is implemented, this should
/// be updated to use the full `Word::unexpectedly_upper_case` logic.
#[derive(Clone, Debug)]
pub struct IfNotCap;
impl InternalNonterminal for IfNotCap {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        // In C, a zero-width internal (min_words==max_words==0) is invoked with
        // an EMPTY wording `Wordings::new(wn, wn-1)`, but `Wordings::first_wn(W)`
        // still returns `wn` (the next unconsumed word). So we peek at the word
        // at `wording.start` WITHOUT consuming it.
        //
        // C reference: `services/words-module/Chapter 4/Basic Nonterminals.w`:
        //   <if-not-cap> internal 0 {
        //       int w1 = Wordings::first_wn(W);
        //       if (Word::unexpectedly_upper_case(w1) == FALSE) return TRUE;
        //       ==> { fail nonterminal };
        //   }
        //
        // `Wordings::first_wn(W)` for an empty wording `Wordings::new(wn, wn-1)`
        // returns `wn`, so we use `wording.start` as the position.
        if (wording.start as usize) >= ctx.word_text.len() {
            return None;
        }
        let first_word = ctx.word_text[wording.start as usize];
        let is_upper = first_word.chars().next().is_some_and(|c| c.is_uppercase());
        if !is_upper {
            Some(InternalResult {
                payload: InternalPayload::None,
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// <preform-nonterminal>
// ---------------------------------------------------------------------------

/// Internal nonterminal that matches a single source word that is the exact
/// name of a declared nonterminal (e.g., `<foo>`) and returns the nonterminal's
/// name as the payload.
///
/// In the C reference, this is implemented in
/// `services/words-module/Chapter 4/Basic Nonterminals.w`:
///
/// ```c
/// <preform-nonterminal> internal 1 {
///     nonterminal *nt = Nonterminals::detect(Lexer::word(Wordings::first_wn(W)));
///     if (nt) { ==> { -, nt }; return TRUE; }
///     ==> { fail nonterminal };
/// }
/// ```
///
/// Our implementation checks if the word looks like `<name>` (starts with `<`
/// and ends with `>`) and if `name` exists in the grammar's nonterminals.
/// On success, it consumes exactly one word and returns the nonterminal name.
#[derive(Clone, Debug)]
pub struct PreformNonterminal;

impl InternalNonterminal for PreformNonterminal {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        if wording.len() != 1 {
            return None;
        }
        let word = ctx.word_text[wording.start as usize];
        // Check if the word looks like <name>.
        if word.starts_with('<') && word.ends_with('>') && word.len() > 2 {
            let inner = &word[1..word.len() - 1];
            // Check if the name exists in the grammar.
            if ctx.grammar.nonterminals.iter().any(|n| n.name == inner) {
                return Some(InternalResult {
                    payload: InternalPayload::Nonterminal(inner.to_string()),
                });
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// InternalRegistry::new() (basic internals)
// ---------------------------------------------------------------------------

impl InternalRegistry {
    /// Create an [`InternalRegistry`] pre-populated with the three basic
    /// internal nonterminals: `if-start-of-paragraph`, `if-not-cap`, and
    /// `preform-nonterminal`.
    ///
    /// These are the names used in the real `Syntax.preform` grammar file.
    pub fn basic() -> Self {
        let mut registry = InternalRegistry::empty();
        registry.register("if-start-of-paragraph", Box::new(IfStartOfParagraph));
        registry.register("if-not-cap", Box::new(IfNotCap));
        registry.register("preform-nonterminal", Box::new(PreformNonterminal));
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preform::{parse_preform_grammar, ProductionTokenCategory};
    use crate::{match_nonterminal_impl, Wording};

    // -----------------------------------------------------------------------
    // <if-start-of-paragraph> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_if_start_of_paragraph_at_start() {
        let grammar = parse_preform_grammar("<if-start-of-paragraph> internal").unwrap();
        let words = &["hello", "world"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: true, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-start-of-paragraph", Wording::new(0, 2));
        assert!(m.is_some(), "should match at paragraph start");
        assert_eq!(m.unwrap().internal.unwrap().payload, InternalPayload::None);
    }

    #[test]
    fn test_if_start_of_paragraph_fails_mid_paragraph() {
        let grammar = parse_preform_grammar("<if-start-of-paragraph> internal").unwrap();
        let words = &["hello", "world"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-start-of-paragraph", Wording::new(0, 2));
        assert!(m.is_none(), "should fail mid-paragraph");
    }

    // -----------------------------------------------------------------------
    // <if-not-cap> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_if_not_cap_lowercase() {
        let grammar = parse_preform_grammar("<if-not-cap> internal").unwrap();
        let words = &["apple", "the"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(0, 1));
        assert!(m.is_some(), "'apple' should match if-not-cap");
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(1, 2));
        assert!(m.is_some(), "'the' should match if-not-cap");
    }

    #[test]
    fn test_if_not_cap_uppercase() {
        let grammar = parse_preform_grammar("<if-not-cap> internal").unwrap();
        let words = &["Apple", "The"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(0, 1));
        assert!(m.is_none(), "'Apple' should fail if-not-cap");
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(1, 2));
        assert!(m.is_none(), "'The' should fail if-not-cap");
    }

    #[test]
    fn test_if_not_cap_empty() {
        let grammar = parse_preform_grammar("<if-not-cap> internal").unwrap();
        let words: &[&str] = &[];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::EMPTY);
        assert!(m.is_none(), "empty wording should fail if-not-cap");
    }

    #[test]
    fn test_if_not_cap_zero_width_with_indefinite_article() {
        // Tests that `<if-not-cap>` succeeds at zero width on lowercase "a",
        // leaving "a" for the following `<indefinite-article>` token.
        // This matches the C semantics where a zero-width internal is invoked
        // with an empty wording but peeks at the next unconsumed word.
        let source = concat!(
            "<if-not-cap> internal\n",
            "<indefinite-article> ::= a | an\n",
            "<test> ::= <if-not-cap> <indefinite-article> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["a", "cat"];

        // if-not-cap at zero width (wording.start=0, wording.end=0) should
        // peek at word 0 ("a") and succeed because "a" is lowercase.
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(0, 0));
        assert!(m.is_some(), "if-not-cap should succeed at zero width on lowercase 'a'");

        // The full production `<if-not-cap> <indefinite-article> cat` should
        // match "a cat": if-not-cap succeeds at zero width (peeking at "a"),
        // then indefinite-article consumes "a", then "cat" matches.
        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_some(), "<if-not-cap> <indefinite-article> cat should match 'a cat'");
    }

    // -----------------------------------------------------------------------
    // <preform-nonterminal> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_preform_nonterminal_matches_declared() {
        let source = "<preform-nonterminal> internal\n<foo> internal\n<bar> ::= x";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["<foo>", "<bar>"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();

        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 1));
        assert!(m.is_some(), "<foo> should match when <foo> is declared");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Nonterminal("foo".to_string())
        );

        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(1, 2));
        assert!(m.is_some(), "<bar> should match when <bar> is declared");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Nonterminal("bar".to_string())
        );
    }

    #[test]
    fn test_preform_nonterminal_fails_undeclared() {
        let source = "<preform-nonterminal> internal\n<foo> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["<baz>"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 1));
        assert!(m.is_none(), "<baz> should fail when not declared");
    }

    #[test]
    fn test_preform_nonterminal_fails_plain_word() {
        let source = "<preform-nonterminal> internal\n<foo> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["foo"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 1));
        assert!(m.is_none(), "'foo' should fail (not in <name> format)");
    }

    #[test]
    fn test_preform_nonterminal_fails_multi_word() {
        let source = "<preform-nonterminal> internal\n<foo> internal\n<bar> ::= x";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["<foo>", "<bar>"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 2));
        assert!(m.is_none(), "should fail on multi-word wording");
    }

    // -----------------------------------------------------------------------
    // Integration test: <dividing-sentence> with <if-start-of-paragraph>
    // -----------------------------------------------------------------------

    #[test]
    fn test_dividing_sentence_with_paragraph_start() {
        let source = concat!(
            "<if-start-of-paragraph> internal\n",
            "<heading> ::= chapter ... | section ...\n",
            "<dividing-sentence> ::= <if-start-of-paragraph> <heading>\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["chapter", "1", "-", "X"];

        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: true, verbs_registry: None };
        let registry = InternalRegistry::new();
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_some(), "dividing-sentence should match at paragraph start");

        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_none(), "dividing-sentence should fail without paragraph start");
    }
    // -----------------------------------------------------------------------
    // Real grammar oracle test
    // -----------------------------------------------------------------------

    #[test]
    fn test_real_syntax_preform_internal_nts() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"if-start-of-paragraph"), "missing if-start-of-paragraph");
        assert!(names.contains(&"if-not-cap"), "missing if-not-cap");
        assert!(names.contains(&"preform-nonterminal"), "missing preform-nonterminal");

        let dividing = grammar.nonterminals.iter()
            .find(|n| n.name == "dividing-sentence")
            .expect("missing dividing-sentence");
        assert!(!dividing.internal, "dividing-sentence should be regular");
        assert!(!dividing.productions.is_empty(), "dividing-sentence should have productions");

        let first_prod = &dividing.productions[0];
        assert_eq!(first_prod.tokens.len(), 2, "first production should have 2 tokens");
        assert_eq!(
            first_prod.tokens[0].category,
            ProductionTokenCategory::SubNonterminal("if-start-of-paragraph".to_string()),
            "first token should be <if-start-of-paragraph>"
        );
        assert_eq!(
            first_prod.tokens[1].category,
            ProductionTokenCategory::SubNonterminal("heading".to_string()),
            "second token should be <heading>"
        );
    }

    #[test]
    fn test_real_syntax_preform_dividing_sentence_match() {
        // Load the real Syntax.preform and test that <dividing-sentence>
        // matches "chapter 1 - X" at paragraph start, and fails without it.
        // Also test that <heading> matches "chapter 1 - X".
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let words = &["chapter", "1", "-", "X"];
        let registry = InternalRegistry::new();

        // <dividing-sentence> should match at paragraph start.
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: true, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_some(), "dividing-sentence should match 'chapter 1 - X' at paragraph start");

        // <dividing-sentence> should fail without paragraph start.
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_none(), "dividing-sentence should fail 'chapter 1 - X' without paragraph start");

        // <heading> should match "chapter 1 - X" via the real grammar.
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: true, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "heading", Wording::new(0, 4));
        assert!(m.is_some(), "heading should match 'chapter 1 - X' via real grammar");
    }

    // -----------------------------------------------------------------------
    // <certainty> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_certainty_always() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["always"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'always'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(2)
        );
    }

    #[test]
    fn test_certainty_certainly() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["certainly"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'certainly'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(2)
        );
    }

    #[test]
    fn test_certainty_usually() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["usually"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'usually'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(1)
        );
    }

    #[test]
    fn test_certainty_normally() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["normally"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'normally'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(1)
        );
    }

    #[test]
    fn test_certainty_rarely() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["rarely"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'rarely'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(-1)
        );
    }

    #[test]
    fn test_certainty_seldom() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["seldom"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'seldom'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(-1)
        );
    }

    #[test]
    fn test_certainty_never() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["never"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'never'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(-2)
        );
    }

    #[test]
    fn test_certainty_initially() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["initially"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_some(), "certainty should match 'initially'");
        assert_eq!(
            m.unwrap().internal.unwrap().payload,
            InternalPayload::Integer(3)
        );
    }

    #[test]
    fn test_certainty_fails_unknown_word() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["xyzzy"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 1));
        assert!(m.is_none(), "certainty should fail on 'xyzzy'");
    }

    #[test]
    fn test_certainty_fails_multi_word() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<certainty> internal").unwrap();
        let words = &["always", "usually"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "certainty", Wording::new(0, 2));
        assert!(m.is_none(), "certainty should fail on multi-word wording");
    }

    // -----------------------------------------------------------------------
    // <pre-verb-rc-marker> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pre_verb_rc_marker_who() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<pre-verb-rc-marker> internal").unwrap();
        let words = &["who"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-rc-marker", Wording::new(0, 1));
        assert!(m.is_some(), "pre-verb-rc-marker should match 'who'");
    }

    #[test]
    fn test_pre_verb_rc_marker_which() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<pre-verb-rc-marker> internal").unwrap();
        let words = &["which"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-rc-marker", Wording::new(0, 1));
        assert!(m.is_some(), "pre-verb-rc-marker should match 'which'");
    }

    #[test]
    fn test_pre_verb_rc_marker_that() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<pre-verb-rc-marker> internal").unwrap();
        let words = &["that"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-rc-marker", Wording::new(0, 1));
        assert!(m.is_some(), "pre-verb-rc-marker should match 'that'");
    }

    #[test]
    fn test_pre_verb_rc_marker_fails_unknown() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<pre-verb-rc-marker> internal").unwrap();
        let words = &["xyzzy"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-rc-marker", Wording::new(0, 1));
        assert!(m.is_none(), "pre-verb-rc-marker should fail on 'xyzzy'");
    }

    // -----------------------------------------------------------------------
    // <nonimperative-verb> stub tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_nonimperative_verb_stub_fails() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<nonimperative-verb> internal").unwrap();
        let words = &["is"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "nonimperative-verb", Wording::new(0, 1));
        assert!(m.is_none(), "nonimperative-verb stub should fail on 'is'");
    }

    // -----------------------------------------------------------------------
    // <negated-noncopular-verb-present> stub tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_negated_noncopular_verb_present_stub_fails() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let grammar = parse_preform_grammar("<negated-noncopular-verb-present> internal").unwrap();
        let words = &["does", "not"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "negated-noncopular-verb-present", Wording::new(0, 2));
        assert!(m.is_none(), "negated-noncopular-verb-present stub should fail");
    }

    // -----------------------------------------------------------------------
    // <pre-verb-certainty> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pre_verb_certainty_always() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<pre-verb-certainty> internal").unwrap();
        let words = &["always"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-certainty", Wording::new(0, 1));
        assert!(m.is_some(), "pre-verb-certainty should match 'always'");
    }

    #[test]
    fn test_pre_verb_certainty_fails_unknown() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<pre-verb-certainty> internal").unwrap();
        let words = &["xyzzy"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "pre-verb-certainty", Wording::new(0, 1));
        assert!(m.is_none(), "pre-verb-certainty should fail on 'xyzzy'");
    }

    // -----------------------------------------------------------------------
    // <post-verb-certainty> tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_post_verb_certainty_never() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<post-verb-certainty> internal").unwrap();
        let words = &["never"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "post-verb-certainty", Wording::new(0, 1));
        assert!(m.is_some(), "post-verb-certainty should match 'never'");
    }

    #[test]
    fn test_post_verb_certainty_fails_unknown() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let grammar = parse_preform_grammar("<post-verb-certainty> internal").unwrap();
        let words = &["xyzzy"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let registry = InternalRegistry::linguistics();
        let m = match_nonterminal_impl(&ctx, &registry, "post-verb-certainty", Wording::new(0, 1));
        assert!(m.is_none(), "post-verb-certainty should fail on 'xyzzy'");
    }

    // -----------------------------------------------------------------------
    // Integration: certainty through matching engine
    // -----------------------------------------------------------------------

    #[test]
    fn test_certainty_through_matching_engine() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let source = concat!(
            "<certainty> internal\n",
            "<test> ::= <certainty> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["always", "cat"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_some(), "<certainty> cat should match 'always cat'");
    }

    #[test]
    fn test_certainty_through_matching_engine_fails() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let source = concat!(
            "<certainty> internal\n",
            "<test> ::= <certainty> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["xyzzy", "cat"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_none(), "<certainty> cat should fail on 'xyzzy cat'");
    }

    // -----------------------------------------------------------------------
    // Integration: pre-verb-rc-marker through matching engine
    // -----------------------------------------------------------------------

    #[test]
    fn test_pre_verb_rc_marker_through_matching_engine() {
        // Reference: services/linguistics-module/Chapter 3/Verbs.w
        let source = concat!(
            "<pre-verb-rc-marker> internal\n",
            "<test> ::= <pre-verb-rc-marker> cat\n",
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let registry = InternalRegistry::linguistics();
        let words = &["who", "cat"];
        let ctx = PreformContext { grammar: &grammar,
        word_text: words,
        is_paragraph_start: false, verbs_registry: None };
        let m = match_nonterminal_impl(&ctx, &registry, "test", Wording::new(0, 2));
        assert!(m.is_some(), "<pre-verb-rc-marker> cat should match 'who cat'");
    }

    // -----------------------------------------------------------------------
    // Real grammar oracle: certainty in Syntax.preform
    // -----------------------------------------------------------------------

    #[test]
    fn test_real_syntax_preform_has_certainty() {
        // Reference: services/linguistics-module/Chapter 3/Adverbs of Certainty.w
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"certainty"), "missing certainty");
        assert!(names.contains(&"nonimperative-verb"), "missing nonimperative-verb");
        assert!(names.contains(&"negated-noncopular-verb-present"), "missing negated-noncopular-verb-present");
        assert!(names.contains(&"pre-verb-rc-marker"), "missing pre-verb-rc-marker");
        assert!(names.contains(&"pre-verb-certainty"), "missing pre-verb-certainty");
        assert!(names.contains(&"post-verb-certainty"), "missing post-verb-certainty");
    }
}

// ---------------------------------------------------------------------------
// <article>, <definite-article>, <indefinite-article>
// ---------------------------------------------------------------------------

/// Internal nonterminal that matches any English article word.
///
/// Matches "the", "a", "an", or "some" and returns the article name as the
/// payload.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 2/Articles.w`
#[derive(Clone, Debug)]
pub struct ArticleInternal {
    words: crate::linguistics::SmallWordSet<String>,
}

impl ArticleInternal {
    /// Create a new article matcher with the given word set.
    pub fn new(words: crate::linguistics::SmallWordSet<String>) -> Self {
        Self { words }
    }
}

impl InternalNonterminal for ArticleInternal {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        // Must consume exactly one word.
        if wording.len() != 1 {
            return None;
        }
        let word_idx = wording.start as usize;
        let word = ctx.word_text.get(word_idx)?;
        let word_lower = word.to_lowercase();
        self.words.get(&word_lower).map(|article_name| {
            InternalResult {
                payload: InternalPayload::Article(article_name.clone()),
            }
        })
    }
}

/// Create the article word sets and return the three article internal NTs.
///
/// Returns (article, definite_article, indefinite_article).
pub fn make_article_internals() -> (
    ArticleInternal,
    ArticleInternal,
    ArticleInternal,
) {
    // Definite article: "the"
    let mut definite_words = crate::linguistics::SmallWordSet::<String>::new();
    definite_words.insert("the".to_string(), "definite".to_string());

    // Indefinite article: "a", "an", "some"
    let mut indefinite_words = crate::linguistics::SmallWordSet::<String>::new();
    indefinite_words.insert("a".to_string(), "indefinite".to_string());
    indefinite_words.insert("an".to_string(), "indefinite".to_string());
    indefinite_words.insert("some".to_string(), "indefinite".to_string());

    // Combined article: all of the above
    let mut all_words = definite_words.clone();
    all_words.extend(indefinite_words.clone());

    (
        ArticleInternal::new(all_words),
        ArticleInternal::new(definite_words),
        ArticleInternal::new(indefinite_words),
    )
}

impl InternalRegistry {
    /// Create an [`InternalRegistry`] pre-populated with the three basic
    /// internal nonterminals plus the three article internal nonterminals:
    /// `article`, `definite-article`, and `indefinite-article`.
    ///
    /// These are the names used in the real `Syntax.preform` grammar file.
    pub fn linguistics() -> Self {
        let mut registry = InternalRegistry::new();
        let (article, definite, indefinite) = make_article_internals();
        registry.register("article", Box::new(article));
        registry.register("definite-article", Box::new(definite));
        registry.register("indefinite-article", Box::new(indefinite));
        registry.register("certainty", Box::new(CertaintyInternal));
        registry.register("nonimperative-verb", Box::new(NonimperativeVerb));
        registry.register("negated-noncopular-verb-present", Box::new(NegatedNoncopularVerbPresent));
        registry.register("pre-verb-rc-marker", Box::new(PreVerbRcMarker));
        registry.register("pre-verb-certainty", Box::new(PreVerbCertainty));
        registry.register("post-verb-certainty", Box::new(PostVerbCertainty));
        registry
    }
}

// ---------------------------------------------------------------------------
// <certainty>
// ---------------------------------------------------------------------------

/// Internal nonterminal that matches certainty adverbs and returns the
/// corresponding certainty level.
///
/// Matches:
/// - `always`/`certainly` → `CERTAIN_CE` (2)
/// - `usually`/`normally` → `LIKELY_CE` (1)
/// - `rarely`/`seldom` → `UNLIKELY_CE` (-1)
/// - `never` → `IMPOSSIBLE_CE` (-2)
/// - `initially` → `INITIALLY_CE` (3)
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
#[derive(Clone, Debug)]
pub struct CertaintyInternal;

impl InternalNonterminal for CertaintyInternal {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        if wording.len() != 1 {
            return None;
        }
        let word_idx = wording.start as usize;
        let word = ctx.word_text.get(word_idx)?;
        let level = match word.to_lowercase().as_str() {
            "always" | "certainly" => 2,  // CERTAIN_CE
            "usually" | "normally" => 1,  // LIKELY_CE
            "rarely" | "seldom" => -1,    // UNLIKELY_CE
            "never" => -2,                // IMPOSSIBLE_CE
            "initially" => 3,             // INITIALLY_CE
            _ => return None,
        };
        Some(InternalResult {
            payload: InternalPayload::Integer(level),
        })
    }
}

// ---------------------------------------------------------------------------
// <nonimperative-verb>
// ---------------------------------------------------------------------------

/// Internal nonterminal that matches known verb usages.
///
/// Takes a wording (single word or multi-word verb phrase) and looks it up
/// in the verb usage search list. Returns a match if the wording matches
/// any known verb usage text, with the verb usage reference as the payload.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
///   the verb usage search list and `<nonimperative-verb>` internal NT.
/// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
///   used by the viability map calculation.
#[derive(Clone, Debug)]
pub struct NonimperativeVerb;

impl InternalNonterminal for NonimperativeVerb {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        let registry = ctx.verbs_registry?;
        if wording.is_empty() {
            return None;
        }

        // Get the word text for the wording range.
        let start = wording.start as usize;
        let end = wording.end as usize;
        let word_slice = ctx.word_text.get(start..end)?;

        // Walk the search list (longest first) to find a matching verb usage.
        let mut current = registry.search_list_head;
        while let Some(vu) = current {
            if let Some(_consumed) = registry.parse_against_verb(word_slice, vu) {
                // Found a match — return the verb usage reference as payload.
                return Some(InternalResult {
                    payload: InternalPayload::Integer(vu as i32),
                });
            }
            current = registry.usages.get(vu)?.next_in_search_list;
        }

        None
    }
}

// ---------------------------------------------------------------------------
// <negated-noncopular-verb-present>
// ---------------------------------------------------------------------------

/// Internal nonterminal for negated non-copular present tense verbs.
///
/// Matches patterns like `does not <verb>`, `do not <verb>`, `did not <verb>`,
/// `doesn't <verb>`, `don't <verb>`, `didn't <verb>` where `<verb>` is a known
/// non-copular verb usage.
///
/// Returns the word position after the full negated verb phrase as the payload.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w` —
///   the `<negated-noncopular-verb-present>` internal NT.
/// - C reference: `services/linguistics-module/Chapter 4/Verb Phrases.w` —
///   used by the viability map calculation (score 3).
#[derive(Clone, Debug)]
pub struct NegatedNoncopularVerbPresent;

impl InternalNonterminal for NegatedNoncopularVerbPresent {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        let registry = ctx.verbs_registry?;
        if wording.len() < 3 {
            return None;
        }

        let start = wording.start as usize;
        let word_slice = ctx.word_text.get(start..)?;

        // Check for negated patterns: "does not VERB", "do not VERB", "did not VERB",
        // "doesn't VERB", "don't VERB", "didn't VERB"
        let first_word = word_slice.first()?.to_lowercase();
        let (neg_prefix_len, verb_offset) = match first_word.as_str() {
            "does" | "do" | "did" => {
                // Need at least 3 words: "does not VERB"
                let second_word = word_slice.get(1).map(|w| w.to_lowercase());
                if second_word.as_deref() == Some("not") {
                    (2, 2)
                } else {
                    return None;
                }
            }
            "doesn't" | "don't" | "didn't" => {
                // Need at least 2 words: "doesn't VERB"
                (1, 1)
            }
            _ => return None,
        };

        // The verb part must be a known non-copular verb usage.
        let verb_word = word_slice.get(verb_offset)?;
        let verb_word_lower = verb_word.to_lowercase();

        // Check if the verb word is a copular verb (e.g., "is", "are", "was", "were", "be").
        // Copular verbs are exempt from score 3.
        let copular_words = ["is", "are", "was", "were", "be", "been", "being", "am"];
        if copular_words.contains(&verb_word_lower.as_str()) {
            return None;
        }

        // Walk the search list to find a matching verb usage for the verb word.
        let verb_slice = &word_slice[verb_offset..];
        let mut current = registry.search_list_head;
        while let Some(vu) = current {
            if let Some(consumed) = registry.parse_against_verb(verb_slice, vu) {
                // Verify the verb is non-copular by checking if it's the copular verb.
                if let Some(verb_ref) = registry.get_verb_from_usage(vu) {
                    if registry.copular_verb == Some(verb_ref) {
                        return None;
                    }
                }
                // Return the total words consumed (neg prefix + verb).
                let total_consumed = (neg_prefix_len + consumed) as i32;
                return Some(InternalResult {
                    payload: InternalPayload::Integer(total_consumed),
                });
            }
            current = registry.usages.get(vu)?.next_in_search_list;
        }

        None
    }
}
// ---------------------------------------------------------------------------
// <pre-verb-rc-marker> (stub)
// ---------------------------------------------------------------------------

/// Internal nonterminal stub for relative clause markers before verbs.
///
/// Matches "who", "which", "that" as relative clause markers.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Verbs.w`
#[derive(Clone, Debug)]
pub struct PreVerbRcMarker;

impl InternalNonterminal for PreVerbRcMarker {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        if wording.len() != 1 {
            return None;
        }
        let word_idx = wording.start as usize;
        let word = ctx.word_text.get(word_idx)?;
        match word.to_lowercase().as_str() {
            "who" | "which" | "that" => Some(InternalResult {
                payload: InternalPayload::Article(word.to_string()),
            }),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// <pre-verb-certainty> (stub)
// ---------------------------------------------------------------------------

/// Internal nonterminal stub for certainty adverbs before the verb.
///
/// Delegates to `<certainty>` for now.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
#[derive(Clone, Debug)]
pub struct PreVerbCertainty;

impl InternalNonterminal for PreVerbCertainty {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        // Delegate to certainty matching.
        if wording.len() != 1 {
            return None;
        }
        let word_idx = wording.start as usize;
        let word = ctx.word_text.get(word_idx)?;
        let level = match word.to_lowercase().as_str() {
            "always" | "certainly" => 2,
            "usually" | "normally" => 1,
            "rarely" | "seldom" => -1,
            "never" => -2,
            "initially" => 3,
            _ => return None,
        };
        Some(InternalResult {
            payload: InternalPayload::Integer(level),
        })
    }
}
// ---------------------------------------------------------------------------
// <post-verb-certainty> (stub)
// ---------------------------------------------------------------------------

/// Internal nonterminal stub for certainty adverbs after the verb.
///
/// Delegates to `<certainty>` for now.
///
/// # References
///
/// - C reference: `services/linguistics-module/Chapter 3/Adverbs of Certainty.w`
#[derive(Clone, Debug)]
pub struct PostVerbCertainty;

impl InternalNonterminal for PostVerbCertainty {
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult> {
        // Same as pre-verb certainty for now.
        if wording.len() != 1 {
            return None;
        }
        let word_idx = wording.start as usize;
        let word = ctx.word_text.get(word_idx)?;
        let level = match word.to_lowercase().as_str() {
            "always" | "certainly" => 2,
            "usually" | "normally" => 1,
            "rarely" | "seldom" => -1,
            "never" => -2,
            "initially" => 3,
            _ => return None,
        };
        Some(InternalResult {
            payload: InternalPayload::Integer(level),
        })
    }
}
