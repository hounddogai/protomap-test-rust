use super::*;

use std::collections::BTreeMap;

use crate::rollback::policy as rollback_policy;

pub fn summarize_rollback_proposals_for_reservation(
    items: &[rollback_policy::RollbackProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rollout_reservations_from_proposal(
    items: &[rollback_policy::RollbackProposal],
) -> Vec<RolloutReservation> {
    let mut out: Vec<RolloutReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_rollout_reservation(item: &RolloutReservation) -> String {
    rollback_policy::normalize_rollback_policy_code(&item.code)
}
