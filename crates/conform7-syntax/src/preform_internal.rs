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
// InternalRegistry::basic()
// ---------------------------------------------------------------------------

impl InternalRegistry {
    /// Create an [`InternalRegistry`] pre-populated with the three basic
    /// internal nonterminals: `if-start-of-paragraph`, `if-not-cap`, and
    /// `preform-nonterminal`.
    ///
    /// These are the names used in the real `Syntax.preform` grammar file.
    pub fn basic() -> Self {
        let mut registry = InternalRegistry::new();
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
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: true,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "if-start-of-paragraph", Wording::new(0, 2));
        assert!(m.is_some(), "should match at paragraph start");
        assert_eq!(m.unwrap().internal.unwrap().payload, InternalPayload::None);
    }

    #[test]
    fn test_if_start_of_paragraph_fails_mid_paragraph() {
        let grammar = parse_preform_grammar("<if-start-of-paragraph> internal").unwrap();
        let words = &["hello", "world"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
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
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(0, 1));
        assert!(m.is_some(), "'apple' should match if-not-cap");
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(1, 2));
        assert!(m.is_some(), "'the' should match if-not-cap");
    }

    #[test]
    fn test_if_not_cap_uppercase() {
        let grammar = parse_preform_grammar("<if-not-cap> internal").unwrap();
        let words = &["Apple", "The"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(0, 1));
        assert!(m.is_none(), "'Apple' should fail if-not-cap");
        let m = match_nonterminal_impl(&ctx, &registry, "if-not-cap", Wording::new(1, 2));
        assert!(m.is_none(), "'The' should fail if-not-cap");
    }

    #[test]
    fn test_if_not_cap_empty() {
        let grammar = parse_preform_grammar("<if-not-cap> internal").unwrap();
        let words: &[&str] = &[];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
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
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
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
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();

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
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 1));
        assert!(m.is_none(), "<baz> should fail when not declared");
    }

    #[test]
    fn test_preform_nonterminal_fails_plain_word() {
        let source = "<preform-nonterminal> internal\n<foo> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["foo"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "preform-nonterminal", Wording::new(0, 1));
        assert!(m.is_none(), "'foo' should fail (not in <name> format)");
    }

    #[test]
    fn test_preform_nonterminal_fails_multi_word() {
        let source = "<preform-nonterminal> internal\n<foo> internal\n<bar> ::= x";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["<foo>", "<bar>"];
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let registry = InternalRegistry::basic();
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

        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: true,
        };
        let registry = InternalRegistry::basic();
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_some(), "dividing-sentence should match at paragraph start");

        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
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
        let registry = InternalRegistry::basic();

        // <dividing-sentence> should match at paragraph start.
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: true,
        };
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_some(), "dividing-sentence should match 'chapter 1 - X' at paragraph start");

        // <dividing-sentence> should fail without paragraph start.
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: false,
        };
        let m = match_nonterminal_impl(&ctx, &registry, "dividing-sentence", Wording::new(0, 4));
        assert!(m.is_none(), "dividing-sentence should fail 'chapter 1 - X' without paragraph start");

        // <heading> should match "chapter 1 - X" via the real grammar.
        let ctx = PreformContext {
            grammar: &grammar,
            word_text: words,
            is_paragraph_start: true,
        };
        let m = match_nonterminal_impl(&ctx, &registry, "heading", Wording::new(0, 4));
        assert!(m.is_some(), "heading should match 'chapter 1 - X' via real grammar");
    }
}
