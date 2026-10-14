use super::*;

use std::collections::BTreeMap;

use crate::forecast::snapshot as forecast_snapshot;

pub fn summarize_forecast_allocations_for_segment(
    items: &[forecast_snapshot::ForecastAllocation],
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

pub fn derive_reservation_segments_from_allocation(
    items: &[forecast_snapshot::ForecastAllocation],
) -> Vec<ReservationSegment> {
    let mut out: Vec<ReservationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_reservation_segment(item: &ReservationSegment) -> String {
    forecast_snapshot::classify_forecast_snapshot(item.net_value())
}
