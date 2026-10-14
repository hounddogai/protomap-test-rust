use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::slot as trust_domain_slot;

pub fn summarize_trust_domain_reservations_for_claim(
    items: &[trust_domain_slot::TrustDomainReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_identity_claims_from_reservation(
    items: &[trust_domain_slot::TrustDomainReservation],
) -> Vec<PrimaryIdentityClaim> {
    let mut out: Vec<PrimaryIdentityClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_identity_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_identity_claim(item: &PrimaryIdentityClaim) -> f64 {
    trust_domain_slot::trust_domain_forecast_weight(5, true)
}
