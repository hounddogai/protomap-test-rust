use super::*;

use std::collections::BTreeMap;

use crate::rollout::metric as rollout_metric;

pub fn summarize_tiered_rollout_proposals_for_claim(
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

pub fn derive_primary_release_claims_from_proposal(
    items: &[rollout_metric::TieredRolloutProposal],
) -> Vec<PrimaryReleaseClaim> {
    let mut out: Vec<PrimaryReleaseClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_release_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_primary_release_claim(item: &PrimaryReleaseClaim) -> String {
    rollout_metric::rollout_metric_bucket(item.projected_load())
}
