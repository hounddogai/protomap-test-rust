use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::agreement as trust_domain_agreement;

pub fn summarize_trust_domain_requests_for_preference(
    items: &[trust_domain_agreement::TrustDomainRequest],
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

pub fn derive_lenient_binding_preferences_from_request(
    items: &[trust_domain_agreement::TrustDomainRequest],
) -> Vec<LenientBindingPreference> {
    let mut out: Vec<LenientBindingPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_binding_preference(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_lenient_binding_preference(item: &LenientBindingPreference) -> f64 {
    trust_domain_agreement::trust_domain_agreement_weight(5, true)
}
