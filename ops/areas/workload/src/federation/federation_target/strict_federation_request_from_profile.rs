use super::*;

use std::collections::BTreeMap;

use crate::identity::profile as identity_profile;

pub fn summarize_fallback_identity_profiles_for_request(
    items: &[identity_profile::FallbackIdentityProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_strict_federation_requests_from_profile(
    items: &[identity_profile::FallbackIdentityProfile],
) -> Vec<StrictFederationRequest> {
    let mut out: Vec<StrictFederationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_federation_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_strict_federation_request(item: &StrictFederationRequest) -> f64 {
    identity_profile::identity_rule_weight(3, true)
}
