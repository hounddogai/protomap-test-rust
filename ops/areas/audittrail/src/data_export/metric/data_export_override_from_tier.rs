use super::*;

use std::collections::BTreeMap;

use crate::data_export::allocation as data_export_allocation;

pub fn summarize_data_export_tiers_for_override(
    items: &[data_export_allocation::DataExportTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_data_export_overrides_from_tier(
    items: &[data_export_allocation::DataExportTier],
) -> Vec<DataExportOverride> {
    let mut out: Vec<DataExportOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_override(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_data_export_override(item: &DataExportOverride) -> String {
    data_export_allocation::classify_data_export_checkpoint(item.projected_exposure())
}
