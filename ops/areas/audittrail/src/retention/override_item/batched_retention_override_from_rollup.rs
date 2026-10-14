use super::*;

use std::collections::BTreeMap;

use crate::data_export::plan as data_export_plan;

pub fn summarize_data_export_rollups_for_override(
    items: &[data_export_plan::DataExportRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_batched_retention_overrides_from_rollup(
    items: &[data_export_plan::DataExportRollup],
) -> Vec<BatchedRetentionOverride> {
    let mut out: Vec<BatchedRetentionOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_retention_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_batched_retention_override(item: &BatchedRetentionOverride) -> String {
    data_export_plan::classify_data_export_plan(item.weighted_total())
}
