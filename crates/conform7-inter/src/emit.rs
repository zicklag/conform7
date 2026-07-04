//! Inter emission primitives — functions for adding instructions to an InterTree.
//!
//! These are the building blocks for code generation. Each function appends an
//! instruction to the specified package's ordered item list.
//!
//! # Usage
//!
//! ```rust
//! use conform7_inter::tree::InterTree;
//! use conform7_inter::emit::{emit_numeric_constant, emit_text_constant, emit_pragma, emit_child_package};
//!
//! let mut tree = InterTree::new();
//! let main = tree.main_package().resource_id;
//!
//! let sym = emit_numeric_constant(&mut tree, main, "MAX_SIZE", 100);
//! emit_pragma(&mut tree, main, "inline_arrays");
//! let child = emit_child_package(&mut tree, main, "config", "_module");
//! let sym2 = emit_text_constant(&mut tree, child, "greeting", "Hello");
//! ```

use crate::instruction::Instruction;
use crate::tree::{InterTree, PackageRef, SymbolType};
use crate::value::ValueFormat;

/// Emit a numeric constant into a package.
///
/// Declares a symbol of type `Constant` in the package, then appends a `constant`
/// instruction with the given `i32` value. Returns the symbol ID.
pub fn emit_numeric_constant(tree: &mut InterTree, package: PackageRef, name: &str, value: i32) -> u32 {
    let symbol = tree.declare_symbol(package, name, SymbolType::Constant);
    let instr = Instruction::constant(
        symbol,
        ValueFormat::Signed as u32,
        value as u32,
    );
    tree.add_instruction(package, instr);
    symbol
}

/// Emit a text constant into a package.
///
/// Declares a symbol of type `Constant`, internes the text in the warehouse,
/// then appends a `constant` instruction with a `TEXT` value.
/// Returns the symbol ID.
pub fn emit_text_constant(tree: &mut InterTree, package: PackageRef, name: &str, text: &str) -> u32 {
    let symbol = tree.declare_symbol(package, name, SymbolType::Constant);
    let text_id = tree.intern_string(text);
    let instr = Instruction::constant(
        symbol,
        ValueFormat::Textual as u32,
        text_id,
    );
    tree.add_instruction(package, instr);
    symbol
}

/// Emit a pragma instruction.
///
/// Appends a `pragma` instruction with the given text (interned as a
/// warehouse string).
pub fn emit_pragma(tree: &mut InterTree, package: PackageRef, text: &str) {
    let text_id = tree.intern_string(text);
    let instr = Instruction::pragma(text_id);
    tree.add_instruction(package, instr);
}

/// Emit a child package.
///
/// Creates a new child package of the given `parent` with the specified
/// `name` and package type keyword (e.g., `"_plain"`, `"_module"`).
/// Returns the new package's resource ID.
pub fn emit_child_package(tree: &mut InterTree, parent: PackageRef, name: &str, pkg_type: &str) -> PackageRef {
    tree.create_child_package(parent, name, pkg_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::PackageItem;

    #[test]
    fn test_emit_numeric_constant() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        let sym = emit_numeric_constant(&mut tree, main, "MAX_SIZE", 100);

        // Verify symbol was declared
        let pkg = tree.find_package("/main").unwrap();
        let symbol = pkg.symbols.get(sym).unwrap();
        assert_eq!(symbol.name, "MAX_SIZE");
        assert_eq!(symbol.symbol_type, SymbolType::Constant);

        // Verify instruction was added
        assert_eq!(pkg.items.len(), 1);
        match &pkg.items[0] {
            PackageItem::Instruction(instr) => {
                assert_eq!(instr.construct, crate::instruction::ConstructId::Constant);
                assert_eq!(instr.field(1), Some(sym));
                assert_eq!(instr.field(2), Some(ValueFormat::Signed as u32));
                assert_eq!(instr.field(3), Some(100u32));
            }
            _ => panic!("expected instruction item"),
        }
    }

    #[test]
    fn test_emit_text_constant() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        let sym = emit_text_constant(&mut tree, main, "GREETING", "Hello, World!");

        // Verify symbol was declared
        let pkg = tree.find_package("/main").unwrap();
        let symbol = pkg.symbols.get(sym).unwrap();
        assert_eq!(symbol.name, "GREETING");
        assert_eq!(symbol.symbol_type, SymbolType::Constant);

        // Verify instruction was added
        assert_eq!(pkg.items.len(), 1);
        match &pkg.items[0] {
            PackageItem::Instruction(instr) => {
                assert_eq!(instr.construct, crate::instruction::ConstructId::Constant);
                assert_eq!(instr.field(1), Some(sym));
                assert_eq!(instr.field(2), Some(ValueFormat::Textual as u32));
                // Verify the interned string
                let text_id = instr.field(3).unwrap();
                let text = tree.get_string(text_id).unwrap();
                assert_eq!(text, "Hello, World!");
            }
            _ => panic!("expected instruction item"),
        }
    }

    #[test]
    fn test_emit_pragma() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        emit_pragma(&mut tree, main, "inline_arrays");

        // Verify pragma instruction was added
        let pkg = tree.find_package("/main").unwrap();
        assert_eq!(pkg.items.len(), 1);
        match &pkg.items[0] {
            PackageItem::Instruction(instr) => {
                assert_eq!(instr.construct, crate::instruction::ConstructId::Pragma);
                let text_id = instr.field(1).unwrap();
                let text = tree.get_string(text_id).unwrap();
                assert_eq!(text, "inline_arrays");
            }
            _ => panic!("expected instruction item"),
        }
    }

    #[test]
    fn test_emit_child_package() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        let child = emit_child_package(&mut tree, main, "config", "_module");

        // Verify child package was created
        let pkg = tree.find_package("/main/config").unwrap();
        assert_eq!(pkg.name, "config");
        assert_eq!(pkg.package_type, crate::tree::PackageType::Module);
        assert_eq!(pkg.resource_id, child);

        // Verify it appears in items
        let main_pkg = tree.find_package("/main").unwrap();
        assert_eq!(main_pkg.items.len(), 1);
        match &main_pkg.items[0] {
            PackageItem::Child(name) => assert_eq!(name, "config"),
            _ => panic!("expected child item"),
        }
    }

    #[test]
    fn test_emit_multiple_constants() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        let sym1 = emit_numeric_constant(&mut tree, main, "A", 1);
        let sym2 = emit_numeric_constant(&mut tree, main, "B", 2);
        let sym3 = emit_text_constant(&mut tree, main, "C", "hello");

        assert!(sym1 != sym2);
        assert!(sym2 != sym3);

        let pkg = tree.find_package("/main").unwrap();
        assert_eq!(pkg.items.len(), 3);
    }

    #[test]
    fn test_emit_into_child_package() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;
        let child = emit_child_package(&mut tree, main, "sub", "_plain");

        let sym = emit_numeric_constant(&mut tree, child, "X", 42);
        let pkg = tree.find_package("/main/sub").unwrap();
        let symbol = pkg.symbols.get(sym).unwrap();
        assert_eq!(symbol.name, "X");
    }
}