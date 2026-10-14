use super::*;

use std::collections::BTreeMap;

use crate::identity::profile as identity_profile;

pub fn summarize_legacy_identity_proposals_for_claim(
    items: &[identity_profile::LegacyIdentityProposal],
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

pub fn derive_credential_claims_from_proposal(
    items: &[identity_profile::LegacyIdentityProposal],
) -> Vec<CredentialClaim> {
    let mut out: Vec<CredentialClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_claim(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_credential_claim(item: &CredentialClaim) -> f64 {
    identity_profile::identity_rule_weight(7, true)
}
