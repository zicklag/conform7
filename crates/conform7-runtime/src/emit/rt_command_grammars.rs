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

        // Verify the child package exists
        let child_pkg = tree.find_package_mut_by_id(child_id.unwrap());
        assert!(child_pkg.is_some());
    }
}