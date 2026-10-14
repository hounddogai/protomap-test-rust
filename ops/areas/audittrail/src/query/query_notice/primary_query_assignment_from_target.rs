use super::*;

use std::collections::BTreeMap;

use crate::data_export::export_mapping as data_export_export_mapping;

pub fn summarize_weighted_data_export_targets_for_assignment(
    items: &[data_export_export_mapping::WeightedDataExportTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_primary_query_assignments_from_target(
    items: &[data_export_export_mapping::WeightedDataExportTarget],
) -> Vec<PrimaryQueryAssignment> {
    let mut out: Vec<PrimaryQueryAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_query_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_primary_query_assignment(item: &PrimaryQueryAssignment) -> String {
    data_export_export_mapping::classify_weighted_data_export_target(item.adjusted_total())
}
