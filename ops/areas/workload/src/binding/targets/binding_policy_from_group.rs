use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::threshold as trust_domain_threshold;

pub fn summarize_trust_domain_groups_for_policy(
    items: &[trust_domain_threshold::TrustDomainGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_binding_policies_from_group(items: &[trust_domain_threshold::TrustDomainGroup]) -> Vec<BindingPolicy> {
    let mut out: Vec<BindingPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_binding_policy(item: &BindingPolicy) -> f64 {
    trust_domain_threshold::trust_domain_threshold_weight(4, true)
}
