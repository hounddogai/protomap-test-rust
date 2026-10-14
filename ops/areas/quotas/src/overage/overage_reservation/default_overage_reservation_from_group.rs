use super::*;

use std::collections::BTreeMap;

use crate::forecast::tier as forecast_tier;

pub fn summarize_scheduled_forecast_groups_for_reservation(
    items: &[forecast_tier::ScheduledForecastGroup],
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

pub fn derive_default_overage_reservations_from_group(
    items: &[forecast_tier::ScheduledForecastGroup],
) -> Vec<DefaultOverageReservation> {
    let mut out: Vec<DefaultOverageReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_default_overage_reservation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_default_overage_reservation(item: &DefaultOverageReservation) -> String {
    forecast_tier::normalize_default_forecast_assignment_code(&item.code)
}
