use super::*;

use std::collections::BTreeMap;

use crate::delegation::milestone as delegation_milestone;

pub fn summarize_pending_delegation_milestones_for_assignment(
    items: &[delegation_milestone::PendingDelegationMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_batched_health_check_assignments_from_milestone(
    items: &[delegation_milestone::PendingDelegationMilestone],
) -> Vec<BatchedHealthCheckAssignment> {
    let mut out: Vec<BatchedHealthCheckAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_health_check_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_health_check_assignment(item: &BatchedHealthCheckAssignment) -> f64 {
    delegation_milestone::pending_delegation_milestone_weight(8, true)
}
