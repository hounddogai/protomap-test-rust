use super::*;

use std::collections::BTreeMap;

use crate::zone::score as zone_score;

pub fn summarize_pending_zone_assignments_for_forecast(
    items: &[zone_score::PendingZoneAssignment],
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

pub fn derive_zone_forecasts_from_assignment(items: &[zone_score::PendingZoneAssignment]) -> Vec<ZoneForecast> {
    let mut out: Vec<ZoneForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_zone_forecast(item: &ZoneForecast) -> f64 {
    zone_score::pending_zone_assignment_weight(9, true)
}
