use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_summary as failover_failover_summary;

pub fn summarize_batched_failover_summaries_for_assignment(
    items: &[failover_failover_summary::BatchedFailoverSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_tiered_zone_assignments_from_summary(
    items: &[failover_failover_summary::BatchedFailoverSummary],
) -> Vec<TieredZoneAssignment> {
    let mut out: Vec<TieredZoneAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_zone_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_tiered_zone_assignment(item: &TieredZoneAssignment) -> String {
    failover_failover_summary::classify_failover_claim(item.projected_load())
}
