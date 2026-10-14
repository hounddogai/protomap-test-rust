use super::*;

use std::collections::BTreeMap;

use crate::token::token_variance as token_token_variance;

pub fn summarize_seasonal_token_reservations_for_bundle(
    items: &[token_token_variance::SeasonalTokenReservation],
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

pub fn derive_global_identity_bundles_from_reservation(
    items: &[token_token_variance::SeasonalTokenReservation],
) -> Vec<GlobalIdentityBundle> {
    let mut out: Vec<GlobalIdentityBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_global_identity_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_identity_bundle(item: &GlobalIdentityBundle) -> String {
    token_token_variance::seasonal_token_reservation_bucket(item.adjusted_exposure())
}
