use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_metric as forecast_forecast_metric;

pub fn summarize_forecast_reservations_for_policy(
    items: &[forecast_forecast_metric::ForecastReservation],
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

pub fn derive_global_overage_policies_from_reservation(
    items: &[forecast_forecast_metric::ForecastReservation],
) -> Vec<GlobalOveragePolicy> {
    let mut out: Vec<GlobalOveragePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_global_overage_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_global_overage_policy(item: &GlobalOveragePolicy) -> String {
    forecast_forecast_metric::classify_weighted_forecast_metric(item.net_value())
}
