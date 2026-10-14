use super::*;

use std::collections::BTreeMap;

use crate::query::audit as query_audit;

pub fn summarize_query_schedules_for_forecast(items: &[query_audit::QuerySchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_seasonal_query_forecasts_from_schedule(
    items: &[query_audit::QuerySchedule],
) -> Vec<SeasonalQueryForecast> {
    let mut out: Vec<SeasonalQueryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_query_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_query_forecast(item: &SeasonalQueryForecast) -> String {
    query_audit::normalize_pending_query_audit_code(&item.code)
}
