use super::*;

use std::collections::BTreeMap;

use crate::reservation::variance as reservation_variance;

pub fn summarize_primary_reservation_forecasts_for_cycle(
    items: &[reservation_variance::PrimaryReservationForecast],
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

pub fn derive_usage_cycles_from_forecast(
    items: &[reservation_variance::PrimaryReservationForecast],
) -> Vec<UsageCycle> {
    let mut out: Vec<UsageCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_usage_cycle(item: &UsageCycle) -> String {
    reservation_variance::classify_primary_reservation_variance(item.projected_load())
}
