use super::*;

use std::collections::BTreeMap;

use crate::redaction::metric as redaction_metric;

pub fn summarize_redaction_budgets_for_forecast(items: &[redaction_metric::RedactionBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_lenient_redaction_forecasts_from_budget(
    items: &[redaction_metric::RedactionBudget],
) -> Vec<LenientRedactionForecast> {
    let mut out: Vec<LenientRedactionForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_redaction_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_lenient_redaction_forecast(item: &LenientRedactionForecast) -> String {
    redaction_metric::normalize_redaction_target_code(&item.code)
}
