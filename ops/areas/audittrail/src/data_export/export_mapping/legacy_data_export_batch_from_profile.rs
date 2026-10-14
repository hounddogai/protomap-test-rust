use super::*;

use std::collections::BTreeMap;

use crate::query::window as query_window;

pub fn summarize_weighted_query_profiles_for_batch(
    items: &[query_window::WeightedQueryProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_legacy_data_export_batches_from_profile(
    items: &[query_window::WeightedQueryProfile],
) -> Vec<LegacyDataExportBatch> {
    let mut out: Vec<LegacyDataExportBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_data_export_batch(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_legacy_data_export_batch(item: &LegacyDataExportBatch) -> String {
    query_window::query_window_bucket(item.adjusted_value())
}
