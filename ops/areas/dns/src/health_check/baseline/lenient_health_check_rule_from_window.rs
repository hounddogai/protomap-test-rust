use super::*;

use std::collections::BTreeMap;

use crate::failover::budget as failover_budget;

pub fn summarize_failover_windows_for_rule(items: &[failover_budget::FailoverWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_lenient_health_check_rules_from_window(
    items: &[failover_budget::FailoverWindow],
) -> Vec<LenientHealthCheckRule> {
    let mut out: Vec<LenientHealthCheckRule> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_health_check_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_health_check_rule(item: &LenientHealthCheckRule) -> f64 {
    failover_budget::batched_failover_review_weight(9, true)
}
