use super::*;

use std::collections::BTreeMap;

use crate::binding::threshold as binding_threshold;

pub fn summarize_batched_binding_thresholds_for_adjustment(
    items: &[binding_threshold::BatchedBindingThreshold],
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

pub fn derive_default_federation_adjustments_from_threshold(
    items: &[binding_threshold::BatchedBindingThreshold],
) -> Vec<DefaultFederationAdjustment> {
    let mut out: Vec<DefaultFederationAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_federation_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_default_federation_adjustment(item: &DefaultFederationAdjustment) -> String {
    binding_threshold::classify_batched_binding_threshold(item.weighted_volume())
}
