use super::*;

use std::collections::BTreeMap;

use crate::policy::quota as policy_quota;

pub fn summarize_lenient_policy_metrics_for_schedule(
    items: &[policy_quota::LenientPolicyMetric],
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

pub fn derive_quote_schedules_from_metric(items: &[policy_quota::LenientPolicyMetric]) -> Vec<QuoteSchedule> {
    let mut out: Vec<QuoteSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_schedule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_quote_schedule(item: &QuoteSchedule) -> f64 {
    policy_quota::lenient_policy_metric_weight(6, true)
}
