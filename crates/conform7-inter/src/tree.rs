//! Inter tree — the core data structure for Inter programs.
//!
//! Based on `Chapter 2/Inter Trees.w`, `Packages.w`, `Symbols Tables.w`,
//! and `The Warehouse.w` from the bytecode module.
//!
//! # Architecture
//!
//! An Inter tree is a hierarchical structure of **packages** (like nested
//! boxes), each containing **symbols** (named references) and **instructions**
//! (bytecode operations). This mirrors the C implementation's `inter_tree`,
//! `inter_package`, and `inter_symbols_table`.
//!
//! ```text
//! InterTree
//! ├── global_scope (SymbolsTable)     ← primitives, package types
//! ├── strings (HashMap<u32, String>)  ← the "warehouse" for text resources
//! └── root (Package, name="")
//!     └── main (Package, type=_plain)
//!         ├── architecture (Package, type=_linkage)
//!         ├── connectors (Package, type=_linkage)
//!         ├── source_text (Package, type=_module)
//!         │   └── kinds (Package, type=_submodule)
//!         └── BasicInformKit (Package, type=_module)
//! ```
//!
//! # The Warehouse
//!
//! In the C implementation, the "warehouse" is a central resource registry
//! that assigns numeric IDs to strings, symbols tables, packages, and node
//! lists. We simplify this: strings are stored in a `HashMap<u32, String>`
//! on the tree, and other resources (symbols tables, packages) carry their
//! own IDs. The `alloc_resource_id` method provides monotonically increasing
//! IDs to match the C behavior.
//!
//! # Symbols and Wiring
//!
//! Symbols are the named entities in an Inter program. Each package has its
//! own symbols table. Symbols can be:
//!
//! - **Defined locally**: the symbol's definition instruction is in the same package
//! - **Wired**: the symbol is connected to a symbol in another package via
//!   `S1 ~~> S2` wiring. This is how cross-package references work.
//! - **A plug**: a placeholder that will be connected during linking
//! - **A socket**: an export point that other trees can connect to
//!
//! Wiring is directional: `S1 ~~> S2` means S1 means whatever S2 means.
//! Circular wirings are forbidden.

use crate::instruction::Instruction;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Symbol
// ---------------------------------------------------------------------------

/// The type of a symbol — what kind of entity it names.
///
/// These values are stored in binary Inter files and must match the C
/// implementation's `*_ISYMT` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SymbolType {
    /// Symbol exists but hasn't been defined yet (or is used miscellaneously).
    /// This is the default state for newly created symbols.
    Misc = 0,
    /// Defined by a `constant` instruction.
    Constant = 1,
    /// Defined by a `variable` instruction.
    Variable = 2,
    /// Defined by a `typename` instruction (a type alias).
    Typename = 3,
    /// Defined by a `package` instruction (a sub-package).
    Package = 4,
    /// Defined by a `primitive` instruction (a built-in operation).
    Primitive = 5,
    /// Defined by a `property` instruction.
    Property = 6,
    /// Defined by an `instance` instruction.
    Instance = 7,
    /// A plug — wired to an external symbol (resolved during linking).
    Plug = 8,
    /// A socket — available for external symbols to wire to.
    Socket = 9,
    /// Defined by a `label` instruction (a jump target in code).
    Label = 10,
}

/// A named entity in a package's symbol table.
///
/// Symbols are the primary way Inter code refers to things. Instead of
/// embedding definitions inline, instructions reference symbols by ID.
/// This level of indirection enables wiring (cross-package references)
/// and linking (cross-tree connections).
///
/// Corresponds to `inter_symbol` in the C implementation.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Unique ID within the owning symbols table. Symbol IDs start at
    /// `SYMBOL_BASE_VAL` (0x40000000) in the C implementation.
    pub id: u32,

    /// The human-readable name (e.g., "K_number", "V_banana", "!print").
    pub name: String,

    /// What kind of entity this symbol names.
    pub symbol_type: SymbolType,

    /// If this symbol is wired to another symbol, the target.
    /// Wiring is directional: this symbol means whatever the target means.
    pub wired_to: Option<WiringTarget>,

    /// If this symbol is a plug wired to a name (not yet resolved to a symbol).
    /// This happens during textual Inter parsing when a forward reference
    /// is encountered. It's resolved in a second pass.
    pub wired_to_name: Option<String>,

    /// Persistent flags from the binary format. These are bitflags that
    /// survive round-tripping through binary Inter.
    pub flags: u32,

    /// Boolean annotations packed into a bitmap. Each bit corresponds to
    /// a boolean annotation declared in the file header.
    pub boolean_annotations: u32,

    /// Non-boolean annotations. Each entry is an `(annotation_id, value)` pair.
    /// Annotations provide metadata about symbols (e.g., `__text="hello"`).
    pub annotations: Vec<(u32, u32)>,
}

