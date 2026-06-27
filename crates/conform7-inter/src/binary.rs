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

use crate::instruction::{ConstructId, Instruction};
use crate::tree::{InterTree, Package, PackageType, SymbolType, SymbolsTable};
use std::cell::Cell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const INTER_SHIBBOLETH: u32 = 0x696E7472;
const TEXT_IRSRC: u32 = 1;
const SYMBOLS_TABLE_IRSRC: u32 = 2;
const NODE_LIST_IRSRC: u32 = 3;
const PACKAGE_REF_IRSRC: u32 = 4;
const INVALID_IANN: u32 = 0xFFFFFFFF;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

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

fn read_word<R: Read>(reader: &mut R) -> Result<u32, BinaryError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
    let c1 = buf[0];
    match c1 & 0xE0 {
        0x00 | 0x20 | 0x40 | 0x60 => Ok(c1 as u32),
        0x80 | 0xA0 => {
            let c1 = (c1 & 0x3F) as u32;
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
            Ok((c1 << 8) | buf[0] as u32)
        }
        0xC0 => {
            let c1 = (c1 & 0x1F) as u32;
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
            Ok((c1 << 16) | ((buf[0] as u32) << 8) | buf[1] as u32)
        }
        0xE0 => {
            if c1 != 0xFF {
                Ok(0x40000000 + (c1 & 0x1F) as u32)
            } else {
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

fn write_word<W: Write>(writer: &mut W, val: u32) -> Result<(), BinaryError> {
    if val < 0x80 {
        writer.write_all(&[val as u8]).map_err(|e| BinaryError::IoError(e.to_string()))
    } else if val < 0x4000 {
        writer.write_all(&[0x80 | (val >> 8) as u8, val as u8])
            .map_err(|e| BinaryError::IoError(e.to_string()))
    } else if val < 0x200000 {
        writer.write_all(&[
            0xC0 | (val >> 16) as u8, ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8,
        ]).map_err(|e| BinaryError::IoError(e.to_string()))
    } else if (0x40000000..0x4000001F).contains(&val) {
        writer.write_all(&[(0xE0 + val - 0x40000000) as u8])
            .map_err(|e| BinaryError::IoError(e.to_string()))
    } else {
        writer.write_all(&[
            0xFF, ((val >> 24) & 0xFF) as u8, ((val >> 16) & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8,
        ]).map_err(|e| BinaryError::IoError(e.to_string()))
    }
}

fn read_text<R: Read>(reader: &mut R) -> Result<String, BinaryError> {
    let len = read_word(reader)? as usize;
    let mut chars = Vec::with_capacity(len);
    for _ in 0..len {
        let c = read_word(reader)?;
        if let Some(ch) = char::from_u32(c) {
            chars.push(ch);
        }
    }
    Ok(chars.into_iter().collect())
}

fn write_text<W: Write>(writer: &mut W, text: &str) -> Result<(), BinaryError> {
    write_word(writer, text.len() as u32)?;
    for ch in text.chars() {
        write_word(writer, ch as u32)?;
    }
    Ok(())
}

fn read_int32_be<R: Read>(reader: &mut R) -> Result<u32, BinaryError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(|e| BinaryError::IoError(e.to_string()))?;
    Ok(u32::from_be_bytes(buf))
}

fn write_int32_be<W: Write>(writer: &mut W, val: u32) -> Result<(), BinaryError> {
    writer.write_all(&val.to_be_bytes()).map_err(|e| BinaryError::IoError(e.to_string()))
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

pub fn read<R: Read>(reader: &mut R) -> Result<InterTree, BinaryError> {
    // --- Header ---
    let shibboleth = read_int32_be(reader)?;
    if shibboleth != INTER_SHIBBOLETH {
        return Err(BinaryError::FormatError("not a binary Inter file".to_string()));
    }
    let zero = read_int32_be(reader)?;
    if zero != 0 {
        return Err(BinaryError::FormatError("not a binary Inter file".to_string()));
    }
    let v1 = read_int32_be(reader)?;
    let v2 = read_int32_be(reader)?;
    let v3 = read_int32_be(reader)?;

    let mut tree = InterTree::new();
    tree.version = (v1, v2, v3);

    // --- Annotations ---
    read_annotations(reader)?;

    // --- Resources ---
    // The resources block has: count, grid_extent, grid_ids..., resource_bodies...
    let count = read_word(reader)?;
    let grid_extent = read_word(reader)?;
    let mut grid: Vec<u32> = vec![0; grid_extent as usize];

    // Read the grid: for each resource i, read its original ID and assign a new ID
    for i in 0..count {
        let original_id = read_word(reader)?;
        let new_id = match i {
            0 => tree.global_scope.resource_id,
            1 => tree.root.resource_id,
            _ => tree.alloc_resource_id(),
        };
        if (original_id as usize) < grid.len() {
            grid[original_id as usize] = new_id;
        }
    }

    // Temporary storage for resources during reading
    let mut symbols_tables: HashMap<u32, SymbolsTable> = HashMap::new();
    // (package, parent_original_id, symbols_table_original_id)
    // We store packages flat and build the hierarchy after all are read
    let mut flat_packages: HashMap<u32, (Package, u32, u32)> = HashMap::new();

    // Read resource bodies
    for _ in 0..count {
        let original_id = read_word(reader)?;
        let new_id = if (original_id as usize) < grid.len() {
            grid[original_id as usize]
        } else {
            original_id
        };
        let resource_type = read_word(reader)?;

        match resource_type {
            TEXT_IRSRC => {
                let text = read_text(reader)?;
                tree.strings.insert(new_id, text);
            }
            SYMBOLS_TABLE_IRSRC => {
                let table = read_symbols_table_body(reader)?;
                symbols_tables.insert(new_id, table);
            }
            PACKAGE_REF_IRSRC => {
                let parent_orig = read_word(reader)?;
                let _flags = read_word(reader)?;
                let symbols_table_orig = read_word(reader)?;
                let name = read_text(reader)?;
                let pkg = Package::new(new_id, name, PackageType::Plain, tree.symbol_counter());
                flat_packages.insert(new_id, (pkg, parent_orig, symbols_table_orig));
            }
            NODE_LIST_IRSRC => {}
            _ => {
                return Err(BinaryError::FormatError(format!(
                    "unknown resource type: {}", resource_type
                )));
            }
        }
    }

    // Attach global scope symbols table
    if let Some(table) = symbols_tables.remove(&tree.global_scope.resource_id) {
        tree.global_scope = table;
    }

    // Build the package hierarchy from the flat list.
    // We process in multiple passes: each pass places packages whose
    // parent has already been placed.
    let mut unplaced: Vec<(Package, u32, u32)> = flat_packages.into_values().collect();
    let mut max_passes = unplaced.len() + 1;
    while !unplaced.is_empty() && max_passes > 0 {
        max_passes -= 1;
        let mut still_unplaced = Vec::new();
        for (mut pkg, parent_orig, symbols_table_orig) in unplaced.drain(..) {
            let parent_id = if (parent_orig as usize) < grid.len() {
                grid[parent_orig as usize]
            } else {
                parent_orig
            };
            let symbols_table_id = if (symbols_table_orig as usize) < grid.len() {
                grid[symbols_table_orig as usize]
            } else {
                symbols_table_orig
            };

            // Attach symbols table if we have it
            if let Some(table) = symbols_tables.remove(&symbols_table_id) {
                pkg.symbols = table;
            }

            // Try to place the package
            if parent_id == 0 || parent_id == tree.root.resource_id {
                tree.root.add_child(pkg);
            } else {
                // Use iterative stack-based approach to find and add to parent.
                // Clone pkg so the compiler knows it's still available after the loop.
                let mut stack: Vec<*mut Package> = vec![&mut tree.root];
                let mut found = false;
                while let Some(ptr) = stack.pop() {
                    let pkg_ref = unsafe { &mut *ptr };
                    if pkg_ref.resource_id == parent_id {
                        pkg_ref.add_child(pkg.clone());
                        found = true;
                        break;
                    }
                    let names: Vec<String> = pkg_ref.child_order.clone();
                    for name in names.iter().rev() {
                        if let Some(child) = pkg_ref.children.get_mut(name) {
                            stack.push(child as *mut Package);
                        }
                    }
                }
                if !found {
                    still_unplaced.push((pkg, parent_orig, symbols_table_orig));
                }
            }
        }
        unplaced = still_unplaced;
    }

    // --- Symbol wirings ---
    read_symbol_wirings(reader, &mut tree, &grid)?;

    // --- Bytecode ---
    read_bytecode(reader, &mut tree, &grid)?;

    Ok(tree)
}

/// Try to place a package in the tree by finding its parent.
/// Uses Option<Package> to handle the move semantics correctly.
fn read_annotations<R: Read>(reader: &mut R) -> Result<(), BinaryError> {
    loop {
        let id = read_word(reader)?;
        if id == INVALID_IANN {
            break;
        }
        let _keyword = read_text(reader)?;
        let _iatype = read_word(reader)?;
    }
    Ok(())
}

/// Read the body of a symbols table resource (symbols and their annotations).
fn read_symbols_table_body<R: Read>(reader: &mut R) -> Result<SymbolsTable, BinaryError> {
    // We don't know the resource ID yet — it's set by the caller
    let mut table = SymbolsTable::new(0, Rc::new(Cell::new(SymbolsTable::SYMBOL_BASE)));
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

        // Read annotations: (bm << 6) + n
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
    Ok(table)
}

fn read_symbol_wirings<R: Read>(
    reader: &mut R,
    _tree: &mut InterTree,
    grid: &[u32],
) -> Result<(), BinaryError> {
    loop {
        let s1_table_orig = read_word(reader)?;
        if s1_table_orig == 0 {
            break;
        }
        let _s1_table_id = if (s1_table_orig as usize) < grid.len() {
            grid[s1_table_orig as usize]
        } else {
            s1_table_orig
        };

        loop {
            let s1_symbol_id = read_word(reader)?;
            if s1_symbol_id == 0 {
                break;
            }
            let s2_table_orig = read_word(reader)?;
            let _s2_table_id = if (s2_table_orig as usize) < grid.len() {
                grid[s2_table_orig as usize]
            } else {
                s2_table_orig
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
    loop {
        let extent = match read_word(reader) {
            Ok(w) => w as usize,
            Err(BinaryError::IoError(_)) => break,
            Err(e) => return Err(e),
        };
        if extent < 2 {
            return Err(BinaryError::FormatError("instruction frame too small".to_string()));
        }

        let pkg_orig = read_word(reader)?;
        let _pkg_id = if (pkg_orig as usize) < grid.len() {
            grid[pkg_orig as usize]
        } else {
            pkg_orig
        };

        let mut words = Vec::with_capacity(extent - 1);
        for _ in 0..extent - 1 {
            words.push(read_word(reader)?);
        }
        if words.is_empty() {
            continue;
        }
        let construct = ConstructId::from_u32(words[0]).unwrap_or(ConstructId::Invalid);
        let _instr = Instruction { construct, words, depth: 0, type_marker: None };
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

pub fn write<W: Write>(tree: &InterTree, writer: &mut W) -> Result<(), BinaryError> {
    write_int32_be(writer, INTER_SHIBBOLETH)?;
    write_int32_be(writer, 0)?;
    write_int32_be(writer, tree.version.0)?;
    write_int32_be(writer, tree.version.1)?;
    write_int32_be(writer, tree.version.2)?;

    write_word(writer, INVALID_IANN)?;

    let mut resources: Vec<(u32, u32)> = Vec::new();
    resources.push((0, SYMBOLS_TABLE_IRSRC));
    resources.push((1, PACKAGE_REF_IRSRC));
    let mut string_ids: Vec<u32> = tree.strings.keys().copied().collect();
    string_ids.sort();
    for &id in &string_ids {
        if id > 1 {
            resources.push((id, TEXT_IRSRC));
        }
    }

    let count = resources.len() as u32;
    write_word(writer, count)?;
    let max_id = resources.iter().map(|(id, _)| *id).max().unwrap_or(0) + 1;
    write_word(writer, max_id)?;
    for &(id, _) in &resources {
        write_word(writer, id)?;
    }
    for &(id, resource_type) in &resources {
        write_word(writer, id)?;
        write_word(writer, resource_type)?;
        match resource_type {
            TEXT_IRSRC => {
                let text = tree.strings.get(&id).map(|s| s.as_str()).unwrap_or("");
                write_text(writer, text)?;
            }
            SYMBOLS_TABLE_IRSRC => write_symbols_table(writer, tree, id)?,
            PACKAGE_REF_IRSRC => write_package_resource(writer, tree, id)?,
            _ => {}
        }
    }

    write_word(writer, 0)?;
    write_bytecode(writer, tree)?;
    Ok(())
}

fn write_symbols_table<W: Write>(
    writer: &mut W, tree: &InterTree, resource_id: u32,
) -> Result<(), BinaryError> {
    let table = if resource_id == tree.global_scope.resource_id {
        &tree.global_scope
    } else {
        return Ok(());
    };
    for sym in table.iter() {
        write_word(writer, sym.id)?;
        write_word(writer, symbol_type_to_u32(sym.symbol_type))?;
        write_word(writer, sym.flags)?;
        write_text(writer, &sym.name)?;
        let bm = 0x20 * sym.boolean_annotations + sym.annotations.len() as u32;
        write_word(writer, bm)?;
        for &(c1, c2) in &sym.annotations {
            write_word(writer, c1)?;
            write_word(writer, c2)?;
        }
        if sym.is_plug() {
            if let Some(ref name) = sym.wired_to_name {
                write_text(writer, name)?;
            }
        }
    }
    write_word(writer, 0)?;
    Ok(())
}

fn write_package_resource<W: Write>(
    writer: &mut W, tree: &InterTree, resource_id: u32,
) -> Result<(), BinaryError> {
    if resource_id == tree.root.resource_id {
        write_word(writer, 0)?;
        write_word(writer, tree.root.flags)?;
        write_word(writer, tree.root.symbols.resource_id)?;
        write_text(writer, &tree.root.name)?;
    }
    Ok(())
}

fn write_bytecode<W: Write>(writer: &mut W, tree: &InterTree) -> Result<(), BinaryError> {
    write_package_bytecode(writer, &tree.root)?;
    Ok(())
}

fn write_package_bytecode<W: Write>(writer: &mut W, pkg: &Package) -> Result<(), BinaryError> {
    for instr in &pkg.instructions {
        write_word(writer, (instr.words.len() + 1) as u32)?;
        write_word(writer, pkg.resource_id)?;
        for &word in &instr.words {
            write_word(writer, word)?;
        }
    }
    for name in &pkg.child_order {
        if let Some(child) = pkg.children.get(name) {
            write_package_bytecode(writer, child)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn symbol_type_from_u32(v: u32) -> SymbolType {
    match v {
        0 => SymbolType::Misc, 1 => SymbolType::Constant, 2 => SymbolType::Variable,
        3 => SymbolType::Typename, 4 => SymbolType::Package, 5 => SymbolType::Primitive,
        6 => SymbolType::Property, 7 => SymbolType::Instance, 8 => SymbolType::Plug,
        9 => SymbolType::Socket, 10 => SymbolType::Label, _ => SymbolType::Misc,
    }
}

fn symbol_type_to_u32(st: SymbolType) -> u32 {
    match st {
        SymbolType::Misc => 0, SymbolType::Constant => 1, SymbolType::Variable => 2,
        SymbolType::Typename => 3, SymbolType::Package => 4, SymbolType::Primitive => 5,
        SymbolType::Property => 6, SymbolType::Instance => 7, SymbolType::Plug => 8,
        SymbolType::Socket => 9, SymbolType::Label => 10,
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
        let vals = [0u32, 1, 0x7F, 0x80, 0xFF, 0x3FFF, 0x4000, 0x1FFFFF,
            0x200000, 0x3FFFFFFF, 0x40000000, 0x40000001, 0x4000001E,
            0x4000001F, 0xFFFFFFFF];
        for &v in &vals {
            let mut buf = Vec::new();
            write_word(&mut buf, v).unwrap();
            let mut c = std::io::Cursor::new(&buf);
            assert_eq!(v, read_word(&mut c).unwrap(), "mismatch for 0x{:X}", v);
        }
    }

    #[test]
    fn test_text_roundtrip() {
        for s in &["", "hello", "Hello, world!\n", "tab\there"] {
            let mut buf = Vec::new();
            write_text(&mut buf, s).unwrap();
            let mut c = std::io::Cursor::new(&buf);
            assert_eq!(s, &read_text(&mut c).unwrap());
        }
    }

    #[test]
    fn test_header_roundtrip() {
        let tree = InterTree::new();
        let mut buf = Vec::new();
        write(&tree, &mut buf).unwrap();
        assert_eq!(&buf[0..4], &[0x69, 0x6E, 0x74, 0x72]);
        assert_eq!(&buf[4..8], &[0, 0, 0, 0]);
    }

    #[test]
    fn test_binary_roundtrip_empty_tree() {
        let tree = InterTree::new();
        let mut buf = Vec::new();
        write(&tree, &mut buf).unwrap();
        let mut c = std::io::Cursor::new(&buf);
        let t2 = read(&mut c).unwrap();
        assert_eq!(t2.version, tree.version);
    }
}
