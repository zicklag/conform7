//! Tables — creating table structures from TABLE_NT nodes.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Tables.w`. It handles the creation
//! of table data structures from TABLE_NT parse nodes.
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Tables.w`

use conform7_syntax::parse_node::ParseNode;

/// A table in the model world.
pub struct Table {
    /// The name of this table (for kind-clash checking).
    pub table_name_text: Option<String>,
}

impl Table {
    pub fn new(name: Option<&str>) -> Self {
        Table {
            table_name_text: name.map(|s| s.to_string()),
        }
    }
}

/// Table creation and management.
pub struct Tables;

impl Tables {
    /// Create a table from a TABLE_NT node (stub).
    ///
    /// Called when a TABLE_NT node is encountered during the pre-pass.
    /// Parses the table structure and creates the corresponding table
    /// data in the model world.
    ///
    /// # References
    ///
    /// - C reference: `inform7/assertions-module/Chapter 2/Tables.w` —
    ///   `Tables::create_table`
    pub fn create_table(_node: &mut ParseNode) {
        // Deferred: table creation
    }

    /// Check if any table name clashes with a kind name that is a subkind of object.
    ///
    /// Returns a list of clash descriptions (empty if no clashes).
    pub fn check_tables_for_kind_clashes(
        tables: &[Table],
        is_kind_name: &dyn Fn(&str) -> bool,
        is_subkind_of_object: &dyn Fn(&str) -> bool,
    ) -> Vec<String> {
        let mut clashes = Vec::new();
        for table in tables {
            if let Some(name) = &table.table_name_text {
                if is_kind_name(name) && is_subkind_of_object(name) {
                    clashes.push(format!(
                        "Table '{}' has the same name as a kind that is a subkind of object",
                        name
                    ));
                }
            }
        }
        clashes
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use conform7_syntax::node_type::NodeType;
    use conform7_syntax::parse_node::ParseNode;
    use conform7_syntax::wording::Wording;

    #[test]
    fn create_table_does_not_panic() {
        let mut node = ParseNode::new(NodeType::Table, Wording::EMPTY);
        Tables::create_table(&mut node);
        // Should not panic
    }
}
