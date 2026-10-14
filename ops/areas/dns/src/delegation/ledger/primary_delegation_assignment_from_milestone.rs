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

pub fn derive_primary_delegation_assignments_from_milestone(
    items: &[delegation_milestone::PendingDelegationMilestone],
) -> Vec<PrimaryDelegationAssignment> {
    let mut out: Vec<PrimaryDelegationAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_delegation_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_primary_delegation_assignment(item: &PrimaryDelegationAssignment) -> String {
    delegation_milestone::classify_delegation_plan(item.adjusted_exposure())
}
