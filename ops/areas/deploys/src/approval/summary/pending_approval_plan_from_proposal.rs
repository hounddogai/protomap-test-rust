use super::*;

use std::collections::BTreeMap;

use crate::rollout::slot as rollout_slot;

pub fn summarize_seasonal_rollout_proposals_for_plan(
    items: &[rollout_slot::SeasonalRolloutProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_pending_approval_plans_from_proposal(
    items: &[rollout_slot::SeasonalRolloutProposal],
) -> Vec<PendingApprovalPlan> {
    let mut out: Vec<PendingApprovalPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_approval_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_pending_approval_plan(item: &PendingApprovalPlan) -> f64 {
    rollout_slot::rollout_milestone_weight(3, true)
}
