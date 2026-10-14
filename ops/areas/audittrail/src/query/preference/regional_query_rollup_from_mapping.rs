use super::*;

use std::collections::BTreeMap;

use crate::query::query_mapping as query_query_mapping;

pub fn summarize_pending_query_mappings_for_rollup(
    items: &[query_query_mapping::PendingQueryMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_regional_query_rollups_from_mapping(
    items: &[query_query_mapping::PendingQueryMapping],
) -> Vec<RegionalQueryRollup> {
    let mut out: Vec<RegionalQueryRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_query_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_regional_query_rollup(item: &RegionalQueryRollup) -> String {
    query_query_mapping::pending_query_mapping_bucket(item.effective_volume())
}
