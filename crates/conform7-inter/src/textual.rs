//! Textual Inter format reader and writer.
//!
//! Based on `Chapter 3/Inter in Text Files.w` from the bytecode module.
//!
//! # The Textual Inter Format
//!
//! Textual Inter (`.intert`) is a human-readable, tab-indented format for
//! Inter programs. It's designed as an interchange format — programs write it,
//! programs read it. Humans can read it too, but it's not optimized for
//! hand-authoring.
//!
//! ## Indentation
//!
//! Nesting is indicated by tab characters at the start of each line. Spaces
//! are not allowed for indentation (this is enforced strictly, matching the
//! C implementation). Each tab level represents one level of package nesting.
//!
//! ```text
//! package main _plain           ← depth 0 (root)
//! \tpackage Main _code          ← depth 1 (inside main)
//! \t\tcode                      ← depth 2 (inside Main)
//! \t\t\tinv !print              ← depth 3 (inside code block)
//! \t\t\t\tval "Hello!"          ← depth 4 (argument to inv)
//! ```
//!
//! ## Constructs
//!
//! Each non-blank line is a construct. The first word on the line is the
//! construct keyword (e.g., `package`, `constant`, `inv`). The rest of the
//! line provides arguments. Comments start with `#`.
//!
//! ## Annotations
//!
//! Lines can end with annotations in the form `__name` or `__name=value`.
//! These provide metadata about the construct (e.g., `__text="hello"`).
//!
//! ## Forward References
//!
//! Symbols can be referenced by URL (e.g., `/main/K_number`) before they
//! are defined. These forward references are resolved in a second pass
//! after the entire file is parsed.
//!
//! # Reading vs Writing
//!
//! The reader ([`read`]) parses textual Inter into an [`InterTree`].
//! The writer ([`write`]) serializes an [`InterTree`] back to text.
//! Together they enable round-trip fidelity: `write(read(text)) == text`
//! (modulo auto-inserted declarations like `packagetype` and `primitive`).

use crate::instruction::{ConstructId, Instruction};
use crate::tree::{InterTree, Package, PackageItem, PackageType, Symbol, SymbolType, WiringTarget};
use crate::value::{unescape_text, InterValue, ValueFormat};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when reading or writing textual Inter.
///
/// The C implementation uses `inter_error_message` for this purpose.
/// We use a simpler enum since we don't need the full error location
/// infrastructure of the C code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextualError {
    ParseError { line: usize, message: String },
    IoError(String),
}

impl std::fmt::Display for TextualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { line, message } => write!(f, "line {}: {}", line, message),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for TextualError {}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

/// Parse a textual Inter string into an [`InterTree`].
///
/// This is the main entry point for reading `.intert` files. It processes
/// the text line by line, tracking indentation to maintain the package
/// hierarchy, and resolves forward references in a second pass.
///
/// # Errors
///
/// Returns [`TextualError::ParseError`] if the input contains invalid syntax
/// (unknown constructs, malformed values, indentation errors).
///
/// # Example
///
/// ```ignore
/// let input = "package main _plain\n\tconstant x = 42\n";
/// let tree = textual::read(input)?;
/// ```
pub fn read(text: &str) -> Result<InterTree, TextualError> {
    let mut tree = InterTree::new();
    let mut state = ReadState::new();

    for (line_num, raw_line) in text.lines().enumerate() {
        let line_num = line_num + 1;
        let line = raw_line.trim_end();

        // Skip empty lines (they're treated as comments)
        if line.trim().is_empty() {
            continue;
        }

        // Count leading tabs for indentation
        let indent = line.chars().take_while(|&c| c == '\t').count();
        let content = &line[indent..];

        // Check for spaces at beginning (illegal in textual Inter)
        if line.starts_with(' ') {
            return Err(TextualError::ParseError {
                line: line_num,
                message: "spaces (rather than tabs) at beginning of line".to_string(),
            });
        }

        // Adjust write position based on indentation
        while state.current_depth() > indent
            && !state.current_package_name().is_empty()
        {
            state.pop_package();
        }

        // Validate indentation: at the root level (no packages entered),
        // jumping more than 1 level is an error since there's no package
        // to contain the content. Inside packages, instruction nesting can
        // increase freely.
        if indent > 1 && state.current_depth() == 0 {
            return Err(TextualError::ParseError {
                line: line_num,
                message: format!(
                    "indentation jumps from level 0 to {} (can only increase by 1 at root)",
                    indent
                ),
            });
        }

        // Parse the line
        state.current_instr_depth = indent.saturating_sub(state.current_depth());
        parse_line(&mut tree, &mut state, content, line_num)?;
    }

    // Resolve forward references (plugs wired to names)
    resolve_forward_references(&mut tree)?;

    Ok(tree)
}

/// Tracks the parser's position in the package hierarchy.
///
/// As we parse lines, we maintain a stack of package names and their
/// baseline indentation levels. When indentation decreases, we pop
/// packages off the stack to return to the correct parent. This mirrors
/// the `inter_bookmark` mechanism in the C implementation.
struct ReadState {
    /// Stack of (package_name, baseline_depth) for nested packages.
    package_stack: Vec<(String, usize)>,
    /// Current package name path (e.g., "/main/source_text").
    current_path: Vec<String>,
    /// Current instruction nesting depth within the current package.
    /// Used for textual Inter output indentation.
    current_instr_depth: usize,
}

impl ReadState {
    fn new() -> Self {
        Self {
            package_stack: Vec::new(),
            current_path: vec![],
            current_instr_depth: 0,
        }
    }

    fn current_depth(&self) -> usize {
        self.package_stack.len()
    }

    fn current_package_name(&self) -> &str {
        self.package_stack.last().map(|(n, _)| n.as_str()).unwrap_or("")
    }

    fn push_package(&mut self, name: String, baseline: usize) {
        self.current_path.push(name.clone());
        self.package_stack.push((name, baseline));
    }

    fn pop_package(&mut self) {
        self.package_stack.pop();
        self.current_path.pop();
    }

    #[allow(dead_code)]
    fn current_url(&self) -> String {
        if self.current_path.is_empty() {
            String::new()
        } else {
            format!("/{}", self.current_path.join("/"))
        }
    }
}

/// Parse a single line of textual Inter.
///
/// This is the core dispatch function. It:
/// 1. Strips trailing annotations (`__foo __bar=2`)
/// 2. Tokenizes the line (respecting quoted strings)
/// 3. Dispatches to the appropriate construct parser based on the keyword
///
/// Unknown constructs produce a [`TextualError::ParseError`].
fn parse_line(
    tree: &mut InterTree,
    state: &mut ReadState,
    line: &str,
    line_num: usize,
) -> Result<(), TextualError> {
    let trimmed = line.trim();

    // Strip trailing annotations (e.g., `_foo _bar=2`)
    let (content, _annotations) = split_annotations(trimmed);

    if content.is_empty() || content.starts_with('#') {
        // Comment line — store as a Comment instruction for round-trip fidelity
        let mut instr = instr_at_depth(state, ConstructId::Comment);
        // Store the comment text (without the leading #)
        let comment_text = content.trim_start_matches('#').trim();
        if !comment_text.is_empty() {
            let text_id = tree.intern_string(comment_text);
            instr.set_field(1, text_id);
        }
        let pkg = get_current_package_mut(tree, state);
        pkg.add_instruction(instr);
        return Ok(());
    }

    // Split into tokens
    let tokens: Vec<&str> = tokenize(content);

    if tokens.is_empty() {
        return Ok(());
    }

    let keyword = tokens[0];

    match keyword {
        "packagetype" => parse_packagetype(tree, state, &tokens, line_num)?,
        "primitive" => parse_primitive(tree, &tokens, line_num)?,
        "package" => parse_package(tree, state, &tokens, line_num)?,
        "constant" => parse_constant(tree, state, &tokens, line_num)?,
        "typename" => parse_typename(tree, state, &tokens, line_num)?,
        "variable" => parse_variable(tree, state, &tokens, line_num)?,
        "code" => parse_code(tree, state, line_num)?,
        "inv" => parse_inv(tree, state, &tokens, line_num)?,
        "val" => parse_val(tree, state, &tokens, line_num)?,
        "instance" => parse_instance(tree, state, &tokens, line_num)?,
        "property" => parse_property(tree, state, &tokens, line_num)?,
        "propertyvalue" => parse_propertyvalue(tree, state, &tokens, line_num)?,
        "permission" => parse_permission(tree, state, &tokens, line_num)?,
        "pragma" => parse_pragma(tree, state, &tokens, line_num)?,
        "insert" => parse_insert(tree, state, &tokens, line_num)?,
        "nop" => { /* nop: no action needed */ }
        "plug" => parse_plug(tree, state, &tokens, line_num)?,
        "socket" => parse_socket(tree, state, &tokens, line_num)?,
        "version" => { /* version pseudo-construct: ignore for now */ }
        "lab" => parse_lab(tree, state, &tokens, line_num)?,
        "label" => parse_label(tree, state, &tokens, line_num)?,
        "local" => parse_local(tree, state, &tokens, line_num)?,
        "assembly" | "evaluation" | "ref" | "reference" => {
            // Code-level constructs — store as generic instruction
            add_instruction(tree, state, keyword, &tokens, line_num)?;
        }
        "cast" => {
            // Cast has special syntax: cast <to_type> <- <from_type>
            parse_cast(tree, state, &tokens, line_num)?;
        }
        "splat" => {
            // Splat is special: its argument is a raw string that may contain
            // embedded quotes and spaces that the tokenizer can't handle.
            // We read the raw content after the keyword.
            parse_splat(tree, state, content, line_num)?;
        }
        // Labels like `.begin` start with a dot — they're label definitions
        kw if kw.starts_with('.') => {
            parse_dot_label(tree, state, &tokens, line_num)?;
        }
        _ => {
            return Err(TextualError::ParseError {
                line: line_num,
                message: format!("unknown construct: {}", keyword),
            });
        }
    }

    Ok(())
}

