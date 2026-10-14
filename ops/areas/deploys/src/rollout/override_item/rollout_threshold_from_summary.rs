use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_summary as canary_canary_summary;

pub fn summarize_primary_canary_summaries_for_threshold(
    items: &[canary_canary_summary::PrimaryCanarySummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollout_thresholds_from_summary(
    items: &[canary_canary_summary::PrimaryCanarySummary],
) -> Vec<RolloutThreshold> {
    let mut out: Vec<RolloutThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollout_threshold(item: &RolloutThreshold) -> String {
    canary_canary_summary::primary_canary_summary_bucket(item.adjusted_total())
}
