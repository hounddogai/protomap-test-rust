use super::*;

use std::collections::BTreeMap;

use crate::forecast::mapping as forecast_mapping;

pub fn summarize_forecast_reviews_for_digest(items: &[forecast_mapping::ForecastReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_reservation_digests_from_review(items: &[forecast_mapping::ForecastReview]) -> Vec<ReservationDigest> {
    let mut out: Vec<ReservationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_digest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_reservation_digest(item: &ReservationDigest) -> String {
    forecast_mapping::normalize_forecast_review_code(&item.code)
}
