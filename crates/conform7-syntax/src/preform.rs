//! Preform grammar parser.
//!
//! Parses the `Syntax.preform` format used to define the I7 grammar.
//! The format is a text-based grammar definition language with ~720
//! nonterminals for English alone.
//!
//! This module handles **parsing** the Preform format into in-memory data
//! structures, and **matching** — the runtime engine that takes a nonterminal
//! and a wording and tries to match it against all productions, with
//! backtracking.
//!
//! # Format
//!
//! ```text
//! language English
//!
//! <nonterminal-name> internal
//!
//! <nonterminal-name> ::=
//!     /a/ production1 |
//!     production2 |
//!     ...
//! ```
//!
//! Where productions contain:
//! - **Fixed words**: literal text like `to`, `is`, `a`, `room`
//! - **Wildcards**: `...` (matches one or more words), `......` (balanced),
//!   `###` (exactly one word), `***` (zero or more words)
//! - **Sub-nonterminals**: `<quoted-text>`, `<if-start-of-paragraph>`, etc.
//! - **Modifiers**: `^` (negation), `_` (disallow unexpected upper case), `\`
//!   (escape the next token, suppressing wildcard recognition)
//! - **Alternatives**: `something/anything` or `_,/and`
//! - **Braces**: `{each other in groups}` mark the start/end of a word range
//! - **Production match numbers**: `/a/`, `/b/`, `/aa/`, `/bb/`
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 4/Loading Preform.w`
//! - C reference: `services/words-module/Chapter 4/Preform.w`
use crate::Wording;
use std::fmt;

/// A complete Preform grammar, containing all nonterminals.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grammar {
    /// The language declared at the top of the file, if any.
    pub language: Option<String>,
    /// All nonterminals in the grammar, in declaration order.
    pub nonterminals: Vec<Nonterminal>,
}

/// A single nonterminal in a Preform grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nonterminal {
    /// The nonterminal name, e.g. `"rule-preamble"`.
    pub name: String,
    /// If true, matching is handled by a Rust function, not grammar rules.
    pub internal: bool,
    /// Productions for this nonterminal (empty for internal NTs).
    pub productions: Vec<Production>,
}

/// A single production (alternative) within a nonterminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production {
    /// Explicit production match number set by `/a/`, `/b/`, `/aa/`, etc.
    ///
    /// If `None`, the production simply takes its index in the nonterminal as
    /// its match number, matching the C default.
    pub match_number: Option<u8>,
    /// The tokens that make up this production.
    pub tokens: Vec<ProductionToken>,
}

/// A token within a production.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionToken {
    /// What kind of token this is.
    pub category: ProductionTokenCategory,
    /// If true, this token is negated (`^`).
    pub negated: bool,
    /// If true, the matched word must not have an unexpected upper-case letter
    /// (`_`).
    pub disallow_unexpected_upper: bool,
    /// If true, the token was escaped with `\`, so wildcard recognition was
    /// suppressed.
    pub escaped: bool,
    /// Alternative forms for this token, connected by `/` in the source.
    pub alternatives: Vec<ProductionTokenCategory>,
    /// Result number assigned by `? N` after a sub-nonterminal.
    pub result_index: Option<usize>,
    /// If set, this token starts a braced word range with the given range id.
    pub range_start: Option<usize>,
    /// If set, this token ends a braced word range with the given range id.
    pub range_end: Option<usize>,
}

/// The category of a [`ProductionToken`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionTokenCategory {
    /// A literal word that must appear verbatim (e.g., `"to"`, `"is"`, `"room"`).
    FixedWord(String),
    /// A wildcard matching exactly one word (`###`).
    SingleWildcard,
    /// A wildcard matching one or more words (`...`).
    MultipleWildcard,
    /// A wildcard matching one or more words while keeping bracket pairs in
    /// balance (`......`).
    BalancedMultipleWildcard,
    /// A wildcard matching zero or more words (`***`).
    PossiblyEmptyWildcard,
    /// A reference to another nonterminal (e.g., `<quoted-text>`).
    SubNonterminal(String),
}

