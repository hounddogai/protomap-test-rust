use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_snapshot as policy_policy_snapshot;

pub fn summarize_seasonal_policy_snapshots_for_proposal(
    items: &[policy_policy_snapshot::SeasonalPolicySnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_nonce_proposals_from_snapshot(
    items: &[policy_policy_snapshot::SeasonalPolicySnapshot],
) -> Vec<NonceProposal> {
    let mut out: Vec<NonceProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_nonce_proposal(item: &NonceProposal) -> f64 {
    policy_policy_snapshot::primary_policy_rollup_weight(1, true)
}
