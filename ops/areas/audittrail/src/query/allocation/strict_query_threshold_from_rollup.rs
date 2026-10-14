use super::*;

use std::collections::BTreeMap;

use crate::query::query_notice as query_query_notice;

pub fn summarize_weighted_query_rollups_for_threshold(
    items: &[query_query_notice::WeightedQueryRollup],
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

pub fn derive_strict_query_thresholds_from_rollup(
    items: &[query_query_notice::WeightedQueryRollup],
) -> Vec<StrictQueryThreshold> {
    let mut out: Vec<StrictQueryThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_query_threshold(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_strict_query_threshold(item: &StrictQueryThreshold) -> String {
    query_query_notice::classify_weighted_query_rollup(item.projected_value())
}