/// Split trailing annotations from a line.
///
/// Annotations start with `__` preceded by whitespace. For example:
/// ```text
/// package main _plain __foo __bar=2
/// ```
/// splits into content `"package main _plain"` and annotations
/// `[("__foo", ""), ("__bar", "2")]`.
///
/// Annotations inside quoted strings are not treated as annotations.
fn split_annotations(line: &str) -> (&str, Vec<(String, String)>) {
    // Find the first `__` that's preceded by whitespace, respecting quoted strings.
    let bytes = line.as_bytes();
    let mut in_quotes = false;
    for i in 1..bytes.len().saturating_sub(1) {
        // Track quote state, respecting backslash escapes
        if bytes[i] == b'"' && bytes[i - 1] != b'\\' {
            in_quotes = !in_quotes;
        }
        if !in_quotes && bytes[i] == b'_' && bytes[i + 1] == b'_' && bytes[i - 1].is_ascii_whitespace() {
            let content = &line[..i - 1];
            let annot_str = &line[i..];
            let annotations = parse_annotation_string(annot_str);
            return (content, annotations);
        }
    }
    (line, Vec::new())
}

fn parse_annotation_string(s: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for part in s.split_whitespace() {
        if let Some(eq_pos) = part.find('=') {
            let key = &part[..eq_pos];
            let val = &part[eq_pos + 1..];
            result.push((key.to_string(), val.to_string()));
        } else {
            result.push((part.to_string(), String::new()));
        }
    }
    result
}

/// Tokenize a line into words, respecting quoted strings.
///
/// Spaces are the token delimiter, but spaces inside double-quoted
/// strings are preserved as part of the token. This is essential for
/// string literals like `"Hello, world!"` which contain spaces.
fn tokenize(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    let bytes = line.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
            in_quotes = !in_quotes;
        }
        if !in_quotes && b == b' ' {
            if i > start {
                tokens.push(&line[start..i]);
            }
            start = i + 1;
        }
    }
    if start < line.len() {
        tokens.push(&line[start..]);
    }
    tokens
}

// --- Individual construct parsers ---
//
// Each function handles one construct keyword. They follow a common pattern:
// 1. Validate the token count
// 2. Extract arguments (name, type, value, etc.)
// 3. Create or resolve symbols in the current package
// 4. Build an Instruction and add it to the package
//
// The order of operations matters: we must extract data from `tree` (e.g.,
// interning strings) BEFORE borrowing the current package mutably, to avoid
// double-borrow issues with Rust's borrow checker.

/// Parse a `packagetype` declaration.
///
/// Example: `packagetype _plain`
///
/// Package types are stored as instructions in the root package so that
/// they are preserved during round-trip serialization.
fn parse_packagetype(
    tree: &mut InterTree,
    _state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "packagetype requires a name".to_string(),
        });
    }
    let name = tokens[1];
    // Store the packagetype declaration as an instruction in the root package
    let name_id = tree.intern_string(name);
    let mut instr = Instruction::new(ConstructId::Packagetype);
    instr.set_field(1, name_id);
    tree.root.add_instruction(instr);
    Ok(())
}

/// Parse a `primitive` declaration.
///
/// Example: `primitive !print val -> void`
///
/// Primitives are built-in operations. They're declared in the global
/// scope and referenced by `inv` instructions. The signature (e.g.,
/// `val val -> val`) is optional in textual Inter.
fn parse_primitive(
    tree: &mut InterTree,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "primitive requires a name".to_string(),
        });
    }
    let name = tokens[1];
    // Signature is optional: `primitive !foo val val -> val`
    if !tree.global_scope.has_name(name) {
        tree.global_scope.create_symbol(name);
    }
    let sym = tree.global_scope.get_by_name_mut(name).unwrap();
    sym.symbol_type = SymbolType::Primitive;

    // Store the primitive as an instruction in the root package so it
    // is preserved during round-trip serialization.
    let name_id = tree.intern_string(name);
    let mut instr = Instruction::new(ConstructId::Primitive);
    instr.set_field(1, name_id);
    // Store the signature (if any) as a string in field 2
    if tokens.len() > 2 {
        let sig = tokens[2..].join(" ");
        let sig_id = tree.intern_string(&sig);
        instr.set_field(2, sig_id);
    }
    tree.root.add_instruction(instr);

    Ok(())
}

/// Parse a `package` declaration.
///
/// Example: `package main _plain`
///
/// Creates a new child package in the current package and pushes it onto
/// the package stack. Subsequent lines at higher indentation will be
/// parsed into this package.
fn parse_package(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 3 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "package requires name and type".to_string(),
        });
    }
    let mut idx = 1;
    let type_marker = parse_type_marker(tokens, idx);
    let pkg_type_marker = type_marker.0;
    idx = type_marker.1;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "package requires name and type".to_string(),
        });
    }
    let name = tokens[idx].to_string();
    idx += 1;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "package requires a package type".to_string(),
        });
    }
    let type_str = tokens[idx];
    let pkg_type = PackageType::from_keyword(type_str);

    // Create the package
    let resource_id = tree.alloc_resource_id();
    let mut pkg = Package::new(resource_id, name.clone(), pkg_type, tree.symbol_counter());
    if let Some(marker) = pkg_type_marker {
        pkg.type_marker = Some(tree.intern_string(&marker));
    }

    // Create a symbol for this package in the parent's symbols table
    {
        let parent = get_current_package_mut(tree, state);
        if !parent.symbols.has_name(&name) {
            parent.symbols.create_symbol(&name);
        }
        let sym = parent.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Package;
        // Add the child to the parent's ordered items
        parent.add_child(pkg);
    }

    // Push onto stack
    let baseline = state.current_depth();
    state.push_package(name, baseline);

    Ok(())
}

