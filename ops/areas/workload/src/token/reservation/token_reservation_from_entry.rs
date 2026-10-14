use super::*;

use std::collections::BTreeMap;

use crate::identity::review as identity_review;

pub fn summarize_identity_entries_for_reservation(items: &[identity_review::IdentityEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_token_reservations_from_entry(items: &[identity_review::IdentityEntry]) -> Vec<TokenReservation> {
    let mut out: Vec<TokenReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_token_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}
