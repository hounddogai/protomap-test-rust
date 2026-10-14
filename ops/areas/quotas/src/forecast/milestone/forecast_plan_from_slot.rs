use super::*;

use std::collections::BTreeMap;

use crate::reservation::quota as reservation_quota;

pub fn summarize_reservation_slots_for_plan(items: &[reservation_quota::ReservationSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_forecast_plans_from_slot(items: &[reservation_quota::ReservationSlot]) -> Vec<ForecastPlan> {
    let mut out: Vec<ForecastPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_forecast_plan(item: &ForecastPlan) -> f64 {
    reservation_quota::reservation_slot_weight(3, true)
}
