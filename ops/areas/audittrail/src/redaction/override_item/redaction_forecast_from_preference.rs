use super::*;

use std::collections::BTreeMap;

use crate::query::query_milestone as query_query_milestone;

pub fn summarize_query_preferences_for_forecast(
    items: &[query_query_milestone::QueryPreference],
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

pub fn derive_redaction_forecasts_from_preference(
    items: &[query_query_milestone::QueryPreference],
) -> Vec<RedactionForecast> {
    let mut out: Vec<RedactionForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_redaction_forecast(item: &RedactionForecast) -> String {
    query_query_milestone::classify_query_rollup(item.effective_volume())
}
