use super::*;

use std::collections::BTreeMap;

use crate::reservation::review as reservation_review;

pub fn summarize_reservation_overrides_for_score(
    items: &[reservation_review::ReservationOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_forecast_scores_from_override(items: &[reservation_review::ReservationOverride]) -> Vec<ForecastScore> {
    let mut out: Vec<ForecastScore> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_forecast_score(item: &ForecastScore) -> f64 {
    reservation_review::scheduled_reservation_override_weight(7, true)
}
