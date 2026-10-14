use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_metric as budget_burn_burn_metric;

pub fn summarize_weighted_budget_burn_metrics_for_batch(
    items: &[budget_burn_burn_metric::WeightedBudgetBurnMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_lenient_slo_batches_from_metric(
    items: &[budget_burn_burn_metric::WeightedBudgetBurnMetric],
) -> Vec<LenientSloBatch> {
    let mut out: Vec<LenientSloBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_slo_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_lenient_slo_batch(item: &LenientSloBatch) -> String {
    budget_burn_burn_metric::normalize_default_budget_burn_metric_code(&item.code)
}
