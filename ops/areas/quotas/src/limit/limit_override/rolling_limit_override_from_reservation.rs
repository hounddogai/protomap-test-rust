use super::*;

use std::collections::BTreeMap;

use crate::limit::reservation as limit_reservation;

pub fn summarize_limit_reservations_for_override(
    items: &[limit_reservation::LimitReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_rolling_limit_overrides_from_reservation(
    items: &[limit_reservation::LimitReservation],
) -> Vec<RollingLimitOverride> {
    let mut out: Vec<RollingLimitOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_limit_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_rolling_limit_override(item: &RollingLimitOverride) -> String {
    limit_reservation::normalize_lenient_limit_agreement_code(&item.code)
}
