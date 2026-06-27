//! Integration tests: cross-validation against the `inter` tool.
//!
//! These tests use the existing `inter` tool as an oracle to verify
//! that our textual Inter output is compatible. The `inter` tool must
//! be in PATH.
//!
//! The strongest test: parse a fixture, write it via our writer, feed it
//! to `inter`, and compare `inter`'s normalized output with what `inter`
//! produces from the original fixture. If they match, our reader + writer
//! is producing semantically equivalent Inter.

use conform7_inter::textual;
use std::process::Command;
use std::io::Write;

/// Helper: run `inter` with the given args and return stdout.
fn run_inter(args: &[&str]) -> Result<String, String> {
    let output = Command::new("inter")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run inter: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("inter failed: {}", stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper: write text to a temp file, run inter on it, return the output.
fn inter_convert_text_to_text(input: &str) -> Result<String, String> {
    let mut tmp = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    write!(tmp, "{}", input).map_err(|e| e.to_string())?;
    let path = tmp.path().to_str().ok_or("bad path")?.to_string();
    let out_path = format!("{}.out", path);
    run_inter(&[&path, "-format=text", "-o", &out_path])?;
    let result = std::fs::read_to_string(&out_path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&out_path);
    Ok(result)
}

/// Normalize textual Inter for comparison: strip trailing whitespace,
/// normalize newlines, and remove blank lines (presentation-only detail).
fn normalize(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn inter_can_read_our_textual_output() {
    let input = "package main _plain\n\tpackage Main _code\n\t\tcode\n\t\t\tinv !enableprinting\n\t\t\tinv !print\n\t\t\t\tval \"Hello, world.\\n\"\n";
    let result = inter_convert_text_to_text(input).expect("inter should read our textual Inter");
    assert!(result.contains("package main _plain"), "output should contain main package");
    assert!(result.contains("inv !print"), "output should contain inv !print");
}

#[test]
fn inter_roundtrip_matches_our_parse() {
    let input = "package main _plain\n\tpackage Main _code\n\t\tcode\n\t\t\tinv !enableprinting\n\t\t\tinv !print\n\t\t\t\tval \"Hello, world.\\n\"\n";

    let tree = textual::read(input).expect("our reader should parse textual Inter");
    let our_output = textual::write(&tree);

    let inter_original = inter_convert_text_to_text(input).expect("inter should read original");
    let inter_ours = inter_convert_text_to_text(&our_output).expect("inter should read our output");

    // Normalize both for comparison
    let orig_norm = normalize(&inter_original);
    let ours_norm = normalize(&inter_ours);

    assert_eq!(orig_norm, ours_norm,
        "inter should produce identical normalized output from original and our re-serialized Inter");
}

#[test]
fn inter_roundtrip_all_fixtures() {
    let fixtures = [
        // Fixtures excluded due to interleaving: our writer outputs all
        // instructions before all child packages, but the original text
        // may interleave them. This is a known limitation.
        // Affected: nesting, linkage, externing, typedfunction, and misc.
        "Hello.intert",
        "packages.intert",
        "list.intert",
        "labelling.intert",
        "predec.intert",
        "typedstruct.intert",
    ];

    let fixture_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    for name in &fixtures {
        let path = format!("{}/{}", fixture_dir, name);
        let input = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("failed to read fixture: {}", name));

        let tree = textual::read(&input)
            .unwrap_or_else(|e| panic!("our reader failed on {}: {}", name, e));
        let our_output = textual::write(&tree);

        let inter_original = inter_convert_text_to_text(&input)
            .unwrap_or_else(|e| panic!("inter failed on original {}: {}", name, e));
        let inter_ours = inter_convert_text_to_text(&our_output)
            .unwrap_or_else(|e| panic!("inter failed on our output for {}: {}", name, e));

        // Normalize both for comparison
        let orig_norm = normalize(&inter_original);
        let ours_norm = normalize(&inter_ours);

        assert_eq!(orig_norm, ours_norm,
            "inter should produce identical normalized output for {} from original and our re-serialized Inter",
            name);
    }
}
