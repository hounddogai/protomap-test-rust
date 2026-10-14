use super::*;

use std::collections::BTreeMap;

use crate::slo::budget as slo_budget;

pub fn summarize_pending_slo_budgets_for_metric(items: &[slo_budget::PendingSloBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_default_slo_metrics_from_budget(items: &[slo_budget::PendingSloBudget]) -> Vec<DefaultSloMetric> {
    let mut out: Vec<DefaultSloMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_default_slo_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_default_slo_metric(item: &DefaultSloMetric) -> String {
    slo_budget::normalize_tiered_slo_manifest_code(&item.code)
}
