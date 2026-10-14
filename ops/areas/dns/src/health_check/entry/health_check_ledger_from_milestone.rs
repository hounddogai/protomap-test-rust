use super::*;

use std::collections::BTreeMap;

use crate::failover::preference as failover_preference;

pub fn summarize_failover_milestones_for_ledger(
    items: &[failover_preference::FailoverMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_health_check_ledgers_from_milestone(
    items: &[failover_preference::FailoverMilestone],
) -> Vec<HealthCheckLedger> {
    let mut out: Vec<HealthCheckLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_ledger(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_health_check_ledger(item: &HealthCheckLedger) -> String {
    failover_preference::failover_preference_bucket(item.measured_value())
}
