use super::*;

use std::collections::BTreeMap;

use crate::binding::slot as binding_slot;

pub fn summarize_binding_slots_for_variance(items: &[binding_slot::BindingSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_adaptive_trust_domain_variances_from_slot(
    items: &[binding_slot::BindingSlot],
) -> Vec<AdaptiveTrustDomainVariance> {
    let mut out: Vec<AdaptiveTrustDomainVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_trust_domain_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_adaptive_trust_domain_variance(item: &AdaptiveTrustDomainVariance) -> String {
    binding_slot::fallback_binding_policy_bucket(item.measured_value())
}