impl Symbol {
    /// Create a new symbol with default settings.
    ///
    /// The symbol starts with type [`SymbolType::Misc`] (undefined) and no
    /// wiring or annotations. The caller should set the type once the
    /// defining instruction is processed.
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            symbol_type: SymbolType::Misc,
            wired_to: None,
            wired_to_name: None,
            flags: 0,
            boolean_annotations: 0,
            annotations: Vec::new(),
        }
    }

    /// Whether this symbol is a plug (needs external wiring).
    pub fn is_plug(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Plug)
    }

    /// Whether this symbol is a socket (available for external wiring).
    pub fn is_socket(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Socket)
    }

    /// Whether this symbol is wired to another symbol.
    pub fn is_wired(&self) -> bool {
        self.wired_to.is_some()
    }

    /// Whether this symbol is wired to a name (forward reference, not yet resolved).
    pub fn is_wired_to_name(&self) -> bool {
        self.wired_to_name.is_some()
    }
}

// ---------------------------------------------------------------------------
// Wiring Target
// ---------------------------------------------------------------------------

/// Identifies the target of a symbol wiring.
///
/// When symbol S1 is wired to symbol S2 (`S1 ~~> S2`), this struct
/// records where S2 lives. The target is identified by the combination
/// of a symbols table (by its warehouse resource ID) and a symbol ID
/// within that table.
///
/// This is needed because symbols in different packages have independent
/// ID spaces. A symbol ID alone is not enough to uniquely identify a
/// symbol across the whole tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiringTarget {
    /// The warehouse resource ID of the symbols table containing the target.
    pub table_id: u32,
    /// The symbol ID within that table.
    pub symbol_id: u32,
}

// ---------------------------------------------------------------------------
// Symbols Table
// ---------------------------------------------------------------------------

/// A collection of named symbols belonging to a package.
///
/// Each package has its own symbols table. Symbols are indexed both by ID
/// (for fast bytecode decoding) and by name (for textual Inter parsing and
/// name resolution).
///
/// In the C implementation, this is `inter_symbols_table`. The C version
/// uses a fixed-size array indexed by symbol ID (which starts at
/// `SYMBOL_BASE_VAL`, a large number). We use a `HashMap` for flexibility.
///
/// # Symbol IDs
///
/// Symbol IDs start at `SYMBOL_BASE_VAL` (0x40000000) and increase
/// monotonically. This high base value is chosen so that symbol IDs
/// are in a range that compresses well in the binary format (the
/// compression scheme has a special short encoding for values in
/// `0x40000000..0x4000001F`).
#[derive(Debug, Clone)]
pub struct SymbolsTable {
    /// Warehouse resource ID of this table. Used for wiring references.
    pub resource_id: u32,

    /// Symbols by ID. The primary index for bytecode decoding.
    pub symbols: HashMap<u32, Symbol>,

    /// Symbols by name. Used for textual Inter parsing and name lookup.
    pub by_name: HashMap<String, u32>,

    /// Next available symbol ID. All tables share a single counter so that
    /// symbol IDs are globally unique across the tree.
    counter: Rc<Cell<u32>>,
}

impl SymbolsTable {
    /// The base value for symbol IDs.
    ///
    /// This matches `SYMBOL_BASE_VAL` in the C implementation. The value
    /// 0x40000000 is chosen because the binary Inter compression scheme
    /// has a compact 1-byte encoding for values in the range
    /// `0x40000000..0x4000001F`, which covers the most common symbol IDs.
    pub const SYMBOL_BASE: u32 = 0x40000000;

    /// Create a new, empty symbols table with the given warehouse resource ID.
    ///
    /// The `counter` is a shared cell holding the next symbol ID to assign.
    /// All tables in a tree share the same counter so that symbol IDs are
    /// globally unique.
    pub fn new(resource_id: u32, counter: Rc<Cell<u32>>) -> Self {
        Self {
            resource_id,
            symbols: HashMap::new(),
            by_name: HashMap::new(),
            counter,
        }
    }

    /// Create a new symbol with an auto-assigned ID.
    ///
    /// Returns a mutable reference to the newly created symbol. The symbol
    /// starts with type [`SymbolType::Misc`].
    pub fn create_symbol(&mut self, name: &str) -> &mut Symbol {
        let id = self.counter.get();
        self.counter.set(id + 1);
        let sym = Symbol::new(id, name.to_string());
        self.symbols.insert(id, sym);
        self.by_name.insert(name.to_string(), id);
        self.symbols.get_mut(&id).unwrap()
    }

