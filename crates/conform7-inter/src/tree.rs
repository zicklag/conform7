//! Inter tree — the core data structure for Inter programs.
//!
//! Based on the `Inter Trees`, `Packages`, and `Symbols Tables` chapters.

use crate::instruction::Instruction;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Symbol
// ---------------------------------------------------------------------------

/// Symbol types. Must match the C `*_ISYMT` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SymbolType {
    /// Symbol is undefined / miscellaneous.
    Misc = 0,
    /// Symbol defined by a constant instruction.
    Constant = 1,
    /// Symbol defined by a variable instruction.
    Variable = 2,
    /// Symbol defined by a typename instruction.
    Typename = 3,
    /// Symbol defined by a package instruction.
    Package = 4,
    /// Symbol defined by a primitive instruction.
    Primitive = 5,
    /// Symbol defined by a property instruction.
    Property = 6,
    /// Symbol defined by an instance instruction.
    Instance = 7,
    /// Symbol is a plug (wired to something external).
    Plug = 8,
    /// Symbol is a socket (available for external wiring).
    Socket = 9,
    /// Symbol defined by a label instruction.
    Label = 10,
}

/// A symbol in a symbols table.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: u32,
    pub name: String,
    pub symbol_type: SymbolType,
    /// If this symbol is wired to another symbol.
    pub wired_to: Option<WiringTarget>,
    /// If this symbol is a plug wired to a name (not yet resolved).
    pub wired_to_name: Option<String>,
    /// Persistent flags.
    pub flags: u32,
    /// Boolean annotations bitmap.
    pub boolean_annotations: u32,
    /// Non-boolean annotations: (annotation_id, value) pairs.
    pub annotations: Vec<(u32, u32)>,
}

impl Symbol {
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

    pub fn is_plug(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Plug)
    }

    pub fn is_socket(&self) -> bool {
        matches!(self.symbol_type, SymbolType::Socket)
    }

    pub fn is_wired(&self) -> bool {
        self.wired_to.is_some()
    }

    pub fn is_wired_to_name(&self) -> bool {
        self.wired_to_name.is_some()
    }
}

/// Target of a symbol wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiringTarget {
    /// The symbols table resource ID containing the target symbol.
    pub table_id: u32,
    /// The target symbol ID.
    pub symbol_id: u32,
}

// ---------------------------------------------------------------------------
// Symbols Table
// ---------------------------------------------------------------------------

/// A symbols table belonging to a package.
#[derive(Debug, Clone)]
pub struct SymbolsTable {
    /// Warehouse resource ID of this table.
    pub resource_id: u32,
    /// Symbols by ID.
    pub symbols: HashMap<u32, Symbol>,
    /// Symbols by name.
    pub by_name: HashMap<String, u32>,
    /// Next available symbol ID.
    next_id: u32,
}

impl SymbolsTable {
    /// The base value for symbol IDs (SYMBOL_BASE_VAL in C).
    pub const SYMBOL_BASE: u32 = 0x40000000;

    pub fn new(resource_id: u32) -> Self {
        Self {
            resource_id,
            symbols: HashMap::new(),
            by_name: HashMap::new(),
            next_id: Self::SYMBOL_BASE,
        }
    }

    pub fn create_symbol(&mut self, name: &str) -> &mut Symbol {
        let id = self.next_id;
        self.next_id += 1;
        let sym = Symbol::new(id, name.to_string());
        self.symbols.insert(id, sym);
        self.by_name.insert(name.to_string(), id);
        self.symbols.get_mut(&id).unwrap()
    }

    pub fn create_symbol_at_id(&mut self, name: &str, id: u32) -> &mut Symbol {
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        let sym = Symbol::new(id, name.to_string());
        self.symbols.insert(id, sym);
        self.by_name.insert(name.to_string(), id);
        self.symbols.get_mut(&id).unwrap()
    }

    pub fn get(&self, id: u32) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Symbol> {
        self.symbols.get_mut(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).and_then(|id| self.symbols.get(id))
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.by_name.get(name).and_then(|id| self.symbols.get_mut(id))
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }
}

// ---------------------------------------------------------------------------
// Package
// ---------------------------------------------------------------------------

/// Package types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageType {
    Plain,
    Code,
    Module,
    Submodule,
    Linkage,
    /// A custom package type (e.g., declared with `packagetype`).
    Custom(String),
}

