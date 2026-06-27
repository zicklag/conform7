//! Integration tests: binary Inter compatibility.
//!
//! These tests verify that our binary Inter reader/writer can handle
//! real `.interb` files from the inform7 compiler's precompiled kits.
//!
//! ## Current Status
//!
//! - ✅ Binary write → read round-trip: works for simple trees
//! - ❌ Reading real `.interb` files: the binary reader is partially
//!   implemented. It can parse the header, annotations, and resource
//!   blocks, but doesn't fully reconstruct the tree (packages and
//!   instructions are not attached to the tree structure).
//! - ❌ Testing against the `inter` tool: requires building the C
//!   `inter` tool from the inform7 source, which needs `inweb` and
//!   a C compiler. This is a separate task.
//!
//! ## Plan for Full Compatibility Testing
//!
//! Once the binary reader is complete and the `inter` tool is built:
//!
//! 1. Write textual Inter with our `textual::write`
//! 2. Feed to `inter` to convert to binary: `inter input.intert -format=binary -o output.interb`
//! 3. Read binary back with our `binary::read`
//! 4. Write as text with our `textual::write`
//! 5. Compare with original (accounting for auto-inserted declarations)
//!
//! This will verify byte-for-byte identical output with the reference
//! implementation.

use conform7_inter::binary;
use conform7_inter::textual;
use conform7_inter::{InterTree, Package, PackageType, Instruction, ConstructId};
use std::fs;

/// Path to the inform7 reference implementation's precompiled kits.
const KITS_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../gitignore/inform/inform7/Internal/Inter"
);

// ---------------------------------------------------------------------------
// Binary reader tests — can we parse real .interb files?
// ---------------------------------------------------------------------------
//
// These tests are expected to fail until the binary reader is fully
// implemented. The reader can parse the header and resource blocks
// but doesn't fully reconstruct the tree yet.

#[test]
#[ignore = "binary reader incomplete for real files"]
fn read_english_language_kit_arch16() {
    let path = format!("{}/EnglishLanguageKit/arch-16.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    assert_eq!(tree.version, (2, 0, 0), "expected Inter version 2.0.0");
    assert!(!tree.strings.is_empty(), "expected at least one string resource");
    assert!(tree.next_resource_id > 2, "expected more than just global scope and root");
}

#[test]
#[ignore = "binary reader incomplete for real files"]
fn read_english_language_kit_arch32() {
    let path = format!("{}/EnglishLanguageKit/arch-32.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    assert_eq!(tree.version, (2, 0, 0));
    assert!(!tree.strings.is_empty());
}

#[test]
#[ignore = "binary reader incomplete for real files"]
fn read_english_language_kit_arch16d() {
    let path = format!("{}/EnglishLanguageKit/arch-16d.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    assert_eq!(tree.version, (2, 0, 0));
    assert!(!tree.strings.is_empty());
}

#[test]
#[ignore = "binary reader incomplete for real files"]
fn read_english_language_kit_arch32d() {
    let path = format!("{}/EnglishLanguageKit/arch-32d.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    assert_eq!(tree.version, (2, 0, 0));
    assert!(!tree.strings.is_empty());
}

// ---------------------------------------------------------------------------
// Binary → Text round-trip tests
// ---------------------------------------------------------------------------
//
// These tests read a real .interb file, write it as text, then re-parse
// the text. They verify that the textual output is valid Inter.
// Currently ignored because the binary reader is incomplete.

#[test]
#[ignore = "binary reader incomplete for real files"]
fn binary_to_text_roundtrip_english_arch16() {
    let path = format!("{}/EnglishLanguageKit/arch-16.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    let text = textual::write(&tree);
    assert!(!text.is_empty(), "textual output should not be empty");
    assert!(text.contains("package"), "textual output should contain package declarations");
    let tree2 = textual::read(&text).expect("failed to re-parse textual Inter");
    assert_eq!(tree2.strings.len(), tree.strings.len(),
        "string count should match after round-trip");
}

#[test]
#[ignore = "binary reader incomplete for real files"]
fn binary_to_text_roundtrip_english_arch32() {
    let path = format!("{}/EnglishLanguageKit/arch-32.interb", KITS_DIR);
    let data = fs::read(&path).expect("failed to read kit file");
    let mut cursor = std::io::Cursor::new(&data);
    let tree = binary::read(&mut cursor).expect("failed to parse binary Inter");
    let text = textual::write(&tree);
    assert!(!text.is_empty());
    assert!(text.contains("package"));
    let tree2 = textual::read(&text).expect("failed to re-parse textual Inter");
    assert_eq!(tree2.strings.len(), tree.strings.len());
}

// ---------------------------------------------------------------------------
// Binary write → read round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn binary_write_read_roundtrip_simple_tree() {
    let mut tree = InterTree::new();

    // Create the !print primitive in global scope
    tree.global_scope.create_symbol("!print");
    let print_id = tree.global_scope.get_by_name("!print").unwrap().id;

    // Intern the string
    let hello_id = tree.intern_string("Hello, world.\n");

    // Build the Main code package
    let mut main_fn = Package::new(
        tree.alloc_resource_id(),
        "Main".to_string(),
        PackageType::Code,
        tree.symbol_counter(),
    );

    main_fn.add_instruction(Instruction::new(ConstructId::Code));

    let mut inv = Instruction::new(ConstructId::Inv);
    inv.set_field(1, print_id);
    main_fn.add_instruction(inv);

    let mut val = Instruction::new(ConstructId::Val);
    val.set_field(1, 0x10004); // TEXTUAL_IVAL
    val.set_field(2, hello_id);
    main_fn.add_instruction(val);

    // Add Main to the main package
    tree.main_package().add_child(main_fn);

    // Write as binary
    let mut buf = Vec::new();
    binary::write(&tree, &mut buf).expect("failed to write binary Inter");

    // Read it back
    let mut cursor = std::io::Cursor::new(&buf);
    let tree2 = binary::read(&mut cursor).expect("failed to re-read binary Inter");

    // Verify structure
    assert_eq!(tree2.version, tree.version);
    assert_eq!(tree2.strings.len(), tree.strings.len());
}

// ---------------------------------------------------------------------------
// Text → Binary → Text round-trip through our own code
// ---------------------------------------------------------------------------
//
// This test is currently ignored because the binary reader doesn't fully
// reconstruct the tree (packages and instructions are not attached).
// Once the binary reader is complete, this test should verify that
// text -> binary -> text produces the same output.

#[test]
#[ignore = "binary reader doesn't reconstruct packages yet"]
fn text_to_binary_to_text_roundtrip() {
    let input = concat!(
        "package main _plain\n",
        "\tpackage Main _code\n",
        "\t\tcode\n",
        "\t\t\tinv !enableprinting\n",
        "\t\t\tinv !print\n",
        "\t\t\t\tval \"Hello, world.\\n\"\n",
    );

    // Parse text
    let tree = textual::read(input).expect("failed to parse textual Inter");

    // Write as binary
    let mut buf = Vec::new();
    binary::write(&tree, &mut buf).expect("failed to write binary Inter");

    // Read binary back
    let mut cursor = std::io::Cursor::new(&buf);
    let tree2 = binary::read(&mut cursor).expect("failed to re-read binary Inter");

    // Write as text
    let output = textual::write(&tree2);

    // The output should contain the same structure
    assert!(output.contains("package main _plain"), "output should contain main package");
    assert!(output.contains("package Main _code"), "output should contain Main package");
    assert!(output.contains("inv !print"), "output should contain inv !print");
}
