use super::*;

use std::collections::BTreeMap;

use crate::binding::template as binding_template;

pub fn summarize_lenient_binding_reservations_for_entry(
    items: &[binding_template::LenientBindingReservation],
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

pub fn derive_trust_domain_entries_from_reservation(
    items: &[binding_template::LenientBindingReservation],
) -> Vec<TrustDomainEntry> {
    let mut out: Vec<TrustDomainEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_trust_domain_entry(item: &TrustDomainEntry) -> String {
    binding_template::classify_legacy_binding_template(item.effective_exposure())
}
