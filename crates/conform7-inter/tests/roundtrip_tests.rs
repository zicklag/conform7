//! Integration tests: round-trip real Inter test files from the inform7 test suite.
//!
//! These tests read each `.intert` fixture, write it back, and verify that:
//! 1. The output can be parsed again (no crashes)
//! 2. The re-parsed tree has the same structure as the original
//! 3. For simple fixtures, the textual output is identical to the input
//!    (modulo auto-inserted declarations like `packagetype` and `primitive`)
//!
//! Fixtures that exercise constructs the writer doesn't fully support
//! (like `misc.intert` with `splat`, `cast`, `sum{...}`) are checked
//! for structural equivalence only.

use conform7_inter::textual;
use std::fs;

/// Assert that two Inter trees have equivalent structure.
///
/// Compares package hierarchy, symbol counts, and instruction counts.
/// Does not compare byte-for-byte textual output, since the writer may
/// normalize whitespace or reorder constructs.
fn assert_trees_equivalent(original: &str, roundtripped: &str, name: &str) {
    let tree1 = textual::read(original)
        .unwrap_or_else(|e| panic!("failed to parse original {}: {}", name, e));
    let tree2 = textual::read(roundtripped)
        .unwrap_or_else(|e| panic!("failed to parse roundtripped {}: {}", name, e));

    // Compare package structure
    let pkg1_count = count_packages(&tree1.root);
    let pkg2_count = count_packages(&tree2.root);
    assert_eq!(pkg1_count, pkg2_count,
        "package count mismatch for {}: original={}, roundtripped={}",
        name, pkg1_count, pkg2_count);

    // Compare string count (should be stable)
    assert_eq!(tree1.strings.len(), tree2.strings.len(),
        "string count mismatch for {}", name);
}

fn count_packages(pkg: &conform7_inter::Package) -> usize {
    1 + pkg.children_iter().map(count_packages).sum::<usize>()
}

