use super::*;

use std::collections::BTreeMap;

use crate::overage::template as overage_template;

pub fn summarize_overage_metrics_for_reservation(items: &[overage_template::OverageMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_forecast_reservations_from_metric(
    items: &[overage_template::OverageMetric],
) -> Vec<WeightedForecastReservation> {
    let mut out: Vec<WeightedForecastReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_forecast_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_weighted_forecast_reservation(item: &WeightedForecastReservation) -> String {
    overage_template::normalize_fallback_overage_target_code(&item.code)
}
