use super::*;

use std::collections::BTreeMap;

use crate::binding::default_binding_group as binding_default_binding_group;

pub fn summarize_default_binding_thresholds_for_target(
    items: &[binding_default_binding_group::DefaultBindingThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_batched_identity_targets_from_threshold(
    items: &[binding_default_binding_group::DefaultBindingThreshold],
) -> Vec<BatchedIdentityTarget> {
    let mut out: Vec<BatchedIdentityTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_identity_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_batched_identity_target(item: &BatchedIdentityTarget) -> f64 {
    binding_default_binding_group::scheduled_binding_allocation_weight(5, true)
}
