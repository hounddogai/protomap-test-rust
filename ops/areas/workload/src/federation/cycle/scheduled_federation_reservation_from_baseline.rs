use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::profile as trust_domain_profile;

pub fn summarize_trust_domain_baselines_for_reservation(
    items: &[trust_domain_profile::TrustDomainBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_scheduled_federation_reservations_from_baseline(
    items: &[trust_domain_profile::TrustDomainBaseline],
) -> Vec<ScheduledFederationReservation> {
    let mut out: Vec<ScheduledFederationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_federation_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_scheduled_federation_reservation(item: &ScheduledFederationReservation) -> String {
    trust_domain_profile::classify_trust_domain_profile(item.weighted_exposure())
}
