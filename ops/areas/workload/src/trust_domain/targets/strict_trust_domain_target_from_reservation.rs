use super::*;

use std::collections::BTreeMap;

use crate::token::adjustment as token_adjustment;

pub fn summarize_lenient_token_reservations_for_target(
    items: &[token_adjustment::LenientTokenReservation],
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

pub fn derive_strict_trust_domain_targets_from_reservation(
    items: &[token_adjustment::LenientTokenReservation],
) -> Vec<StrictTrustDomainTarget> {
    let mut out: Vec<StrictTrustDomainTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_trust_domain_target(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_strict_trust_domain_target(item: &StrictTrustDomainTarget) -> String {
    token_adjustment::classify_lenient_token_reservation(item.weighted_load())
}
