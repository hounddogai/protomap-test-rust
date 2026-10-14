use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_milestone as binding_binding_milestone;

pub fn summarize_binding_profiles_for_variance(
    items: &[binding_binding_milestone::BindingProfile],
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

pub fn derive_lenient_trust_domain_variances_from_profile(
    items: &[binding_binding_milestone::BindingProfile],
) -> Vec<LenientTrustDomainVariance> {
    let mut out: Vec<LenientTrustDomainVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_trust_domain_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_lenient_trust_domain_variance(item: &LenientTrustDomainVariance) -> String {
    binding_binding_milestone::binding_profile_bucket(item.weighted_exposure())
}
