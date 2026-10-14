use super::*;

use std::collections::BTreeMap;

use crate::federation::cycle as federation_cycle;

pub fn summarize_scheduled_federation_reservations_for_review(
    items: &[federation_cycle::ScheduledFederationReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_token_reviews_from_reservation(
    items: &[federation_cycle::ScheduledFederationReservation],
) -> Vec<TokenReview> {
    let mut out: Vec<TokenReview> = Vec::new();
    for item in items.iter() {
        let derived = new_token_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_token_review(item: &TokenReview) -> f64 {
    federation_cycle::scheduled_federation_reservation_weight(1, true)
}
