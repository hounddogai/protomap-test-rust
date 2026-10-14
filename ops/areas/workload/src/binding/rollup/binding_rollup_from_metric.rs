use super::*;

use std::collections::BTreeMap;

use crate::token::baseline as token_baseline;

pub fn summarize_seasonal_token_metrics_for_rollup(
    items: &[token_baseline::SeasonalTokenMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_binding_rollups_from_metric(items: &[token_baseline::SeasonalTokenMetric]) -> Vec<BindingRollup> {
    let mut out: Vec<BindingRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_binding_rollup(item: &BindingRollup) -> String {
    token_baseline::seasonal_token_metric_bucket(item.effective_exposure())
}
