//! Lexer for Inform 7 source text.
//!
//! The lexer is a simple state machine that reads I7 source characters and
//! produces a flat sequence of tokens. It mirrors the C implementation in
//! `services/words-module/Chapter 3/Lexer.w`.
//!
//! # State machine
//!
//! The lexer has two main modes:
//!
//! - **Ordinary mode** (default): reading natural language text, punctuation,
//!   numbers, comments, and I6 escape blocks.
//! - **Literal mode**: reading inside a quoted string, comment, or I6 block.
//!
//! Within literal mode, the `kind` field tracks which kind of literal we are
//! inside (string, comment, or I6 block).
//!
//! # Paragraph breaks
//!
//! A paragraph break is a blank line (two consecutive newlines with only
//! whitespace between them). In I7, paragraph breaks are semantically
//! significant — they end the current sentence.
//!
//! # Headings
//!
//! A heading is a paragraph whose first word is one of `Volume`, `Book`,
//! `Part`, `Chapter`, or `Section` (capitalized as shown). The lexer
//! detects this and emits a `HeadingMarker` token for the first word.
//!
//! # References
//!
//! - C reference: `services/words-module/Chapter 3/Lexer.w`

use crate::syntax_kind::SyntaxKind;
use crate::token::Token;

/// The mode the lexer is currently in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LexMode {
    /// Reading ordinary text (words, punctuation, etc.)
    Ordinary,
    /// Inside a quoted string `"..."`
    String,
    /// Inside a comment `[...]`
    Comment,
    /// Inside an I6 escape block `(- ... -)`
    I6Block,
}

/// The kind of literal being read in literal mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiteralKind {
    String,
    Comment,
    I6Block,
}

/// I7 source lexer.
///
/// Produces a flat sequence of tokens from I7 source text.
///
/// # Example
///
/// ```
/// use conform7_syntax::Lexer;
///
/// let source = r#"The Lab is a room. "Hello, world!""#;
/// let tokens = Lexer::tokenize(source).unwrap();
/// assert!(tokens.iter().any(|t| t.kind == conform7_syntax::SyntaxKind::WORD));
/// ```
pub struct Lexer;

impl Lexer {
    /// Tokenize I7 source text into a vector of tokens.
    ///
    /// Returns `Ok(tokens)` on success, or `Err(error_tokens)` if the source
    /// contains malformed input (unclosed quotes, unclosed I6 blocks, etc.).
    /// Error tokens are included in the returned vector for diagnostic purposes.
    pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<Token>> {
        let mut lexer = LexerState::new(source);
        lexer.run();
        if lexer.has_errors() {
            Err(lexer.tokens)
        } else {
            Ok(lexer.tokens)
        }
    }
}

/// Internal lexer state machine.
struct LexerState<'a> {
    /// The source text being lexed.
    source: &'a str,
    /// The byte position in the source.
    pos: usize,
    /// The current line number (1-indexed).
    line: usize,
    /// The current column number (1-indexed, in bytes).
    column: usize,
    /// Accumulated tokens.
    tokens: Vec<Token>,
    /// The current lexer mode.
    mode: LexMode,
    /// The kind of literal being read (valid only in literal mode).
    literal_kind: LiteralKind,
    /// Nesting depth for comments (supports nested `[...]`).
    comment_nesting: u32,
    /// Whether we are at the start of a new paragraph.
    at_paragraph_start: bool,
    /// Whether the current line is empty so far (only whitespace seen).
    line_is_empty: bool,
    /// Whether the current word is empty so far (no non-whitespace seen).
    word_is_empty: bool,
    /// The start position of the current word being accumulated.
    word_start: usize,
    /// The start line of the current word.
    word_start_line: usize,
    /// The start column of the current word.
    word_start_column: usize,
    /// Whether we have seen any errors.
    has_errors: bool,
}

