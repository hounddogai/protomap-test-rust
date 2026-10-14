use super::*;

use std::collections::BTreeMap;

use crate::approval::quota as approval_quota;

pub fn summarize_approval_adjustments_for_proposal(
    items: &[approval_quota::ApprovalAdjustment],
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

pub fn derive_canary_proposals_from_adjustment(items: &[approval_quota::ApprovalAdjustment]) -> Vec<CanaryProposal> {
    let mut out: Vec<CanaryProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_proposal(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_canary_proposal(item: &CanaryProposal) -> String {
    approval_quota::approval_adjustment_bucket(item.net_exposure())
}