impl ProductionToken {
    /// Build a plain fixed-word token.
    pub fn fixed(word: impl Into<String>) -> Self {
        Self {
            category: ProductionTokenCategory::FixedWord(word.into()),
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Build a sub-nonterminal token.
    pub fn sub(name: impl Into<String>) -> Self {
        Self {
            category: ProductionTokenCategory::SubNonterminal(name.into()),
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Build a `...` wildcard token.
    pub fn multiple_wildcard() -> Self {
        Self {
            category: ProductionTokenCategory::MultipleWildcard,
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Build a `......` balanced wildcard token.
    pub fn balanced_multiple_wildcard() -> Self {
        Self {
            category: ProductionTokenCategory::BalancedMultipleWildcard,
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Build a `###` single-word wildcard token.
    pub fn single_wildcard() -> Self {
        Self {
            category: ProductionTokenCategory::SingleWildcard,
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Build a `***` possibly-empty wildcard token.
    pub fn possibly_empty_wildcard() -> Self {
        Self {
            category: ProductionTokenCategory::PossiblyEmptyWildcard,
            negated: false,
            disallow_unexpected_upper: false,
            escaped: false,
            alternatives: Vec::new(),
            result_index: None,
            range_start: None,
            range_end: None,
        }
    }

    /// Mark this token as negated (`^`).
    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }

    /// Mark this token as disallowing unexpected upper case (`_`).
    pub fn lower(mut self) -> Self {
        self.disallow_unexpected_upper = true;
        self
    }

    /// Mark this token as escaped (`\`).
    pub fn escaped(mut self) -> Self {
        self.escaped = true;
        self
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(lang) = &self.language {
            writeln!(f, "language {}", lang)?;
            writeln!(f)?;
        }
        for (i, nt) in self.nonterminals.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", nt)?;
        }
        Ok(())
    }
}

impl fmt::Display for Nonterminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.internal {
            write!(f, "<{}> internal", self.name)?;
        } else if self.productions.is_empty() {
            write!(f, "<{}> ::=", self.name)?;
        } else {
            writeln!(f, "<{}> ::=", self.name)?;
            for (i, prod) in self.productions.iter().enumerate() {
                let sep = if i < self.productions.len() - 1 { " |" } else { "" };
                writeln!(f, "    {}{}", prod, sep)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.match_number {
            write!(f, "/{}/ ", match_number_to_letters(n))?;
        }
        let mut first = true;
        for token in &self.tokens {
            if !first {
                write!(f, " ")?;
            }
            if token.range_start.is_some() {
                write!(f, "{{")?;
            }
            write!(f, "{}", token)?;
            if token.range_end.is_some() {
                write!(f, "}}")?;
            }
            first = false;
        }
        Ok(())
    }
}

impl fmt::Display for ProductionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.escaped {
            write!(f, "\\")?;
        }
        if self.negated {
            write!(f, "^")?;
        }
        if self.disallow_unexpected_upper {
            write!(f, "_")?;
        }
        self.category.fmt(f)?;
        for alt in &self.alternatives {
            write!(f, "/{}", alt)?;
        }
        if let Some(n) = self.result_index {
            write!(f, " ? {}", n)?;
        }
        Ok(())
    }
}

impl fmt::Display for ProductionTokenCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProductionTokenCategory::FixedWord(word) => write!(f, "{}", word),
            ProductionTokenCategory::SingleWildcard => write!(f, "###"),
            ProductionTokenCategory::MultipleWildcard => write!(f, "..."),
            ProductionTokenCategory::BalancedMultipleWildcard => write!(f, "......"),
            ProductionTokenCategory::PossiblyEmptyWildcard => write!(f, "***"),
            ProductionTokenCategory::SubNonterminal(name) => write!(f, "<{}>", name),
        }
    }
}

fn match_number_to_letters(n: u8) -> String {
    if n < 26 {
        ((b'a' + n) as char).to_string()
    } else {
        let c = (b'a' + n - 26) as char;
        format!("{c}{c}")
    }
}

// ---------------------------------------------------------------------------
// PEG parser for the Preform format
// ---------------------------------------------------------------------------

/// Internal elements parsed within a single production.
enum ProductionElement {
    Token(ProductionToken),
    OpenBrace,
    CloseBrace,
    MatchNumber(u8),
}

/// Internal top-level item returned by the grammar parser.
enum TopLevelItem {
    Language(String),
    Nonterminal(Nonterminal),
}

fn resolve_elements(elements: Vec<ProductionElement>) -> Option<Production> {
    let mut tokens: Vec<ProductionToken> = Vec::new();
    let mut match_number = None;
    let mut pending_open: Option<usize> = None;
    let mut active_ranges: Vec<usize> = Vec::new();
    let mut next_range_id = 1usize;

    for elem in elements {
        match elem {
            ProductionElement::MatchNumber(n) => {
                match_number = Some(n);
            }
            ProductionElement::OpenBrace => {
                let id = next_range_id;
                next_range_id += 1;
                pending_open = Some(id);
            }
            ProductionElement::CloseBrace => {
                if let Some(id) = active_ranges.pop() {
                    if let Some(last) = tokens.last_mut() {
                        last.range_end = Some(id);
                    }
                }
            }
            ProductionElement::Token(mut token) => {
                if let Some(id) = pending_open.take() {
                    token.range_start = Some(id);
                    active_ranges.push(id);
                }
                tokens.push(token);
            }
        }
    }

    if tokens.is_empty() {
        None
    } else {
        Some(Production {
            match_number,
            tokens,
        })
    }
}

fn new_production_token(category: ProductionTokenCategory) -> ProductionToken {
    ProductionToken {
        category,
        negated: false,
        disallow_unexpected_upper: false,
        escaped: false,
        alternatives: Vec::new(),
        result_index: None,
        range_start: None,
        range_end: None,
    }
}

fn split_word(text: &str) -> ProductionToken {
    let chars: Vec<char> = text.chars().collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();

    for (i, c) in chars.iter().enumerate() {
        if *c == '/' {
            // C only splits at slashes that have word material on both sides.
            if !current.is_empty() && i + 1 < chars.len() {
                segments.push(current.clone());
                current.clear();
            } else {
                current.push(*c);
            }
        } else {
            current.push(*c);
        }
    }
    segments.push(current);

    let head = ProductionTokenCategory::FixedWord(segments.remove(0));
    let alternatives: Vec<_> = segments
        .into_iter()
        .map(ProductionTokenCategory::FixedWord)
        .collect();
    ProductionToken {
        category: head,
        alternatives,
        ..new_production_token(ProductionTokenCategory::FixedWord(String::new()))
    }
}

peg::parser! {
    grammar preform_parser() for str {
        /// Parse an entire Preform grammar source.
        pub(crate) rule grammar() -> Grammar
            = items:top_level_item()* EOI()
              {
                  let mut language = None;
                  let mut nonterminals = Vec::new();
                  for item in items {
                      match item {
                          TopLevelItem::Language(l) => language = Some(l),
                          TopLevelItem::Nonterminal(n) => nonterminals.push(n),
                      }
                  }
                  Grammar { language, nonterminals }
              }

        // A top-level item: nonterminal declaration or language switch.
        rule top_level_item() -> TopLevelItem
            = n:nonterminal_declaration() { TopLevelItem::Nonterminal(n) }
            / l:language_declaration() { TopLevelItem::Language(l) }

        // Language switch: "language English"
        rule language_declaration() -> String
            = _ "language" ws() name:$((![' ' | '\t' | '\n' | '\r'] [_])+) _ { name.to_string() }

        // Whitespace within a line (spaces and tabs only)
        rule ws()
            = quiet!{ ([' ' | '\t'] / comment())* }

        // Whitespace including newlines (for between declarations)
        rule _()
            = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }

        // Comments: [ ... ]
        rule comment()
            = "[" $((![']'] [_])*) "]"

        // A nonterminal declaration: either internal or with productions.
        rule nonterminal_declaration() -> Nonterminal
            = _ name:nonterminal_name() ws() "internal" _
              {
                  Nonterminal {
                      name,
                      internal: true,
                      productions: Vec::new(),
                  }
              }
            / _ name:nonterminal_name() ws() "::=" _ prods:production_list() _
              {
                  Nonterminal {
                      name,
                      internal: false,
                      productions: prods,
                  }
              }

        // Nonterminal name: <...>
        rule nonterminal_name() -> String
            = "<" n:$((!['>'] [_])+) ">" { n.trim().to_string() }

        // List of productions separated by |
        // Stops when the next non-whitespace content looks like a new
        // nonterminal declaration (<name> ::= or <name> internal).
        // Trailing | on the last production is allowed (creates an empty
        // production that is ignored, matching C behavior).
        rule production_list() -> Vec<Production>
            = p:production() rest:(_ "|" _ p2:production() { p2 })* _ "|"?
              {
                  let mut v = Vec::new();
                  if let Some(p) = p { v.push(p); }
                  v.extend(rest.into_iter().flatten());
                  v
              }

        // A single production: a sequence of elements.
        // Fails if the next content looks like a new nonterminal declaration.
        rule production() -> Option<Production>
            = !nonterminal_start() elems:production_element()* { resolve_elements(elems) }

        // Check if the next content looks like a nonterminal declaration start.
        rule nonterminal_start()
            = "<" n:$((!['>'] [_])+) ">" ws() ("::=" / "internal")

        // Whitespace within a production: spaces, tabs, comments, and single newlines.
        // Does NOT consume blank lines (two consecutive newlines), which separate
        // nonterminal declarations. This allows tabular multi-line productions
        // like `<en-article-declension>` where tokens are separated by newlines.
        rule production_ws()
            = quiet!{ ([' ' | '\t'] / comment() / "\n" !"\n")* }

        // One element of a production: a token, brace marker, or match number.
        // Stops when the next content looks like a new nonterminal declaration.
        // Consumes whitespace first, then checks for nonterminal start, to handle
        // tabular multi-line productions like `<en-article-declension>` where
        // tokens are separated by newlines.
        rule production_element() -> ProductionElement
            = production_ws() !nonterminal_start() n:production_match_number() { ProductionElement::MatchNumber(n) }
            / production_ws() !nonterminal_start() "{" { ProductionElement::OpenBrace }
            / production_ws() !nonterminal_start() "}" { ProductionElement::CloseBrace }
            / production_ws() !nonterminal_start() token:production_token() { ProductionElement::Token(token) }

        // Production match number: /a/ to /z/ and /aa/ to /zz/.
        rule production_match_number() -> u8
            = "/" c:['a'..='z'] "/" { (c as u8) - b'a' }
            / "/" a:['a'..='z'] b:['a'..='z'] "/" { (a as u8) - b'a' + 26 + (b as u8 - b'a') }

        // A token with optional modifiers.
        rule production_token() -> ProductionToken
            = negated:("^" { true })*
              lower:("_" { true })*
              "\\" ws() t:escaped_core_token()
              {
                  ProductionToken {
                      negated: !negated.is_empty(),
                      disallow_unexpected_upper: !lower.is_empty(),
                      escaped: true,
                      ..t
                  }
              }
            / negated:("^" { true })*
              lower:("_" { true })*
              ws() t:plain_core_token()
              {
                  ProductionToken {
                      negated: !negated.is_empty(),
                      disallow_unexpected_upper: !lower.is_empty(),
                      escaped: false,
                      ..t
                  }
              }

        // Core of a plain (non-escaped) token: sub-nonterminal, wildcard, or word
        // (which may contain slash alternatives).
        rule plain_core_token() -> ProductionToken
            = t:sub_nonterminal_with_result() { t }
            / w:wildcard() { new_production_token(w) }
            / w:plain_word() { w }

        // Core of an escaped token: only fixed words (slash alternatives allowed),
        // with wildcard recognition suppressed.
        rule escaped_core_token() -> ProductionToken
            = w:escaped_word() { w }

        // Sub-nonterminal, optionally followed by a result number.
        rule sub_nonterminal_with_result() -> ProductionToken
            = n:nonterminal_name() r:result_number()?
              {
                  ProductionToken {
                      category: ProductionTokenCategory::SubNonterminal(n),
                      result_index: r,
                      ..new_production_token(ProductionTokenCategory::FixedWord(String::new()))
                  }
              }

        // Result number suffix: ? N
        rule result_number() -> usize
            = "?" ws() n:$(['0'..='9']+) { n.parse().unwrap() }

        // Wildcards. The negative lookahead ensures we don't split a longer word
        // (e.g. "****" must remain a fixed word, not "***" + "*").
        rule wildcard() -> ProductionTokenCategory
            = "......" !continuation_char() { ProductionTokenCategory::BalancedMultipleWildcard }
            / "..." !continuation_char() { ProductionTokenCategory::MultipleWildcard }
            / "###" !continuation_char() { ProductionTokenCategory::SingleWildcard }
            / "***" !continuation_char() { ProductionTokenCategory::PossiblyEmptyWildcard }

        // A plain word. Since `/` is not word-breaking punctuation in Preform,
        // a word may contain internal slashes that create alternatives.
        rule plain_word() -> ProductionToken
            = w:$(plain_word_char()+) { split_word(w) }

        // An escaped word. Wildcard recognition is suppressed, but slash
        // alternatives still work.
        rule escaped_word() -> ProductionToken
            = w:$(escaped_word_char()+) { split_word(w) }

        rule plain_word_char() -> ()
            = ![' ' | '\t' | '\n' | '\r' | '|' | '{' | '}' | '[' | ']' | '_' | '^' | '?' | '&' | '\\' | '<'] [_]

        rule escaped_word_char() -> ()
            = ![' ' | '\t' | '\n' | '\r' | '|'] [_]

        // A character that would continue a word after a wildcard. Wildcards must
        // not match as a prefix of a longer word.
        rule continuation_char() -> ()
            = plain_word_char()

        // End of input
        rule EOI()
            = ![_]
    }
}

/// Parse a Preform grammar from a source string.
///
/// Returns the parsed `Grammar` on success, or an error message on failure.
///
/// # Examples
///
/// ```
/// use conform7_syntax::parse_preform_grammar;
///
/// let source = "<test> internal";
/// let grammar = parse_preform_grammar(source).unwrap();
/// assert_eq!(grammar.nonterminals.len(), 1);
/// assert_eq!(grammar.nonterminals[0].name, "test");
/// assert!(grammar.nonterminals[0].internal);
/// ```
pub fn parse_preform_grammar(source: &str) -> Result<Grammar, String> {
    preform_parser::grammar(source).map_err(|e| format!("Preform parse error: {}", e))
}

// ---------------------------------------------------------------------------
// Preform matching engine
// ---------------------------------------------------------------------------

/// Payload produced by a successful internal nonterminal match.
///
/// Corresponds to the `Q` (integer) and `QP` (pointer) results in the C
/// reference (`services/words-module/Chapter 4/Preform.w`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InternalPayload {
    /// No meaningful payload (zero-width internals like `<if-start-of-paragraph>`).
    None,
    /// An integer result (e.g., the word number for `<quoted-text>`).
    Integer(i32),
    /// The name of a matched nonterminal (for `<preform-nonterminal>`).
    Nonterminal(String),
    /// An article name (for article internal NTs).
    Article(String),
}

/// Result of a successful internal nonterminal match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InternalResult {
    /// The payload produced by the internal NT.
    pub payload: InternalPayload,
}

