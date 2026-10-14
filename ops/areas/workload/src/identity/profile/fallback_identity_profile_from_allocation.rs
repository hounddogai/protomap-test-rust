use super::*;

use std::collections::BTreeMap;

use crate::credential::policy as credential_policy;

pub fn summarize_credential_allocations_for_profile(
    items: &[credential_policy::CredentialAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_fallback_identity_profiles_from_allocation(
    items: &[credential_policy::CredentialAllocation],
) -> Vec<FallbackIdentityProfile> {
    let mut out: Vec<FallbackIdentityProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_identity_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_identity_profile(item: &FallbackIdentityProfile) -> f64 {
    credential_policy::credential_policy_weight(7, true)
}
