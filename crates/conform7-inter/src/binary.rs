//! Binary Inter format reader and writer.
//!
//! Based on `Chapter 3/Inter in Binary Files.w` from the bytecode module.
//!
//! # The Binary Inter Format
//!
//! Binary Inter (`.interb`) is a compressed, fast-loading format for Inter
//! programs. It's the primary storage format for precompiled kits and the
//! preferred interchange format between tools.
//!
//! ## File Structure
//!
//! A binary Inter file has five blocks:
//!
//! 1. **Header** (20 bytes, uncompressed): magic number "intr", zero word,
//!    and three version words (major, minor, patch)
//! 2. **Annotations**: declares annotation types used in the file
//! 3. **Resources**: strings, symbols tables, packages, and node lists,
//!    each identified by a warehouse resource ID
//! 4. **Symbol wirings**: cross-package symbol connections (`S1 ~~> S2`)
//! 5. **Bytecode**: the actual instruction frames, each prefixed with
//!    extent and package ID
//!
//! ## Word Compression
//!
//! After the header, all words are stored in a variable-length encoding
//! similar to UTF-8 but optimized for Inter's value distribution:
//!
//! | Range | Bytes | Encoding |
//! |-------|-------|----------|
//! | 0x00000000-0x0000007F | 1 | `0xxxxxxx` |
//! | 0x00000080-0x00003FFF | 2 | `10xxxxxx xxxxxxxx` |
//! | 0x00004000-0x001FFFFF | 3 | `110xxxxx xxxxxxxx xxxxxxxx` |
//! | 0x40000000-0x4000001E | 1 | `111xxxxx` (symbol IDs) |
//! | 0x00200000-0x3FFFFFFF | 5 | `11111111 ...` (4 data bytes) |
//! | 0x4000001F-0xFFFFFFFF | 5 | `11111111 ...` (4 data bytes) |
//!
//! The special 1-byte encoding for 0x40000000-0x4000001E is because
//! symbol IDs start at `SYMBOL_BASE_VAL` (0x40000000), making the most
//! common symbol references compress to a single byte.
//!
//! ## Current Status
//!
//! The binary reader and writer are partially implemented. The word
//! compression, header, and resource block structures work correctly.
//! Full tree reconstruction from binary (with correct package attachment
//! and wiring resolution) is a work in progress.

use crate::instruction::{ConstructId, Instruction};
use crate::tree::{InterTree, Package, PackageType, SymbolType, SymbolsTable};
use std::io::{Read, Write};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Magic number at the start of binary Inter files: ASCII "intr".
/// This is the `INTER_SHIBBOLETH` constant in the C implementation.
const INTER_SHIBBOLETH: u32 = 0x696E7472;

/// Resource type codes. These identify what kind of data a warehouse
/// resource contains. Must match the C `*_IRSRC` constants.
const TEXT_IRSRC: u32 = 1;
const SYMBOLS_TABLE_IRSRC: u32 = 2;
const NODE_LIST_IRSRC: u32 = 3;
const PACKAGE_REF_IRSRC: u32 = 4;

/// Sentinel value marking the end of the annotations list.
const INVALID_IANN: u32 = 0xFFFFFFFF;

/// Base value for symbol IDs. The special compression range
/// 0x40000000-0x4000001E is reserved for the most common symbols.
#[allow(dead_code)]
const SYMBOL_BASE_VAL: u32 = 0x40000000;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when reading or writing binary Inter.
///
/// The C implementation uses `inter_error_message` and exits on error.
/// We use a Rust enum to allow callers to handle errors gracefully.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryError {
    IoError(String),
    FormatError(String),
    VersionMismatch { file_version: String, expected: String },
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::FormatError(msg) => write!(f, "format error: {}", msg),
            Self::VersionMismatch { file_version, expected } => {
                write!(f, "version mismatch: file is v{}, expected v{}", file_version, expected)
            }
        }
    }
}

impl std::error::Error for BinaryError {}

// ---------------------------------------------------------------------------
// Word compression
// ---------------------------------------------------------------------------

