use super::*;

use std::collections::BTreeMap;

use crate::canary::allocation as canary_allocation;

pub fn summarize_seasonal_canary_thresholds_for_threshold(
    items: &[canary_allocation::SeasonalCanaryThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_capped_canary_thresholds_from_threshold(
    items: &[canary_allocation::SeasonalCanaryThreshold],
) -> Vec<CappedCanaryThreshold> {
    let mut out: Vec<CappedCanaryThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_canary_threshold(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_capped_canary_threshold(item: &CappedCanaryThreshold) -> String {
    canary_allocation::classify_seasonal_canary_threshold(item.net_volume())
}
