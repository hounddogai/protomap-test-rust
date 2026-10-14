use super::*;

use std::collections::BTreeMap;

use crate::rollback::policy as rollback_policy;

pub fn summarize_rollback_proposals_for_adjustment(
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

pub fn derive_approval_adjustments_from_proposal(
    items: &[rollback_policy::RollbackProposal],
) -> Vec<ApprovalAdjustment> {
    let mut out: Vec<ApprovalAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_adjustment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_approval_adjustment(item: &ApprovalAdjustment) -> f64 {
    rollback_policy::rollback_policy_weight(2, true)
}
