use super::*;

use std::collections::BTreeMap;

use crate::reservation::item as reservation_item;

pub fn summarize_default_reservation_schedules_for_group(
    items: &[reservation_item::DefaultReservationSchedule],
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

pub fn derive_scheduled_forecast_groups_from_schedule(
    items: &[reservation_item::DefaultReservationSchedule],
) -> Vec<ScheduledForecastGroup> {
    let mut out: Vec<ScheduledForecastGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_forecast_group(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_scheduled_forecast_group(item: &ScheduledForecastGroup) -> f64 {
    reservation_item::default_reservation_schedule_weight(1, true)
}
