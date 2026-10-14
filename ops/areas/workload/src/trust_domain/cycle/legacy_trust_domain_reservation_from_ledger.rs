use super::*;

use std::collections::BTreeMap;

use crate::identity::milestone as identity_milestone;

pub fn summarize_legacy_identity_ledgers_for_reservation(
    items: &[identity_milestone::LegacyIdentityLedger],
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

pub fn derive_legacy_trust_domain_reservations_from_ledger(
    items: &[identity_milestone::LegacyIdentityLedger],
) -> Vec<LegacyTrustDomainReservation> {
    let mut out: Vec<LegacyTrustDomainReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_trust_domain_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_legacy_trust_domain_reservation(item: &LegacyTrustDomainReservation) -> String {
    identity_milestone::classify_legacy_identity_ledger(item.net_exposure())
}
