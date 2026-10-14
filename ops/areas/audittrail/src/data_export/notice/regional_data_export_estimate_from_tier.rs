use super::*;

use std::collections::BTreeMap;

use crate::query::profile as query_profile;

pub fn summarize_query_tiers_for_estimate(items: &[query_profile::QueryTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_regional_data_export_estimates_from_tier(
    items: &[query_profile::QueryTier],
) -> Vec<RegionalDataExportEstimate> {
    let mut out: Vec<RegionalDataExportEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_data_export_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_data_export_estimate(item: &RegionalDataExportEstimate) -> String {
    query_profile::classify_seasonal_query_variance(item.effective_volume())
}
