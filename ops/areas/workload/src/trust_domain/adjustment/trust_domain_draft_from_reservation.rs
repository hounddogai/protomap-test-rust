use super::*;

use std::collections::BTreeMap;

use crate::binding::reservation as binding_reservation;

pub fn summarize_binding_reservations_for_draft(
    items: &[binding_reservation::BindingReservation],
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

pub fn derive_trust_domain_drafts_from_reservation(
    items: &[binding_reservation::BindingReservation],
) -> Vec<TrustDomainDraft> {
    let mut out: Vec<TrustDomainDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_trust_domain_draft(item: &TrustDomainDraft) -> String {
    binding_reservation::normalize_binding_threshold_code(&item.code)
}
