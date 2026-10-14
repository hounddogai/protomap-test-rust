use super::*;

use std::collections::BTreeMap;

use crate::approval::assignment as approval_assignment;

pub fn summarize_approval_assignments_for_digest(
    items: &[approval_assignment::ApprovalAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_global_rollout_digests_from_assignment(
    items: &[approval_assignment::ApprovalAssignment],
) -> Vec<GlobalRolloutDigest> {
    let mut out: Vec<GlobalRolloutDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_rollout_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_global_rollout_digest(item: &GlobalRolloutDigest) -> String {
    approval_assignment::approval_assignment_bucket(item.weighted_exposure())
}
