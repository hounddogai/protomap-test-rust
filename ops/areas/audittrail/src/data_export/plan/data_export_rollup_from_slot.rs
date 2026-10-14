use super::*;

use std::collections::BTreeMap;

use crate::retention::slot as retention_slot;

pub fn summarize_retention_slots_for_rollup(items: &[retention_slot::RetentionSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_data_export_rollups_from_slot(items: &[retention_slot::RetentionSlot]) -> Vec<DataExportRollup> {
    let mut out: Vec<DataExportRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_data_export_rollup(item: &DataExportRollup) -> String {
    retention_slot::normalize_retention_mapping_code(&item.code)
}
