use super::*;

use std::collections::BTreeMap;

use crate::usage::batch as usage_batch;

pub fn summarize_global_usage_batches_for_forecast(items: &[usage_batch::GlobalUsageBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_reservation_forecasts_from_batch(items: &[usage_batch::GlobalUsageBatch]) -> Vec<ReservationForecast> {
    let mut out: Vec<ReservationForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_forecast(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_reservation_forecast(item: &ReservationForecast) -> f64 {
    usage_batch::global_usage_batch_weight(9, true)
}