/// Parse a `constant` declaration.
///
/// Examples:
/// - `constant lucky_number = 7`
/// - `constant (K_number) C_death = -5`
/// - `constant message = "hello"`
///
/// Constants are named values. The optional type marker in parentheses
/// specifies the constant's type. The value is parsed as an Inter value
/// pair (number, string, symbol reference, etc.).
fn parse_constant(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 3 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "constant requires name and value".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "constant requires name and value".to_string(),
        });
    }
    let name = tokens[idx].to_string();
    idx += 1;

    if idx >= tokens.len() || tokens[idx] != "=" {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "constant requires '=' before value".to_string(),
        });
    }
    idx += 1;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "constant requires a value".to_string(),
        });
    }

    let value_str = tokens[idx..].join(" ");

    // Intern the type marker early so we don't need to borrow tree again
    // while the current package is borrowed.
    let marker_id = type_marker.as_ref().map(|m| tree.intern_string(m));

    // Parse value first (needs &mut tree)
    let value = parse_value_literal(tree, state, &value_str, line_num)?;

    // Now create symbol and instruction
    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym_id = {
        let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Constant;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Constant);
    instr.set_field(1, sym_id);
    instr.set_field(2, value.format as u32);
    instr.set_field(3, value.content);
    instr.type_marker = marker_id;
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `typename` declaration.
///
/// Examples:
/// - `typename K_number = int32`
/// - `typename K_list_of_number = list of /main/K_number`
/// - `typename K_piece <= K_object`  (subkind)
///
/// Typenames create type aliases. The `<=` syntax declares a subkind
/// (a subtype relationship). The type on the right can be a base type,
/// a compound type, or a reference to another typename.
fn parse_typename(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 3 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "typename requires name and type".to_string(),
        });
    }
    let name = tokens[1].to_string();
    // Support both `=` and `<=` for subkind declarations
    let eq_idx = tokens.iter().position(|&t| t == "=" || t == "<=");
    if eq_idx.is_none() || eq_idx.unwrap() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "typename requires '=' or '<=' before type".to_string(),
        });
    }
    let eq_idx = eq_idx.unwrap();
    let operator = tokens[eq_idx]; // "=" or "<="
    let type_str = tokens[eq_idx + 1..].join(" ");

    // Intern strings first (needs &mut tree)
    let type_id = tree.intern_string(&type_str);
    let op_id = tree.intern_string(operator);

    // Now create symbol and instruction
    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym_id = {
        let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Typename;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Typename);
    instr.set_field(1, sym_id);
    instr.set_field(2, type_id);
    instr.set_field(3, op_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `variable` declaration.
///
/// Examples:
/// - `variable (K_number) V_banana = 100`
/// - `variable (K_colour) V_shade = I_red`
///
/// Variables are named storage locations. The type marker is required
/// in well-formed Inter. The initial value is optional.
fn parse_variable(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "variable requires a name".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "variable requires a name".to_string(),
        });
    }
    let name = tokens[idx].to_string();
    idx += 1;

    let value_str = if idx < tokens.len() && tokens[idx] == "=" {
        idx += 1;
        if idx < tokens.len() {
            Some(tokens[idx..].join(" "))
        } else {
            None
        }
    } else {
        None
    };

    // Intern the type marker early so we don't need to borrow tree again
    // while the current package is borrowed.
    let marker_id = type_marker.as_ref().map(|m| tree.intern_string(m));

    // Parse value first if present (needs &mut tree)
    let value = if let Some(ref val_str) = value_str {
        Some(parse_value_literal(tree, state, val_str, line_num)?)
    } else {
        None
    };

    // Now create symbol and instruction
    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym_id = {
        let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Variable;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Variable);
    instr.set_field(1, sym_id);
    if let Some(val) = value {
        instr.set_field(2, val.format as u32);
        instr.set_field(3, val.content);
    }
    instr.type_marker = marker_id;
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `code` marker.
///
/// Example: `code`
///
/// The `code` construct marks the beginning of executable code within
/// a `_code` package. It has no arguments.
fn parse_code(
    tree: &mut InterTree,
    state: &mut ReadState,
    _line_num: usize,
) -> Result<(), TextualError> {
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr_at_depth(state, ConstructId::Code));
    Ok(())
}

/// Parse a `local` variable declaration inside a `_code` package.
///
/// Examples:
/// - `local (int32) argument`
/// - `local (/main/K_number) x`
///
/// Locals are variables scoped to a function body. They are written just
/// like top-level `variable` declarations but use the `local` construct.
fn parse_local(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "local requires a name".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "local requires a name".to_string(),
        });
    }
    let name = tokens[idx].to_string();

    // Intern the type marker early so we don't need to borrow tree again
    // while the current package is borrowed.
    let marker_id = type_marker.as_ref().map(|m| tree.intern_string(m));

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym_id = {
        let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Variable;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Local);
    instr.set_field(1, sym_id);
    instr.type_marker = marker_id;
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `lab` (label reference) instruction.
///
/// Example: `lab .begin`
///
/// The referenced label is resolved to a symbol in the current package.
/// If it doesn't exist yet, it's created as a placeholder (forward
/// references to labels are allowed).
fn parse_lab(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "lab requires a label name".to_string(),
        });
    }
    let name = tokens[1].trim_start_matches('.').to_string();

    let pkg = get_current_package_mut(tree, state);
    let sym_id = if let Some(sym) = pkg.symbols.get_by_name(&name) {
        sym.id
    } else {
        let sym = pkg.symbols.create_symbol(&name);
        sym.symbol_type = SymbolType::Label;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Lab);
    instr.set_field(1, sym_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse an explicit `label` definition.
///
/// Example: `label .begin`
///
/// This is the keyword form of a label definition; dotted names like
/// `.begin` are more common and handled by [`parse_dot_label`].
fn parse_label(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "label requires a name".to_string(),
        });
    }
    let name = tokens[1].to_string();
    parse_dot_label_by_name(tree, state, &name)
}

/// Parse a dotted label definition (e.g., `.begin`).
fn parse_dot_label(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    _line_num: usize,
) -> Result<(), TextualError> {
    let name = tokens[0].to_string();
    parse_dot_label_by_name(tree, state, &name)
}

fn parse_dot_label_by_name(
    tree: &mut InterTree,
    state: &mut ReadState,
    name: &str,
) -> Result<(), TextualError> {
    let name = name.trim_start_matches('.');
    let pkg = get_current_package_mut(tree, state);
    let sym_id = if let Some(sym) = pkg.symbols.get_by_name(name) {
        sym.id
    } else {
        let sym = pkg.symbols.create_symbol(name);
        sym.symbol_type = SymbolType::Label;
        sym.id
    };

    let mut instr = instr_at_depth(state, ConstructId::Label);
    instr.set_field(1, sym_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse an `inv` (invoke) instruction.
///
/// Examples:
/// - `inv !print`
/// - `inv /main/OtherFunction`
///
/// Invokes either a primitive operation (`!name`) or a function/package
/// reference. Arguments are child instructions at higher indentation.
fn parse_inv(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "inv requires a primitive or function name".to_string(),
        });
    }
    let target = tokens[1].to_string();

    let target_id = if target.starts_with('!') || target.starts_with('@') {
        // Primitive or I6 opcode invocation
        if !tree.global_scope.has_name(&target) {
            tree.global_scope.create_symbol(&target);
            let sym = tree.global_scope.get_by_name_mut(&target).unwrap();
            sym.symbol_type = SymbolType::Primitive;
        }
        tree.global_scope.get_by_name(&target).unwrap().id
    } else {
        // Function or symbol invocation — create a wiring symbol for the target
        let pkg = get_current_package_mut(tree, state);
        let sym = pkg.symbols.create_symbol(&format!("__inv_ref_{}", pkg.symbols.symbols.len()));
        let sym_id = sym.id;
        sym.wired_to_name = Some(target);
        sym_id
    };

    let mut instr = instr_at_depth(state, ConstructId::Inv);
    instr.set_field(1, target_id);
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `val` (value) instruction.
///
/// Examples:
/// - `val 42`
/// - `val "Hello, world!\n"`
/// - `val (K_number) 17`
///
/// Pushes a literal value. The optional type marker specifies the type.
/// This is the most common code-level construct.
fn parse_val(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "val requires a value".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "val requires a value".to_string(),
        });
    }
    let value_str = tokens[idx..].join(" ");
    let value = parse_value_literal(tree, state, &value_str, line_num)?;

    let mut instr = instr_at_depth(state, ConstructId::Val);
    instr.set_field(1, value.format as u32);
    instr.set_field(2, value.content);
    if let Some(marker) = type_marker {
        instr.type_marker = Some(tree.intern_string(&marker));
    }
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse an `instance` declaration.
///
/// Example: `instance (K_colour) I_green = 1`
///
/// Instances are named values of an enumerated kind. They're similar to
/// constants but specifically for enum types.
fn parse_instance(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "instance requires a name".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "instance requires a name".to_string(),
        });
    }
    let name = tokens[idx].to_string();

    // Intern type marker early
    let marker_id = type_marker.as_ref().map(|m| tree.intern_string(m));

    // Parse optional value (= N)
    idx += 1;
    let value = if idx < tokens.len() && tokens[idx] == "=" {
        idx += 1;
        if idx < tokens.len() {
            Some(parse_value_literal(tree, state, &tokens[idx..].join(" "), line_num)?)
        } else {
            None
        }
    } else {
        None
    };

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Instance;

    let mut instr = instr_at_depth(state, ConstructId::Instance);
    instr.set_field(1, sym.id);
    instr.type_marker = marker_id;
    if let Some(val) = value {
        instr.set_field(2, val.format as u32);
        instr.set_field(3, val.content);
    }
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `property` declaration.
///
/// Example: `property (K_number) P_strength`
///
/// Properties are named attributes that can be attached to instances.
/// The type marker specifies the property's value type.
fn parse_property(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "property requires a name".to_string(),
        });
    }

    let mut idx = 1;
    let (type_marker, next_idx) = parse_type_marker(tokens, idx);
    idx = next_idx;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "property requires a name".to_string(),
        });
    }
    let name = tokens[idx].to_string();

    // Intern type marker early
    let marker_id = type_marker.as_ref().map(|m| tree.intern_string(m));

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Property;

    let mut instr = instr_at_depth(state, ConstructId::Property);
    instr.set_field(1, sym.id);
    instr.type_marker = marker_id;
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `propertyvalue` instruction.
///
/// Examples:
/// - `propertyvalue P_strength of I_citrus = 20`
/// - `propertyvalue P_strength I_citrus = 20`
///
/// Sets the value of a property for a specific owner (instance or kind).
/// The syntax is `propertyvalue <property> [of] <owner> = <value>`.
/// The `of` keyword is optional.
fn parse_propertyvalue(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 4 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "propertyvalue requires property, owner, and value".to_string(),
        });
    }

    // Syntax: propertyvalue <property> [of] <owner> = <value>
    // tokens[0] = "propertyvalue"
    // tokens[1] = property name
    // tokens[2] = "of" (optional) or owner name
    // tokens[3] = owner name (if tokens[2] was "of") or "="
    // tokens[4] = value (if tokens[3] was "=") or "="
    // tokens[5] = value (if tokens[4] was "=")

    let property = tokens[1].to_string();
    let mut idx = 2;

    // Check for optional "of" keyword
    if tokens[idx] == "of" {
        idx += 1;
    }

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "propertyvalue requires owner after property".to_string(),
        });
    }
    let owner = tokens[idx].to_string();
    idx += 1;

    if idx >= tokens.len() || tokens[idx] != "=" {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "propertyvalue requires '=' before value".to_string(),
        });
    }
    idx += 1;

    if idx >= tokens.len() {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "propertyvalue requires a value".to_string(),
        });
    }
    let value_str = tokens[idx..].join(" ");

    // Parse value first (needs &mut tree)
    let value = parse_value_literal(tree, state, &value_str, line_num)?;

    // Now resolve symbols and create instruction
    let pkg = get_current_package_mut(tree, state);
    let owner_id = resolve_or_create_symbol(pkg, &owner, SymbolType::Instance);
    let prop_id = resolve_or_create_symbol(pkg, &property, SymbolType::Property);

    let mut instr = instr_at_depth(state, ConstructId::Propertyvalue);
    instr.set_field(1, owner_id);
    instr.set_field(2, prop_id);
    instr.set_field(3, value.format as u32);
    instr.set_field(4, value.content);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `permission` declaration.
