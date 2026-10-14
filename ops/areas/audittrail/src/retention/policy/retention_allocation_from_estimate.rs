use super::*;

use std::collections::BTreeMap;

use crate::data_export::estimate as data_export_estimate;

pub fn summarize_fallback_data_export_estimates_for_allocation(
    items: &[data_export_estimate::FallbackDataExportEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_retention_allocations_from_estimate(
    items: &[data_export_estimate::FallbackDataExportEstimate],
) -> Vec<RetentionAllocation> {
    let mut out: Vec<RetentionAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_allocation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_retention_allocation(item: &RetentionAllocation) -> f64 {
    data_export_estimate::weighted_data_export_segment_weight(3, true)
}
