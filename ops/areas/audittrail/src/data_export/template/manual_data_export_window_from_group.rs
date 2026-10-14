use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_proposal as retention_retention_proposal;

pub fn summarize_regional_retention_groups_for_window(
    items: &[retention_retention_proposal::RegionalRetentionGroup],
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

pub fn derive_manual_data_export_windows_from_group(
    items: &[retention_retention_proposal::RegionalRetentionGroup],
) -> Vec<ManualDataExportWindow> {
    let mut out: Vec<ManualDataExportWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_data_export_window(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_manual_data_export_window(item: &ManualDataExportWindow) -> String {
    retention_retention_proposal::classify_regional_retention_group(item.net_value())
}
