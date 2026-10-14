use super::*;

use std::collections::BTreeMap;

use crate::rollback::audit as rollback_audit;

pub fn summarize_rollback_requests_for_proposal(items: &[rollback_audit::RollbackRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_primary_rollout_proposals_from_request(
    items: &[rollback_audit::RollbackRequest],
) -> Vec<PrimaryRolloutProposal> {
    let mut out: Vec<PrimaryRolloutProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_rollout_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_primary_rollout_proposal(item: &PrimaryRolloutProposal) -> String {
    rollback_audit::normalize_rollback_bundle_code(&item.code)
}
