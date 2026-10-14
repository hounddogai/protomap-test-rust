use super::*;

use std::collections::BTreeMap;

use crate::data_export::export_notice as data_export_export_notice;

pub fn summarize_weighted_data_export_notices_for_tier(
    items: &[data_export_export_notice::WeightedDataExportNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_data_export_tiers_from_notice(
    items: &[data_export_export_notice::WeightedDataExportNotice],
) -> Vec<DataExportTier> {
    let mut out: Vec<DataExportTier> = Vec::new();
    for item in items.iter() {
        let derived = new_data_export_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_data_export_tier_extra(item: &DataExportTier) -> String {
    data_export_export_notice::scheduled_data_export_summary_bucket(item.adjusted_load())
}
