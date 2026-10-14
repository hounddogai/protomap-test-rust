use super::*;

use std::collections::BTreeMap;

use crate::retention::checkpoint as retention_checkpoint;

pub fn summarize_default_retention_proposals_for_assignment(
    items: &[retention_checkpoint::DefaultRetentionProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_weighted_retention_assignments_from_proposal(
    items: &[retention_checkpoint::DefaultRetentionProposal],
) -> Vec<WeightedRetentionAssignment> {
    let mut out: Vec<WeightedRetentionAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_retention_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_weighted_retention_assignment(item: &WeightedRetentionAssignment) -> f64 {
    retention_checkpoint::default_retention_proposal_weight(7, true)
}
