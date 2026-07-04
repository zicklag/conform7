//! TABLES_CSEQ — Tables and Grammar bench dispatch.
//!
//! This module corresponds to the C reference
//! `inform7/assertions-module/Chapter 2/Tables.w` section "Tables and grammar".
//! It dispatches three benches:
//!
//! 1. `Measurements::validate_definitions`
//! 2. `BinaryPredicateFamilies::second_stock`
//! 3. `Tables::check_tables_for_kind_clashes`
//!
//! # References
//!
//! - C reference: `inform7/assertions-module/Chapter 2/Tables.w` — "Tables and grammar"
//! - C reference: `inform7/assertions-module/Chapter 2/Tables.w` — `Tables::check_tables_for_kind_clashes`

use crate::assertions::tables::{Table, Tables};
use crate::calculus::binary_predicate_families::{BpFamily, BinaryPredicateFamilies};
use crate::calculus::binary_predicates::BinaryPredicate;
use crate::knowledge::measurements::{MeasurementDefinition, Measurements};
use crate::knowledge::properties::Property;

/// Run all three TABLES_CSEQ benches.
///
/// Returns a list of clash descriptions from the tables-vs-kinds check
/// (empty if no clashes).
#[allow(clippy::too_many_arguments)]
pub fn run_tables_cseq(
    measurements: &mut [MeasurementDefinition],
    properties: &[Property],
    families: &mut [BpFamily],
    bp_registry: &mut Vec<BinaryPredicate>,
    property_registry: &[Property],
    tables: &[Table],
    is_kind_name: &dyn Fn(&str) -> bool,
    is_subkind_of_object: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    // 1. Validate measurement definitions
    Measurements::validate_definitions(measurements, properties);

    // 2. Second stock for BP families
    BinaryPredicateFamilies::second_stock(families, bp_registry, property_registry);

    // 3. Check tables for kind clashes
    Tables::check_tables_for_kind_clashes(tables, is_kind_name, is_subkind_of_object)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_tables_cseq_empty_inputs() {
        let mut measurements = Vec::new();
        let properties = Vec::new();
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        let property_registry = Vec::new();
        let tables = Vec::new();
        let is_kind_name = |_: &str| false;
        let is_subkind_of_object = |_: &str| false;

        let clashes = run_tables_cseq(
            &mut measurements,
            &properties,
            &mut families,
            &mut bp_registry,
            &property_registry,
            &tables,
            &is_kind_name,
            &is_subkind_of_object,
        );

        assert!(clashes.is_empty());
    }

    #[test]
    fn run_tables_cseq_detects_clashes() {
        let mut measurements = Vec::new();
        let properties = Vec::new();
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        let property_registry = Vec::new();
        let tables = vec![
            Table::new(Some("room")),
            Table::new(Some("container")),
            Table::new(Some("score")),
        ];
        let is_kind_name = |name: &str| matches!(name, "room" | "container" | "thing");
        let is_subkind_of_object = |name: &str| matches!(name, "room" | "container");

        let clashes = run_tables_cseq(
            &mut measurements,
            &properties,
            &mut families,
            &mut bp_registry,
            &property_registry,
            &tables,
            &is_kind_name,
            &is_subkind_of_object,
        );

        assert_eq!(clashes.len(), 2);
        assert!(clashes[0].contains("room"));
        assert!(clashes[1].contains("container"));
    }

    #[test]
    fn run_tables_cseq_no_clashes_when_not_subkind_of_object() {
        let mut measurements = Vec::new();
        let properties = Vec::new();
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        let property_registry = Vec::new();
        let tables = vec![
            Table::new(Some("room")),
            Table::new(Some("container")),
        ];
        // "room" is a kind name but NOT a subkind of object
        let is_kind_name = |name: &str| matches!(name, "room" | "container");
        let is_subkind_of_object = |_: &str| false;

        let clashes = run_tables_cseq(
            &mut measurements,
            &properties,
            &mut families,
            &mut bp_registry,
            &property_registry,
            &tables,
            &is_kind_name,
            &is_subkind_of_object,
        );

        assert!(clashes.is_empty());
    }

    #[test]
    fn run_tables_cseq_unnamed_table_no_clash() {
        let mut measurements = Vec::new();
        let properties = Vec::new();
        let mut families = Vec::new();
        let mut bp_registry = Vec::new();
        let property_registry = Vec::new();
        let tables = vec![Table::new(None)];
        let is_kind_name = |_: &str| true;
        let is_subkind_of_object = |_: &str| true;

        let clashes = run_tables_cseq(
            &mut measurements,
            &properties,
            &mut families,
            &mut bp_registry,
            &property_registry,
            &tables,
            &is_kind_name,
            &is_subkind_of_object,
        );

        assert!(clashes.is_empty());
    }
}
