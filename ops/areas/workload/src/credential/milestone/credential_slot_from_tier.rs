use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::tier as trust_domain_tier;

pub fn summarize_trust_domain_tiers_for_slot(items: &[trust_domain_tier::TrustDomainTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_credential_slots_from_tier(items: &[trust_domain_tier::TrustDomainTier]) -> Vec<CredentialSlot> {
    let mut out: Vec<CredentialSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_credential_slot(item: &CredentialSlot) -> f64 {
    trust_domain_tier::trust_domain_score_weight(7, true)
}
