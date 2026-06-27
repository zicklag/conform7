//! Integration tests: round-trip real Inter test files from the inform7 test suite.

use conform7_inter::textual;
use std::fs;

#[test]
fn roundtrip_hello() {
    let input = fs::read_to_string("tests/fixtures/Hello.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let output = textual::write(&tree);

    // Re-parse the output
    let tree2 = textual::read(&output).unwrap();

    // Verify structure
    let main = tree2.find_package("/main").unwrap();
    let main_fn = main.get_child("Main").unwrap();
    assert_eq!(main_fn.instructions.len(), 4); // code, inv, inv, val
}

#[test]
fn roundtrip_packages() {
    let input = fs::read_to_string("tests/fixtures/packages.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let output = textual::write(&tree);

    // Re-parse
    let tree2 = textual::read(&output).unwrap();
    let main = tree2.find_package("/main").unwrap();
    assert!(main.get_child("sub").is_some());
}

#[test]
fn roundtrip_misc() {
    let input = fs::read_to_string("tests/fixtures/misc.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
    // Just verify it doesn't panic
}

#[test]
fn roundtrip_nesting() {
    let input = fs::read_to_string("tests/fixtures/nesting.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_list() {
    let input = fs::read_to_string("tests/fixtures/list.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_linkage() {
    let input = fs::read_to_string("tests/fixtures/linkage.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_labelling() {
    let input = fs::read_to_string("tests/fixtures/labelling.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_externing() {
    let input = fs::read_to_string("tests/fixtures/externing.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_predec() {
    let input = fs::read_to_string("tests/fixtures/predec.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_typedfunction() {
    let input = fs::read_to_string("tests/fixtures/typedfunction.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}

#[test]
fn roundtrip_typedstruct() {
    let input = fs::read_to_string("tests/fixtures/typedstruct.intert").unwrap();
    let tree = textual::read(&input).unwrap();
    let _output = textual::write(&tree);
}