impl<'a> LexerState<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            mode: LexMode::Ordinary,
            literal_kind: LiteralKind::String,
            comment_nesting: 0,
            at_paragraph_start: true,
            line_is_empty: true,
            word_is_empty: true,
            word_start: 0,
            word_start_line: 1,
            word_start_column: 1,
            has_errors: false,
        }
    }

    fn has_errors(&self) -> bool {
        self.has_errors
    }

    /// Run the lexer to completion.
    fn run(&mut self) {
        while self.pos < self.source.len() {
            match self.mode {
                LexMode::Ordinary => self.lex_ordinary(),
                LexMode::String => self.lex_string(),
                LexMode::Comment => self.lex_comment(),
                LexMode::I6Block => self.lex_i6_block(),
            }
        }
        // Flush any remaining word
        self.flush_word();
        // Check for unclosed literals
        match self.mode {
            LexMode::String => {
                self.emit_error("unclosed quoted string");
            }
            LexMode::Comment => {
                self.emit_error("unclosed comment");
            }
            LexMode::I6Block => {
                self.emit_error("unclosed I6 escape block");
            }
            LexMode::Ordinary => {}
        }
    }

    /// Peek at the current character without consuming it.
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    /// Peek at the character at `offset` bytes ahead.
    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.source.as_bytes().get(self.pos + offset).copied()
    }

    /// Consume and return the current character, advancing position.
    fn advance(&mut self) -> Option<u8> {
        let b = self.source.as_bytes().get(self.pos).copied()?;
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(b)
    }

    /// Emit a token.
    fn emit_token(&mut self, kind: SyntaxKind, text: &str) {
        let start = self.pos.saturating_sub(text.len());
        let end = self.pos;
        let line = self.line;
        let column = self.column.saturating_sub(text.len() as u32 as usize);
        self.tokens.push(Token::new(kind, text, start..end, line, column));
    }

    /// Emit an error token.
    fn emit_error(&mut self, message: &str) {
        self.has_errors = true;
        let start = self.pos;
        let end = self.pos;
        self.tokens.push(Token::error(message, start..end, self.line, self.column));
    }

    /// Start accumulating a word at the current position.
    fn start_word(&mut self) {
        if self.word_is_empty {
            self.word_start = self.pos;
            self.word_start_line = self.line;
            self.word_start_column = self.column;
            self.word_is_empty = false;
        }
    }

    /// Flush the current accumulated word as a token.
    fn flush_word(&mut self) {
        if !self.word_is_empty {
            let text = &self.source[self.word_start..self.pos];
            if !text.is_empty() {
                let kind = match self.mode {
                    LexMode::Comment => SyntaxKind::COMMENT,
                    LexMode::String => SyntaxKind::STRING,
                    LexMode::I6Block => SyntaxKind::I6Block,
                    LexMode::Ordinary => self.classify_word(text),
                };
                let start = self.word_start;
                let end = self.pos;
                let line = self.word_start_line;
                let column = self.word_start_column;
                self.tokens.push(Token::new(kind, text, start..end, line, column));
            }
            self.word_is_empty = true;
        }
    }

    /// Classify an accumulated word into a SyntaxKind.
    fn classify_word(&self, text: &str) -> SyntaxKind {
        // Check for heading markers at paragraph start
        if self.at_paragraph_start && self.word_start_column == 1 {
            match text {
                "Volume" | "Book" | "Part" | "Chapter" | "Section" => {
                    return SyntaxKind::HeadingMarker;
                }
                _ => {}
            }
        }
        // Check for numbers
        if self.is_number(text) {
            return SyntaxKind::NUMBER;
        }
        SyntaxKind::WORD
    }

    /// Check if text is a numeric literal.
    fn is_number(&self, text: &str) -> bool {
        if text.is_empty() {
            return false;
        }
        let bytes = text.as_bytes();
        let mut i = 0;
        // Allow leading minus
        if bytes[0] == b'-' {
            i = 1;
        }
        if i >= bytes.len() {
            return false;
        }
        bytes[i..].iter().all(|b| b.is_ascii_digit())
    }

    /// Check if a character is a punctuation mark.
    fn is_punctuation(c: u8) -> bool {
        matches!(c, b'.' | b',' | b':' | b';' | b'?' | b'!' | b'(' | b')' | b'{' | b'}')
    }

    /// Check if a character is whitespace.
    fn is_whitespace(c: u8) -> bool {
        matches!(c, b' ' | b'\t' | b'\r')
    }

    /// Check if a character is a newline.
    fn is_newline(c: u8) -> bool {
        c == b'\n'
    }

    /// Check if a character is a dash or hyphen.
    fn is_dash(c: u8) -> bool {
        c == b'-'
    }

    // ── Ordinary mode ──────────────────────────────────────────────────

    fn lex_ordinary(&mut self) {
        let Some(c) = self.peek() else { return };


        // Handle newlines
        if Self::is_newline(c) {
            self.flush_word();
            self.advance();
            self.emit_token(SyntaxKind::NEWLINE, "\n");
            self.line_is_empty = true;

            // Check for paragraph break (blank line)
            // A paragraph break is two consecutive newlines with only
            // whitespace between them.
            let mut lookahead = self.pos;
            while lookahead < self.source.len() {
                let b = self.source.as_bytes()[lookahead];
                if Self::is_newline(b) {
                    // Found a blank line — paragraph break
                    self.at_paragraph_start = true;
                    self.emit_token(SyntaxKind::ParagraphBreak, "\n\n");
                    // Consume the second newline
                    self.advance();
                    return;
                } else if Self::is_whitespace(b) {
                    lookahead += 1;
                } else {
                    break;
                }
            }
            self.at_paragraph_start = true;
            return;
        }

        // Handle whitespace
        if Self::is_whitespace(c) {
            self.flush_word();
            let start = self.pos;
            self.advance();
            let text = &self.source[start..self.pos];
            self.emit_token(SyntaxKind::WHITESPACE, text);
            return;
        }

        // We've seen a non-whitespace character on this line
        self.line_is_empty = false;

        // Handle comments `[...]` outside strings
        if c == b'[' {
            self.flush_word();
            self.advance(); // consume `[`
            self.mode = LexMode::Comment;
            self.literal_kind = LiteralKind::Comment;
            self.comment_nesting = 1;
            self.word_start = self.pos - 1;
            self.word_start_line = self.line;
            self.word_start_column = self.column - 1;
            self.word_is_empty = false;
            return;
        }

        // Handle I6 escape blocks `(- ... -)`
        if c == b'(' && self.peek_at(1) == Some(b'-') {
            self.flush_word();
            self.advance(); // consume `(`
            self.advance(); // consume `-`
            self.mode = LexMode::I6Block;
            self.literal_kind = LiteralKind::I6Block;
            self.word_start = self.pos - 2;
            self.word_start_line = self.line;
            self.word_start_column = self.column - 2;
            self.word_is_empty = false;
            return;
        }

        // Handle quoted strings
        if c == b'"' {
            self.flush_word();
            self.advance(); // consume `"`
            self.mode = LexMode::String;
            self.literal_kind = LiteralKind::String;
            self.word_start = self.pos - 1;
            self.word_start_line = self.line;
            self.word_start_column = self.column - 1;
            self.word_is_empty = false;
            return;
        }

        // Handle punctuation
        if Self::is_punctuation(c) {
            self.flush_word();
            self.advance();
            let text = &self.source[self.pos - 1..self.pos];
            self.emit_token(SyntaxKind::PUNCTUATION, text);
            return;
        }

        // Handle dashes — but `-` followed by a digit is a negative number
        if Self::is_dash(c) {
            // Check if this is a negative number: `-` followed by a digit
            if c == b'-' && self.peek_at(1).is_some_and(|n| n.is_ascii_digit()) {
                self.start_word();
                self.advance();
                return;
            }
            self.flush_word();
            self.advance();
            let text = &self.source[self.pos - 1..self.pos];
            self.emit_token(SyntaxKind::DASH, text);
            return;
        }

        // Handle equals
        if c == b'=' {
            self.flush_word();
            self.advance();
            let text = &self.source[self.pos - 1..self.pos];
            self.emit_token(SyntaxKind::EQUALS, text);
            return;
        }

        // Handle slash
        if c == b'/' {
            self.flush_word();
            self.advance();
            let text = &self.source[self.pos - 1..self.pos];
            self.emit_token(SyntaxKind::SLASH, text);
            return;
        }

        // Ordinary word character — accumulate
        self.start_word();
        self.advance();
    }

    // ── String mode ────────────────────────────────────────────────────

    fn lex_string(&mut self) {
        let Some(c) = self.peek() else {
            // End of source while in string — error
            self.flush_word();
            self.emit_error("unclosed quoted string");
            self.mode = LexMode::Ordinary;
            return;
        };

        // Handle escape sequences
        if c == b'\\' {
            self.advance(); // consume backslash
            if self.peek().is_some() {
                self.advance(); // consume escaped character
            }
            return;
        }

        // Handle text substitutions inside strings
        if c == b'[' {
            // Emit the string text up to this point as a STRING token
            // Then switch to text substitution mode
            // Actually, for simplicity, we'll include text substitutions
            // as part of the string. The C lexer stores the whole string
            // as one word, including substitutions.
            self.advance();
            return;
        }

        // Handle end of string
        if c == b'"' {
            self.advance(); // consume closing quote
            self.flush_word();
            self.mode = LexMode::Ordinary;
            return;
        }

        // Handle newlines inside strings (allowed in I7)
        if Self::is_newline(c) {
            self.advance();
            return;
        }

        // Regular character inside string
        self.advance();
    }

    // ── Comment mode ───────────────────────────────────────────────────

    fn lex_comment(&mut self) {
        let Some(c) = self.peek() else {
            // End of source while in comment — error
            self.flush_word();
            self.emit_error("unclosed comment");
            self.mode = LexMode::Ordinary;
            return;
        };

        // Handle nested comments
        if c == b'[' {
            self.comment_nesting += 1;
            self.advance();
            return;
        }

        // Handle end of comment
        if c == b']' {
            self.comment_nesting -= 1;
            self.advance();
            if self.comment_nesting == 0 {
                // Comment is complete — emit as COMMENT token
                self.flush_word();
                self.mode = LexMode::Ordinary;
            }
            return;
        }

        // Regular character inside comment
        self.advance();
    }

    // ── I6 block mode ──────────────────────────────────────────────────

    fn lex_i6_block(&mut self) {
        let Some(c) = self.peek() else {
            // End of source while in I6 block — error
            self.flush_word();
            self.emit_error("unclosed I6 escape block");
            self.mode = LexMode::Ordinary;
            return;
        };

        // Handle end of I6 block: `-)`
        if c == b'-' && self.peek_at(1) == Some(b')') {
            self.advance(); // consume `-`
            self.advance(); // consume `)`
            self.flush_word();
            self.mode = LexMode::Ordinary;
            return;
        }

        // Handle I7 escape back into I7: `(+ ... +)`
        // For now, just consume characters — we'll handle this in the parser
        if c == b'(' && self.peek_at(1) == Some(b'+') {
            self.advance(); // consume `(`
            self.advance(); // consume `+`
            // Skip until `+)`
            while self.pos < self.source.len() {
                if self.peek() == Some(b'+') && self.peek_at(1) == Some(b')') {
                    self.advance(); // consume `+`
                    self.advance(); // consume `)`
                    break;
                }
                self.advance();
            }
            return;
        }

        // Regular character inside I6 block
        self.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(tokens: &[Token]) -> Vec<SyntaxKind> {
        tokens.iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty_source() {
        let tokens = Lexer::tokenize("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_simple_words() {
        let tokens = Lexer::tokenize("The Lab is a room.").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::WORD), "expected WORD tokens, got {:?}", kinds);
        assert!(kinds.contains(&SyntaxKind::PUNCTUATION), "expected PUNCTUATION, got {:?}", kinds);
    }

    #[test]
    fn test_quoted_string() {
        let tokens = Lexer::tokenize(r#"say "Hello, world!""#).unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::STRING), "expected STRING, got {:?}", kinds);
        // Find the string token
        for t in &tokens {
            if t.kind == SyntaxKind::STRING {
                assert_eq!(t.text, r#""Hello, world!""#);
            }
        }
    }

    #[test]
    fn test_i6_block() {
        let tokens = Lexer::tokenize("(- Constant BLOB = 12 -)").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::I6Block), "expected I6Block, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::I6Block {
                assert_eq!(t.text, "(- Constant BLOB = 12 -)");
            }
        }
    }

    #[test]
    fn test_comment() {
        let tokens = Lexer::tokenize("The Lab [this is a comment] is a room.").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::COMMENT), "expected COMMENT, got {:?}", kinds);
    }

    #[test]
    fn test_nested_comment() {
        let tokens = Lexer::tokenize("[outer [inner]]").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::COMMENT), "expected COMMENT, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::COMMENT {
                assert_eq!(t.text, "[outer [inner]]");
            }
        }
    }

    #[test]
    fn test_paragraph_break() {
        let tokens = Lexer::tokenize("First paragraph.\n\nSecond paragraph.").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::ParagraphBreak), "expected ParagraphBreak, got {:?}", kinds);
    }

    #[test]
    fn test_heading_marker() {
        let tokens = Lexer::tokenize("Chapter 1 - The Beginning").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::HeadingMarker), "expected HeadingMarker, got {:?}", kinds);
    }

    #[test]
    fn test_heading_marker_not_at_start() {
        // "Chapter" at the start of a paragraph is a heading marker
        let tokens = Lexer::tokenize("Chapter 1 - The Beginning").unwrap();
        assert_eq!(tokens[0].kind, SyntaxKind::HeadingMarker);

        // "Chapter" in the middle of a paragraph is just a word
        let tokens = Lexer::tokenize("See Chapter 5 for details.").unwrap();
        // Find the "Chapter" token
        for t in &tokens {
            if t.text == "Chapter" {
                assert_eq!(t.kind, SyntaxKind::WORD, "Chapter in middle should be WORD, got {:?}", t.kind);
            }
        }
    }

    #[test]
    fn test_number() {
        let tokens = Lexer::tokenize("The tally is 42.").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::NUMBER), "expected NUMBER, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::NUMBER {
                assert_eq!(t.text, "42");
            }
        }
    }

    #[test]
    fn test_negative_number() {
        let tokens = Lexer::tokenize("-3").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::NUMBER), "expected NUMBER, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::NUMBER {
                assert_eq!(t.text, "-3");
            }
        }
    }

    #[test]
    fn test_punctuation() {
        let tokens = Lexer::tokenize("Hello, world!").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::PUNCTUATION), "expected PUNCTUATION, got {:?}", kinds);
        // Should have both `,` and `!`
        let punct: Vec<&str> = tokens.iter().filter(|t| t.kind == SyntaxKind::PUNCTUATION).map(|t| t.text.as_str()).collect();
        assert!(punct.contains(&","), "expected comma, got {:?}", punct);
        assert!(punct.contains(&"!"), "expected exclamation, got {:?}", punct);
    }

    #[test]
    fn test_dash() {
        let tokens = Lexer::tokenize("Chapter 1 - The Beginning").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::DASH), "expected DASH, got {:?}", kinds);
    }

    #[test]
    fn test_equals() {
        let tokens = Lexer::tokenize("x = 5").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::EQUALS), "expected EQUALS, got {:?}", kinds);
    }

    #[test]
    fn test_slash() {
        let tokens = Lexer::tokenize("http://example.com").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::SLASH), "expected SLASH, got {:?}", kinds);
    }

    #[test]
    fn test_unclosed_string() {
        let result = Lexer::tokenize(r#"say "Hello"#);
        assert!(result.is_err(), "expected error for unclosed string");
        let tokens = result.unwrap_err();
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::ERROR));
    }

    #[test]
    fn test_unclosed_comment() {
        let result = Lexer::tokenize("The Lab [this comment never ends");
        assert!(result.is_err(), "expected error for unclosed comment");
        let tokens = result.unwrap_err();
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::ERROR));
    }

    #[test]
    fn test_unclosed_i6_block() {
        let result = Lexer::tokenize("(- Constant BLOB = 12");
        assert!(result.is_err(), "expected error for unclosed I6 block");
        let tokens = result.unwrap_err();
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::ERROR));
    }

    #[test]
    fn test_text_substitution_in_string() {
        // Text substitutions inside strings are part of the string
        let tokens = Lexer::tokenize(r#"say "[the noun] falls." "#).unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::STRING), "expected STRING, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::STRING {
                assert_eq!(t.text, r#""[the noun] falls.""#);
            }
        }
    }

    #[test]

    fn test_whitespace_preserved() {
        let tokens = Lexer::tokenize("The   Lab").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::WHITESPACE), "expected WHITESPACE, got {:?}", kinds);
    }

    #[test]
    fn test_newline() {
        let tokens = Lexer::tokenize("line1\nline2").unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::NEWLINE), "expected NEWLINE, got {:?}", kinds);
    }

    #[test]
    fn test_source_locations() {
        let source = "The Lab\n  is a room.";
        let tokens = Lexer::tokenize(source).unwrap();
        // "The" should be at line 1, column 1
        let the = tokens.iter().find(|t| t.text == "The").unwrap();
        assert_eq!(the.line, 1);
        assert_eq!(the.column, 1);
        // "Lab" should be at line 1, column 5
        let lab = tokens.iter().find(|t| t.text == "Lab").unwrap();
        assert_eq!(lab.line, 1);
        assert_eq!(lab.column, 5);
        // "is" should be at line 2, column 3 (after 2 spaces of indent)
        let is_token = tokens.iter().find(|t| t.text == "is").unwrap();
        assert_eq!(is_token.line, 2);
        assert_eq!(is_token.column, 3);
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
        // Should have heading markers
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::HeadingMarker),
            "expected HeadingMarker, got kinds: {:?}", kinds(&tokens));
        // Should have strings
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::STRING),
            "expected STRING, got kinds: {:?}", kinds(&tokens));
        // Should have paragraph breaks
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::ParagraphBreak),
            "expected ParagraphBreak, got kinds: {:?}", kinds(&tokens));
        // Should have punctuation
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::PUNCTUATION),
            "expected PUNCTUATION, got kinds: {:?}", kinds(&tokens));
        // Should have words
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::WORD),
            "expected WORD, got kinds: {:?}", kinds(&tokens));
        // Should have newlines
        assert!(tokens.iter().any(|t| t.kind == SyntaxKind::NEWLINE),
            "expected NEWLINE, got kinds: {:?}", kinds(&tokens));
    }

    #[test]
    fn test_heading_not_at_paragraph_start() {
        // "Section" in the middle of a sentence should be a word
        let tokens = Lexer::tokenize("See Section 5 for details.").unwrap();
        for t in &tokens {
            if t.text == "Section" {
                assert_eq!(t.kind, SyntaxKind::WORD,
                    "Section in middle of sentence should be WORD, got {:?}", t.kind);
            }
        }
    }

    #[test]
    fn test_heading_after_paragraph_break() {
        // "Chapter" after a paragraph break should be a heading marker
        let source = "Some text.\n\nChapter 1 - The Beginning";
        let tokens = Lexer::tokenize(source).unwrap();
        let chapter_token = tokens.iter().find(|t| t.text == "Chapter").unwrap();
        assert_eq!(chapter_token.kind, SyntaxKind::HeadingMarker,
            "Chapter after paragraph break should be HeadingMarker, got {:?}", chapter_token.kind);
    }

    #[test]
    fn test_comment_with_quotes_inside() {
        // Comments can contain quotes without ending the string
        let tokens = Lexer::tokenize(r#"[this is a "comment" with quotes]"#).unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::COMMENT), "expected COMMENT, got {:?}", kinds);
    }

    #[test]
    fn test_i6_block_with_nested_i7() {
        // I6 blocks can contain I7 escapes: (+ ... +)
        let source = r#"(- Constant BLOB = (+ the total weight of things in (- selfobj -) +) -)"#;
        let tokens = Lexer::tokenize(source).unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::I6Block), "expected I6Block, got {:?}", kinds);
    }

    #[test]
    fn test_escape_in_string() {
        let tokens = Lexer::tokenize(r#""Hello \"world\"""#).unwrap();
        let kinds = kinds(&tokens);
        assert!(kinds.contains(&SyntaxKind::STRING), "expected STRING, got {:?}", kinds);
        for t in &tokens {
            if t.kind == SyntaxKind::STRING {
                assert_eq!(t.text, r#""Hello \"world\"""#);
            }
        }
    }
}
