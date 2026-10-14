use super::*;

use std::collections::BTreeMap;

use crate::binding::group as binding_group;

pub fn summarize_binding_groups_for_tier(items: &[binding_group::BindingGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_trust_domain_tiers_from_group(items: &[binding_group::BindingGroup]) -> Vec<TrustDomainTier> {
    let mut out: Vec<TrustDomainTier> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_trust_domain_tier(item: &TrustDomainTier) -> f64 {
    binding_group::binding_window_weight(7, true)
}
