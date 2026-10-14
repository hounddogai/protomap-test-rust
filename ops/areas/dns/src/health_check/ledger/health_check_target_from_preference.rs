use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_preference as failover_failover_preference;

pub fn summarize_primary_failover_preferences_for_target(
    items: &[failover_failover_preference::PrimaryFailoverPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_health_check_targets_from_preference(
    items: &[failover_failover_preference::PrimaryFailoverPreference],
) -> Vec<HealthCheckTarget> {
    let mut out: Vec<HealthCheckTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_target(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_health_check_target(item: &HealthCheckTarget) -> String {
    failover_failover_preference::classify_default_failover_cycle(item.measured_value())
}
