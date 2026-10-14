use super::*;

use std::collections::BTreeMap;

use crate::usage::manifest as usage_manifest;

pub fn summarize_usage_reservations_for_score(items: &[usage_manifest::UsageReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_primary_forecast_scores_from_reservation(
    items: &[usage_manifest::UsageReservation],
) -> Vec<PrimaryForecastScore> {
    let mut out: Vec<PrimaryForecastScore> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_forecast_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_primary_forecast_score(item: &PrimaryForecastScore) -> String {
    usage_manifest::classify_usage_reservation(item.net_total())
}
