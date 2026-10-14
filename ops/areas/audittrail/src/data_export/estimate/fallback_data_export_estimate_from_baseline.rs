use super::*;

use std::collections::BTreeMap;

use crate::data_export::mapping as data_export_mapping;

pub fn summarize_data_export_baselines_for_estimate(
    items: &[data_export_mapping::DataExportBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_fallback_data_export_estimates_from_baseline(
    items: &[data_export_mapping::DataExportBaseline],
) -> Vec<FallbackDataExportEstimate> {
    let mut out: Vec<FallbackDataExportEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_data_export_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_fallback_data_export_estimate(item: &FallbackDataExportEstimate) -> String {
    data_export_mapping::normalize_data_export_baseline_code(&item.code)
}
