use super::*;

use std::collections::BTreeMap;

use crate::reservation::lenient_reservation_summary as reservation_lenient_reservation_summary;

pub fn summarize_scheduled_reservation_summaries_for_group(
    items: &[reservation_lenient_reservation_summary::ScheduledReservationSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_forecast_groups_from_summary(
    items: &[reservation_lenient_reservation_summary::ScheduledReservationSummary],
) -> Vec<ForecastGroup> {
    let mut out: Vec<ForecastGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_forecast_group(item: &ForecastGroup) -> f64 {
    reservation_lenient_reservation_summary::scheduled_reservation_summary_weight(8, true)
}
