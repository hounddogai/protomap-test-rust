use super::*;

use std::collections::BTreeMap;

use crate::binding::proposal as binding_proposal;

pub fn summarize_binding_snapshots_for_proposal(items: &[binding_proposal::BindingSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_identity_proposals_from_snapshot(
    items: &[binding_proposal::BindingSnapshot],
) -> Vec<ManualIdentityProposal> {
    let mut out: Vec<ManualIdentityProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_identity_proposal(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_manual_identity_proposal(item: &ManualIdentityProposal) -> String {
    binding_proposal::binding_snapshot_bucket(item.effective_volume())
}
