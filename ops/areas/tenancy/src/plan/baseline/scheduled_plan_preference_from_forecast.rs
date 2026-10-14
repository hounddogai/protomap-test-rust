use super::*;

use std::collections::BTreeMap;

use crate::suspension::forecast as suspension_forecast;

pub fn summarize_lenient_suspension_forecasts_for_preference(
    items: &[suspension_forecast::LenientSuspensionForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_scheduled_plan_preferences_from_forecast(
    items: &[suspension_forecast::LenientSuspensionForecast],
) -> Vec<ScheduledPlanPreference> {
    let mut out: Vec<ScheduledPlanPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_plan_preference(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_scheduled_plan_preference(item: &ScheduledPlanPreference) -> String {
    suspension_forecast::lenient_suspension_forecast_bucket(item.weighted_value())
}