    /// Create a new symbol with a specific ID.
    ///
    /// This is used when reading binary Inter files, where symbol IDs are
    /// assigned by the original compiler and must be preserved for correct
    /// wiring references.
    pub fn create_symbol_at_id(&mut self, name: &str, id: u32) -> &mut Symbol {
        let current = self.counter.get();
        if id >= current {
            self.counter.set(id + 1);
        }
        let sym = Symbol::new(id, name.to_string());
        self.symbols.insert(id, sym);
        self.by_name.insert(name.to_string(), id);
        self.symbols.get_mut(&id).unwrap()
    }

    /// Return the next symbol ID that would be assigned.
    pub fn next_id(&self) -> u32 {
        self.counter.get()
    }

    /// Look up a symbol by ID. Returns `None` if not found.
    pub fn get(&self, id: u32) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    /// Look up a symbol by ID, mutable. Returns `None` if not found.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Symbol> {
        self.symbols.get_mut(&id)
    }

    /// Look up a symbol by name. Returns `None` if not found.
    pub fn get_by_name(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).and_then(|id| self.symbols.get(id))
    }

    /// Look up a symbol by name, mutable. Returns `None` if not found.
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.by_name.get(name).and_then(|id| self.symbols.get_mut(id))
    }

    /// Check whether a symbol with the given name exists in this table.
    pub fn has_name(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Iterate over all symbols in this table.
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }
}

// ---------------------------------------------------------------------------
// Package Type
// ---------------------------------------------------------------------------

/// The type of a package — determines what can appear inside it.
///
/// Package types are declared with `packagetype` at the root level and
/// then used in `package` instructions. The standard types are:
///
/// - `_plain`: General-purpose container. The `main` package and most
///   subpackages use this.
/// - `_code`: Contains executable code (function bodies).
/// - `_module`: A compilation unit (source text, extension, or kit).
/// - `_submodule`: A subdivision within a module (e.g., `kinds`, `variables`).
/// - `_linkage`: Special packages for cross-tree linking (`architecture`,
///   `connectors`).
///
/// Custom package types (like `R_101` or `OtherFunction`) are also supported
/// via the `Custom` variant. These are declared with `packagetype` and can
/// be used anywhere a standard type can.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageType {
    /// General-purpose container. The default package type.
    Plain,
    /// Contains executable code (function body).
    Code,
    /// A compilation unit (source text, extension, or kit).
    Module,
    /// A subdivision within a module.
    Submodule,
    /// Special package for cross-tree linking.
    Linkage,
    /// A custom package type declared with `packagetype`.
    Custom(String),
}

