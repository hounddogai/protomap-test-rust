use super::*;

use std::collections::BTreeMap;

use crate::federation::claim as federation_claim;

pub fn summarize_capped_federation_targets_for_milestone(
    items: &[federation_claim::CappedFederationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_adaptive_binding_milestones_from_target(
    items: &[federation_claim::CappedFederationTarget],
) -> Vec<AdaptiveBindingMilestone> {
    let mut out: Vec<AdaptiveBindingMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_binding_milestone(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_adaptive_binding_milestone(item: &AdaptiveBindingMilestone) -> String {
    federation_claim::classify_legacy_federation_reservation(item.projected_value())
}
