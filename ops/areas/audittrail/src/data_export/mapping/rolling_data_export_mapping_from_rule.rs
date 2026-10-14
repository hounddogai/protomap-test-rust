use super::*;

use std::collections::BTreeMap;

use crate::data_export::metric as data_export_metric;

pub fn summarize_manual_data_export_rules_for_mapping(
    items: &[data_export_metric::ManualDataExportRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_rolling_data_export_mappings_from_rule(
    items: &[data_export_metric::ManualDataExportRule],
) -> Vec<RollingDataExportMapping> {
    let mut out: Vec<RollingDataExportMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_data_export_mapping(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_rolling_data_export_mapping(item: &RollingDataExportMapping) -> String {
    data_export_metric::normalize_manual_data_export_rule_code(&item.code)
}
