use super::*;

use std::collections::BTreeMap;

use crate::canary::scheduled_canary_assignment as canary_scheduled_canary_assignment;

pub fn summarize_adaptive_canary_assignments_for_request(
    items: &[canary_scheduled_canary_assignment::AdaptiveCanaryAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_approval_requests_from_assignment(
    items: &[canary_scheduled_canary_assignment::AdaptiveCanaryAssignment],
) -> Vec<WeightedApprovalRequest> {
    let mut out: Vec<WeightedApprovalRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_approval_request(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_weighted_approval_request(item: &WeightedApprovalRequest) -> String {
    canary_scheduled_canary_assignment::adaptive_canary_assignment_bucket(item.projected_total())
}
