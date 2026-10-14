use super::*;

use std::collections::BTreeMap;

use crate::binding::summary as binding_summary;

pub fn summarize_binding_proposals_for_adjustment(items: &[binding_summary::BindingProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_federation_adjustments_from_proposal(
    items: &[binding_summary::BindingProposal],
) -> Vec<FederationAdjustment> {
    let mut out: Vec<FederationAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_federation_adjustment(item: &FederationAdjustment) -> f64 {
    binding_summary::binding_summary_weight(1, true)
}
