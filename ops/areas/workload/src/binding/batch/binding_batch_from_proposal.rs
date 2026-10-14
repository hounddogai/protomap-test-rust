use super::*;

use std::collections::BTreeMap;

use crate::binding::review as binding_review;

pub fn summarize_weighted_binding_proposals_for_batch(
    items: &[binding_review::WeightedBindingProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_binding_batches_from_proposal(items: &[binding_review::WeightedBindingProposal]) -> Vec<BindingBatch> {
    let mut out: Vec<BindingBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_binding_batch(item: &BindingBatch) -> String {
    binding_review::classify_binding_review(item.projected_value())
}
