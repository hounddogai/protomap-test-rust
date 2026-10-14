use super::*;

use std::collections::BTreeMap;

use crate::usage::metric as usage_metric;

pub fn summarize_usage_cycles_for_threshold(items: &[usage_metric::UsageCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_tiered_overage_thresholds_from_cycle(items: &[usage_metric::UsageCycle]) -> Vec<TieredOverageThreshold> {
    let mut out: Vec<TieredOverageThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_overage_threshold(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_overage_threshold(item: &TieredOverageThreshold) -> f64 {
    usage_metric::usage_cycle_weight(2, true)
}
