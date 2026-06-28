//! Preform grammar parser.
//!
//! Parses the `Syntax.preform` format used to define the I7 grammar.
//! The format is a text-based grammar definition language with ~720
//! nonterminals for English alone.
//!
//! This module handles **parsing only** — loading the grammar into in-memory
//! data structures. The matching engine (which takes a nonterminal and a
//! wording and tries to match it) comes in a later plan.
//!
//! # Format
//!
//! ```text
//! <nonterminal-name> internal
//!
//! <nonterminal-name> ::=
//!     production1 |
//!     production2 |
//!     ...
//! ```
//!
//! Where productions contain:
//! - **Fixed words**: literal text like `to`, `is`, `a`, `room`
//! - **Wildcards**: `...` (matches any number of words)
//! - **Sub-nonterminals**: `<quoted-text>`, `<if-start-of-paragraph>`, etc.
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 4/Loading Preform.w`
//! - C reference: `services/words-module/Chapter 4/Preform.w`

use std::fmt;

/// A complete Preform grammar, containing all nonterminals.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grammar {
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
    /// The tokens that make up this production.
    pub tokens: Vec<ProductionToken>,
}

/// A token within a production.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionToken {
    /// A literal word that must appear verbatim (e.g., `"to"`, `"is"`, `"room"`).
    FixedWord(String),
    /// A wildcard matching any number of words (`...`).
    Wildcard,
    /// A reference to another nonterminal (e.g., `<quoted-text>`).
    SubNonterminal(String),
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        for (i, token) in self.tokens.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}

impl fmt::Display for ProductionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProductionToken::FixedWord(word) => write!(f, "{}", word),
            ProductionToken::Wildcard => write!(f, "..."),
            ProductionToken::SubNonterminal(name) => write!(f, "<{}>", name),
        }
    }
}

// ---------------------------------------------------------------------------
// PEG parser for the Preform format
// ---------------------------------------------------------------------------

peg::parser! {
    grammar preform_parser() for str {
        /// Parse an entire Preform grammar source.
        pub(crate) rule grammar() -> Grammar
            = decls:top_level_item()* EOI()
              { Grammar { nonterminals: decls.into_iter().flatten().collect() } }

        // A top-level item: nonterminal declaration or language switch.
        rule top_level_item() -> Option<Nonterminal>
            = n:nonterminal_declaration() { Some(n) }
            / language_declaration() { None }

        // Language switch: "language English"
        rule language_declaration()
            = _ "language" ws() name:$((![' ' | '\t' | '\n' | '\r'] [_])+) _

        // Whitespace within a line (spaces and tabs only)
        rule ws()
            = quiet!{ ([' ' | '\t'] / comment())* }

        // Whitespace including newlines (for between declarations)
        rule _()
            = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }

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
                  let mut v = vec![p];
                  v.extend(rest);
                  v
              }

        // A single production: a sequence of tokens
        // Fails if the next content looks like a new nonterminal declaration.
        rule production() -> Production
            = !nonterminal_start() tokens:(ws() token:production_token() { token })+
              { Production { tokens } }

        // Check if the next content looks like a nonterminal declaration start.
        rule nonterminal_start()
            = "<" n:$((!['>'] [_])+) ">" ws() ("::=" / "internal")

        // A single token in a production
        rule production_token() -> ProductionToken
            = negation_modifier()
            / wildcard()
            / sub_nonterminal()
            / fixed_word()

        // Negation modifier: ^
        rule negation_modifier() -> ProductionToken
            = "^" { ProductionToken::FixedWord("^".to_string()) }

        // Wildcard: ... or ......
        rule wildcard() -> ProductionToken
            = "......" { ProductionToken::Wildcard }
            / "..." { ProductionToken::Wildcard }

        // Sub-nonterminal reference: <name>
        rule sub_nonterminal() -> ProductionToken
            = n:nonterminal_name() { ProductionToken::SubNonterminal(n) }

        // A fixed word: any non-whitespace sequence that isn't a special token.
        rule fixed_word() -> ProductionToken
            = !"|" w:$((![' ' | '\t' | '\n' | '\r'] [_])+)
              { ProductionToken::FixedWord(w.to_string()) }

        // Comments: [ ... ]
        rule comment()
            = "[" $((![']'] [_])*) "]"

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
    fn test_nonterminal_with_productions() {
        let source = "<greeting> ::= hello world | hi there";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 1);
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.name, "greeting");
        assert!(!nt.internal);
        assert_eq!(nt.productions.len(), 2);

        // First production: hello world
        assert_eq!(nt.productions[0].tokens.len(), 2);
        assert_eq!(
            nt.productions[0].tokens[0],
            ProductionToken::FixedWord("hello".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1],
            ProductionToken::FixedWord("world".to_string())
        );

        // Second production: hi there
        assert_eq!(nt.productions[1].tokens.len(), 2);
        assert_eq!(
            nt.productions[1].tokens[0],
            ProductionToken::FixedWord("hi".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[1],
            ProductionToken::FixedWord("there".to_string())
        );
    }

    #[test]
    fn test_wildcard() {
        let source = "<anything> ::= ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 1);
        assert_eq!(nt.productions[0].tokens.len(), 1);
        assert_eq!(nt.productions[0].tokens[0], ProductionToken::Wildcard);
    }

    #[test]
    fn test_sub_nonterminal() {
        let source = "<sentence> ::= <subject> <verb> <object>";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 1);
        assert_eq!(nt.productions[0].tokens.len(), 3);
        assert_eq!(
            nt.productions[0].tokens[0],
            ProductionToken::SubNonterminal("subject".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1],
            ProductionToken::SubNonterminal("verb".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[2],
            ProductionToken::SubNonterminal("object".to_string())
        );
    }

    #[test]
    fn test_mixed_production() {
        let source = "<rule> ::= to ... <action>";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 1);
        assert_eq!(nt.productions[0].tokens.len(), 3);
        assert_eq!(
            nt.productions[0].tokens[0],
            ProductionToken::FixedWord("to".to_string())
        );
        assert_eq!(nt.productions[0].tokens[1], ProductionToken::Wildcard);
        assert_eq!(
            nt.productions[0].tokens[2],
            ProductionToken::SubNonterminal("action".to_string())
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
            nt.productions[0].tokens[0],
            ProductionToken::FixedWord("chapter".to_string())
        );
        assert_eq!(
            nt.productions[0].tokens[1],
            ProductionToken::FixedWord(":".to_string())
        );
        assert_eq!(nt.productions[0].tokens[2], ProductionToken::Wildcard);
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
    }

    #[test]
    fn test_comments() {
        let source = "<a> internal\n[this is a comment]\n<b> ::= x";
        let grammar = parse_preform_grammar(source).unwrap();
        assert_eq!(grammar.nonterminals.len(), 2);
    }

    #[test]
    fn test_six_dot_wildcard() {
        let source = "<balanced> ::= ......";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions[0].tokens.len(), 1);
        assert_eq!(nt.productions[0].tokens[0], ProductionToken::Wildcard);
    }

    #[test]
    fn test_punctuation_fixed_words() {
        let source = "<entry> ::= * | ** | ***";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 3);
        assert_eq!(
            nt.productions[0].tokens[0],
            ProductionToken::FixedWord("*".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[0],
            ProductionToken::FixedWord("**".to_string())
        );
        assert_eq!(
            nt.productions[2].tokens[0],
            ProductionToken::FixedWord("***".to_string())
        );
    }

    #[test]
    fn test_colon_and_dash() {
        let source = "<heading> ::= chapter : ... | chapter - ...";
        let grammar = parse_preform_grammar(source).unwrap();
        let nt = &grammar.nonterminals[0];
        assert_eq!(nt.productions.len(), 2);
        assert_eq!(
            nt.productions[0].tokens[1],
            ProductionToken::FixedWord(":".to_string())
        );
        assert_eq!(
            nt.productions[1].tokens[1],
            ProductionToken::FixedWord("-".to_string())
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
        // Verify a few known nonterminals are present.
        let names: Vec<&str> = grammar.nonterminals.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"quoted-text"), "missing quoted-text");
        assert!(names.contains(&"table-column-heading"), "missing table-column-heading");
        assert!(names.contains(&"extension-documentation-heading"), "missing extension-documentation-heading");
    }
}
