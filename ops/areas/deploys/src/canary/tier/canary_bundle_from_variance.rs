use super::*;

use std::collections::BTreeMap;

use crate::rollout::variance as rollout_variance;

pub fn summarize_rollout_variances_for_bundle(items: &[rollout_variance::RolloutVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_canary_bundles_from_variance(items: &[rollout_variance::RolloutVariance]) -> Vec<CanaryBundle> {
    let mut out: Vec<CanaryBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_canary_bundle(item: &CanaryBundle) -> String {
    rollout_variance::normalize_fallback_rollout_slot_code(&item.code)
}
