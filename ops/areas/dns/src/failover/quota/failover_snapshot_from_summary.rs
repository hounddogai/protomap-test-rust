use super::*;

use std::collections::BTreeMap;

use crate::health_check::milestone as health_check_milestone;

pub fn summarize_health_check_summaries_for_snapshot(
    items: &[health_check_milestone::HealthCheckSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_failover_snapshots_from_summary(
    items: &[health_check_milestone::HealthCheckSummary],
) -> Vec<FailoverSnapshot> {
    let mut out: Vec<FailoverSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_failover_snapshot(item: &FailoverSnapshot) -> String {
    health_check_milestone::health_check_summary_bucket(item.adjusted_volume())
}