///
/// Example: `permission for K_odour to have P_strength`
///
/// Grants a kind permission to have a property. Without permission,
/// propertyvalue assignments for that kind are invalid.
///
/// The syntax is `permission for <kind> to have <property>`.
fn parse_permission(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 5 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "permission requires 'for <kind> to have <property>'".to_string(),
        });
    }

    // Syntax: permission for <kind> to have <property>
    // tokens[0] = "permission"
    // tokens[1] = "for"
    // tokens[2] = kind name
    // tokens[3] = "to"
    // tokens[4] = "have"
    // tokens[5] = property name
    if tokens[1] != "for" || tokens[3] != "to" || tokens[4] != "have" {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "permission syntax: 'permission for <kind> to have <property>'".to_string(),
        });
    }

    let kind_name = tokens[2].to_string();
    let prop_name = if tokens.len() > 5 { tokens[5].to_string() } else {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "permission requires a property name".to_string(),
        });
    };

    let pkg = get_current_package_mut(tree, state);
    let kind_id = resolve_or_create_symbol(pkg, &kind_name, SymbolType::Typename);
    let prop_id = resolve_or_create_symbol(pkg, &prop_name, SymbolType::Property);

    let mut instr = instr_at_depth(state, ConstructId::Permission);
    instr.set_field(1, kind_id);
    instr.set_field(2, prop_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `pragma` directive.
///
/// Example: `pragma target_I6 "$MAX_STATIC_DATA=180000"`
///
/// Pragmas are platform-specific tuning directives. They're passed
/// through to the code generator.
fn parse_pragma(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    _line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Ok(()); // malformed pragma, just skip
    }

    let target = tokens[1].to_string();
    let value = if tokens.len() > 2 {
        let raw = tokens[2..].join(" ");
        // Strip surrounding quotes if present
        if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
            raw[1..raw.len() - 1].to_string()
        } else {
            raw
        }
    } else {
        String::new()
    };

    // Intern strings first (needs &mut tree)
    let target_id = tree.intern_string(&target);
    let value_id = if !value.is_empty() {
        Some(tree.intern_string(&value))
    } else {
        None
    };

    let pkg = get_current_package_mut(tree, state);
    let mut instr = instr_at_depth(state, ConstructId::Pragma);
    instr.set_field(1, target_id);
    if let Some(vid) = value_id {
        instr.set_field(2, vid);
    }
    pkg.add_instruction(instr);
    Ok(())
}

/// Parse an `insert` directive.
///
/// Example: `insert`
///
/// Parse a  (raw I6 code) instruction.
///
/// Example: 
///
/// Splat embeds raw Inform 6 code inline. The argument is a string literal
/// that may contain embedded quotes and spaces. We read the raw content
/// after the keyword to avoid issues with the tokenizer.
fn parse_splat(
    tree: &mut InterTree,
    state: &mut ReadState,
    line: &str,
    line_num: usize,
) -> Result<(), TextualError> {
    // Find the content after 'splat' keyword
    let content = line.trim_start();
    let value_str = if let Some(pos) = content.find(' ') {
        content[pos + 1..].trim().to_string()
    } else {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "splat requires a string argument".to_string(),
        });
    };

    // Parse as a string literal
    let value = parse_value_literal(tree, state, &value_str, line_num)?;

    let mut instr = instr_at_depth(state, ConstructId::Splat);
    instr.set_field(1, value.format as u32);
    instr.set_field(2, value.content);
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `cast` instruction.
///
/// Example: `cast /main/K_number <- /main/K_colour`
///
/// Cast converts a value from one type to another. The `<-` is syntactic
/// sugar in the textual format. We store both the target type and source
/// type as interned strings.
fn parse_cast(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 4 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "cast requires <to_type> <- <from_type>".to_string(),
        });
    }
    let to_type = tokens[1].to_string();
    // tokens[2] should be "<-"
    let from_type = tokens[3..].join(" ");

    let to_id = tree.intern_string(&to_type);
    let from_id = tree.intern_string(&from_type);

    let mut instr = instr_at_depth(state, ConstructId::Cast);
    instr.set_field(1, to_id);
    instr.set_field(2, from_id);
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `splat` (raw I6 code) instruction.
///
/// Example: `splat "Sing a song of \"six splats\"...\nand don't wait up"`
///
/// Splat embeds raw Inform 6 code inline. The argument is a string literal
/// that may contain embedded quotes and spaces. We read the raw content
/// after the keyword to avoid issues with the tokenizer.
/// Marks a position where another package's contents will be inserted
/// during linking. Used for the connectors mechanism.
fn parse_insert(
    tree: &mut InterTree,
    state: &mut ReadState,
    _tokens: &[&str],
    _line_num: usize,
) -> Result<(), TextualError> {
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr_at_depth(state, ConstructId::Insert));
    Ok(())
}

/// Parse a `plug` declaration.
///
/// Examples:
/// - `plug my_symbol`
/// - `plug my_symbol ~~> "/target/name"`
///
/// Plugs are symbols that need to be connected to external definitions
/// during linking. The optional `~~>` syntax specifies the wiring target.
fn parse_plug(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "plug requires a name".to_string(),
        });
    }
    let name = tokens[1].to_string();

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Plug;
    let sym_id = sym.id;

    // plug may have `~~> target` wiring
    if tokens.len() >= 4 && tokens[2] == "~~>" {
        let target = tokens[3..].join(" ");
        if target.starts_with('"') {
            // Wired to a name (forward reference)
            let name_ref = target.trim_matches('"').to_string();
            sym.wired_to_name = Some(name_ref);
        } else {
            // Wired to a URL
            sym.wired_to_name = Some(target);
        }
    }

    // Add a plug instruction to preserve it during round-trip
    let mut instr = instr_at_depth(state, ConstructId::Plug);
    instr.set_field(1, sym_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `socket` declaration.
///
/// Examples:
/// - `socket my_symbol`
/// - `socket my_symbol ~~> "/target/name"`
///
/// Sockets are symbols offered for external connection. They're the
/// counterpart to plugs. The optional `~~>` syntax specifies the
/// wiring target.
fn parse_socket(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "socket requires a name".to_string(),
        });
    }
    let name = tokens[1].to_string();

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Socket;
    let sym_id = sym.id;

    // socket may have `~~> target` wiring
    if tokens.len() >= 4 && tokens[2] == "~~>" {
        let target = tokens[3..].join(" ");
        if target.starts_with('"') {
            let name_ref = target.trim_matches('"').to_string();
            sym.wired_to_name = Some(name_ref);
        } else {
            sym.wired_to_name = Some(target);
        }
    }

    // Add a socket instruction to preserve it during round-trip
    let mut instr = instr_at_depth(state, ConstructId::Socket);
    instr.set_field(1, sym_id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Generic instruction parser for constructs we don't fully handle yet.
///
/// This is a fallback for code-level constructs like `assembly`, `cast`,
/// `evaluation`, `ref`, `reference`, `splat`, and `label`. These are
/// stored as instructions with their construct ID and any remaining
/// tokens as frame words, so that round-trip fidelity is preserved.
fn add_instruction(
    tree: &mut InterTree,
    state: &mut ReadState,
    keyword: &str,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    let construct = ConstructId::from_keyword(keyword).ok_or_else(|| {
        TextualError::ParseError {
            line: line_num,
            message: format!("unknown construct: {}", keyword),
        }
    })?;

    // Preserve remaining tokens as frame words for round-trip fidelity.
    // The first token is the keyword itself, so we skip it.
    let mut instr = instr_at_depth(state, construct);
    for (i, token) in tokens[1..].iter().enumerate() {
        // Try to parse as a number first; otherwise store as a string ID
        if let Ok(n) = token.parse::<u32>() {
            instr.set_field(i + 1, n);
        } else if token.starts_with('"') && token.ends_with('"') {
            // String literal — intern it
            let inner = &token[1..token.len() - 1];
            if let Ok(unescaped) = unescape_text(inner) {
                let id = tree.intern_string(&unescaped);
                instr.set_field(i + 1, id);
            }
        } else {
            // Symbol reference — create a wiring symbol
            let pkg = get_current_package_mut(tree, state);
            let sym = pkg.symbols.create_symbol(&format!(
                "__gen_ref_{}", pkg.symbols.symbols.len()
            ));
            let sym_id = sym.id;
            sym.wired_to_name = Some(token.to_string());
            instr.set_field(i + 1, sym_id);
        }
    }
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(instr);
    Ok(())
}

// --- Helpers ---

/// Create an instruction with the current nesting depth from the parse state.
fn instr_at_depth(state: &ReadState, construct: ConstructId) -> Instruction {
    let mut instr = Instruction::new(construct);
    instr.depth = state.current_instr_depth;
    instr
}

/// Navigate to the current package based on the parse state's path stack.
///
/// The path stack records the sequence of package names from root to
/// current (e.g., `["main", "source_text"]`). This function walks the
/// tree following that path.
///
/// Panics if the path doesn't match the tree structure — this indicates
/// a bug in the parser's state management.
fn get_current_package_mut<'a>(tree: &'a mut InterTree, state: &ReadState) -> &'a mut Package {
    let mut current = &mut tree.root;
    for name in &state.current_path {
        current = current.children.get_mut(name).expect("package not found");
    }
    current
}

