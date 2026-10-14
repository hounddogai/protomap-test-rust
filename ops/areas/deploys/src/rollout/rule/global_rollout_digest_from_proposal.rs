use super::*;

use std::collections::BTreeMap;

use crate::rollout::metric as rollout_metric;

pub fn summarize_tiered_rollout_proposals_for_digest(
    items: &[rollout_metric::TieredRolloutProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_global_rollout_digests_from_proposal(
    items: &[rollout_metric::TieredRolloutProposal],
) -> Vec<GlobalRolloutDigest> {
    let mut out: Vec<GlobalRolloutDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_rollout_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_rollout_digest(item: &GlobalRolloutDigest) -> String {
    rollout_metric::rollout_metric_bucket(item.weighted_exposure())
}
