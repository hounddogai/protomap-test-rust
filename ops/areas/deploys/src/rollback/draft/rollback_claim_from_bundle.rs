use super::*;

use std::collections::BTreeMap;

use crate::rollout::estimate as rollout_estimate;

pub fn summarize_rollout_bundles_for_claim(items: &[rollout_estimate::RolloutBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_rollback_claims_from_bundle(items: &[rollout_estimate::RolloutBundle]) -> Vec<RollbackClaim> {
    let mut out: Vec<RollbackClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_rollback_claim(item: &RollbackClaim) -> f64 {
    rollout_estimate::rollout_schedule_weight(9, true)
}