/// Resolve or create a symbol in the given package.
///
/// If the symbol doesn't exist, it's created with the given type.
/// If it exists but has type [`SymbolType::Misc`] (undefined), its type
/// is updated. Returns the symbol's ID.
///
/// This is used by construct parsers that reference symbols which may
/// not have been declared yet (forward references).
fn resolve_or_create_symbol(pkg: &mut Package, name: &str, stype: SymbolType) -> u32 {
    if !pkg.symbols.has_name(name) {
        pkg.symbols.create_symbol(name);
    }
    let sym = pkg.symbols.get_by_name_mut(name).unwrap();
    if matches!(sym.symbol_type, SymbolType::Misc) {
        sym.symbol_type = stype;
    }
    sym.id
}

/// Parse a value literal from textual Inter syntax.
///
/// Handles all value formats:
/// - Numbers: `42`, `-5`, `0x7f2a`, `0b1010`
/// - Strings: `"hello"`, `"line\nbreak"`
/// - Reals: `r"3.14159"`
/// - Dictionary words: `dw"frogs"`, `dwp"frogs"`
/// - Globs: `glob"SOME_I6_DRIVEL"`
/// - Symbols: bare identifiers or URLs like `/main/K_number`
/// - Undef: `!undef`
///
/// Strings are automatically interned in the tree's warehouse.
fn parse_value_literal(
    tree: &mut InterTree,
    state: &mut ReadState,
    s: &str,
    line_num: usize,
) -> Result<InterValue, TextualError> {
    let s = s.trim();

    if s == "!undef" {
        return Ok(InterValue::undef());
    }

    // Quoted string: "text"
    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        let unescaped = unescape_text(inner).map_err(|e| TextualError::ParseError {
            line: line_num,
            message: e,
        })?;
        let id = tree.intern_string(&unescaped);
        return Ok(InterValue::text(id));
    }

    // Real: r"text"
    if s.starts_with("r\"") && s.ends_with('"') {
        let inner = &s[2..s.len() - 1];
        let unescaped = unescape_text(inner).map_err(|e| TextualError::ParseError {
            line: line_num,
            message: e,
        })?;
        let id = tree.intern_string(&unescaped);
        return Ok(InterValue::real(id));
    }

    // Dictionary word: dw"text"
    if s.starts_with("dw\"") && s.ends_with('"') {
        let inner = &s[3..s.len() - 1];
        let unescaped = unescape_text(inner).map_err(|e| TextualError::ParseError {
            line: line_num,
            message: e,
        })?;
        let id = tree.intern_string(&unescaped);
        return Ok(InterValue::dword(id));
    }

    // Plural dictionary word: dwp"text"
    if s.starts_with("dwp\"") && s.ends_with('"') {
        let inner = &s[4..s.len() - 1];
        let unescaped = unescape_text(inner).map_err(|e| TextualError::ParseError {
            line: line_num,
            message: e,
        })?;
        let id = tree.intern_string(&unescaped);
        return Ok(InterValue::pdword(id));
    }

    // Glob: glob"text"
    if s.starts_with("glob\"") && s.ends_with('"') {
        let inner = &s[5..s.len() - 1];
        let unescaped = unescape_text(inner).map_err(|e| TextualError::ParseError {
            line: line_num,
            message: e,
        })?;
        let id = tree.intern_string(&unescaped);
        return Ok(InterValue::glob(id));
    }

    // Hex: 0x...
    if s.starts_with("0x") || s.starts_with("0X") {
        let hex_str = &s[2..];
        let n = u32::from_str_radix(hex_str, 16).map_err(|_| {
            TextualError::ParseError { line: line_num, message: format!("invalid hex: {}", s) }
        })?;
        return Ok(InterValue::number_in_base(n, 16));
    }

    // Binary: 0b...
    if s.starts_with("0b") || s.starts_with("0B") {
        let bin_str = &s[2..];
        let n = u32::from_str_radix(bin_str, 2).map_err(|_| {
            TextualError::ParseError { line: line_num, message: format!("invalid binary: {}", s) }
        })?;
        return Ok(InterValue::number_in_base(n, 2));
    }

    // Signed decimal: -...
    if s.starts_with('-') {
        let n: i32 = s.parse().map_err(|_| {
            TextualError::ParseError { line: line_num, message: format!("invalid number: {}", s) }
        })?;
        return Ok(InterValue::signed_number(n));
    }

    // List literal: { ... }
    if s.starts_with('{') {
        let id = tree.intern_string(s);
        return Ok(InterValue::list(id));
    }

    // Struct literal: struct{ ... }
    if s.starts_with("struct{") {
        let id = tree.intern_string(s);
        return Ok(InterValue::struct_lit(id));
    }

    // Sum literal: sum{ ... }
    if s.starts_with("sum{") {
        let id = tree.intern_string(s);
        return Ok(InterValue::list(id));
    }

    // Unsigned decimal
    if let Ok(n) = s.parse::<u32>() {
        return Ok(InterValue::number(n));
    }

    // Symbol reference (identifier or URL).
    // First try to resolve against existing symbols in the global scope
    // (checked before borrowing the current package to avoid borrow conflicts).
    // If found, use the existing symbol's ID directly.
    // Otherwise, create a wiring symbol for forward reference resolution.
    if let Some(sym) = tree.global_scope.get_by_name(s) {
        return Ok(InterValue::symbolic(sym.id));
    }
    let pkg = get_current_package_mut(tree, state);
    let sym_id = if let Some(sym) = pkg.symbols.get_by_name(s) {
        sym.id
    } else {
        let sym = pkg.symbols.create_symbol(&format!("__sym_ref_{}", pkg.symbols.symbols.len()));
        let id = sym.id;
        sym.wired_to_name = Some(s.to_string());
        id
    };
    Ok(InterValue::symbolic(sym_id))
}

// ---------------------------------------------------------------------------
// Forward reference resolution
// ---------------------------------------------------------------------------

/// Resolve forward references after parsing is complete.
///
/// During parsing, symbols referenced by URL before they're defined are
/// temporarily wired to a name string. This pass walks the entire tree,
/// finds those temporary wirings, and resolves them to actual symbol
/// targets.
///
/// This is a two-phase process to avoid issues with the Rust borrow
/// checker:
/// 1. [`collect_resolutions`] walks the tree immutably to find all
///    symbols that need resolution
/// 2. [`apply_resolutions`] walks the tree mutably to apply the fixes
fn resolve_forward_references(tree: &mut InterTree) -> Result<(), TextualError> {
    // Collect all resolutions needed, then apply them
    let resolutions = collect_resolutions(&tree.root, &[]);
    apply_resolutions(tree, &resolutions);
    Ok(())
}

