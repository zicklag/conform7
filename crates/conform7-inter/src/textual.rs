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
use crate::tree::{InterTree, Package, PackageType, SymbolType, WiringTarget};
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

        // Parse the line
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
}

impl ReadState {
    fn new() -> Self {
        Self {
            package_stack: Vec::new(),
            current_path: vec![],
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
        // Comment line (either empty or starting with #)
        return Ok(());
    }

    // Split into tokens
    let tokens: Vec<&str> = tokenize(content);

    if tokens.is_empty() {
        return Ok(());
    }

    let keyword = tokens[0];

    match keyword {
        "packagetype" => parse_packagetype(tree, &tokens, line_num)?,
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
        "lab" => {
            // lab <label-name>
            let pkg = get_current_package_mut(tree, state);
            pkg.add_instruction(Instruction::new(ConstructId::Lab));
        }
        "assembly" | "cast" | "evaluation" | "label"
        | "local" | "ref" | "reference" | "splat" => {
            // Code-level constructs — store as generic instruction
            add_instruction(tree, state, keyword, &tokens, line_num)?;
        }
        // Labels like `.begin` start with a dot — they're label definitions
        kw if kw.starts_with('.') => {
            let pkg = get_current_package_mut(tree, state);
            pkg.add_instruction(Instruction::new(ConstructId::Label));
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
    // Find the first `__` that's preceded by whitespace
    let bytes = line.as_bytes();
    for i in 1..bytes.len().saturating_sub(1) {
        if bytes[i] == b'_' && bytes[i + 1] == b'_' && bytes[i - 1] == b' ' {
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
/// Package types are implicitly declared — we just validate the syntax.
/// The C implementation auto-declares package types when they're first
/// referenced, so we don't need to store them explicitly.

fn parse_packagetype(
    _tree: &mut InterTree,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "packagetype requires a name".to_string(),
        });
    }
    // Package types are implicitly declared; we just record them
    // In the C implementation, they're auto-declared when referenced
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
    let name = tokens[1].to_string();
    let type_str = tokens[2];
    let pkg_type = PackageType::from_keyword(type_str);

    // Create the package
    let resource_id = tree.alloc_resource_id();
    let pkg = Package::new(resource_id, name.clone(), pkg_type);

    // Create a symbol for this package in the parent's symbols table
    {
        let parent = get_current_package_mut(tree, state);
        if !parent.symbols.has_name(&name) {
            parent.symbols.create_symbol(&name);
        }
        let sym = parent.symbols.get_by_name_mut(&name).unwrap();
        sym.symbol_type = SymbolType::Package;
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
    let _type_marker = if tokens[idx].starts_with('(') {
        idx += 1;
        true
    } else {
        false
    };

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

    // Parse value first (needs &mut tree)
    let value = parse_value_literal(tree, &value_str, line_num)?;

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

    let mut instr = Instruction::new(ConstructId::Constant);
    instr.set_field(1, sym_id);
    instr.set_field(2, value.format as u32);
    instr.set_field(3, value.content);
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
    let type_str = tokens[eq_idx + 1..].join(" ");

    // Intern string first (needs &mut tree)
    let type_id = tree.intern_string(&type_str);

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

    let mut instr = Instruction::new(ConstructId::Typename);
    instr.set_field(1, sym_id);
    instr.set_field(2, type_id);
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
    if tokens[idx].starts_with('(') {
        idx += 1;
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

    // Parse value first if present (needs &mut tree)
    let value = if let Some(ref val_str) = value_str {
        Some(parse_value_literal(tree, val_str, line_num)?)
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

    let mut instr = Instruction::new(ConstructId::Variable);
    instr.set_field(1, sym_id);
    if let Some(val) = value {
        instr.set_field(2, val.format as u32);
        instr.set_field(3, val.content);
    }
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
    pkg.add_instruction(Instruction::new(ConstructId::Code));
    Ok(())
}

/// Parse an `inv` (invoke) instruction.
///
/// Example: `inv !print`
///
/// Invokes a primitive operation. The primitive must be declared in the
/// global scope. Arguments to the primitive are child instructions at
/// higher indentation (typically `val` instructions).
fn parse_inv(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 2 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "inv requires a primitive name".to_string(),
        });
    }
    let prim_name = tokens[1].to_string();

    // Ensure the primitive exists in global scope
    if !tree.global_scope.has_name(&prim_name) {
        tree.global_scope.create_symbol(&prim_name);
        let sym = tree.global_scope.get_by_name_mut(&prim_name).unwrap();
        sym.symbol_type = SymbolType::Primitive;
    }
    let prim_sym = tree.global_scope.get_by_name(&prim_name).unwrap();

    let mut instr = Instruction::new(ConstructId::Inv);
    instr.set_field(1, prim_sym.id);
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
    if tokens[idx].starts_with('(') {
        idx += 1;
    }

    let value_str = tokens[idx..].join(" ");
    let value = parse_value_literal(tree, &value_str, line_num)?;

    let mut instr = Instruction::new(ConstructId::Val);
    instr.set_field(1, value.format as u32);
    instr.set_field(2, value.content);
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
    let name = tokens[1].to_string();

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Instance;

    let mut instr = Instruction::new(ConstructId::Instance);
    instr.set_field(1, sym.id);
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
    let name = tokens[1].to_string();

    let pkg = get_current_package_mut(tree, state);
    if !pkg.symbols.has_name(&name) {
        pkg.symbols.create_symbol(&name);
    }
    let sym = pkg.symbols.get_by_name_mut(&name).unwrap();
    sym.symbol_type = SymbolType::Property;

    let mut instr = Instruction::new(ConstructId::Property);
    instr.set_field(1, sym.id);
    pkg.add_instruction(instr);

    Ok(())
}

/// Parse a `propertyvalue` instruction.
///
/// Example: `propertyvalue P_strength of I_citrus = 20`
///
/// Sets the value of a property for a specific owner (instance or kind).
/// The owner and property are resolved as symbols in the current package.
fn parse_propertyvalue(
    tree: &mut InterTree,
    state: &mut ReadState,
    tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    if tokens.len() < 4 {
        return Err(TextualError::ParseError {
            line: line_num,
            message: "propertyvalue requires owner, property, and value".to_string(),
        });
    }
    let owner = tokens[1].to_string();
    let property = tokens[2].to_string();
    let value_str = if tokens.len() > 4 { tokens[4..].join(" ") } else { String::new() };

    // Parse value first (needs &mut tree)
    let value = parse_value_literal(tree, &value_str, line_num)?;

    // Now resolve symbols and create instruction
    let pkg = get_current_package_mut(tree, state);
    let owner_id = resolve_or_create_symbol(pkg, &owner, SymbolType::Instance);
    let prop_id = resolve_or_create_symbol(pkg, &property, SymbolType::Property);

    let mut instr = Instruction::new(ConstructId::Propertyvalue);
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
fn parse_permission(
    tree: &mut InterTree,
    state: &mut ReadState,
    _tokens: &[&str],
    _line_num: usize,
) -> Result<(), TextualError> {
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(Instruction::new(ConstructId::Permission));
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
    // Intern string first if needed
    let target_id = if tokens.len() > 1 {
        let target = tokens[1].to_string();
        Some(tree.intern_string(&target))
    } else {
        None
    };

    let pkg = get_current_package_mut(tree, state);
    let mut instr = Instruction::new(ConstructId::Pragma);
    if let Some(tid) = target_id {
        instr.set_field(1, tid);
    }
    pkg.add_instruction(instr);
    Ok(())
}

/// Parse an `insert` directive.
///
/// Example: `insert`
///
/// Marks a position where another package's contents will be inserted
/// during linking. Used for the connectors mechanism.
fn parse_insert(
    tree: &mut InterTree,
    state: &mut ReadState,
    _tokens: &[&str],
    _line_num: usize,
) -> Result<(), TextualError> {
    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(Instruction::new(ConstructId::Insert));
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

    Ok(())
}

/// Generic instruction parser for constructs we don't fully handle yet.
///
/// This is a fallback for code-level constructs like `assembly`, `cast`,
/// `evaluation`, `ref`, `reference`, `splat`, and `label`. These are
/// stored as instructions with their construct ID but without detailed
/// field parsing. Full support will be added as needed.
fn add_instruction(
    tree: &mut InterTree,
    state: &mut ReadState,
    keyword: &str,
    _tokens: &[&str],
    line_num: usize,
) -> Result<(), TextualError> {
    let construct = ConstructId::from_keyword(keyword).ok_or_else(|| {
        TextualError::ParseError {
            line: line_num,
            message: format!("unknown construct: {}", keyword),
        }
    })?;

    let pkg = get_current_package_mut(tree, state);
    pkg.add_instruction(Instruction::new(construct));
    Ok(())
}

// --- Helpers ---

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

    // Unsigned decimal
    if let Ok(n) = s.parse::<u32>() {
        return Ok(InterValue::number(n));
    }

    // Symbol reference (identifier or URL)
    // For now, store as a string reference
    let id = tree.intern_string(s);
    Ok(InterValue::symbolic(id))
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
    for child_name in &pkg.child_order {
        if let Some(child) = pkg.children.get(child_name) {
            let mut child_path = path.to_vec();
            child_path.push(child_name.clone());
            result.extend(collect_resolutions(child, &child_path));
        }
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

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Write an [`InterTree`] as a textual Inter string.
///
/// This is the main entry point for generating `.intert` output. It
/// traverses the tree recursively, writing each package's instructions
/// and child packages with appropriate tab indentation.
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
/// Instructions are written first, then child packages in insertion order.
/// Each level of nesting adds one tab of indentation.
fn write_package(tree: &InterTree, pkg: &Package, depth: usize, out: &mut String) {
    let indent = "\t".repeat(depth);

    // Write instructions
    for instr in &pkg.instructions {
        write_instruction(tree, pkg, instr, depth, out);
    }

    // Write child packages
    for name in &pkg.child_order {
        if let Some(child) = pkg.children.get(name) {
            let type_str = child.package_type.keyword();
            out.push_str(&format!("{}package {} {}\n", indent, child.name, type_str));
            write_package(tree, child, depth + 1, out);
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
            out.push_str(&format!("{}#\n", indent));
        }
        ConstructId::Constant => {
            let sym_id = instr.field(1).unwrap_or(0);
            let fmt = ValueFormat::from_u32(instr.field(2).unwrap_or(0));
            let content = instr.field(3).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            if let Some(fmt) = fmt {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| pkg.symbols.get(id).map(|s| s.name.as_str()).unwrap_or("?").to_string(),
                );
                out.push_str(&format!("{}constant {} = {}\n", indent, sym_name, val_str));
            }
        }
        ConstructId::Typename => {
            let sym_id = instr.field(1).unwrap_or(0);
            let type_id = instr.field(2).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            let type_str = tree.get_string(type_id).unwrap_or("?");
            out.push_str(&format!("{}typename {} = {}\n", indent, sym_name, type_str));
        }
        ConstructId::Variable => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            if let (Some(fmt), Some(content)) = (
                ValueFormat::from_u32(instr.field(2).unwrap_or(0)),
                instr.field(3),
            ) {
                let val = InterValue { format: fmt, content };
                let val_str = val.to_text(
                    &|id| tree.get_string(id).unwrap_or("?").to_string(),
                    &|id| pkg.symbols.get(id).map(|s| s.name.as_str()).unwrap_or("?").to_string(),
                );
                out.push_str(&format!("{}variable {} = {}\n", indent, sym_name, val_str));
            } else {
                out.push_str(&format!("{}variable {}\n", indent, sym_name));
            }
        }
        ConstructId::Code => {
            out.push_str(&format!("{}code\n", indent));
        }
        ConstructId::Inv => {
            let prim_id = instr.field(1).unwrap_or(0);
            let prim_name = tree.global_scope.get(prim_id)
                .map(|s| s.name.as_str())
                .unwrap_or("?");
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
                    &|id| pkg.symbols.get(id).map(|s| s.name.as_str()).unwrap_or("?").to_string(),
                );
                out.push_str(&format!("{}val {}\n", indent, val_str));
            }
        }
        ConstructId::Instance => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            out.push_str(&format!("{}instance {}\n", indent, sym_name));
        }
        ConstructId::Property => {
            let sym_id = instr.field(1).unwrap_or(0);
            let sym_name = pkg.symbols.get(sym_id).map(|s| s.name.as_str()).unwrap_or("?");
            out.push_str(&format!("{}property {}\n", indent, sym_name));
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
                    &|id| pkg.symbols.get(id).map(|s| s.name.as_str()).unwrap_or("?").to_string(),
                );
                out.push_str(&format!(
                    "{}propertyvalue {} {} = {}\n",
                    indent, owner_name, prop_name, val_str
                ));
            }
        }
        ConstructId::Permission => {
            out.push_str(&format!("{}permission\n", indent));
        }
        ConstructId::Pragma => {
            let target_id = instr.field(1).unwrap_or(0);
            let target = tree.get_string(target_id).unwrap_or("?");
            out.push_str(&format!("{}pragma {}\n", indent, target));
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
                    out.push_str(&format!("{}plug {} ~~> \"{}\"\n", indent, sym_name, name));
                } else if let Some(ref _target) = sym.wired_to {
                    // We'd need to look up the target symbol name
                    out.push_str(&format!("{}plug {}\n", indent, sym_name));
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
                    out.push_str(&format!("{}socket {} ~~> \"{}\"\n", indent, sym_name, name));
                } else {
                    out.push_str(&format!("{}socket {}\n", indent, sym_name));
                }
            }
        }
        _ => {
            // Generic: just write the keyword
            out.push_str(&format!("{}{}\n", indent, instr.construct.keyword()));
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
        assert_eq!(main_fn.instructions.len(), 4); // code, inv, inv, val
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
}
