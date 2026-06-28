//! Sentence breaker for Inform 7 source text.
//!
//! The sentence breaker takes a token stream from the lexer and splits it
//! into sentences, classifying each sentence by type. This mirrors the C
//! implementation in `services/syntax-module/Chapter 3/Sentences.w`.
//!
//! # Sentence breaking
//!
//! Sentences are delimited by:
//! - Full stops `.` (standard sentence end)
//! - Semicolons `;` (rule phrase separator)
//! - Colons `:` (rule preamble end)
//! - Paragraph breaks (blank lines)
//! - Quoted text ending with `?!.` followed by a capital letter ("X break")
//!
//! Multiple consecutive stop characters (e.g., `.;;`) are treated as a
//! single division — only the last one counts for classification.
//!
//! # Rule mode
//!
//! After a colon, the sentence breaker enters "rule mode". Subsequent
//! sentences ending with semicolons are rule phrases. A sentence ending
//! with a full stop or paragraph break exits rule mode.
//!
//! # Table mode
//!
//! After a "Table ..." sentence, the sentence breaker enters "table mode".
//! In table mode, X breaks (quoted text ending with punctuation) are
//! disabled, because table entries routinely contain quoted text with
//! punctuation that shouldn't end the sentence.
//!
//! # References
//!
//! - C reference: `services/syntax-module/Chapter 3/Sentences.w`

use crate::token::Token;
use crate::SyntaxKind;
use std::ops::Range;

/// A classified sentence from an I7 source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    /// The range of tokens this sentence covers.
    ///
    /// This range is **exclusive** of the terminating punctuation token
    /// (full stop, semicolon, colon, or paragraph break). The stop
    /// character is recorded in the `classification` instead.
    pub token_range: Range<usize>,

    /// The classification of this sentence.
    pub classification: SentenceClassification,
}

/// How a sentence is classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SentenceClassification {
    /// A heading: `Chapter 1 - The Beginning`
    Heading {
        /// The heading level (1 = Volume, 5 = Section).
        level: HeadingLevel,
    },

    /// A structural sentence: `Include ...`, `Table ...`, etc.
    Structural(StructuralType),

    /// A regular sentence (assertion, phrase, rule, etc.)
    Regular,

    /// A rule preamble ending with a colon
    RulePreamble,

    /// A rule body phrase ending with a semicolon
    RulePhrase,
}

/// Heading levels matching the C hierarchy.
///
/// Lower numbers are higher in the hierarchy. Volume (1) is the highest,
/// Section (5) is the lowest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadingLevel {
    Volume = 1,
    Book = 2,
    Part = 3,
    Chapter = 4,
    Section = 5,
}

/// Types of structural sentences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralType {
    /// `Include ... by ...`
    Include,
    /// `Table of ...`
    Table,
    /// `Equation ...`
    Equation,
    /// `Use ...`
    Use,
}

/// Break a token stream into sentences.
///
/// Returns a vector of `Sentence` values, each covering a contiguous
/// range of tokens (exclusive of the terminating punctuation).
///
/// # Example
///
/// ```
/// use conform7_syntax::{break_sentences, Lexer};
///
/// let source = "The Lab is a room. Peter is a man.";
/// let tokens = Lexer::tokenize(source).unwrap();
/// let sentences = break_sentences(&tokens);
/// assert_eq!(sentences.len(), 2);
/// ```
pub fn break_sentences(tokens: &[Token]) -> Vec<Sentence> {
    let mut breaker = BreakerState::new(tokens);
    breaker.run();
    breaker.sentences
}

/// Internal sentence breaker state machine.
struct BreakerState<'a> {
    /// The token stream being broken into sentences.
    tokens: &'a [Token],
    /// Current position in the token stream.
    pos: usize,
    /// Accumulated sentences.
    sentences: Vec<Sentence>,
    /// Start position of the current sentence being accumulated.
    sentence_start: usize,
    /// Whether we are inside a rule definition (after a colon).
    inside_rule_mode: bool,
    /// Whether we are inside a table definition.
    inside_table_mode: bool,
}