/// Context passed to internal nonterminal matchers.
///
/// Bundles the information needed by the simple position/name internals:
/// the grammar, the word-text slice, and paragraph-start information.
#[derive(Clone, Debug)]
pub struct PreformContext<'a> {
    /// The grammar being matched against.
    pub grammar: &'a Grammar,
    /// The text of each word by index.
    pub word_text: &'a [&'a str],
    /// True if the first word of the wording is at the start of a paragraph.
    pub is_paragraph_start: bool,
    /// The verb registry, if available.
    pub verbs_registry: Option<&'a crate::verbs::Verbs>,
}

/// Trait for internal nonterminal implementations.
///
/// Each internal NT in the Preform grammar (declared with `internal` in
/// `Syntax.preform`) has a corresponding Rust implementation that implements
/// this trait.
///
/// # References
///
/// - C reference: `services/words-module/Chapter 4/Nonterminals.w` — the
///   `internal_definition` function pointer on `nonterminal`.
/// - C reference: `services/words-module/Chapter 4/Basic Nonterminals.w` —
///   example implementations like `<if-start-of-paragraph>`, `<if-not-cap>`,
///   `<preform-nonterminal>`.
pub trait InternalNonterminal: Send + Sync {
    /// Try to match this internal nonterminal against the given wording.
    ///
    /// Returns `Some(InternalResult)` on success, or `None` on failure.
    fn match_nonterminal(&self, ctx: &PreformContext, wording: Wording) -> Option<InternalResult>;
}

