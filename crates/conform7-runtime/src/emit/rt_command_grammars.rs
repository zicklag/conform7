use conform7_inter::tree::InterTree;
use conform7_inter::emit::{emit_numeric_constant, emit_child_package};

/// Compile non-generic constants for command grammars.
///
/// Corresponds to `RTCommandGrammars::compile_non_generic_constants` in the C
/// reference (`runtime-module/Chapter 7/Command Grammars.w`).
///
/// Emits two numeric constants into a child package:
/// - `DICT_WORD_SIZE` = 4
/// - `DICT_ENTRY_BYTES` = 64
pub fn compile_non_generic_constants(tree: &mut InterTree, main_package: u32) {
    // Create a child package for command grammars
    let pkg = emit_child_package(tree, main_package, "command_grammars", "_module");
    // Emit DICT_WORD_SIZE and DICT_ENTRY_BYTES
    emit_numeric_constant(tree, pkg, "DICT_WORD_SIZE", 4);
    emit_numeric_constant(tree, pkg, "DICT_ENTRY_BYTES", 64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use conform7_inter::tree::InterTree;

    #[test]
    fn test_compile_non_generic_constants() {
        let mut tree = InterTree::new();
        let main = tree.main_package().resource_id;

        compile_non_generic_constants(&mut tree, main);

        // Verify the child package was created
        let child_id = tree.find_package_by_path(&["main", "command_grammars"]);
        assert!(child_id.is_some(), "command_grammars package should exist");

        // Verify constants exist with correct values
        let child_pkg = tree.find_package_mut_by_id(child_id.unwrap()).unwrap();

        let word_size_sym = child_pkg.symbols.get_by_name("DICT_WORD_SIZE").unwrap();
        assert_eq!(word_size_sym.name, "DICT_WORD_SIZE");
        assert_eq!(word_size_sym.symbol_type, conform7_inter::tree::SymbolType::Constant);

        let entry_bytes_sym = child_pkg.symbols.get_by_name("DICT_ENTRY_BYTES").unwrap();
        assert_eq!(entry_bytes_sym.name, "DICT_ENTRY_BYTES");
        assert_eq!(entry_bytes_sym.symbol_type, conform7_inter::tree::SymbolType::Constant);

        // Verify instruction values
        let instrs: Vec<&conform7_inter::instruction::Instruction> = child_pkg.instructions().collect();
        assert_eq!(instrs.len(), 2, "should have 2 constant instructions");

        // DICT_WORD_SIZE = 4
        assert_eq!(instrs[0].field(3), Some(4), "DICT_WORD_SIZE should be 4");
        // DICT_ENTRY_BYTES = 64
        assert_eq!(instrs[1].field(3), Some(64), "DICT_ENTRY_BYTES should be 64");
    }
}