use super::*;

use std::collections::BTreeMap;

use crate::query::forecast as query_forecast;

pub fn summarize_query_checkpoints_for_profile(items: &[query_forecast::QueryCheckpoint]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_primary_data_export_profiles_from_checkpoint(
    items: &[query_forecast::QueryCheckpoint],
) -> Vec<PrimaryDataExportProfile> {
    let mut out: Vec<PrimaryDataExportProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_data_export_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_primary_data_export_profile(item: &PrimaryDataExportProfile) -> f64 {
    query_forecast::query_checkpoint_weight(9, true)
}
