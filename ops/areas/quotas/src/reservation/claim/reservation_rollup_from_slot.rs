use super::*;

use std::collections::BTreeMap;

use crate::forecast::group as forecast_group;

pub fn summarize_global_forecast_slots_for_rollup(
    items: &[forecast_group::GlobalForecastSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_reservation_rollups_from_slot(items: &[forecast_group::GlobalForecastSlot]) -> Vec<ReservationRollup> {
    let mut out: Vec<ReservationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_reservation_rollup(item: &ReservationRollup) -> String {
    forecast_group::classify_forecast_group(item.adjusted_total())
}
