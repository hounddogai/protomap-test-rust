use super::*;

use std::collections::BTreeMap;

use crate::canary::bundle as canary_bundle;

pub fn summarize_canary_thresholds_for_estimate(items: &[canary_bundle::CanaryThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_strict_rollout_estimates_from_threshold(
    items: &[canary_bundle::CanaryThreshold],
) -> Vec<StrictRolloutEstimate> {
    let mut out: Vec<StrictRolloutEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_rollout_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_strict_rollout_estimate(item: &StrictRolloutEstimate) -> String {
    canary_bundle::tiered_canary_bundle_bucket(item.adjusted_exposure())
}