fn collect_resolutions(pkg: &Package, path: &[String]) -> Vec<(Vec<String>, u32, String)> {
    let mut result = Vec::new();
    for sym in pkg.symbols.iter() {
        if let Some(ref name) = sym.wired_to_name {
            if !sym.is_plug() {
                result.push((path.to_vec(), sym.id, name.clone()));
            }
        }
    }
    for child in pkg.children_iter() {
        let mut child_path = path.to_vec();
        child_path.push(child.name.clone());
        result.extend(collect_resolutions(child, &child_path));
    }
    result
}

fn apply_resolutions(tree: &mut InterTree, resolutions: &[(Vec<String>, u32, String)]) {
    for (path, sym_id, target_name) in resolutions {
        let target_sym = if target_name.starts_with('/') {
            find_symbol_by_url(tree, target_name)
        } else {
            // Navigate to the package and find the symbol
            let pkg = navigate_package_mut(&mut tree.root, path);
            pkg.symbols.get_by_name(target_name).map(|s| (s.id, pkg.symbols.resource_id))
        };

        if let Some((target_id, target_table_id)) = target_sym {
            let pkg = navigate_package_mut(&mut tree.root, path);
            if let Some(sym) = pkg.symbols.get_mut(*sym_id) {
                sym.wired_to = Some(WiringTarget {
                    table_id: target_table_id,
                    symbol_id: target_id,
                });
                sym.wired_to_name = None;
            }
        }
    }
}

/// Navigate to a package by path, returning a mutable reference.
///
/// The path is a sequence of package names from root (e.g.,
/// `["main", "source_text"]`). Panics if any component is not found.
fn navigate_package_mut<'a>(root: &'a mut Package, path: &[String]) -> &'a mut Package {
    let mut current = root;
    for name in path {
        current = current.children.get_mut(name).expect("package not found");
    }
    current
}

/// Find a symbol by URL path (e.g., `/main/K_number`).
///
/// The URL is split into a package path and a symbol name. The package
/// is navigated to, then the symbol is looked up by name.
/// Returns `(symbol_id, table_resource_id)` if found.
fn find_symbol_by_url(tree: &InterTree, url: &str) -> Option<(u32, u32)> {
    // Split URL into package path and symbol name
    let parts: Vec<&str> = url.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return None;
    }

    let symbol_name = parts.last().unwrap();
    let pkg_path = &parts[..parts.len() - 1];

    // Navigate to the package
    let mut current = &tree.root;
    for part in pkg_path {
        current = current.get_child(part)?;
    }

    // Find the symbol
    current.symbols.get_by_name(symbol_name).map(|s| (s.id, current.symbols.resource_id))
}

/// Resolve a symbol ID to its display name, handling URL references.
/// If the symbol is wired to another symbol in a different package,
/// returns the URL of the target. If the target is in the same package
/// (or the global scope), returns just the target's name.
fn resolve_symbol_name(tree: &InterTree, pkg: &Package, id: u32) -> String {
    // First check the current package's symbols
    if let Some(sym) = pkg.symbols.get(id) {
        if let Some(ref target) = sym.wired_to {
            // Same package — use bare name
            if target.table_id == pkg.symbols.resource_id {
                if let Some(target_sym) = pkg.symbols.get(target.symbol_id) {
                    return target_sym.name.clone();
                }
            }
            // Global scope — use bare name
            if target.table_id == tree.global_scope.resource_id {
                if let Some(target_sym) = tree.global_scope.get(target.symbol_id) {
                    return target_sym.name.clone();
                }
            }
            // Cross-package — construct URL
            if let Some(target_sym) = find_symbol_in_tree(tree, target.table_id, target.symbol_id) {
                return format!("/{}/{}", find_package_path(tree, target.table_id), target_sym.name);
            }
        }
        return sym.name.clone();
    }
    // Check the global scope
    if let Some(sym) = tree.global_scope.get(id) {
        return sym.name.clone();
    }
    "?".to_string()
}

/// Find a symbol by table ID and symbol ID anywhere in the tree.
fn find_symbol_in_tree(tree: &InterTree, table_id: u32, symbol_id: u32) -> Option<&Symbol> {
    if table_id == tree.global_scope.resource_id {
        return tree.global_scope.get(symbol_id);
    }
    find_symbol_in_package(&tree.root, table_id, symbol_id)
}

/// Find a symbol in a package or its children.
fn find_symbol_in_package(pkg: &Package, table_id: u32, symbol_id: u32) -> Option<&Symbol> {
    if pkg.symbols.resource_id == table_id {
        return pkg.symbols.get(symbol_id);
    }
    for child in pkg.children_iter() {
        if let Some(sym) = find_symbol_in_package(child, table_id, symbol_id) {
            return Some(sym);
        }
    }
    None
}

/// Find the URL path for a symbols table by its resource ID.
fn find_package_path(tree: &InterTree, table_id: u32) -> String {
    find_package_path_internal(&tree.root, table_id, Vec::new())
        .map(|parts| parts.join("/"))
        .unwrap_or_default()
}

fn find_package_path_internal(pkg: &Package, table_id: u32, mut path: Vec<String>) -> Option<Vec<String>> {
    if pkg.symbols.resource_id == table_id {
        return Some(path);
    }
    for child in pkg.children_iter() {
        path.push(child.name.clone());
        if let Some(result) = find_package_path_internal(child, table_id, path.clone()) {
            return Some(result);
        }
        path.pop();
    }
    None
}

/// Render an optional type marker as it appears in textual Inter.
///
/// If the instruction has a type marker, returns `"(<marker>) "`,
/// otherwise returns an empty string.
fn type_marker_str(tree: &InterTree, instr: &Instruction) -> String {
    instr.type_marker
        .and_then(|id| tree.get_string(id))
        .map(|m| format!("({}) ", m))
        .unwrap_or_default()
}

/// Format a wiring target as a URL or bare name for textual Inter output.
///
/// Looks up the target symbol by table ID and symbol ID, and formats it
/// as a URL (e.g., `/main/sub/C_gamma`) or a bare name if it's in the
/// global scope (e.g., `!print`).
fn format_wiring_target(tree: &InterTree, target: &WiringTarget) -> String {
    // Global scope — use bare name
    if target.table_id == tree.global_scope.resource_id {
        if let Some(sym) = tree.global_scope.get(target.symbol_id) {
            return sym.name.clone();
        }
        return "?".to_string();
    }
    // Cross-package — construct URL
    if let Some(target_sym) = find_symbol_in_tree(tree, target.table_id, target.symbol_id) {
        let path = find_package_path(tree, target.table_id);
        if path.is_empty() {
            return target_sym.name.clone();
        }
        return format!("/{}/{}", path, target_sym.name);
    }
    "?".to_string()
}

