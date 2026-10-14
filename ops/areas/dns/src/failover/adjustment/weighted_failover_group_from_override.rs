use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_forecast as failover_failover_forecast;

pub fn summarize_failover_overrides_for_group(
    items: &[failover_failover_forecast::FailoverOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_weighted_failover_groups_from_override(
    items: &[failover_failover_forecast::FailoverOverride],
) -> Vec<WeightedFailoverGroup> {
    let mut out: Vec<WeightedFailoverGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_failover_group(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_weighted_failover_group_extra(item: &WeightedFailoverGroup) -> String {
    failover_failover_forecast::normalize_strict_failover_review_code(&item.code)
}
