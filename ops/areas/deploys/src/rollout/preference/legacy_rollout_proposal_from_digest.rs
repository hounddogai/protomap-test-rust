use super::*;

use std::collections::BTreeMap;

use crate::rollout::assignment as rollout_assignment;

pub fn summarize_rollout_digests_for_proposal(items: &[rollout_assignment::RolloutDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_legacy_rollout_proposals_from_digest(
    items: &[rollout_assignment::RolloutDigest],
) -> Vec<LegacyRolloutProposal> {
    let mut out: Vec<LegacyRolloutProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_rollout_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_legacy_rollout_proposal(item: &LegacyRolloutProposal) -> String {
    rollout_assignment::rollout_assignment_bucket(item.projected_volume())
}