/// Registry mapping nonterminal names to their Rust implementations.
///
/// Internal NTs are looked up by name (without angle brackets) when the
/// matching engine encounters an `internal` nonterminal.
#[derive(Default)]
pub struct InternalRegistry {
    implementations: std::collections::HashMap<String, Box<dyn InternalNonterminal>>,
}

impl InternalRegistry {
    /// Create an empty registry (no internals registered).
    pub fn empty() -> Self {
        Self {
            implementations: std::collections::HashMap::new(),
        }
    }

    /// Create a registry with the three basic internals.
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register("if-start-of-paragraph", Box::new(crate::preform_internal::IfStartOfParagraph));
        registry.register("if-not-cap", Box::new(crate::preform_internal::IfNotCap));
        registry.register("preform-nonterminal", Box::new(crate::preform_internal::PreformNonterminal));
        registry
    }

    /// Register an internal nonterminal implementation.
    pub fn register(&mut self, name: &str, impl_: Box<dyn InternalNonterminal>) {
        self.implementations.insert(name.to_string(), impl_);
    }

    /// Look up an internal nonterminal by name.
    pub fn get(&self, name: &str) -> Option<&dyn InternalNonterminal> {
        self.implementations.get(name).map(|b| b.as_ref())
    }
}

impl fmt::Debug for InternalRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InternalRegistry")
            .field("names", &self.implementations.keys().collect::<Vec<_>>())
            .finish()
    }
}
use std::ops::Range;

/// Result of a successful match against a nonterminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// Index of the production within the nonterminal that matched.
    pub production_index: usize,
    /// The match number of the production (either explicit or the production index).
    pub match_number: u8,
    /// The word range consumed by the entire match (inclusive start, exclusive end).
    pub word_range: Range<u32>,
    /// If the match was produced by an internal nonterminal, the internal result.
    pub internal: Option<InternalResult>,
}


/// Match a nonterminal against a wording with full context and registry.
///
/// This is the main entry point for the Preform matching engine. It handles
/// both regular nonterminals (defined by grammar productions) and internal
/// nonterminals (implemented in Rust via the `InternalRegistry`).
///
/// For internal nonterminals, looks up the implementation in `registry` and
/// delegates to it. If no implementation is registered, returns `None`.
pub fn match_nonterminal_impl(
    ctx: &PreformContext,
    registry: &InternalRegistry,
    name: &str,
    wording: Wording,
) -> Option<Match> {
    let nt = ctx.grammar.nonterminals.iter().find(|n| n.name == name)?;
    if nt.internal {
        // Look up the internal NT implementation in the registry.
        if let Some(impl_) = registry.get(name) {
            let result = impl_.match_nonterminal(ctx, wording)?;
            let word_start = wording.start;
            let word_end = wording.end;
            return Some(Match {
                production_index: 0,
                match_number: 0,
                word_range: word_start..word_end,
                internal: Some(result),
            });
        }
        return None;
    }

    let word_start = wording.start as usize;
    let word_end = wording.end as usize;

    for (prod_idx, production) in nt.productions.iter().enumerate() {
        if try_match_production(&production.tokens, ctx, registry, word_start, word_end).is_some() {
            let match_number = production.match_number.unwrap_or(prod_idx as u8);
            return Some(Match {
                production_index: prod_idx,
                match_number,
                word_range: word_start as u32..word_end as u32,
                internal: None,
            });
        }
    }

    None
}

/// Try to match a sequence of tokens against a range of words.
///
/// Returns the word ranges consumed by each token on success, or `None` on failure.
/// Handles backtracking: for elastic tokens (wildcards, sub-nonterminals), tries
/// different consumption lengths from minimum to maximum.
fn try_match_production(
    tokens: &[ProductionToken],
    ctx: &PreformContext,
    registry: &InternalRegistry,
    word_start: usize,
    word_end: usize,
) -> Option<Vec<Range<u32>>> {
    if tokens.is_empty() {
        return if word_start == word_end { Some(vec![]) } else { None };
    }

    let token = &tokens[0];
    let rest = &tokens[1..];
    let available = word_end.saturating_sub(word_start);
    let word_text = ctx.word_text;

    match &token.category {
        ProductionTokenCategory::FixedWord(expected) => {
            // Check if the word at the current position matches the expected word
            // or any of its alternatives.
            let word_matches = |w: &str| -> bool {
                if token.disallow_unexpected_upper && w.chars().any(|c| c.is_uppercase()) {
                    return false;
                }
                true
            };
            let matches = word_start < word_end
                && word_matches(word_text[word_start])
                && (word_text[word_start] == expected.as_str()
                    || token.alternatives.iter().any(|alt| {
                        matches!(alt, ProductionTokenCategory::FixedWord(w) if word_text[word_start] == w.as_str())
                    }));

            let success = if token.negated { !matches } else { matches };

            if success {
                let mut result = try_match_production(rest, ctx, registry, word_start + 1, word_end)?;
                result.insert(0, word_start as u32..(word_start + 1) as u32);
                Some(result)
            } else {
                None
            }
        }
        ProductionTokenCategory::SingleWildcard => {
            if available >= 1 {
                let mut result = try_match_production(rest, ctx, registry, word_start + 1, word_end)?;
                result.insert(0, word_start as u32..(word_start + 1) as u32);
                Some(result)
            } else {
                None
            }
        }
        ProductionTokenCategory::MultipleWildcard => {
            // Try consuming 1 to `available` words, shortest first (non-greedy).
            for len in 1..=available {
                if let Some(mut result) = try_match_production(rest, ctx, registry, word_start + len, word_end) {
                    result.insert(0, word_start as u32..(word_start + len) as u32);
                    return Some(result);
                }
            }
            None
        }
        ProductionTokenCategory::PossiblyEmptyWildcard => {
            // Try consuming 0 to `available` words, shortest first.
            for len in 0..=available {
                if let Some(mut result) = try_match_production(rest, ctx, registry, word_start + len, word_end) {
                    result.insert(0, word_start as u32..(word_start + len) as u32);
                    return Some(result);
                }
            }
            None
        }
        ProductionTokenCategory::BalancedMultipleWildcard => {
            // Try consuming 1 to `available` words, shortest first, but only
            // where brackets are balanced.
            for len in 1..=available {
                if is_balanced_brackets(word_text, word_start, word_start + len) {
                    if let Some(mut result) = try_match_production(rest, ctx, registry, word_start + len, word_end) {
                        result.insert(0, word_start as u32..(word_start + len) as u32);
                        return Some(result);
                    }
                }
            }
            None
        }
        ProductionTokenCategory::SubNonterminal(sub_name) => {
            if token.negated {
                // Negated sub-nonterminal: succeeds if the sub-nonterminal does NOT match
                // against any possible range. If it fails for all ranges, the negation
                // succeeds and we consume 1 word (the minimum).
                //
                // Note: The C reference uses strut/lookahead logic to determine the exact
                // range to check, then consumes that range on success. Our simple approach
                // of consuming 1 word is correct for common cases but may fail for
                // multi-word negated sub-NTs (e.g., `^<phrase>` where `<phrase>` matches
                // 2+ words). Full strut support would fix this.
                for len in 1..=available {
                    let sub_wording = Wording::new(word_start as u32, (word_start + len) as u32);
                    if match_nonterminal_impl(ctx, registry, sub_name, sub_wording).is_some() {
                        // Sub-nonterminal matched, so negation fails.
                        return None;
                    }
                }
                // No range matched the sub-nonterminal, so negation succeeds.
                // Consume 1 word (the minimum) and continue.
                let mut result = try_match_production(rest, ctx, registry, word_start + 1, word_end)?;
                result.insert(0, word_start as u32..(word_start + 1) as u32);
                Some(result)
            } else {
                // Non-negated sub-nonterminal: try matching against each possible range,
                // including 0-length (for sub-NTs that can match empty text).
                for len in 0..=available {
                    let sub_wording = Wording::new(word_start as u32, (word_start + len) as u32);
                    if match_nonterminal_impl(ctx, registry, sub_name, sub_wording).is_some() {
                        if let Some(mut result) = try_match_production(rest, ctx, registry, word_start + len, word_end) {
                            result.insert(0, word_start as u32..(word_start + len) as u32);
                            return Some(result);
                        }
                    }
                }
                None
            }
        }
    }
}

