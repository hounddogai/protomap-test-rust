use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_metric as redaction_redaction_metric;

pub fn summarize_fallback_redaction_summaries_for_forecast(
    items: &[redaction_redaction_metric::FallbackRedactionSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_query_forecasts_from_summary(
    items: &[redaction_redaction_metric::FallbackRedactionSummary],
) -> Vec<QueryForecast> {
    let mut out: Vec<QueryForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_query_forecast(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_query_forecast(item: &QueryForecast) -> String {
    redaction_redaction_metric::fallback_redaction_summary_bucket(item.effective_value())
}
