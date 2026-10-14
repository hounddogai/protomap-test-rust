use super::*;

use std::collections::BTreeMap;

use crate::identity::forecast as identity_forecast;

pub fn summarize_identity_profiles_for_threshold(
    items: &[identity_forecast::IdentityProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_trust_domain_thresholds_from_profile(
    items: &[identity_forecast::IdentityProfile],
) -> Vec<TrustDomainThreshold> {
    let mut out: Vec<TrustDomainThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_trust_domain_threshold(item: &TrustDomainThreshold) -> String {
    identity_forecast::normalize_identity_forecast_code(&item.code)
}
