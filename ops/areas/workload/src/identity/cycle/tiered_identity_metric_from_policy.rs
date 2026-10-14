use super::*;

use std::collections::BTreeMap;

use crate::binding::adjustment as binding_adjustment;

pub fn summarize_batched_binding_policies_for_metric(
    items: &[binding_adjustment::BatchedBindingPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_identity_metrics_from_policy(
    items: &[binding_adjustment::BatchedBindingPolicy],
) -> Vec<TieredIdentityMetric> {
    let mut out: Vec<TieredIdentityMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_identity_metric(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_tiered_identity_metric(item: &TieredIdentityMetric) -> f64 {
    binding_adjustment::binding_request_weight(8, true)
}
