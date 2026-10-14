use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::cycle as trust_domain_cycle;

pub fn summarize_legacy_trust_domain_reservations_for_quota(
    items: &[trust_domain_cycle::LegacyTrustDomainReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_pending_binding_quotas_from_reservation(
    items: &[trust_domain_cycle::LegacyTrustDomainReservation],
) -> Vec<PendingBindingQuota> {
    let mut out: Vec<PendingBindingQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_binding_quota(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_pending_binding_quota(item: &PendingBindingQuota) -> String {
    trust_domain_cycle::legacy_trust_domain_reservation_bucket(item.effective_load())
}
