use super::*;

use std::collections::BTreeMap;

use crate::data_export::score as data_export_score;

pub fn summarize_rolling_data_export_policies_for_forecast(
    items: &[data_export_score::RollingDataExportPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_seasonal_query_forecasts_from_policy(
    items: &[data_export_score::RollingDataExportPolicy],
) -> Vec<SeasonalQueryForecast> {
    let mut out: Vec<SeasonalQueryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_query_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_seasonal_query_forecast(item: &SeasonalQueryForecast) -> String {
    data_export_score::classify_default_data_export_draft(item.projected_total())
}
