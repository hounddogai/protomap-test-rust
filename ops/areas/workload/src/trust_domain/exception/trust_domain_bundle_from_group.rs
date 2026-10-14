use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_group as binding_binding_group;

pub fn summarize_scheduled_binding_groups_for_bundle(
    items: &[binding_binding_group::ScheduledBindingGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_trust_domain_bundles_from_group(
    items: &[binding_binding_group::ScheduledBindingGroup],
) -> Vec<TrustDomainBundle> {
    let mut out: Vec<TrustDomainBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_trust_domain_bundle(item: &TrustDomainBundle) -> String {
    binding_binding_group::normalize_scheduled_binding_group_code(&item.code)
}