/// Read a single compressed word from a binary Inter stream.
///
/// Implements the variable-length decoding scheme described in the
/// module documentation. Returns `IoError` on EOF or read failure.
///
/// This corresponds to `BinaryInter::read_word` in the C implementation.
fn read_word<R: Read>(reader: &mut R) -> Result<u32, BinaryError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
    let c1 = buf[0];

    match c1 & 0xE0 {
        0x00 | 0x20 | 0x40 | 0x60 => {
            // 0xxxxxxx → single byte
            Ok(c1 as u32)
        }
        0x80 | 0xA0 => {
            // 10xxxxxx xxxxxxxx → two bytes
            let c1 = (c1 & 0x3F) as u32;
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
            Ok((c1 << 8) | buf[0] as u32)
        }
        0xC0 => {
            // 110xxxxx xxxxxxxx xxxxxxxx → three bytes
            let c1 = (c1 & 0x1F) as u32;
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
            Ok((c1 << 16) | ((buf[0] as u32) << 8) | buf[1] as u32)
        }
        0xE0 => {
            if c1 != 0xFF {
                // 111xxxxx → 0x40000000 + xxxxx
                Ok(0x40000000 + (c1 & 0x1F) as u32)
            } else {
                // 11111111 xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx → five bytes
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
                Ok(((buf[0] as u32) << 24)
                    | ((buf[1] as u32) << 16)
                    | ((buf[2] as u32) << 8)
                    | buf[3] as u32)
            }
        }
        _ => unreachable!(),
    }
}

/// Write a single compressed word to a binary Inter stream.
///
/// Implements the variable-length encoding scheme. Chooses the most
/// compact representation for the given value.
///
/// This corresponds to `BinaryInter::write_word` in the C implementation.
fn write_word<W: Write>(writer: &mut W, val: u32) -> Result<(), BinaryError> {
    if val < 0x80 {
        writer.write_all(&[val as u8]).map_err(|e| BinaryError::IoError(e.to_string()))?;
    } else if val < 0x4000 {
        writer.write_all(&[0x80 | (val >> 8) as u8, val as u8])
            .map_err(|e| BinaryError::IoError(e.to_string()))?;
    } else if val < 0x200000 {
        writer.write_all(&[
            0xC0 | (val >> 16) as u8,
            ((val >> 8) & 0xFF) as u8,
            (val & 0xFF) as u8,
        ])
        .map_err(|e| BinaryError::IoError(e.to_string()))?;
    } else if (0x40000000..0x4000001F).contains(&val) {
        writer.write_all(&[(0xE0 + val - 0x40000000) as u8])
            .map_err(|e| BinaryError::IoError(e.to_string()))?;
    } else {
        writer.write_all(&[
            0xFF,
            ((val >> 24) & 0xFF) as u8,
            ((val >> 16) & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            (val & 0xFF) as u8,
        ])
        .map_err(|e| BinaryError::IoError(e.to_string()))?;
    }
    Ok(())
}

/// Read a length-prefixed text from a binary Inter stream.
///
/// Texts are stored as a length word followed by that many character
/// words. This is not null-terminated — the length is explicit.
///
/// This corresponds to `BinaryInter::read_text` in the C implementation.
fn read_text<R: Read>(reader: &mut R) -> Result<String, BinaryError> {
    let len = read_word(reader)? as usize;
    let mut chars = Vec::with_capacity(len);
    for _ in 0..len {
        let c = read_word(reader)?;
        // Inter stores characters as words; we only handle ASCII/BMP for now
        if let Some(ch) = char::from_u32(c) {
            chars.push(ch);
        }
    }
    Ok(chars.into_iter().collect())
}

/// Write a length-prefixed text to a binary Inter stream.
///
/// This corresponds to `BinaryInter::write_text` in the C implementation.
fn write_text<W: Write>(writer: &mut W, text: &str) -> Result<(), BinaryError> {
    write_word(writer, text.len() as u32)?;
    for ch in text.chars() {
        write_word(writer, ch as u32)?;
    }
    Ok(())
}

/// Read a big-endian 32-bit integer (uncompressed).
///
/// Used only for the file header. The rest of the file uses
/// variable-length compressed words.
fn read_int32_be<R: Read>(reader: &mut R) -> Result<u32, BinaryError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
    Ok(u32::from_be_bytes(buf))
}