impl<'a> BreakerState<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            sentences: Vec::new(),
            sentence_start: 0,
            inside_rule_mode: false,
            inside_table_mode: false,
        }
    }

    /// Run the sentence breaker to completion.
    fn run(&mut self) {
        // Skip leading whitespace, newlines, comments, and paragraph breaks
        self.skip_leading_junk();

        // Check if the first sentence is a table
        self.check_table_mode();

        while self.pos < self.tokens.len() {
            let token = &self.tokens[self.pos];
            let mut stop_char: Option<char> = None;

            // Check for sentence-ending tokens
            match token.kind {
                SyntaxKind::ParagraphBreak => {
                    stop_char = Some('|');
                }
                SyntaxKind::PUNCTUATION => match token.text.as_str() {
                    "." => {
                        stop_char = Some('.');
                    }
                    ";" => {
                        stop_char = Some(';');
                    }
                    ":" if !self.is_time_colon() => {
                        stop_char = Some(':');
                    }
                    _ => {}
                },
                _ => {}
            }

            // Check for X break: quoted text ending with punctuation
            // followed by a capital letter (disabled in table mode)
            if stop_char.is_none() && !self.inside_table_mode && self.check_x_break() {
                // X break: emit sentence up to current position
                // (no stop token to skip)
                self.emit_sentence('.');
                // Only advance if we made progress (avoid infinite loop
                // when the sentence is empty)
                if self.sentence_start < self.pos {
                    self.sentence_start = self.pos;
                } else {
                    self.pos += 1;
                    self.sentence_start = self.pos;
                }
                self.update_rule_mode('.');
                self.skip_leading_junk();
                self.check_table_mode();
                continue;
            }

            if let Some(sc) = stop_char {
                // Count consecutive stop tokens (e.g., ".;;")
                let no_stop_tokens = self.count_consecutive_stops();

                // Emit sentence (exclusive of stop tokens)
                self.emit_sentence(sc);

                // Advance past stop tokens
                self.pos += no_stop_tokens;
                self.sentence_start = self.pos;

                // Update rule mode
                self.update_rule_mode(sc);

                // Skip leading junk at new sentence start
                self.skip_leading_junk();

                // Check if the new sentence is a table
                self.check_table_mode();
            } else {
                self.pos += 1;
            }
        }

        // Flush any remaining text as a sentence
        if self.sentence_start < self.pos {
            self.emit_sentence('.');
        }
    }

    /// Skip whitespace, newlines, comments, and paragraph breaks at the
    /// start of a sentence.
    fn skip_leading_junk(&mut self) {
        while self.pos < self.tokens.len() {
            let kind = self.tokens[self.pos].kind;
            if matches!(
                kind,
                SyntaxKind::WHITESPACE
                    | SyntaxKind::NEWLINE
                    | SyntaxKind::COMMENT
                    | SyntaxKind::ParagraphBreak
            ) {
                self.pos += 1;
                self.sentence_start = self.pos;
            } else {
                break;
            }
        }
    }

    /// Check if the colon at the current position is part of a time
    /// notation (e.g., `1:34`). Returns `true` if the preceding and
    /// following tokens are both numbers.
    fn is_time_colon(&self) -> bool {
        if self.pos > 0
            && self.pos + 1 < self.tokens.len()
            && self.tokens[self.pos - 1].kind == SyntaxKind::NUMBER
            && self.tokens[self.pos + 1].kind == SyntaxKind::NUMBER
        {
            return true;
        }
        false
    }

    /// Check for an X break: the current token is a WORD starting with
    /// an uppercase letter, and the preceding non-whitespace token is a
    /// STRING whose text ends with `.`, `?`, or `!`.
    ///
    /// Note: We only skip WHITESPACE and NEWLINE when looking back for
    /// the preceding token. If a COMMENT token sits between the quoted
    /// text and the uppercase word, the X break won't fire. This is a
    /// known limitation — the C lexer strips comments, so they never
    /// appear between words.
    ///
    /// Returns `true` if an X break is detected at the current position.
    fn check_x_break(&self) -> bool {
        if self.pos == 0 {
            return false;
        }
        let current = &self.tokens[self.pos];

        // Current token must be a WORD starting with uppercase
        if current.kind != SyntaxKind::WORD {
            return false;
        }
        let first_char = current.text.chars().next();
        if !first_char.is_some_and(|c| c.is_uppercase()) {
            return false;
        }

        // Look back past whitespace/newlines to find the previous
        // non-whitespace token
        let mut prev_pos = self.pos.saturating_sub(1);
        while prev_pos > 0 {
            let kind = self.tokens[prev_pos].kind;
            if kind != SyntaxKind::WHITESPACE && kind != SyntaxKind::NEWLINE {
                break;
            }
            prev_pos = prev_pos.saturating_sub(1);
        }
        let prev = &self.tokens[prev_pos];

        // Previous token must be a STRING ending with sentence-ending
        // punctuation. Strip the surrounding quotes before checking.
        if prev.kind != SyntaxKind::STRING {
            return false;
        }
        let text = &prev.text;
        // STRING tokens include the surrounding quotes, e.g., "\"Look out!\""
        // We need to check the content between the quotes.
        let content = if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
            &text[1..text.len() - 1]
        } else {
            text.as_str()
        };
        let last_char = content.chars().last();
        matches!(last_char, Some('.') | Some('?') | Some('!'))
    }

    /// Count how many consecutive stop tokens (paragraph breaks, full
    /// stops, semicolons) follow the current position.
    ///
    /// Note: In the C implementation, whitespace is not a word token,
    /// so `. ;` (period, space, semicolon) counts as two consecutive
    /// stops. In our token stream, whitespace IS a token, so we stop
    /// counting at the whitespace. This means the stop character used
    /// for classification may differ from C in rare cases (the first
    /// stop vs. the last). The end result (number of non-empty
    /// sentences) is the same.
    fn count_consecutive_stops(&self) -> usize {
        let mut count = 1; // current token is a stop
        while self.pos + count < self.tokens.len() {
            let next = &self.tokens[self.pos + count];
            match next.kind {
                SyntaxKind::ParagraphBreak => {
                    count += 1;
                }
                SyntaxKind::PUNCTUATION => match next.text.as_str() {
                    "." | ";" => {
                        count += 1;
                    }
                    _ => break,
                },
                _ => break,
            }
        }
        count
    }

    /// Emit a sentence from `sentence_start` to `pos` (exclusive of the
    /// stop token at `pos`).
    fn emit_sentence(&mut self, stop_character: char) {
        let range = self.sentence_start..self.pos;
        if range.is_empty() {
            return;
        }

        let classification = self.classify_sentence(stop_character);

        self.sentences.push(Sentence {
            token_range: range,
            classification,
        });
    }

    /// Classify a sentence based on its first relevant token and stop
    /// character.
    fn classify_sentence(&self, stop_character: char) -> SentenceClassification {
        let first_token = self.first_relevant_token(self.sentence_start);

        // Check for heading
        if let Some(token) = first_token {
            if token.kind == SyntaxKind::HeadingMarker {
                let level = match token.text.as_str() {
                    "Volume" => HeadingLevel::Volume,
                    "Book" => HeadingLevel::Book,
                    "Part" => HeadingLevel::Part,
                    "Chapter" => HeadingLevel::Chapter,
                    "Section" => HeadingLevel::Section,
                    _ => unreachable!("unknown heading marker: {}", token.text),
                };
                return SentenceClassification::Heading { level };
            }
        }

        // Check for structural sentences
        if let Some(token) = first_token {
            if token.kind == SyntaxKind::WORD {
                match token.text.as_str() {
                    "Include" => {
                        return SentenceClassification::Structural(StructuralType::Include)
                    }
                    "Table" => {
                        return SentenceClassification::Structural(StructuralType::Table)
                    }
                    "Equation" => {
                        return SentenceClassification::Structural(StructuralType::Equation)
                    }
                    "Use" => return SentenceClassification::Structural(StructuralType::Use),
                    _ => {}
                }
            }
        }

        // Check for rule mode
        if stop_character == ':' {
            return SentenceClassification::RulePreamble;
        }

        if self.inside_rule_mode && stop_character == ';' {
            return SentenceClassification::RulePhrase;
        }

        SentenceClassification::Regular
    }

    /// Update rule mode based on the stop character.
    fn update_rule_mode(&mut self, stop_character: char) {
        match stop_character {
            ':' => {
                // Colon enters rule mode
                self.inside_rule_mode = true;
            }
            '.' | '|' => {
                // Full stop or paragraph break exits rule mode
                self.inside_rule_mode = false;
            }
            _ => {}
        }
    }

    /// Check if the current sentence (starting at `sentence_start`) is
    /// a table definition, and if so, enter table mode.
    ///
    /// Looks at the first non-whitespace token at or after `sentence_start`
    /// to determine if this is a table sentence.
    ///
    /// Note: The C `<structural-sentence>` nonterminal detects multiple
    /// tabbed structural types. In practice, only `Table` uses tabbed
    /// rows in I7, so we simplify by checking for the word "Table" alone.
    fn check_table_mode(&mut self) {
        let first = self.first_token_of_sentence();
        self.inside_table_mode = first.is_some_and(|t| t.kind == SyntaxKind::WORD && t.text == "Table");
    }

    /// Find the first non-whitespace, non-newline, non-comment token at
    /// or after `sentence_start`, scanning forward through all remaining
    /// tokens (not bounded by `self.pos`).
    fn first_token_of_sentence(&self) -> Option<&Token> {
        self.tokens[self.sentence_start..]
            .iter()
            .find(|t| {
                !matches!(
                    t.kind,
                    SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
                )
            })
    }

    /// Find the first non-whitespace, non-newline, non-comment token at
    /// or after the given position, searching only within the current
    /// sentence (up to `self.pos`, which is at the stop token).
    ///
    /// This is used by `classify_sentence` to inspect the first token
    /// of the sentence that was just scanned. The bound at `self.pos`
    /// is intentional: we only want tokens that are part of the
    /// sentence, not tokens beyond the stop.
    fn first_relevant_token(&self, start: usize) -> Option<&Token> {
        self.tokens[start..self.pos]
            .iter()
            .find(|t| {
                !matches!(
                    t.kind,
                    SyntaxKind::WHITESPACE | SyntaxKind::NEWLINE | SyntaxKind::COMMENT
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lexer;

    /// Helper: extract classifications from sentences.
    fn classifications(sentences: &[Sentence]) -> Vec<&str> {
        sentences
            .iter()
            .map(|s| match s.classification {
                SentenceClassification::Heading { level: _ } => "heading",
                SentenceClassification::Structural(StructuralType::Include) => "include",
                SentenceClassification::Structural(StructuralType::Table) => "table",
                SentenceClassification::Structural(StructuralType::Equation) => "equation",
                SentenceClassification::Structural(StructuralType::Use) => "use",
                SentenceClassification::Regular => "regular",
                SentenceClassification::RulePreamble => "preamble",
                SentenceClassification::RulePhrase => "phrase",
            })
            .collect()
    }

    /// Helper: extract sentence text from tokens.
    fn sentence_texts(tokens: &[Token], sentences: &[Sentence]) -> Vec<String> {
        sentences
            .iter()
            .map(|s| {
                tokens[s.token_range.clone()]
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    #[test]
    fn test_empty_source() {
        let tokens = Lexer::tokenize("").unwrap();
        let sentences = break_sentences(&tokens);
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_simple_sentences() {
        let source = "The Lab is a room. Peter is a man.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);
        assert_eq!(classifications(&sentences), vec!["regular", "regular"]);
    }

    #[test]
    fn test_heading_detection() {
        let source = "Chapter 1 - The Beginning\n\nThe Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);
        assert_eq!(classifications(&sentences), vec!["heading", "regular"]);
    }

    #[test]
    fn test_heading_levels() {
        let source = "Volume 1\n\nBook 1\n\nPart 1\n\nChapter 1\n\nSection 1";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 5);
        let levels: Vec<HeadingLevel> = sentences
            .iter()
            .map(|s| match s.classification {
                SentenceClassification::Heading { level } => level,
                _ => panic!("expected heading"),
            })
            .collect();
        assert_eq!(
            levels,
            vec![
                HeadingLevel::Volume,
                HeadingLevel::Book,
                HeadingLevel::Part,
                HeadingLevel::Chapter,
                HeadingLevel::Section,
            ]
        );
    }

    #[test]
    fn test_rule_preamble_and_phrases() {
        let source = "To look upwards:\n    say \"Look out!\";\n    do something else.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 3);
        assert_eq!(
            classifications(&sentences),
            vec!["preamble", "phrase", "regular"]
        );
    }

    #[test]
    fn test_structural_include() {
        let source = "Include Standard Rules by Graham Nelson.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
        assert_eq!(classifications(&sentences), vec!["include"]);
    }

    #[test]
    fn test_structural_table() {
        let source = "Table of TestData\nA\tB\n1\t2";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
        assert_eq!(classifications(&sentences), vec!["table"]);
    }

    #[test]
    fn test_structural_use() {
        let source = "Use American dialect.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
        assert_eq!(classifications(&sentences), vec!["use"]);
    }

    #[test]
    fn test_paragraph_break_ends_sentence() {
        let source = "First paragraph.\n\nSecond paragraph.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_multiple_stop_characters() {
        // ".;;" should be treated as a single division
        let source = "First.;;Second.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_time_colon_not_break() {
        // "1:34" should not be a sentence break
        let source = "He went out at 1:34 PM.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn test_x_break_quoted_text() {
        // Quoted text ending with punctuation followed by capital letter
        // should be a sentence break
        let source = "\"Look out!\" The explosion shattered the calm.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn test_no_x_break_without_capital() {
        // Quoted text ending with punctuation followed by lowercase
        // should NOT be a sentence break
        let source = "\"Look out!\" he shouted.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn test_table_mode_disables_x_break() {
        // In table mode, quoted text punctuation should not end sentences
        let source = "Table of TestData\n\"Of cabbages and kings.\"\tWalrus\t\"Carroll\"";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn test_heading_not_in_middle() {
        // "Chapter" in the middle of a sentence should not be a heading
        let source = "See Chapter 5 for details.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
        assert_eq!(classifications(&sentences), vec!["regular"]);
    }

    #[test]
    fn test_real_i7_snippet() {
        let source = r#""Hello" by Test

Chapter 1 - The Beginning

The Lab is a room. "A sterile white laboratory."

Peter is a man in the Lab.

Every turn:
    say "The tally is [tally].";
    increment the tally.

To expose (X - a value):
    say "You admire [X]."

Instead of taking the beaker:
    say "It's bolted to the table.""#;

        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        // Should have at least: heading, regular sentences, rule preamble,
        // rule phrases, and regular sentences
        let classes = classifications(&sentences);
        assert!(
            classes.contains(&"heading"),
            "expected heading, got {:?}",
            classes
        );
        assert!(
            classes.contains(&"regular"),
            "expected regular, got {:?}",
            classes
        );
        assert!(
            classes.contains(&"preamble"),
            "expected preamble, got {:?}",
            classes
        );
        assert!(
            classes.contains(&"phrase"),
            "expected phrase, got {:?}",
            classes
        );
    }

    #[test]
    fn test_sentence_texts() {
        let source = "The Lab is a room. Peter is a man.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        let texts = sentence_texts(&tokens, &sentences);
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "The Lab is a room");
        assert_eq!(texts[1], "Peter is a man");
    }

    #[test]
    fn test_rule_mode_exits_on_period() {
        let source = "To look:\n    say \"Hi.\"\n\nThe Lab is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        // preamble, regular (phrase ending with .), paragraph break,
        // then regular sentence
        let classes = classifications(&sentences);
        // After the preamble, the "say \"Hi.\"" ends with a period,
        // which should exit rule mode
        assert_eq!(classes[1], "regular", "expected regular after preamble, got {:?}", classes);
    }

    #[test]
    fn test_equation_structural() {
        let source = "Equation NewtonLaw\nF = ma";
        let tokens = Lexer::tokenize(source).unwrap();
        let sentences = break_sentences(&tokens);
        assert_eq!(sentences.len(), 1);
        assert_eq!(classifications(&sentences), vec!["equation"]);
    }
}