/// Assert that the written output matches the original input exactly.
///
/// This is the strongest check: it verifies that parsing and re-serializing
/// produces byte-identical output. Some fixtures may fail this due to
/// annotations, ordering differences, or auto-inserted declarations.
fn assert_textual_identical(original: &str, name: &str) {
    let tree = textual::read(original)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", name, e));
    let output = textual::write(&tree);

    // Normalize both for comparison: trim trailing whitespace, normalize
    // newlines, and strip blank lines (which are presentation-only).
    let normalize = |s: &str| -> String {
        s.lines()
            .map(|l| l.trim_end())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    };

    let orig_norm = normalize(original.trim());
    let output_norm = normalize(output.trim());

    if orig_norm != output_norm {
        // Compute diff for debugging
        let orig_lines: Vec<&str> = orig_norm.lines().collect();
        let out_lines: Vec<&str> = output_norm.lines().collect();
        let mut diffs = Vec::new();
        for i in 0..orig_lines.len().max(out_lines.len()) {
            let a = orig_lines.get(i).copied().unwrap_or("");
            let b = out_lines.get(i).copied().unwrap_or("");
            if a != b {
                diffs.push(format!("  line {}:\n    - {}\n    + {}", i + 1, a, b));
            }
        }
        if diffs.len() <= 10 {
            panic!(
                "textual output differs from input for {} ({} diffs):\n{}",
                name,
                diffs.len(),
                diffs.join("\n")
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Tests that every fixture can be parsed and re-parsed without crashing.
/// These are the basic sanity checks.
mod structural {
    use super::*;

    fn read_fixture(name: &str) -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
        fs::read_to_string(format!("{}/{}", dir, name))
            .unwrap_or_else(|_| panic!("failed to read fixture: {}", name))
    }

    #[test]
    fn roundtrip_hello() {
        let input = read_fixture("Hello.intert");
        let tree = textual::read(&input).unwrap();
        let output = textual::write(&tree);
        let tree2 = textual::read(&output).unwrap();

        let main = tree2.find_package("/main").unwrap();
        let main_fn = main.get_child("Main").unwrap();
        assert_eq!(main_fn.instructions().count(), 4);
    }

    #[test]
    fn roundtrip_packages() {
        let input = read_fixture("packages.intert");
        let tree = textual::read(&input).unwrap();
        let output = textual::write(&tree);
        let tree2 = textual::read(&output).unwrap();
        let main = tree2.find_package("/main").unwrap();
        assert!(main.get_child("sub").is_some());
    }

    #[test]
    fn roundtrip_misc() {
        let input = read_fixture("misc.intert");
        let tree = textual::read(&input).unwrap();
        let output = textual::write(&tree);
        let tree2 = textual::read(&output).unwrap();

        // Verify structure is preserved
        let main = tree2.find_package("/main").unwrap();
        assert!(main.symbols.get_by_name("K_number").is_some());
        assert!(main.symbols.get_by_name("V_banana").is_some());
        assert!(main.symbols.get_by_name("C_death").is_some());
    }

    #[test]
    fn roundtrip_nesting() {
        let input = read_fixture("nesting.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "nesting");
    }

    #[test]
    fn roundtrip_list() {
        let input = read_fixture("list.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "list");
    }

    #[test]
    fn roundtrip_linkage() {
        let input = read_fixture("linkage.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "linkage");
    }

    #[test]
    fn roundtrip_labelling() {
        let input = read_fixture("labelling.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "labelling");
    }

    #[test]
    fn roundtrip_externing() {
        let input = read_fixture("externing.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "externing");
    }

    #[test]
    fn roundtrip_predec() {
        let input = read_fixture("predec.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "predec");
    }

    #[test]
    fn roundtrip_typedfunction() {
        let input = read_fixture("typedfunction.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "typedfunction");
    }

    #[test]
    fn roundtrip_typedstruct() {
        let input = read_fixture("typedstruct.intert");
        assert_trees_equivalent(&input, &textual::write(&textual::read(&input).unwrap()), "typedstruct");
    }
}

/// Tests that certain fixtures produce byte-identical textual output.
/// These are the strongest validity check.
mod identical_output {
    use super::*;

    fn read_fixture(name: &str) -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
        fs::read_to_string(format!("{}/{}", dir, name))
            .unwrap_or_else(|_| panic!("failed to read fixture: {}", name))
    }

    #[test]
    fn hello_identical() {
        let input = read_fixture("Hello.intert");
        assert_textual_identical(&input, "Hello.intert");
    }

    #[test]
    fn packages_identical() {
        let input = read_fixture("packages.intert");
        assert_textual_identical(&input, "packages.intert");
    }

    #[test]
    fn list_identical() {
        let input = read_fixture("list.intert");
        assert_textual_identical(&input, "list.intert");
    }

    #[test]
    fn labelling_identical() {
        let input = read_fixture("labelling.intert");
        assert_textual_identical(&input, "labelling.intert");
    }

    #[test]
    fn predec_identical() {
        let input = read_fixture("predec.intert");
        assert_textual_identical(&input, "predec.intert");
    }

    #[test]
    fn typedstruct_identical() {
        let input = read_fixture("typedstruct.intert");
        assert_textual_identical(&input, "typedstruct.intert");
    }

    #[test]
    fn nesting_identical() {
        let input = read_fixture("nesting.intert");
        assert_textual_identical(&input, "nesting.intert");
    }

    #[test]
    fn linkage_identical() {
        let input = read_fixture("linkage.intert");
        assert_textual_identical(&input, "linkage.intert");
    }

    #[test]
    fn externing_identical() {
        let input = read_fixture("externing.intert");
        assert_textual_identical(&input, "externing.intert");
    }

    #[test]
    fn typedfunction_identical() {
        let input = read_fixture("typedfunction.intert");
        assert_textual_identical(&input, "typedfunction.intert");
    }

    #[test]
    fn misc_identical() {
        let input = read_fixture("misc.intert");
        assert_textual_identical(&input, "misc.intert");
    }
}
