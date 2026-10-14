use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_request as verifier_verifier_request;

pub fn summarize_lenient_verifier_requests_for_reservation(
    items: &[verifier_verifier_request::LenientVerifierRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_nonce_reservations_from_request(
    items: &[verifier_verifier_request::LenientVerifierRequest],
) -> Vec<NonceReservation> {
    let mut out: Vec<NonceReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_nonce_reservation(item: &NonceReservation) -> f64 {
    verifier_verifier_request::lenient_verifier_request_weight(5, true)
}
