use super::*;

use std::collections::BTreeMap;

use crate::delegation::forecast as delegation_forecast;

pub fn summarize_fallback_delegation_forecasts_for_preference(
    items: &[delegation_forecast::FallbackDelegationForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_pending_health_check_preferences_from_forecast(
    items: &[delegation_forecast::FallbackDelegationForecast],
) -> Vec<PendingHealthCheckPreference> {
    let mut out: Vec<PendingHealthCheckPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_health_check_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_pending_health_check_preference(item: &PendingHealthCheckPreference) -> String {
    delegation_forecast::fallback_delegation_forecast_bucket(item.projected_load())
}
