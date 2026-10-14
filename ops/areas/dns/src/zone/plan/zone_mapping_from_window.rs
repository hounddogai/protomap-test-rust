use super::*;

use std::collections::BTreeMap;

use crate::failover::budget as failover_budget;

pub fn summarize_failover_windows_for_mapping(items: &[failover_budget::FailoverWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_zone_mappings_from_window(items: &[failover_budget::FailoverWindow]) -> Vec<ZoneMapping> {
    let mut out: Vec<ZoneMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_zone_mapping(item: &ZoneMapping) -> f64 {
    failover_budget::batched_failover_review_weight(5, true)
}