/// Parse an optional parenthesised type marker from a token sequence.
///
/// If `tokens[idx]` starts with `(`, returns the marker text (with the
/// outer parentheses removed) and the index of the next token.
/// Otherwise returns `None` and the original index.
///
/// Handles multi-token markers like `(list of int32)` by consuming
/// tokens until the closing `)`.
fn parse_type_marker<'a>(tokens: &'a [&'a str], mut idx: usize) -> (Option<String>, usize) {
    if idx < tokens.len() && tokens[idx].starts_with('(') {
        let mut marker = tokens[idx];
        if marker.ends_with(')') {
            // Single-token marker: (K_number)
            marker = &marker[1..marker.len() - 1];
            idx += 1;
            return (Some(marker.to_string()), idx);
        }
        // Multi-token marker: (list of int32)
        // Consume tokens until we find one ending with ')'
        let mut parts = vec![&marker[1..]]; // strip leading '('
        idx += 1;
        while idx < tokens.len() {
            let t = tokens[idx];
            if let Some(stripped) = t.strip_suffix(')') {
                parts.push(stripped); // strip trailing ')'
                idx += 1;
                return (Some(parts.join(" ")), idx);
            }
            parts.push(t);
            idx += 1;
        }
        // Unclosed '(' — return what we have
        return (Some(parts.join(" ")), idx);
    }
    (None, idx)
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Write an [`InterTree`] as a textual Inter string.
///
/// This is the main entry point for generating `.intert` output. It
/// traverses the tree recursively, writing each package's items
/// (instructions and child packages) with appropriate tab indentation.
///
/// The output is designed to be readable by the existing `inter` tool
/// and by this crate's own [`read`] function (round-trip fidelity).
pub fn write(tree: &InterTree) -> String {
    let mut out = String::new();
    write_package(tree, &tree.root, 0, &mut out);
    out
}

/// Write a single package and all its descendants.
///
/// Items are written in order: instructions and child packages are
/// interleaved as they appear in the [`Package::items`] list. This
/// preserves the original ordering from the textual Inter source.
///
/// Each level of nesting adds one tab of indentation.
fn write_package(tree: &InterTree, pkg: &Package, depth: usize, out: &mut String) {
    let indent = "\t".repeat(depth);

    for item in &pkg.items {
        match item {
            PackageItem::Instruction(instr) => {
                write_instruction(tree, pkg, instr, depth + instr.depth, out);
            }
            PackageItem::Child(name) => {
                if let Some(child) = pkg.children.get(name) {
                    let type_str = child.package_type.keyword();
                    let marker = child.type_marker
                        .and_then(|id| tree.get_string(id))
                        .map(|m| format!("({}) ", m))
                        .unwrap_or_default();
                    out.push_str(&format!("{}package {}{} {}\n", indent, marker, child.name, type_str));
                    write_package(tree, child, depth + 1, out);
                }
            }
        }
    }
}

/// Write a single instruction in textual Inter format.
///
/// Each construct has its own formatting logic. For constructs we don't
/// fully handle, we fall back to writing just the keyword. The output
/// is designed to match the C implementation's textual Inter format.
fn write_instruction(
    tree: &InterTree,
    pkg: &Package,
    instr: &Instruction,
    depth: usize,
    out: &mut String,
) {
    let indent = "\t".repeat(depth);

    match instr.construct {
        ConstructId::Comment => {
            if let Some(text_id) = instr.field(1) {
                let text = tree.get_string(text_id).unwrap_or("");
                out.push_str(&format!("{}# {}\n", indent, text));
            } else {
                out.push_str(&format!("{}#\n", indent));
            }
        }
        ConstructId::Packagetype => {
            let name_id = instr.field(1).unwrap_or(0);
            let name = tree.get_string(name_id).unwrap_or("?");
            out.push_str(&format!("{}packagetype {}\n", indent, name));
        }
        ConstructId::Primitive => {
            let name_id = instr.field(1).unwrap_or(0);
            let name = tree.get_string(name_id).unwrap_or("?");
            if let Some(sig_id) = instr.field(2) {
                let sig = tree.get_string(sig_id).unwrap_or("?");
                out.push_str(&format!("{}primitive {} {}\n", indent, name, sig));
            } else {
                out.push_str(&format!("{}primitive {}\n", indent, name));
            }
        }
        ConstructId::Constant => {
            let sym_id = instr.field(1).unwrap_or(0);
            let fmt = ValueFormat::from_u32(instr.field(2).unwrap_or(0));
            let content = instr.field(3).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let marker = type_marker_str(tree, instr);
            if let Some(fmt) = fmt {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                out.push_str(&format!("{}constant {}{} = {}\n", indent, marker, sym_name, val_str));
            }
        }
        ConstructId::Typename => {
            let sym_id = instr.field(1).unwrap_or(0);
            let type_id = instr.field(2).unwrap_or(0);
            let op_id = instr.field(3).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let type_str = tree.get_string(type_id).unwrap_or("?");
            let operator = tree.get_string(op_id).unwrap_or("=");
            out.push_str(&format!("{}typename {} {} {}\n", indent, sym_name, operator, type_str));
        }
        ConstructId::Variable => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let marker = type_marker_str(tree, instr);
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(2).unwrap_or(0)),
                instr.field(3),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                out.push_str(&format!("{}variable {}{} = {}\n", indent, marker, sym_name, val_str));
            } else {
                out.push_str(&format!("{}variable {}{}\n", indent, marker, sym_name));
            }
        }
        ConstructId::Code => {
            out.push_str(&format!("{}code\n", indent));
        }
        ConstructId::Inv => {
            let prim_id = instr.field(1).unwrap_or(0);
            let prim_name = if let Some(sym) = tree.global_scope.get(prim_id) {
                sym.name.clone()
            } else {
                resolve_symbol_name(tree, pkg, prim_id)
            };
            out.push_str(&format!("{}inv {}\n", indent, prim_name));
        }
        ConstructId::Val => {
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(1).unwrap_or(0)),
                instr.field(2),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                let marker = type_marker_str(tree, instr);
                out.push_str(&format!("{}val {}{}\n", indent, marker, val_str));
            }
        }
        ConstructId::Lab => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = resolve_symbol_name(tree, pkg, sym_id);
            out.push_str(&format!("{}lab .{}\n", indent, sym_name));
        }
        ConstructId::Label => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = resolve_symbol_name(tree, pkg, sym_id);
            out.push_str(&format!("{}.{}\n", indent, sym_name));
        }
        ConstructId::Local => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let marker = type_marker_str(tree, instr);
            out.push_str(&format!("{}local {}{}\n", indent, marker, sym_name));
        }
        ConstructId::Instance => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let marker = type_marker_str(tree, instr);
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(2).unwrap_or(0)),
                instr.field(3),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                out.push_str(&format!("{}instance {}{} = {}\n", indent, marker, sym_name, val_str));
            } else {
                out.push_str(&format!("{}instance {}{}\n", indent, marker, sym_name));
            }
        }
        ConstructId::Property => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let marker = type_marker_str(tree, instr);
            out.push_str(&format!("{}property {}{}\n", indent, marker, sym_name));
        }
        ConstructId::Propertyvalue => {
            let owner_id = instr.field(1).unwrap_or(0);
            let prop_id = instr.field(2).unwrap_or(0);
            let owner_name = pkg.symbols.get(owner_id).map(|s| s.name.as_str()).unwrap_or("?");
            let prop_name = pkg.symbols.get(prop_id).map(|s| s.name.as_str()).unwrap_or("?");
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(3).unwrap_or(0)),
                instr.field(4),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                out.push_str(&format!(
                    "{}propertyvalue {} of {} = {}\n",
                    indent, prop_name, owner_name, val_str
                ));
            }
        }
        ConstructId::Permission => {
            let kind_id = instr.field(1).unwrap_or(0);
            let prop_id = instr.field(2).unwrap_or(0);
            let kind_name = pkg.symbols.get(kind_id).map(|s| s.name.as_str()).unwrap_or("?");
            let prop_name = pkg.symbols.get(prop_id).map(|s| s.name.as_str()).unwrap_or("?");
            out.push_str(&format!(
                "{}permission for {} to have {}\n",
                indent, kind_name, prop_name
            ));
        }
        ConstructId::Pragma => {
            let target_id = instr.field(1).unwrap_or(0);
            let target = tree.get_string(target_id).unwrap_or("?");
            if let Some(value_id) = instr.field(2) {
                let value = tree.get_string(value_id).unwrap_or("?");
                out.push_str(&format!("{}pragma {} \"{}\"\n", indent, target, value));
            } else {
                out.push_str(&format!("{}pragma {}\n", indent, target));
            }
        }
        ConstructId::Cast => {
            let to_id = instr.field(1).unwrap_or(0);
            let from_id = instr.field(2).unwrap_or(0);
            let to_type = tree.get_string(to_id).unwrap_or("?");
            let from_type = tree.get_string(from_id).unwrap_or("?");
            out.push_str(&format!("{}cast {} <- {}\n", indent, to_type, from_type));
        }
        ConstructId::Splat => {
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(1).unwrap_or(0)),
                instr.field(2),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| resolve_symbol_name(tree, pkg, id),
                );
                out.push_str(&format!("{}splat {}\n", indent, val_str));
            }
        }
        ConstructId::Insert => {
            out.push_str(&format!("{}insert\n", indent));
        }
        ConstructId::Nop => {
            out.push_str(&format!("{}nop\n", indent));
        }
        ConstructId::Plug => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym = pkg.symbols.get(sym_id);
            let sym_name = sym.map(|s| s.name.as_str()).unwrap_or("?");
            if let Some(sym) = sym {
                if let Some(ref name) = sym.wired_to_name {
                    // URL targets (starting with /) are written bare;
                    // other targets (forward references) are quoted.
                    if name.starts_with('/') {
                        out.push_str(&format!("{}plug {} ~~> {}\n", indent, sym_name, name));
                    } else {
                        out.push_str(&format!("{}plug {} ~~> \"{}\"\n", indent, sym_name, name));
                    }
                } else if let Some(ref target) = sym.wired_to {
                    // Resolved wiring — look up the target and write its URL
                    let target_url = format_wiring_target(tree, target);
                    out.push_str(&format!("{}plug {} ~~> {}\n", indent, sym_name, target_url));
                } else {
                    out.push_str(&format!("{}plug {}\n", indent, sym_name));
                }
            }
        }
        ConstructId::Socket => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym = pkg.symbols.get(sym_id);
            let sym_name = sym.map(|s| s.name.as_str()).unwrap_or("?");
            if let Some(sym) = sym {
                if let Some(ref name) = sym.wired_to_name {
                    // URL targets (starting with /) are written bare;
                    // other targets (forward references) are quoted.
                    if name.starts_with('/') {
                        out.push_str(&format!("{}socket {} ~~> {}\n", indent, sym_name, name));
                    } else {
                        out.push_str(&format!("{}socket {} ~~> \"{}\"\n", indent, sym_name, name));
                    }
                } else if let Some(ref target) = sym.wired_to {
                    // Resolved wiring — look up the target and write its URL
                    let target_url = format_wiring_target(tree, target);
                    out.push_str(&format!("{}socket {} ~~> {}\n", indent, sym_name, target_url));
                } else {
                    out.push_str(&format!("{}socket {}\n", indent, sym_name));
                }
            }
        }
        _ => {
            // Generic: write the keyword followed by any frame words.
            // Frame words may be symbol IDs (resolved via resolve_symbol_name)
            // or raw values. We try to resolve as symbols first, then fall
            // back to raw numbers.
            let keyword = instr.construct.keyword();
            let mut line = format!("{}{}", indent, keyword);
            for i in 1..instr.words.len() {
                let val = instr.words[i];
                let name = resolve_symbol_name(tree, pkg, val);
                if name != "?" {
                    line.push(' ');
                    line.push_str(&name);
                } else {
                    line.push(' ');
                    line.push_str(&val.to_string());
                }
            }
            line.push('\n');
            out.push_str(&line);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hello_world() {
        let input = r#"package main _plain
	package Main _code
		code
			inv !enableprinting
			inv !print
				val "Hello, world.\n"
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        let main_fn = main.get_child("Main").unwrap();
        assert_eq!(main_fn.package_type, PackageType::Code);
        assert_eq!(main_fn.instructions().count(), 4); // code, inv, inv, val
    }

    #[test]
    fn test_roundtrip_hello() {
        let input = r#"package main _plain
	package Main _code
		code
			inv !enableprinting
			inv !print
				val "Hello, world.\n"
"#;
        let tree = read(input).unwrap();
        let output = write(&tree);

        // Re-parse the output
        let tree2 = read(&output).unwrap();
        let main = tree2.find_package("/main").unwrap();
        let main_fn = main.get_child("Main").unwrap();
        assert_eq!(main_fn.package_type, PackageType::Code);
    }

    #[test]
    fn test_parse_packages() {
        let input = r#"packagetype _plain
packagetype _code

package main _plain
	typename K_number = int32
	constant (K_number) x = 11
	package sub _plain
		constant (/main/K_number) y = 17
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        assert!(main.get_child("sub").is_some());
    }

    #[test]
    fn test_parse_constant() {
        let input = r#"package main _plain
	constant lucky_number = 7
	constant message = "hello"
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        assert!(main.symbols.get_by_name("lucky_number").is_some());
        assert!(main.symbols.get_by_name("message").is_some());
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize(r#"constant x = "hello world""#);
        assert_eq!(tokens, vec!["constant", "x", "=", r#""hello world""#]);
    }

    #[test]
    fn test_split_annotations() {
        let (content, annotations) = split_annotations("package main _plain __foo __bar=2");
        assert_eq!(content, "package main _plain");
        assert_eq!(annotations.len(), 2);
    }

    #[test]
    fn test_split_annotations_with_tabs() {
        // Annotations preceded by tabs should also be recognized
        let (content, annotations) = split_annotations("package main _plain\t__foo");
        assert_eq!(content, "package main _plain");
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].0, "__foo");
    }

    #[test]
    fn test_split_annotations_inside_quotes() {
        // Annotations inside quoted strings should NOT be treated as annotations
        let (content, annotations) = split_annotations(r#"constant x = "foo __bar""#);
        assert_eq!(content, r#"constant x = "foo __bar""#);
        assert_eq!(annotations.len(), 0);
    }

    // --- Negative tests ---

    #[test]
    fn test_error_spaces_instead_of_tabs() {
        let input = "package main _plain\n    package sub _code\n";
        let result = read(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("spaces"));
    }

    #[test]
    fn test_error_unknown_construct() {
        let input = "package main _plain\n\tfoobar 42\n";
        let result = read(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown construct"));
    }

    #[test]
    fn test_error_indentation_jump_at_root() {
        // At root level (before any package is entered),
        // jumping more than 1 level is an error.
        let input = "\t\t\tcode\n";
        let result = read(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("indentation"));
    }

    #[test]
    fn test_unclosed_quote_currently_accepted() {
        // A quoted string that never closes. Since tokenize doesn't
        // error on this (it just keeps the token open), the parser
        // should still succeed. This documents current behavior.
        let input = "package main _plain\n\tconstant x = \"hello\n";
        let result = read(input);
        // Currently succeeds because tokenize doesn't validate quote balance
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_constant_no_value() {
        let input = "package main _plain\n\tconstant x =\n";
        let result = read(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_package_no_type() {
        let input = "package main\n";
        let result = read(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_typename_no_type() {
        let input = "package main _plain\n\ttypename K_number\n";
        let result = read(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_hex() {
        let input = "package main _plain\n\tconstant x = 0xGGGG\n";
        let result = read(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_invalid_binary() {
        let input = "package main _plain\n\tconstant x = 0b1234\n";
        let result = read(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unknown_escape() {
        let input = "package main _plain\n\tconstant x = \"hello\\rworld\"\n";
        let result = read(input);
        assert!(result.is_err());
    }

    // --- Propertyvalue tests ---

    #[test]
    fn test_parse_propertyvalue_with_of() {
        let input = r#"package main _plain
	propertyvalue P_strength of I_citrus = 20
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        // Check that symbols were created correctly
        assert!(main.symbols.get_by_name("P_strength").is_some());
        assert!(main.symbols.get_by_name("I_citrus").is_some());
        // Check the instruction
        let instrs: Vec<&Instruction> = main.instructions().collect();
        assert_eq!(instrs.len(), 1);
        assert_eq!(instrs[0].construct, ConstructId::Propertyvalue);
        let owner_id = instrs[0].field(1).unwrap_or(0);
        let prop_id = instrs[0].field(2).unwrap_or(0);
        let owner = main.symbols.get(owner_id).unwrap();
        let prop = main.symbols.get(prop_id).unwrap();
        assert_eq!(owner.name, "I_citrus");
        assert_eq!(prop.name, "P_strength");
    }

    #[test]
    fn test_parse_propertyvalue_without_of() {
        let input = r#"package main _plain
	propertyvalue P_strength I_citrus = 20
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        let instrs: Vec<&Instruction> = main.instructions().collect();
        assert_eq!(instrs.len(), 1);
        let owner_id = instrs[0].field(1).unwrap_or(0);
        let prop_id = instrs[0].field(2).unwrap_or(0);
        let owner = main.symbols.get(owner_id).unwrap();
        let prop = main.symbols.get(prop_id).unwrap();
        assert_eq!(owner.name, "I_citrus");
        assert_eq!(prop.name, "P_strength");
    }

    #[test]
    fn test_parse_permission() {
        let input = r#"package main _plain
	permission for K_odour to have P_strength
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        assert!(main.symbols.get_by_name("K_odour").is_some());
        assert!(main.symbols.get_by_name("P_strength").is_some());
        let instrs: Vec<&Instruction> = main.instructions().collect();
        assert_eq!(instrs.len(), 1);
        assert_eq!(instrs[0].construct, ConstructId::Permission);
        let kind_id = instrs[0].field(1).unwrap_or(0);
        let prop_id = instrs[0].field(2).unwrap_or(0);
        let kind = main.symbols.get(kind_id).unwrap();
        let prop = main.symbols.get(prop_id).unwrap();
        assert_eq!(kind.name, "K_odour");
        assert_eq!(prop.name, "P_strength");
    }

    #[test]
    fn test_parse_pragma_with_value() {
        let input = r#"package main _plain
	pragma target_I6 "$MAX_STATIC_DATA=180000"
"#;
        let tree = read(input).unwrap();
        let main = tree.find_package("/main").unwrap();
        let instrs: Vec<&Instruction> = main.instructions().collect();
        assert_eq!(instrs.len(), 1);
        assert_eq!(instrs[0].construct, ConstructId::Pragma);
        let target_id = instrs[0].field(1).unwrap_or(0);
        let value_id = instrs[0].field(2).unwrap_or(0);
        let target = tree.get_string(target_id).unwrap_or("?");
        let value = tree.get_string(value_id).unwrap_or("?");
        assert_eq!(target, "target_I6");
        assert_eq!(value, "$MAX_STATIC_DATA=180000");
    }

    #[test]
    fn test_parse_type_marker_multi_token() {
        let (marker, idx) = parse_type_marker(&["(list", "of", "int32)"], 0);
        assert_eq!(marker, Some("list of int32".to_string()));
        assert_eq!(idx, 3);
    }

    #[test]
    fn test_parse_type_marker_single_token() {
        let (marker, idx) = parse_type_marker(&["(K_number)", "x"], 0);
        assert_eq!(marker, Some("K_number".to_string()));
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_parse_type_marker_none() {
        let (marker, idx) = parse_type_marker(&["x", "=", "42"], 0);
        assert!(marker.is_none());
        assert_eq!(idx, 0);
    }
}