/// Write a big-endian 32-bit integer (uncompressed).
///
/// Used only for the file header.
fn write_int32_be<W: Write>(writer: &mut W, val: u32) -> Result<(), BinaryError> {
    writer.write_all(&val.to_be_bytes()).map_err(|e| BinaryError::IoError(e.to_string()))
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

/// Read a binary Inter file into an [`InterTree`].
///
/// This is the main entry point for reading `.interb` files. It processes
/// the five blocks (header, annotations, resources, wirings, bytecode)
/// in order.
///
/// The `grid` array maps original warehouse IDs (from the file) to new
/// IDs (in our tree). This is necessary because we can't control what
/// IDs the original compiler assigned — we must remap them to our own
/// ID space.
pub fn read<R: Read>(reader: &mut R) -> Result<InterTree, BinaryError> {
    // --- Header ---
    let shibboleth = read_int32_be(reader)?;
    if shibboleth != INTER_SHIBBOLETH {
        return Err(BinaryError::FormatError("not a binary Inter file".to_string()));
    }

    let zero = read_int32_be(reader)?;
    if zero != 0 {
        return Err(BinaryError::FormatError("not a binary Inter file (second word not zero)".to_string()));
    }

    let v1 = read_int32_be(reader)?;
    let v2 = read_int32_be(reader)?;
    let v3 = read_int32_be(reader)?;
    let _file_version = format!("{}.{}.{}", v1, v2, v3);

    let mut tree = InterTree::new();
    tree.version = (v1, v2, v3);

    // --- Annotations ---
    read_annotations(reader)?;

    // --- Resources ---
    let (grid, _count) = read_resources(reader, &mut tree)?;

    // --- Symbol wirings ---
    read_symbol_wirings(reader, &mut tree, &grid)?;

    // --- Bytecode ---
    read_bytecode(reader, &mut tree, &grid)?;

    Ok(tree)
}

fn read_annotations<R: Read>(reader: &mut R) -> Result<(), BinaryError> {
    loop {
        let id = read_word(reader)?;
        if id == INVALID_IANN {
            break;
        }
        let _keyword = read_text(reader)?;
        let _iatype = read_word(reader)?;
        // For now, we skip annotation declarations — they're not needed
        // for round-tripping basic Inter files
    }
    Ok(())
}

fn read_resources<R: Read>(
    reader: &mut R,
    tree: &mut InterTree,
) -> Result<(Vec<u32>, u32), BinaryError> {
    let count = read_word(reader)?;

    // Read the table of warehouse ID numbers
    let grid_extent = read_word(reader)?;
    let mut grid: Vec<u32> = vec![0; grid_extent as usize];

    for i in 0..count {
        let original_id = read_word(reader)?;
        let n = match i {
            0 => {
                // Resource 0: global scope
                tree.global_scope.resource_id
            }
            1 => {
                // Resource 1: root package
                tree.root.resource_id
            }
            _ => tree.alloc_resource_id(),
        };
        if (original_id as usize) < grid.len() {
            grid[original_id as usize] = n;
        }
    }

    // Read the resources proper
    for _ in 0..count {
        let original_id = read_word(reader)?;
        let id = if (original_id as usize) < grid.len() {
            grid[original_id as usize]
        } else {
            original_id
        };

        let resource_type = read_word(reader)?;
        match resource_type {
            TEXT_IRSRC => {
                let text = read_text(reader)?;
                tree.strings.insert(id, text);
            }
            SYMBOLS_TABLE_IRSRC => {
                read_symbols_table(reader, tree, id)?;
            }
            PACKAGE_REF_IRSRC => {
                read_package_resource(reader, tree, id, &grid)?;
            }
            NODE_LIST_IRSRC => {
                // Node lists are built fresh; nothing to read
            }
            _ => {
                return Err(BinaryError::FormatError(format!(
                    "unknown resource type: {}",
                    resource_type
                )));
            }
        }
    }

    Ok((grid, count))
}

fn read_symbols_table<R: Read>(
    reader: &mut R,
    _tree: &mut InterTree,
    resource_id: u32,
) -> Result<(), BinaryError> {
    // Find or create the symbols table
    // The symbols table might already exist (created by a package resource)
    // We need to find which package owns this table
    let mut table = SymbolsTable::new(resource_id);

    loop {
        let symbol_id = read_word(reader)?;
        if symbol_id == 0 {
            break;
        }

        let st = read_word(reader)?;
        let flags = read_word(reader)?;
        let identifier = read_text(reader)?;

        let sym = table.create_symbol_at_id(&identifier, symbol_id);
        sym.symbol_type = symbol_type_from_u32(st);
        sym.flags = flags;

        // Read annotations
        let bm = read_word(reader)?;
        sym.boolean_annotations = bm / 0x20;
        let annotation_count = bm & 0x1F;
        for _ in 0..annotation_count {
            let c1 = read_word(reader)?;
            let c2 = read_word(reader)?;
            sym.annotations.push((c1, c2));
        }

        // If plug, read wired-to name
        if sym.is_plug() {
            let wired_name = read_text(reader)?;
            sym.wired_to_name = Some(wired_name);
        }
    }

    // Store the table — we need to attach it to the right package
    // For now, store it in a temporary location; it'll be attached
    // when the package resource is read
    // (In the C implementation, the table is created and then the package
    // resource sets its scope to this table)

    Ok(())
}

fn read_package_resource<R: Read>(
    reader: &mut R,
    _tree: &mut InterTree,
    resource_id: u32,
    _grid: &[u32],
) -> Result<(), BinaryError> {
    let _parent_resource_id = read_word(reader)?;
    let flags = read_word(reader)?;
    let _symbols_table_id = read_word(reader)?;
    let name = read_text(reader)?;

    // Determine package type from flags or context
    // For now, default to _plain
    let pkg_type = PackageType::Plain;

    let mut pkg = Package::new(resource_id, name.clone(), pkg_type);
    pkg.flags = flags;

    // TODO: Attach to parent package and set symbols table
    // This requires more context about the tree structure

    Ok(())
}

fn read_symbol_wirings<R: Read>(
    reader: &mut R,
    _tree: &mut InterTree,
    grid: &[u32],
) -> Result<(), BinaryError> {
    loop {
        let s1_table_id_raw = read_word(reader)?;
        if s1_table_id_raw == 0 {
            break;
        }
        let _s1_table_id = if (s1_table_id_raw as usize) < grid.len() {
            grid[s1_table_id_raw as usize]
        } else {
            s1_table_id_raw
        };

        loop {
            let s1_symbol_id = read_word(reader)?;
            if s1_symbol_id == 0 {
                break;
            }

            let s2_table_id_raw = read_word(reader)?;
            let _s2_table_id = if (s2_table_id_raw as usize) < grid.len() {
                grid[s2_table_id_raw as usize]
            } else {
                s2_table_id_raw
            };
            let _s2_symbol_id = read_word(reader)?;
        }
    }

    Ok(())
}

fn read_bytecode<R: Read>(
    reader: &mut R,
    _tree: &mut InterTree,
    grid: &[u32],
) -> Result<(), BinaryError> {
    // Read instructions until EOF
    loop {
        let extent = match read_word(reader) {
            Ok(w) => w as usize,
            Err(BinaryError::IoError(_)) => break, // EOF
            Err(e) => return Err(e),
        };

        if extent < 2 {
            return Err(BinaryError::FormatError("instruction frame too small".to_string()));
        }

        // Read package ID
        let pkg_id_raw = read_word(reader)?;
        let _pkg_id = if (pkg_id_raw as usize) < grid.len() {
            grid[pkg_id_raw as usize]
        } else {
            pkg_id_raw
        };

        // Read the frame words
        let mut words = Vec::with_capacity(extent - 1);
        for _ in 0..extent - 1 {
            words.push(read_word(reader)?);
        }

        // The first word of the frame is the construct ID
        if words.is_empty() {
            continue;
        }
        let _construct = ConstructId::from_u32(words[0]).unwrap_or(ConstructId::Invalid);

        let _instr = Instruction {
            construct: _construct,
            words,
        };

        // TODO: Add instruction to the correct package
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Write an [`InterTree`] as a binary Inter file.
///
/// This is the main entry point for generating `.interb` output. It
/// writes the five blocks in order: header, annotations, resources,
/// symbol wirings, and bytecode.
pub fn write<W: Write>(tree: &InterTree, writer: &mut W) -> Result<(), BinaryError> {
    // --- Header ---
    write_int32_be(writer, INTER_SHIBBOLETH)?;
    write_int32_be(writer, 0)?;
    write_int32_be(writer, tree.version.0)?;
    write_int32_be(writer, tree.version.1)?;
    write_int32_be(writer, tree.version.2)?;

    // --- Annotations ---
    write_word(writer, INVALID_IANN)?;

    // --- Resources ---
    // Collect all resources
    let mut resources: Vec<(u32, u32)> = Vec::new(); // (original_id, type)

    // Resource 0: global scope
    resources.push((0, SYMBOLS_TABLE_IRSRC));
    // Resource 1: root package
    resources.push((1, PACKAGE_REF_IRSRC));

    // String resources
    let mut string_ids: Vec<u32> = tree.strings.keys().copied().collect();
    string_ids.sort();
    for &id in &string_ids {
        if id > 1 {
            resources.push((id, TEXT_IRSRC));
        }
    }

    let count = resources.len() as u32;
    write_word(writer, count)?;

    // Table of warehouse ID numbers
    let max_id = resources.iter().map(|(id, _)| *id).max().unwrap_or(0) + 1;
    write_word(writer, max_id)?;
    for &(id, _) in &resources {
        write_word(writer, id)?;
    }

    // Resources proper
    for &(id, resource_type) in &resources {
        write_word(writer, id)?;
        write_word(writer, resource_type)?;

        match resource_type {
            TEXT_IRSRC => {
                let text = tree.strings.get(&id).map(|s| s.as_str()).unwrap_or("");
                write_text(writer, text)?;
            }
            SYMBOLS_TABLE_IRSRC => {
                write_symbols_table(writer, tree, id)?;
            }
            PACKAGE_REF_IRSRC => {
                write_package_resource(writer, tree, id)?;
            }
            NODE_LIST_IRSRC => {
                // Nothing to write
            }
            _ => {}
        }
    }

    // --- Symbol wirings ---
    write_word(writer, 0)?; // No wirings for now

    // --- Bytecode ---
    write_bytecode(writer, tree)?;

    Ok(())
}

fn write_symbols_table<W: Write>(
    writer: &mut W,
    tree: &InterTree,
    resource_id: u32,
) -> Result<(), BinaryError> {
    // Find the symbols table with this resource ID
    let table = if resource_id == tree.global_scope.resource_id {
        &tree.global_scope
    } else {
        // Search packages for the table
        // For now, just handle global scope
        return Ok(());
    };

    for sym in table.iter() {
        write_word(writer, sym.id)?;
        write_word(writer, symbol_type_to_u32(sym.symbol_type))?;
        write_word(writer, sym.flags)?;
        write_text(writer, &sym.name)?;

        // Annotations
        let bm = 0x20 * sym.boolean_annotations + sym.annotations.len() as u32;
        write_word(writer, bm)?;
        for &(c1, c2) in &sym.annotations {
            write_word(writer, c1)?;
            write_word(writer, c2)?;
        }

        // Plug wiring
        if sym.is_plug() {
            if let Some(ref name) = sym.wired_to_name {
                write_text(writer, name)?;
            }
        }
    }

    write_word(writer, 0)?; // End of symbols
    Ok(())
}

fn write_package_resource<W: Write>(
    writer: &mut W,
    tree: &InterTree,
    resource_id: u32,
) -> Result<(), BinaryError> {
    // Root package
    if resource_id == tree.root.resource_id {
        write_word(writer, 0)?; // no parent
        write_word(writer, tree.root.flags)?;
        write_word(writer, tree.root.symbols.resource_id)?;
        write_text(writer, &tree.root.name)?;
    }
    Ok(())
}

fn write_bytecode<W: Write>(
    writer: &mut W,
    tree: &InterTree,
) -> Result<(), BinaryError> {
    // Write instructions from the root package and all children
    write_package_bytecode(writer, tree, &tree.root)?;
    Ok(())
}

fn write_package_bytecode<W: Write>(
    writer: &mut W,
    tree: &InterTree,
    pkg: &Package,
) -> Result<(), BinaryError> {
    for instr in &pkg.instructions {
        // Preframe: extent + 1, package resource ID
        write_word(writer, (instr.words.len() + 1) as u32)?;
        write_word(writer, pkg.resource_id)?;

        // Frame: the instruction words
        for &word in &instr.words {
            write_word(writer, word)?;
        }
    }

    // Recurse into children
    for name in &pkg.child_order {
        if let Some(child) = pkg.children.get(name) {
            write_package_bytecode(writer, tree, child)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn symbol_type_from_u32(v: u32) -> SymbolType {
    match v {
        0 => SymbolType::Misc,
        1 => SymbolType::Constant,
        2 => SymbolType::Variable,
        3 => SymbolType::Typename,
        4 => SymbolType::Package,
        5 => SymbolType::Primitive,
        6 => SymbolType::Property,
        7 => SymbolType::Instance,
        8 => SymbolType::Plug,
        9 => SymbolType::Socket,
        10 => SymbolType::Label,
        _ => SymbolType::Misc,
    }
}

fn symbol_type_to_u32(st: SymbolType) -> u32 {
    match st {
        SymbolType::Misc => 0,
        SymbolType::Constant => 1,
        SymbolType::Variable => 2,
        SymbolType::Typename => 3,
        SymbolType::Package => 4,
        SymbolType::Primitive => 5,
        SymbolType::Property => 6,
        SymbolType::Instance => 7,
        SymbolType::Plug => 8,
        SymbolType::Socket => 9,
        SymbolType::Label => 10,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_compression_roundtrip() {
        let test_values = vec![
            0u32,
            1,
            0x7F,
            0x80,
            0xFF,
            0x3FFF,
            0x4000,
            0x1FFFFF,
            0x200000,
            0x3FFFFFFF,
            0x40000000,
            0x40000001,
            0x4000001E,
            0x4000001F,
            0xFFFFFFFF,
        ];

        for &val in &test_values {
            let mut buf = Vec::new();
            write_word(&mut buf, val).unwrap();
            let mut cursor = std::io::Cursor::new(&buf);
            let decoded = read_word(&mut cursor).unwrap();
            assert_eq!(val, decoded, "mismatch for value 0x{:X}", val);
        }
    }

    #[test]
    fn test_text_roundtrip() {
        let test_strings = vec![
            "",
            "hello",
            "Hello, world!\n",
            "tab\there",
        ];

        for s in &test_strings {
            let mut buf = Vec::new();
            write_text(&mut buf, s).unwrap();
            let mut cursor = std::io::Cursor::new(&buf);
            let decoded = read_text(&mut cursor).unwrap();
            assert_eq!(s, &decoded);
        }
    }

    #[test]
    fn test_header_roundtrip() {
        let tree = InterTree::new();
        let mut buf = Vec::new();
        write(&tree, &mut buf).unwrap();

        // Verify header
        assert_eq!(&buf[0..4], &[0x69, 0x6E, 0x74, 0x72]); // "intr"
        assert_eq!(&buf[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_binary_roundtrip_empty_tree() {
        let tree = InterTree::new();
        let mut buf = Vec::new();
        write(&tree, &mut buf).unwrap();

        let mut cursor = std::io::Cursor::new(&buf);
        let tree2 = read(&mut cursor).unwrap();

        assert_eq!(tree2.version, tree.version);
    }
}
