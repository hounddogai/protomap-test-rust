use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_budget as nonce_nonce_budget;

pub fn summarize_nonce_reservations_for_variance(
    items: &[nonce_nonce_budget::NonceReservation],
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

pub fn derive_manual_quote_variances_from_reservation(
    items: &[nonce_nonce_budget::NonceReservation],
) -> Vec<ManualQuoteVariance> {
    let mut out: Vec<ManualQuoteVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_quote_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_manual_quote_variance(item: &ManualQuoteVariance) -> String {
    nonce_nonce_budget::normalize_nonce_reservation_code(&item.code)
}
