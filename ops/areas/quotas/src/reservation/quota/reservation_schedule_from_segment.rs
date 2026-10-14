use super::*;

use std::collections::BTreeMap;

use crate::forecast::estimate as forecast_estimate;

pub fn summarize_default_forecast_segments_for_schedule(
    items: &[forecast_estimate::DefaultForecastSegment],
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

pub fn derive_reservation_schedules_from_segment(
    items: &[forecast_estimate::DefaultForecastSegment],
) -> Vec<ReservationSchedule> {
    let mut out: Vec<ReservationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_reservation_schedule(item: &ReservationSchedule) -> f64 {
    forecast_estimate::forecast_manifest_weight(3, true)
}
