use super::*;

use std::collections::BTreeMap;

use crate::forecast::threshold as forecast_threshold;

pub fn summarize_forecast_rules_for_policy(items: &[forecast_threshold::ForecastRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_reservation_policies_from_rule(items: &[forecast_threshold::ForecastRule]) -> Vec<ReservationPolicy> {
    let mut out: Vec<ReservationPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_reservation_policy(item: &ReservationPolicy) -> String {
    forecast_threshold::normalize_global_forecast_baseline_code(&item.code)
}
