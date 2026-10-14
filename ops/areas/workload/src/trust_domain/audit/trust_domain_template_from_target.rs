use super::*;

use std::collections::BTreeMap;

use crate::federation::targets as federation_targets;

pub fn summarize_federation_targets_for_template(
    items: &[federation_targets::FederationTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_trust_domain_templates_from_target(
    items: &[federation_targets::FederationTarget],
) -> Vec<TrustDomainTemplate> {
    let mut out: Vec<TrustDomainTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_template(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_trust_domain_template(item: &TrustDomainTemplate) -> String {
    federation_targets::normalize_adaptive_federation_cycle_code(&item.code)
}
