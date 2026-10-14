use super::*;

use std::collections::BTreeMap;

use crate::failover::variance as failover_variance;

pub fn summarize_failover_variances_for_rule(items: &[failover_variance::FailoverVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_health_check_rules_from_variance(
    items: &[failover_variance::FailoverVariance],
) -> Vec<ScheduledHealthCheckRule> {
    let mut out: Vec<ScheduledHealthCheckRule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_health_check_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_scheduled_health_check_rule(item: &ScheduledHealthCheckRule) -> String {
    failover_variance::normalize_failover_slot_code(&item.code)
}
