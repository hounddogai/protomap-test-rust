use super::*;

use std::collections::BTreeMap;

use crate::retention::policy as retention_policy;

pub fn summarize_retention_snapshots_for_allocation(
    items: &[retention_policy::RetentionSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_legacy_data_export_allocations_from_snapshot(
    items: &[retention_policy::RetentionSnapshot],
) -> Vec<LegacyDataExportAllocation> {
    let mut out: Vec<LegacyDataExportAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_data_export_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_legacy_data_export_allocation(item: &LegacyDataExportAllocation) -> String {
    retention_policy::normalize_retention_allocation_code(&item.code)
}