/// Check that brackets are balanced in a range of words.
///
/// Counts `(`/`)`, `[`/`]`, and `{`/`}` pairs, ensuring no closing bracket
/// appears without a matching opener and that all openers are closed by the end.
fn is_balanced_brackets(word_text: &[&str], start: usize, end: usize) -> bool {
    let mut depth = 0i32;
    for word in &word_text[start..end] {
        match *word {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_nonterminal() {
        let source = "<test> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 1);
        assert_eq!(grammar.nonterminals[0].name, "test");
        assert!(grammar.nonterminals[0].internal);
        assert!(grammar.nonterminals[0].productions.is_empty());
    }

    #[test]
    fn test_language_declaration() {
        let source = "language English\n\n<test> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.language.as_deref(), Some("English"));
        assert_eq!(grammar.nonterminals.len(), 1);
    }

    #[test]
    fn test_nonterminal_with_productions() {
        let source = "<greeting> ::= hello world | hi there";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 1);
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.name, "greeting");
        assert!(!nt.internal);
        assert_eq!(nt.productions.len(), 2);

        assert_eq!(nt.productions[0].tokens.len(), 2);
        assert_eq!(
            nt.productions[0].tokens[0].category,
            ProductionTokenCategory::FixedWord("hello".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1].category,
            ProductionTokenCategory::FixedWord("world".to_string())
        );

        assert_eq!(nt.productions[1].tokens.len(), 2);
        assert_eq!(
            nt.productions[1].tokens[0].category,
            ProductionTokenCategory::FixedWord("hi".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[1].category,
            ProductionTokenCategory::FixedWord("there".to_string())
        );
    }

    #[test]
    fn test_wildcards() {
        let cases = [
            ("<a> ::= ...", ProductionTokenCategory::MultipleWildcard),
            ("<a> ::= ......", ProductionTokenCategory::BalancedMultipleWildcard),
            ("<a> ::= ###", ProductionTokenCategory::SingleWildcard),
            ("<a> ::= ***", ProductionTokenCategory::PossiblyEmptyWildcard),
        ];
        for (source, expected) in cases {
            let grammar = parse_preform_grammar(source).unwrap();
            assert_eq!(grammar.nonterminals[0].productions[0].tokens[0].category, expected);
        }
    }

    #[test]
    fn test_four_asterisks_is_fixed_word() {
        let source = "<row> ::= ****";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals[0].productions[0].tokens.len(), 1);
        assert_eq!(
            grammar.nonterminals[0].productions[0].tokens[0].category,
            ProductionTokenCategory::FixedWord("****".to_string())
        );
    }

    #[test]
    fn test_sub_nonterminal() {
        let source = "<sentence> ::= <subject> <verb> <object>";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 1);
        assert_eq!(nt.productions[0].tokens.len(), 3);
        assert_eq!(
            nt.productions[0].tokens[0].category,
            ProductionTokenCategory::SubNonterminal("subject".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1].category,
            ProductionTokenCategory::SubNonterminal("verb".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[2].category,
            ProductionTokenCategory::SubNonterminal("object".to_string())
        );
    }

    #[test]
    fn test_mixed_production() {
        let source = "<rule> ::= to ... <action>";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 1);
        let tokens = &nt.productions[0].tokens;
        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens[0].category,
            ProductionTokenCategory::FixedWord("to".to_string())
        );
        assert_eq!(tokens[1].category, ProductionTokenCategory::MultipleWildcard);
        assert_eq!(
            tokens[2].category,
            ProductionTokenCategory::SubNonterminal("action".to_string())
        );
    }

    #[test]
    fn test_multiple_nonterminals() {
        let source = "<a> internal\n\n<b> ::= x | y\n\n<c> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 3);
        assert_eq!(grammar.nonterminals[0].name, "a");
        assert!(grammar.nonterminals[0].internal);
        assert_eq!(grammar.nonterminals[1].name, "b");
        assert!(!grammar.nonterminals[1].internal);
        assert_eq!(grammar.nonterminals[2].name, "c");
        assert!(grammar.nonterminals[2].internal);
    }

    #[test]
    fn test_multi_line_productions() {
        let source = "<heading> ::=\n    chapter : ... |\n    section : ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);
        assert_eq!(nt.productions[0].tokens.len(), 3);
        assert_eq!(
            nt.productions[0].tokens[0].category,
            ProductionTokenCategory::FixedWord("chapter".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1].category,
            ProductionTokenCategory::FixedWord(":".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[2].category,
            ProductionTokenCategory::MultipleWildcard
        );
    }

    #[test]
    fn test_display_roundtrip() {
        let source = "<test> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let displayed = format!("{}", grammar);
        let reparsed = parse_preform_grammar(&displayed).unwrap();
        assert_eq!(grammar, reparsed);
    }

    #[test]
    fn test_display_roundtrip_with_productions() {
        let source = "<greeting> ::=\n    hello world |\n    hi there\n";
        let grammar = parse_preform_grammar(source).unwrap();
        let displayed = format!("{}", grammar);
        let reparsed = parse_preform_grammar(&displayed).unwrap();
        assert_eq!(grammar, reparsed);
    }

    #[test]
    fn test_empty_source() {
        let grammar = parse_preform_grammar("").unwrap();
        assert!(grammar.nonterminals.is_empty());
        assert!(grammar.language.is_none());
    }

    #[test]
    fn test_comments() {
        let source = "<a> internal\n[this is a comment]\n<b> ::= x";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 2);
    }

    #[test]
    fn test_punctuation_fixed_words() {
        // `*` and `**` are fixed words; `***` is a possibly-empty wildcard;
        // `****` is a fixed word.
        let source = "<entry> ::= * | ** | *** | ****";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 4);
        assert_eq!(
            nt.productions[0].tokens[0].category,
            ProductionTokenCategory::FixedWord("*".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[0].category,
            ProductionTokenCategory::FixedWord("**".to_string())
        );
        assert_eq!(
            nt.productions[2].tokens[0].category,
            ProductionTokenCategory::PossiblyEmptyWildcard
        );
        assert_eq!(
            nt.productions[3].tokens[0].category,
            ProductionTokenCategory::FixedWord("****".to_string())
        );
    }

    #[test]
    fn test_colon_and_dash() {
        let source = "<heading> ::= chapter : ... | chapter - ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);
        assert_eq!(
            nt.productions[0].tokens[1].category,
            ProductionTokenCategory::FixedWord(":".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[1].category,
            ProductionTokenCategory::FixedWord("-".to_string())
        );
    }

    #[test]
    fn test_blank_lines_between_declarations() {
        let source = "<a> internal\n\n<b> ::= x | y\n\n<c> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 3);
        assert_eq!(grammar.nonterminals[0].name, "a");
        assert!(grammar.nonterminals[0].internal);
        assert_eq!(grammar.nonterminals[1].name, "b");
        assert!(!grammar.nonterminals[1].internal);
        assert_eq!(grammar.nonterminals[1].productions.len(), 2);
        assert_eq!(grammar.nonterminals[2].name, "c");
        assert!(grammar.nonterminals[2].internal);
    }

    #[test]
    fn test_display_roundtrip_internal() {
        let source = "<test> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let displayed = format!("{}", grammar);
        let reparsed = parse_preform_grammar(&displayed).unwrap();
        assert_eq!(grammar, reparsed);
    }

    #[test]
    fn test_negation_modifier() {
        let source = "<test> ::= ^<name>";
        let grammar = parse_preform_grammar(source).unwrap();
        let token = &grammar.nonterminals[0].productions[0].tokens[0];
        assert!(token.negated);
        assert_eq!(
            token.category,
            ProductionTokenCategory::SubNonterminal("name".to_string())
        );
        assert_eq!(format!("{}", token), "^<name>");
    }

    #[test]
    fn test_lower_case_modifier() {
        let source = "<test> ::= , _and <rest>";
        let grammar = parse_preform_grammar(source).unwrap();
        let tokens = &grammar.nonterminals[0].productions[0].tokens;
        assert_eq!(tokens[0].category, ProductionTokenCategory::FixedWord(",".to_string()));
        assert!(tokens[1].disallow_unexpected_upper);
        assert_eq!(
            tokens[1].category,
            ProductionTokenCategory::FixedWord("and".to_string())
        );
    }

    #[test]
    fn test_alternatives() {
        let source = "<test> ::= something/anything | it/he/she";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);

        let t1 = &nt.productions[0].tokens[0];
        assert_eq!(
            t1.category,
            ProductionTokenCategory::FixedWord("something".to_string())
        );
        assert_eq!(t1.alternatives.len(), 1);
        assert_eq!(
            t1.alternatives[0],
            ProductionTokenCategory::FixedWord("anything".to_string())
        );

        let t2 = &nt.productions[1].tokens[0];
        assert_eq!(
            t2.category,
            ProductionTokenCategory::FixedWord("it".to_string())
        );
        assert_eq!(t2.alternatives.len(), 2);
        assert_eq!(
            t2.alternatives[0],
            ProductionTokenCategory::FixedWord("he".to_string())
        );
        assert_eq!(
            t2.alternatives[1],
            ProductionTokenCategory::FixedWord("she".to_string())
        );
    }

    #[test]
    fn test_escaped_token() {
        let source = "<test> ::= \\{ \\} | \\***";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);

        assert!(nt.productions[0].tokens[0].escaped);
        assert_eq!(
            nt.productions[0].tokens[0].category,
            ProductionTokenCategory::FixedWord("{".to_string())
        );
        assert!(nt.productions[0].tokens[1].escaped);
        assert_eq!(
            nt.productions[0].tokens[1].category,
            ProductionTokenCategory::FixedWord("}".to_string())
        );

        assert!(nt.productions[1].tokens[0].escaped);
        assert_eq!(
            nt.productions[1].tokens[0].category,
            ProductionTokenCategory::FixedWord("***".to_string())
        );
    }

    #[test]
    fn test_production_match_number() {
        let source = "<article> ::= /a/ a/an | /d/ some";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);
        assert_eq!(nt.productions[0].match_number, Some(0));
        assert_eq!(nt.productions[1].match_number, Some(3));
        assert_eq!(nt.productions[0].tokens[0].category, ProductionTokenCategory::FixedWord("a".to_string()));
    }

    #[test]
    fn test_braced_word_ranges() {
        let source = "<test> ::= {another} | {each other in groups}";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);

        let p1 = &nt.productions[0];
        assert_eq!(p1.tokens.len(), 1);
        assert_eq!(p1.tokens[0].range_start, Some(1));
        assert_eq!(p1.tokens[0].range_end, Some(1));

        let p2 = &nt.productions[1];
        assert_eq!(p2.tokens.len(), 4);
        assert_eq!(p2.tokens[0].range_start, Some(1));
        assert!(p2.tokens[0].range_end.is_none());
        assert!(p2.tokens[1].range_start.is_none());
        assert!(p2.tokens[1].range_end.is_none());
        assert!(p2.tokens[2].range_start.is_none());
        assert!(p2.tokens[2].range_end.is_none());
        assert!(p2.tokens[3].range_start.is_none());
        assert_eq!(p2.tokens[3].range_end, Some(1));
    }

    #[test]
    fn test_result_number() {
        let source = "<test> ::= <foo>? 1 <bar>";
        let grammar = parse_preform_grammar(source).unwrap();
        let tokens = &grammar.nonterminals[0].productions[0].tokens;
        assert_eq!(tokens[0].result_index, Some(1));
        assert_eq!(tokens[1].result_index, None);
    }

    #[test]
    fn test_display_roundtrip_full_featured() {
        let source = concat!(
            "language English\n\n",
            "<a> internal\n\n",
            "<b> ::=\n",
            "    ^<x> |\n",
            "    {each other in groups} |\n",
            "    /a/ _and <y> |\n",
            "    \\{ \\} |\n",
            "    ### ... ***\n"
        );
        let grammar = parse_preform_grammar(source).unwrap();
        let displayed = format!("{}", grammar);
        let reparsed = parse_preform_grammar(&displayed).unwrap();
        assert_eq!(grammar, reparsed);
    }

    #[test]
    fn test_load_real_syntax_preform() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/retrospective/6M62/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");
        // The English Syntax.preform has ~720 nonterminals.
        assert!(
            grammar.nonterminals.len() > 700,
            "expected ~720 nonterminals, got {}",
            grammar.nonterminals.len()
        );
        assert_eq!(grammar.language.as_deref(), Some("English"));

        // Verify a few known nonterminals are present.
        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"quoted-text"), "missing quoted-text");
        assert!(names.contains(&"table-column-heading"), "missing table-column-heading");
        assert!(names.contains(&"extension-documentation-heading"), "missing extension-documentation-heading");

        // Sanity-check row-of-asterisks: escaped `***` is a literal fixed word;
        // `****` is a fixed word; `*` and `**` are fixed words.
        let row = grammar.nonterminals.iter().find(|n| n.name == "row-of-asterisks").unwrap();
        assert_eq!(row.productions.len(), 4);
        assert_eq!(row.productions[0].tokens[0].category, ProductionTokenCategory::FixedWord("*".to_string()));
        assert_eq!(row.productions[1].tokens[0].category, ProductionTokenCategory::FixedWord("**".to_string()));
        assert!(row.productions[2].tokens[0].escaped);
        assert_eq!(row.productions[2].tokens[0].category, ProductionTokenCategory::FixedWord("***".to_string()));
        assert_eq!(row.productions[3].tokens[0].category, ProductionTokenCategory::FixedWord("****".to_string()));

        // Sanity-check escaped braces in s-literal-list.
        let lit = grammar.nonterminals.iter().find(|n| n.name == "s-literal-list").unwrap();
        let first = &lit.productions[0].tokens[0];
        assert!(first.escaped);
        assert_eq!(first.category, ProductionTokenCategory::FixedWord("{".to_string()));

        // Sanity-check a negated sub-nonterminal.
        let np = grammar.nonterminals.iter().find(|n| n.name == "s-noun-phrase").unwrap();
        let negated = np.productions.iter().find(|p| {
            p.tokens.iter().any(|t| t.negated)
        }).expect("expected a negated production");
        assert!(negated.tokens.iter().any(|t| t.negated && matches!(t.category, ProductionTokenCategory::SubNonterminal(_))));

        // Sanity-check a `_` modifier with slash alternatives.
        let tail = grammar.nonterminals.iter().find(|n| n.name == "equation-where-tail").unwrap();
        let and_token = tail.productions.iter()
            .flat_map(|p| &p.tokens)
            .find(|t| {
                t.disallow_unexpected_upper
                    && t.category == ProductionTokenCategory::FixedWord(",".to_string())
                    && t.alternatives.iter().any(|a| *a == ProductionTokenCategory::FixedWord("and".to_string()))
            })
            .expect("expected a `_,/and` token");
        assert_eq!(and_token.category, ProductionTokenCategory::FixedWord(",".to_string()));
        assert!(and_token.alternatives.iter().any(|a| *a == ProductionTokenCategory::FixedWord("and".to_string())));
    }

    // -----------------------------------------------------------------------
    // Matching engine tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_match_simple_fixed_words() {
        let source = "<greeting> ::= hello world";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "world"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "greeting", Wording::new(0, 2));
        assert!(m.is_some());
        let m = m.unwrap();
        assert_eq!(m.match_number, 0);
        assert_eq!(m.production_index, 0);
        assert_eq!(m.word_range, 0..2);
    }

    #[test]
    fn test_match_no_match() {
        let source = "<greeting> ::= hello world";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "there"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "greeting", Wording::new(0, 2));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_multiple_productions_first_wins() {
        let source = "<greeting> ::= hi there | hello world";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "world"];

        // First production "hi there" doesn't match, second "hello world" does.
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "greeting", Wording::new(0, 2));
        assert!(m.is_some());
        let m = m.unwrap();
        assert_eq!(m.production_index, 1);
        assert_eq!(m.match_number, 1);
    }

    #[test]
    fn test_match_single_wildcard() {
        let source = "<a> ::= ### ###";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "world"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_multiple_wildcard() {
        let source = "<a> ::= start ... end";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["start", "hello", "world", "end"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 4));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_multiple_wildcard_minimum() {
        let source = "<a> ::= start ... end";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["start", "end"];

        // `...` must match at least 1 word, so this should fail.
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_possibly_empty_wildcard() {
        let source = "<a> ::= start *** end";
        let grammar = parse_preform_grammar(source).unwrap();

        // `***` can match zero words.
        let words = &["start", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_some());

        // `***` can match one or more words.
        let words = &["start", "hello", "world", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 4));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_sub_nonterminal() {
        let source = "<word> ::= hello | world\n<phrase> ::= <word> <word>";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "world"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "phrase", Wording::new(0, 2));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_sub_nonterminal_recursive() {
        let source = "<a> ::= x | x <a>";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["x", "x", "x"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 3));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_internal_nonterminal() {
        let source = "<internal-nt> internal";
        let grammar = parse_preform_grammar(source).unwrap();
        let words: &[&str] = &[];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "internal-nt", Wording::new(0, 0));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_nonexistent_nonterminal() {
        let source = "<a> ::= hello";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "nonexistent", Wording::new(0, 1));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_empty_wording() {
        let source = "<a> ::= ***";
        let grammar = parse_preform_grammar(source).unwrap();
        let words: &[&str] = &[];

        // `***` matches zero words, so empty wording should match.
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 0));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_empty_wording_no_empty_wildcard() {
        let source = "<a> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let words: &[&str] = &[];

        // `...` requires at least 1 word, so empty wording should not match.
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 0));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_alternatives() {
        let source = "<a> ::= hello/world";
        let grammar = parse_preform_grammar(source).unwrap();

        let words = &["hello"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_some());

        let words = &["world"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_some());

        let words = &["foo"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_negated_fixed_word() {
        let source = "<a> ::= ^hello world";
        let grammar = parse_preform_grammar(source).unwrap();

        // "hello" is negated, so "hello world" should NOT match.
        let words = &["hello", "world"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_none());

        // "foo world" should match because "foo" != "hello".
        let words = &["foo", "world"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_negated_sub_nonterminal() {
        let source = "<word> ::= hello | world\n<a> ::= ^<word> foo";
        let grammar = parse_preform_grammar(source).unwrap();

        // "hello foo" should NOT match because "hello" matches <word>.
        let words = &["hello", "foo"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_none());

        // "bar foo" should match because "bar" does not match <word>.
        let words = &["bar", "foo"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_backtracking_wildcard_then_fixed() {
        // Production: ... end
        // Input: "hello end world end"
        // `...` should match "hello end world" (greedy would fail because
        // "end" would be consumed by `...` and then there's no "end" left).
        // Non-greedy: `...` matches "hello", then "end" matches, then
        // "world end" is left over → fail. Backtrack: `...` matches "hello end",
        // then "end" matches "world" → fail. Backtrack: `...` matches "hello end world",
        // then "end" matches "end" → success.
        let source = "<a> ::= ... end";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "end", "world", "end"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 4));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_backtracking_sub_nt_then_fixed() {
        let source = "<word> ::= hello | world\n<a> ::= <word> world";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello", "world"];

        // <word> tries "hello" (1 word), then "world" must match → success.
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 2));
        assert!(m.is_some());
    }

    #[test]
    fn test_match_production_match_number() {
        let source = "<article> ::= /a/ a | /d/ the";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["a"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "article", Wording::new(0, 1));
        assert!(m.is_some());
        assert_eq!(m.unwrap().match_number, 0);

        let words = &["the"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "article", Wording::new(0, 1));
        assert!(m.is_some());
        assert_eq!(m.unwrap().match_number, 3);
    }

    #[test]
    fn test_match_balanced_wildcard() {
        let source = "<a> ::= start ...... end";
        let grammar = parse_preform_grammar(source).unwrap();

        // Balanced brackets.
        let words = &["start", "(", "hello", ")", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 5));
        assert!(m.is_some());

        // Unbalanced brackets should fail.
        let words = &["start", "(", "hello", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 4));
        assert!(m.is_none());
    }

    #[test]
    fn test_match_real_syntax_preform_nonterminal() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/retrospective/6M62/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");

        // Test matching "row-of-asterisks" — a simple nonterminal with fixed words.
        let row_nt = grammar.nonterminals.iter().find(|n| n.name == "row-of-asterisks")
            .expect("row-of-asterisks not found");
        assert_eq!(row_nt.productions.len(), 4);

        // Match "*" against the first production.
        let words = &["*"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "row-of-asterisks", Wording::new(0, 1));
        assert!(m.is_some(), "row-of-asterisks should match '*'");

        // Match "**" against the second production.
        let words = &["**"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "row-of-asterisks", Wording::new(0, 1));
        assert!(m.is_some(), "row-of-asterisks should match '**'");

        // Match "***" against the third production (escaped).
        let words = &["***"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "row-of-asterisks", Wording::new(0, 1));
        assert!(m.is_some(), "row-of-asterisks should match '***'");

        // Match "****" against the fourth production.
        let words = &["****"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "row-of-asterisks", Wording::new(0, 1));
        assert!(m.is_some(), "row-of-asterisks should match '****'");

        // "*****" should not match any production.
        let words = &["*****"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "row-of-asterisks", Wording::new(0, 1));
        assert!(m.is_none(), "row-of-asterisks should not match '*****'");
    }

    #[test]
    fn test_match_disallow_unexpected_upper() {
        let source = "<a> ::= _hello";
        let grammar = parse_preform_grammar(source).unwrap();

        // Lowercase "hello" should match.
        let words = &["hello"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_some(), "lowercase 'hello' should match _hello");

        // Uppercase "Hello" should NOT match because of the `_` modifier.
        let words = &["Hello"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_none(), "uppercase 'Hello' should not match _hello");
    }

    #[test]
    fn test_match_sub_nonterminal_zero_length() {
        // A sub-NT that can match empty text (via `***`).
        let source = "<empty> ::= ***\n<a> ::= <empty> hello";
        let grammar = parse_preform_grammar(source).unwrap();
        let words = &["hello"];

        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 1));
        assert!(m.is_some(), "<empty> should match 0 words before 'hello'");
    }

    #[test]
    fn test_match_balanced_wildcard_with_braces() {
        let source = "<a> ::= start ...... end";
        let grammar = parse_preform_grammar(source).unwrap();

        // Balanced braces.
        let words = &["start", "{", "hello", "}", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 5));
        assert!(m.is_some(), "balanced braces should match ......");

        // Unbalanced braces should fail.
        let words = &["start", "{", "hello", "end"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 4));
        assert!(m.is_none(), "unbalanced braces should not match ......");
    }

    #[test]
    fn test_match_negated_sub_nonterminal_multi_word() {
        // Note: This test documents a known limitation. The current implementation
        // consumes only 1 word for negated sub-NTs, which may fail for multi-word
        // negated sub-NTs. Full strut/lookahead support would fix this.
        let source = "<word> ::= hello world\n<a> ::= ^<word> foo";
        let grammar = parse_preform_grammar(source).unwrap();

        // "hello there foo": <word> doesn't match "hello there" (2 words),
        // so negation succeeds. Current impl consumes 1 word ("hello"),
        // then "there foo" doesn't match "foo" → fails.
        // This is a known limitation.
        let words = &["hello", "there", "foo"];
        let m = match_nonterminal_impl(&PreformContext { grammar: &grammar, word_text: words, is_paragraph_start: false, verbs_registry: None }, &InternalRegistry::new(), "a", Wording::new(0, 3));
        // Currently expected to fail due to the limitation.
        // When strut support is added, this should succeed.
        assert!(m.is_none(), "known limitation: multi-word negated sub-NT");
    }

    #[test]
    fn test_current_syntax_preform_parses() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../gitignore/inform/inform7/Internal/Languages/English/Syntax.preform"
        );
        let source = std::fs::read_to_string(path)
            .expect("failed to read Syntax.preform");
        let grammar = parse_preform_grammar(&source)
            .expect("failed to parse Syntax.preform");
        // The English Syntax.preform has ~720 nonterminals.
        assert!(
            grammar.nonterminals.len() > 700,
            "expected ~720 nonterminals, got {}",
            grammar.nonterminals.len()
        );
        assert_eq!(grammar.language.as_deref(), Some("English"));
        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"if-start-of-paragraph"), "missing if-start-of-paragraph");
        assert!(names.contains(&"if-not-cap"), "missing if-not-cap");
        assert!(names.contains(&"preform-nonterminal"), "missing preform-nonterminal");
        assert!(names.contains(&"dividing-sentence"), "missing dividing-sentence");
    }

    #[test]
    fn test_multi_line_tabular_production() {
        let source = "<en-article-declension> ::=
    a           a    a
                some some |
    the         the  the
                the  the";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 1);
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.name, "en-article-declension");
        assert_eq!(nt.productions.len(), 2);
        assert_eq!(nt.productions[0].tokens.len(), 5);
        assert_eq!(nt.productions[1].tokens.len(), 5);
    }
}