impl PackageType {
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

/// A package in the Inter tree.
#[derive(Debug, Clone)]
pub struct Package {
    /// Warehouse resource ID.
    pub resource_id: u32,
    /// Package name (e.g., "main", "Main", "connectors").
    pub name: String,
    /// Package type.
    pub package_type: PackageType,
    /// Symbols table for this package.
    pub symbols: SymbolsTable,
    /// Instructions in this package (in order).
    pub instructions: Vec<Instruction>,
    /// Child packages, indexed by name.
    pub children: HashMap<String, Package>,
    /// Child packages in insertion order.
    pub child_order: Vec<String>,
    /// Persistent flags.
    pub flags: u32,
}

impl Package {
    pub fn new(resource_id: u32, name: String, package_type: PackageType) -> Self {
        Self {
            resource_id,
            name,
            package_type,
            symbols: SymbolsTable::new(resource_id + 1), // symbols table gets its own resource
            instructions: Vec::new(),
            children: HashMap::new(),
            child_order: Vec::new(),
            flags: 0,
        }
    }

    pub fn add_child(&mut self, child: Package) {
        let name = child.name.clone();
        self.child_order.push(name.clone());
        self.children.insert(name, child);
    }

    pub fn get_child(&self, name: &str) -> Option<&Package> {
        self.children.get(name)
    }

    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut Package> {
        self.children.get_mut(name)
    }

    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    /// Whether this is a function body (code package).
    pub fn is_function_body(&self) -> bool {
        matches!(self.package_type, PackageType::Code)
    }

    /// Whether this is the root package.
    pub fn is_root(&self) -> bool {
        self.name.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Inter Tree
// ---------------------------------------------------------------------------

/// The top-level Inter tree.
#[derive(Debug, Clone)]
pub struct InterTree {
    /// The root package (empty name, contains `main`).
    pub root: Package,
    /// Global symbols table (for primitives, package types, etc.).
    pub global_scope: SymbolsTable,
    /// String table: warehouse ID → string content.
    pub strings: HashMap<u32, String>,
    /// Next available warehouse resource ID.
    next_resource_id: u32,
    /// Inter version (major, minor, patch).
    pub version: (u32, u32, u32),
}

impl InterTree {
    pub fn new() -> Self {
        let global = SymbolsTable::new(0);
        // Resource 0 is the global scope, resource 1 is the root package
        let root = Package::new(1, String::new(), PackageType::Plain);
        Self {
            root,
            global_scope: global,
            strings: HashMap::new(),
            next_resource_id: 2,
            version: (1, 0, 0),
        }
    }

    /// Allocate a new warehouse resource ID.
    pub fn alloc_resource_id(&mut self) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }

    /// Store a string and return its warehouse ID.
    pub fn intern_string(&mut self, s: &str) -> u32 {
        // Check if already interned
        for (&id, existing) in &self.strings {
            if existing == s {
                return id;
            }
        }
        let id = self.alloc_resource_id();
        self.strings.insert(id, s.to_string());
        id
    }

    /// Get a string by warehouse ID.
    pub fn get_string(&self, id: u32) -> Option<&str> {
        self.strings.get(&id).map(|s| s.as_str())
    }

    /// Get or create the `main` package.
    pub fn main_package(&mut self) -> &mut Package {
        if !self.root.children.contains_key("main") {
            let main = Package::new(self.alloc_resource_id(), "main".to_string(), PackageType::Plain);
            self.root.add_child(main);
        }
        self.root.get_child_mut("main").unwrap()
    }

    /// Find a package by URL path (e.g., "/main/BasicInformKit/properties").
    pub fn find_package(&self, url: &str) -> Option<&Package> {
        let parts: Vec<&str> = url.split('/').filter(|p| !p.is_empty()).collect();
        let mut current = &self.root;
        for part in parts {
            current = current.get_child(part)?;
        }
        Some(current)
    }

    /// Find a package by URL path, mutable.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree() {
        let tree = InterTree::new();
        assert!(tree.root.children.is_empty());
    }

    #[test]
    fn test_main_package() {
        let mut tree = InterTree::new();
        let main = tree.main_package();
        assert_eq!(main.name, "main");
    }

    #[test]
    fn test_symbols_table() {
        let mut table = SymbolsTable::new(0);
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
        assert_eq!(id1, id2);
        assert_eq!(tree.get_string(id1), Some("hello"));
    }
}
