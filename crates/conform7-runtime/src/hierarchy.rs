/// Simple hierarchy lookup — find a package by path components.
///
/// Returns the package's resource ID, or `None` if not found.
use conform7_inter::tree::InterTree;

/// Look up a package by path (e.g., `["main", "source_text", "kinds"]`).
pub fn find_package(tree: &InterTree, path: &[&str]) -> Option<u32> {
    tree.find_package_by_path(path)
}