impl PackageType {
    /// The keyword used in textual Inter for this package type.
    ///
    /// Standard types use their underscore-prefixed names (e.g., `_plain`).
    /// Custom types use their declared name as-is.
    pub fn keyword(&self) -> &str {
        match self {
            Self::Plain => "_plain",
            Self::Code => "_code",
            Self::Module => "_module",
            Self::Submodule => "_submodule",
            Self::Linkage => "_linkage",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Parse a package type from its textual Inter keyword.
    ///
    /// Standard types are recognized by their underscore-prefixed names.
    /// Anything else is treated as a custom type. This matches the C
    /// implementation's behavior where `packagetype` declarations make
    /// any name a valid package type.
    pub fn from_keyword(kw: &str) -> Self {
        match kw {
            "_plain" => Self::Plain,
            "_code" => Self::Code,
            "_module" => Self::Module,
            "_submodule" => Self::Submodule,
            "_linkage" => Self::Linkage,
            other => Self::Custom(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Package
// ---------------------------------------------------------------------------

/// A package in the Inter tree — a named container for symbols and instructions.
///
/// Packages are the primary organizational unit in Inter. They form a tree:
/// the root package contains `main`, which contains modules, which contain
/// submodules. Each package has:
///
/// - A **name** (e.g., "main", "source_text", "kinds")
/// - A **type** (e.g., `_plain`, `_code`, `_module`)
/// - A **symbols table** for named entities defined in this package
/// - A list of **instructions** (the actual Inter bytecode)
/// - **Child packages** (forming the hierarchy)
///
/// Corresponds to `inter_package` in the C implementation.
#[derive(Debug, Clone)]
pub struct Package {
    /// Warehouse resource ID. Used for binary Inter serialization and
    /// cross-reference in wiring.
    pub resource_id: u32,

    /// The package name (e.g., "main", "Main", "connectors").
    pub name: String,

    /// The package type, which determines what can appear inside.
    pub package_type: PackageType,

    /// Optional type marker (kind) written before the package name in
    /// textual Inter. For example, `package (K_func) R_101 _code` has
    /// type marker `K_func`.
    pub type_marker: Option<u32>,

    /// The symbols table for this package. All named entities defined
    /// by instructions in this package are recorded here.
    pub symbols: SymbolsTable,

    /// Instructions in this package, in order. These are the actual
    /// Inter bytecode nodes.
    pub instructions: Vec<Instruction>,

    /// Child packages, indexed by name for fast lookup.
    pub children: HashMap<String, Package>,

    /// Child package names in insertion order. This preserves the
    /// order packages were added, which matters for textual Inter output.
    pub child_order: Vec<String>,

    /// Persistent flags from the binary format.
    pub flags: u32,
}

impl Package {
    /// Create a new package.
    ///
    /// The symbols table is automatically created with a resource ID of
    /// `resource_id + 1`. The `symbol_counter` is the shared tree-wide
    /// counter, ensuring symbol IDs are globally unique.
    pub fn new(
        resource_id: u32,
        name: String,
        package_type: PackageType,
        symbol_counter: Rc<Cell<u32>>,
    ) -> Self {
        let symbols = SymbolsTable::new(resource_id + 1, symbol_counter);
        Self {
            resource_id,
            name,
            package_type,
            type_marker: None,
            symbols,
            instructions: Vec::new(),
            children: HashMap::new(),
            child_order: Vec::new(),
            flags: 0,
        }
    }

    /// Add a child package. The child is indexed by name and appended to
    /// the insertion order.
    pub fn add_child(&mut self, child: Package) {
        let name = child.name.clone();
        self.child_order.push(name.clone());
        self.children.insert(name, child);
    }

    /// Look up a child package by name.
    pub fn get_child(&self, name: &str) -> Option<&Package> {
        self.children.get(name)
    }

    /// Look up a child package by name, mutable.
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut Package> {
        self.children.get_mut(name)
    }

    /// Append an instruction to this package's instruction list.
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    /// Whether this package is a function body (type `_code`).
    /// Code packages have different rules about what constructs can appear.
    pub fn is_function_body(&self) -> bool {
        matches!(self.package_type, PackageType::Code)
    }

    /// Whether this is the root package (empty name).
    /// The root package is a special container that exists outside the
    /// normal package hierarchy.
    pub fn is_root(&self) -> bool {
        self.name.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Inter Tree
// ---------------------------------------------------------------------------

/// The top-level Inter tree — the complete intermediate representation
/// of a program.
///
/// An `InterTree` is the root of all Inter data. It contains:
///
/// - A **root package** (empty name) that holds the `main` package and
///   any root-level instructions (primitives, package types, pragmas)
/// - A **global scope** symbols table for primitives and other
///   tree-wide symbols
/// - A **string table** (the "warehouse") mapping resource IDs to
///   string content
/// - A **version** number tracking the Inter specification version
///
/// Corresponds to `inter_tree` in the C implementation.
#[derive(Debug, Clone)]
pub struct InterTree {
    /// The root package. Contains `main` and any root-level instructions.
    /// The root package has an empty name and type `_plain`.
    pub root: Package,

    /// Global symbols table. Contains primitives (`!print`, `!add`, etc.)
    /// and other symbols accessible from anywhere in the tree.
    pub global_scope: SymbolsTable,

    /// String table: warehouse resource ID → string content.
    /// This is the "warehouse" for text resources. When an instruction
    /// needs a string literal, the string is stored here and referenced
    /// by its warehouse ID.
    pub strings: HashMap<u32, String>,

    /// Shared counter for globally unique symbol IDs. All symbols tables
    /// hold a clone of this counter, so IDs never overlap between tables.
    next_symbol_id: Rc<Cell<u32>>,

    /// Next available warehouse resource ID. Increments monotonically.
    /// Resource IDs 0 and 1 are reserved for the global scope and root
    /// package respectively.
    pub next_resource_id: u32,

    /// The Inter specification version (major, minor, patch).
    /// Written to binary Inter file headers for compatibility checking.
    pub version: (u32, u32, u32),
}

impl InterTree {
    /// Create a new, empty Inter tree.
    ///
    /// Initializes the global scope (resource 0), root package (resource 1),
    /// and sets the next resource ID to 2. The version defaults to 1.0.0.
    pub fn new() -> Self {
        let symbol_counter = Rc::new(Cell::new(SymbolsTable::SYMBOL_BASE));
        let global = SymbolsTable::new(0, symbol_counter.clone());
        let root = Package::new(1, String::new(), PackageType::Plain, symbol_counter.clone());
        Self {
            root,
            global_scope: global,
            strings: HashMap::new(),
            next_resource_id: 2,
            next_symbol_id: symbol_counter,
            version: (1, 0, 0),
        }
    }

    /// Allocate a new warehouse resource ID.
    pub fn alloc_resource_id(&mut self) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }

    /// Allocate a new symbol ID.
    ///
    /// Symbol IDs are shared across all symbols tables in the tree to ensure
    /// that a raw ID uniquely identifies a symbol even without knowing which
    /// table it belongs to.
    pub fn alloc_symbol_id(&self) -> u32 {
        let id = self.next_symbol_id.get();
        self.next_symbol_id.set(id + 1);
        id
    }

    /// Return a clone of the shared symbol ID counter.
    ///
    /// Used when creating a new package, which needs its own symbols table
    /// that continues sharing the tree-wide counter.
    pub fn symbol_counter(&self) -> Rc<Cell<u32>> {
        self.next_symbol_id.clone()
    }

    /// Store a string in the warehouse and return its resource ID.
    ///
    /// If the string is already stored, returns the existing ID (interning).
    /// This deduplication saves memory and ensures that identical strings
    /// compare equal by ID.
    pub fn intern_string(&mut self, s: &str) -> u32 {
        for (&id, existing) in &self.strings {
            if existing == s {
                return id;
            }
        }
        let id = self.alloc_resource_id();
        self.strings.insert(id, s.to_string());
        id
    }

    /// Get a string by its warehouse resource ID.
    pub fn get_string(&self, id: u32) -> Option<&str> {
        self.strings.get(&id).map(|s| s.as_str())
    }

    /// Get or create the `main` package.
    ///
    /// The `main` package is the top-level container for all program content.
    /// It's created automatically on first access if it doesn't exist.
    pub fn main_package(&mut self) -> &mut Package {
        if !self.root.children.contains_key("main") {
            let main = Package::new(
                self.alloc_resource_id(),
                "main".to_string(),
                PackageType::Plain,
                self.next_symbol_id.clone(),
            );
            self.root.add_child(main);
        }
        self.root.get_child_mut("main").unwrap()
    }

    /// Find a package by URL path (e.g., "/main/BasicInformKit/properties").
    ///
    /// The path is split on `/` and each component is used to navigate
    /// down the package hierarchy. Returns `None` if any component is
    /// not found.
    pub fn find_package(&self, url: &str) -> Option<&Package> {
        let parts: Vec<&str> = url.split('/').filter(|p| !p.is_empty()).collect();
        let mut current = &self.root;
        for part in parts {
            current = current.get_child(part)?;
        }
        Some(current)
    }

    /// Find a package by URL path, mutable version.
    pub fn find_package_mut(&mut self, url: &str) -> Option<&mut Package> {
        let parts: Vec<&str> = url.split('/').filter(|p| !p.is_empty()).collect();
        let mut current = &mut self.root;
        for part in parts {
            current = current.children.get_mut(part)?;
        }
        Some(current)
    }
}

impl Default for InterTree {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree() {
        let tree = InterTree::new();
        assert!(tree.root.children.is_empty());
        assert_eq!(tree.version, (1, 0, 0));
    }

    #[test]
    fn test_main_package() {
        let mut tree = InterTree::new();
        let main = tree.main_package();
        assert_eq!(main.name, "main");
        assert_eq!(main.package_type, PackageType::Plain);
    }

    #[test]
    fn test_symbols_table() {
        let counter = Rc::new(Cell::new(SymbolsTable::SYMBOL_BASE));
        let mut table = SymbolsTable::new(0, counter);
        let sym_id = {
            let sym = table.create_symbol("hello");
            assert_eq!(sym.name, "hello");
            assert!(sym.id >= SymbolsTable::SYMBOL_BASE);
            sym.id
        };

        let found = table.get_by_name("hello").unwrap();
        assert_eq!(found.id, sym_id);
    }

    #[test]
    fn test_string_interning() {
        let mut tree = InterTree::new();
        let id1 = tree.intern_string("hello");
        let id2 = tree.intern_string("hello");
        // Same string should get the same ID
        assert_eq!(id1, id2);
        assert_eq!(tree.get_string(id1), Some("hello"));

        // Different string should get a different ID
        let id3 = tree.intern_string("world");
        assert_ne!(id1, id3);
    }
}
